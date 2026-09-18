//! Safe wrappers for the cvc5 parser API.
//!
//! This module is only available when the `parser` feature is enabled.
//!
//! # Example
//!
//! ```rust
//! use cvc5::{TermManager, Solver, InputParser, SymbolManager, InputLanguage};
//!
//! let mut tm = TermManager::new();
//! let mut solver = Solver::new(&tm);
//! let mut sm = SymbolManager::new(&tm);
//!
//! // The parser mutates both as it parses, so it borrows both exclusively.
//! let mut parser = InputParser::new(&mut solver, &mut sm);
//! parser.set_str_input(
//!     InputLanguage::SmtLib26,
//!     "(set-logic QF_LIA)(declare-const x Int)(assert (> x 0))(check-sat)",
//!     "example",
//! )?;
//!
//! while !parser.done() {
//!     match parser.next_command() {
//!         Ok(Some(cmd)) => { parser.invoke(&cmd)?; }
//!         Ok(None) => break,
//!         Err(e) => panic!("parse error: {e}"),
//!     }
//! }
//! # Ok::<(), cvc5::Error>(())
//! ```

use cvc5_sys::InputLanguage;
use cvc5_sys::parser::*;
use std::ffi::CString;
use std::fmt;

use crate::error::{Error, Result};
use crate::ffi::{checked, cstr_or_empty, cstr_to_string, non_null, raw_slice};
use crate::{Solver, Sort, Term, TermManager};

/// Manages symbols for the parser.
///
/// Internally tracks a symbol table and meta-information from SMT-LIB inputs
/// (named assertions, declared functions/sorts, etc.).
///
/// Owns a reference to the [`TermManager`] it was created from, and can be
/// shared with an [`InputParser`] by reference so that parsed commands update
/// the same table.
pub struct SymbolManager {
    inner: *mut cvc5_sys::parser::SymbolManager,
}

impl Drop for SymbolManager {
    fn drop(&mut self) {
        unsafe { symbol_manager_delete(self.inner) };
    }
}

impl SymbolManager {
    /// Create a new symbol manager associated with the given term manager.
    ///
    /// The C wrapper takes its own reference to the term manager, so the
    /// `SymbolManager` may outlive the [`TermManager`] binding.
    pub fn new(tm: &TermManager) -> Self {
        Self {
            inner: non_null(unsafe { symbol_manager_new(tm.ptr()) }, "SymbolManager"),
        }
    }

    pub(crate) fn ptr(&self) -> *mut cvc5_sys::parser::SymbolManager {
        self.inner
    }

    /// Return whether the logic has been set.
    pub fn is_logic_set(&self) -> bool {
        unsafe { sm_is_logic_set(self.ptr()) }
    }

    /// Get the logic string (e.g. `"QF_LIA"`).
    ///
    /// # Panics
    ///
    /// The underlying C API asserts that the logic has been set.
    pub fn get_logic(&self) -> Result<&str> {
        let p = unsafe { sm_get_logic(self.ptr()) };
        let p = checked(p, "get_logic")?;
        Ok(unsafe { cstr_or_empty(p) })
    }

    /// Get the sorts declared via `declare-sort` commands.
    ///
    /// These are the sorts printed as part of a `get-model` response.
    pub fn get_declared_sorts(&self) -> Vec<Sort> {
        let mut size = 0usize;
        let ptr = unsafe { sm_get_declared_sorts(self.ptr(), &mut size) };
        unsafe { raw_slice(ptr, size) }
            .iter()
            .map(|&p| Sort::from_raw(p))
            .collect()
    }

    /// Get the terms declared via `declare-fun` and `declare-const` commands.
    ///
    /// These are the terms printed in a `get-model` response.
    pub fn get_declared_terms(&self) -> Vec<Term> {
        let mut size = 0usize;
        let ptr = unsafe { sm_get_declared_terms(self.ptr(), &mut size) };
        unsafe { raw_slice(ptr, size) }
            .iter()
            .map(|&p| Term::from_raw(p))
            .collect()
    }

    /// Get terms that have been given names via the `:named` attribute.
    ///
    /// Returns a list of `(term, name)` pairs.
    pub fn get_named_terms(&self) -> Vec<(Term, String)> {
        let mut size = 0usize;
        let mut terms: *mut cvc5_sys::Term = std::ptr::null_mut();
        let mut names: *mut *const std::os::raw::c_char = std::ptr::null_mut();
        unsafe { sm_get_named_terms(self.ptr(), &mut size, &mut terms, &mut names) };
        // Out-params are only written on success; on failure they keep the
        // null/zero initializers above.
        let terms = unsafe { raw_slice(terms, size) };
        let names = unsafe { raw_slice(names, size) };
        terms
            .iter()
            .zip(names)
            .map(|(&t, &n)| {
                let t = Term::from_raw(t);
                let n = unsafe { cstr_to_string(n, "SymbolManager::get_named_terms name") };
                (t, n)
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Command
// ---------------------------------------------------------------------------

/// A parsed command (e.g. `assert`, `check-sat`, `declare-const`).
///
/// Commands are produced by [`InputParser::next_command`] and can be executed
/// on a solver and symbol manager via [`Command::invoke`].
///
/// A command owns a reference to the [`InputParser`] that produced it (commands
/// live in `Cvc5InputParser::d_alloc_cmds`), so it stays valid even after that
/// parser has been dropped.
pub struct Command {
    pub(crate) inner: cvc5_sys::parser::Command,
}

impl Clone for Command {
    fn clone(&self) -> Self {
        Self::from_raw(unsafe { cmd_copy(self.inner) })
    }
}

impl Drop for Command {
    fn drop(&mut self) {
        unsafe { cmd_release(self.inner) }
    }
}

impl Command {
    pub(crate) fn from_raw(raw: cvc5_sys::parser::Command) -> Self {
        Self {
            inner: non_null(raw, "Command"),
        }
    }

    /// Return `true` if this is a null (empty) command.
    ///
    /// Always `false`: [`InputParser::next_command`] reports the end of input
    /// as `Ok(None)` rather than handing back a null command, so a `Command`
    /// value never wraps a null pointer. Retained for API compatibility.
    pub fn is_null(&self) -> bool {
        self.inner.is_null()
    }

    /// Execute this command on the given solver and symbol manager.
    ///
    /// Returns any output produced by the command (e.g. `sat`, `unsat`,
    /// model output, etc.).
    /// Both the solver and the symbol manager are mutated: a command may declare
    /// symbols, assert formulas or run a check. While an [`InputParser`] is alive
    /// it holds both exclusively, so use [`InputParser::invoke`] inside a parse
    /// loop and this method only once the parser has been dropped.
    pub fn invoke(&self, solver: &mut Solver, sm: &mut SymbolManager) -> Result<String> {
        let p = unsafe { cmd_invoke(self.inner, solver.inner, sm.ptr()) };
        let p = checked(p, "invoke")?;
        Ok(unsafe { cstr_or_empty(p) }.to_owned())
    }

    /// Get the name of this command (e.g. `"assert"`, `"check-sat"`).
    pub fn name(&self) -> &str {
        unsafe { cstr_or_empty(cmd_get_name(self.inner)) }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { cmd_to_string(self.inner) };
        let cs = unsafe { std::ffi::CStr::from_ptr(s) };
        write!(f, "{}", cs.to_string_lossy())
    }
}

impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Command({self})")
    }
}

// ---------------------------------------------------------------------------
// InputParser
// ---------------------------------------------------------------------------

/// Parses SMT-LIB or SyGuS input into commands and terms.
///
/// After construction, configure an input source with one of:
/// - [`set_file_input`](InputParser::set_file_input) — read from a file
/// - [`set_str_input`](InputParser::set_str_input) — read from a string
/// - [`set_inc_str_input`](InputParser::set_inc_str_input) +
///   [`append_inc_str_input`](InputParser::append_inc_str_input) — incremental string feeding
///
/// Then call [`next_command`](InputParser::next_command) or
/// [`next_term`](InputParser::next_term) in a loop until
/// [`done`](InputParser::done) returns `true`.
/// # Lifetime
///
/// `'s` is the borrow of the [`Solver`] and [`SymbolManager`] this parser reads
/// and writes; both must outlive it. This is the one lifetime the crate still
/// needs: `Cvc5InputParser` stores a bare `Cvc5*` and `Cvc5SymbolManager*` and
/// takes no reference on either, so unlike every other wrapper it cannot keep
/// its owners alive. A parser therefore cannot escape the scope of the solver it
/// was built from:
///
/// ```compile_fail,E0597
/// use cvc5::{InputParser, Solver, SymbolManager, TermManager};
/// let mut tm = TermManager::new();
/// let mut sm = SymbolManager::new(&tm);
/// let parser = {
///     let mut solver = Solver::new(&tm);
///     InputParser::new(&mut solver, &mut sm)
/// };
/// let _ = parser.done();
/// ```
///
/// The borrow is **exclusive**, because parsing mutates both: each command may
/// declare symbols, assert formulas or run a check. Reaching around the parser
/// to touch the solver it is driving is therefore rejected:
///
/// ```compile_fail,E0499
/// use cvc5::{InputParser, Solver, SymbolManager, TermManager};
/// let mut tm = TermManager::new();
/// let mut sm = SymbolManager::new(&tm);
/// let mut solver = Solver::new(&tm);
/// let mut parser = InputParser::new(&mut solver, &mut sm);
/// solver.set_logic("QF_LIA").unwrap();   // solver is exclusively borrowed
/// let _ = parser.done();
/// ```
pub struct InputParser<'s> {
    inner: *mut cvc5_sys::parser::InputParser,
    solver: &'s mut Solver,
    sm: &'s mut SymbolManager,
}

impl<'s> InputParser<'s> {
    /// Create a new input parser.
    ///
    /// Both the solver and the symbol manager are borrowed, so both must outlive
    /// the parser. If both have their logic set, the logics must be the same.
    pub fn new(solver: &'s mut Solver, sm: &'s mut SymbolManager) -> Self {
        Self {
            inner: non_null(unsafe { parser_new(solver.inner, sm.ptr()) }, "InputParser"),
            solver,
            sm,
        }
    }

    /// Execute a command against the solver and symbol manager this parser
    /// borrows.
    ///
    /// The parser holds both exclusively, so this is how a command is invoked
    /// from inside a parse loop; [`Command::invoke`] is for after the parser is
    /// gone.
    pub fn invoke(&mut self, cmd: &Command) -> Result<String> {
        cmd.invoke(self.solver, self.sm)
    }

    /// Return the solver associated with this parser.
    pub fn get_solver(&self) -> &Solver {
        self.solver
    }

    /// Mutable access to the solver this parser borrows.
    pub fn get_solver_mut(&mut self) -> &mut Solver {
        self.solver
    }

    /// Get the symbol manager associated with this parser.
    pub fn get_symbol_manager(&self) -> &SymbolManager {
        self.sm
    }

    /// Mutable access to the symbol manager this parser borrows.
    pub fn get_symbol_manager_mut(&mut self) -> &mut SymbolManager {
        self.sm
    }

    /// Configure a file as the input source.
    ///
    /// - `lang` — the input language (e.g.
    ///   [`SmtLib26`](cvc5_sys::InputLanguage::SmtLib26)).
    /// - `filename` — path to the file.
    pub fn set_file_input(&mut self, lang: InputLanguage, filename: &str) -> Result<()> {
        let f = CString::new(filename).unwrap();
        unsafe { parser_set_file_input(self.inner, lang, f.as_ptr()) };
        checked((), "set_file_input")
    }

    /// Configure a concrete string as the input source.
    ///
    /// - `lang` — the input language.
    /// - `input` — the input string to parse.
    /// - `name` — a name used in error messages (e.g. `"<stdin>"`).
    pub fn set_str_input(&mut self, lang: InputLanguage, input: &str, name: &str) -> Result<()> {
        let i = CString::new(input).unwrap();
        let n = CString::new(name).unwrap();
        unsafe { parser_set_str_input(self.inner, lang, i.as_ptr(), n.as_ptr()) };
        checked((), "set_str_input")
    }

    /// Configure incremental string input mode.
    ///
    /// After calling this, feed input with
    /// [`append_inc_str_input`](InputParser::append_inc_str_input).
    ///
    /// - `lang` — the input language.
    /// - `name` — a name used in error messages.
    pub fn set_inc_str_input(&mut self, lang: InputLanguage, name: &str) -> Result<()> {
        let n = CString::new(name).unwrap();
        unsafe { parser_set_inc_str_input(self.inner, lang, n.as_ptr()) };
        checked((), "set_inc_str_input")
    }

    /// Append a string to the incremental input stream.
    ///
    /// Must be called after [`set_inc_str_input`](InputParser::set_inc_str_input).
    pub fn append_inc_str_input(&mut self, input: &str) -> Result<()> {
        let i = CString::new(input).unwrap();
        unsafe { parser_append_inc_str_input(self.inner, i.as_ptr()) };
        checked((), "append_inc_str_input")
    }

    /// Parse and return the next command.
    ///
    /// Returns:
    /// - `Ok(Some(cmd))` — a successfully parsed command.
    /// - `Ok(None)` — no more commands (end of input).
    /// - `Err(e)` — a parse error, with cvc5's message.
    ///
    /// If no logic has been set, the first command that requires one will
    /// initialize the logic to `"ALL"`.
    pub fn next_command(&mut self) -> Result<Option<Command>> {
        let mut error_msg: *const std::os::raw::c_char = std::ptr::null();
        let cmd = unsafe { parser_next_command(self.inner, &mut error_msg) };
        // The parser reports failures through this out-param rather than the
        // thread-local error state, so check it instead of calling `checked`.
        if !error_msg.is_null() {
            return Err(Error::from_message(unsafe {
                cstr_to_string(error_msg, "next_command error")
            }));
        }
        if cmd.is_null() {
            Ok(None)
        } else {
            Ok(Some(Command::from_raw(cmd)))
        }
    }

    /// Parse and return the next term.
    ///
    /// Returns:
    /// - `Ok(Some(term))` — a successfully parsed term.
    /// - `Ok(None)` — no more terms (end of input).
    /// - `Err(e)` — a parse error, with cvc5's message.
    ///
    /// The logic must be set before calling this method.
    pub fn next_term(&mut self) -> Result<Option<Term>> {
        let mut error_msg: *const std::os::raw::c_char = std::ptr::null();
        let term = unsafe { parser_next_term(self.inner, &mut error_msg) };
        // As `next_command`: the error arrives via the out-param.
        if !error_msg.is_null() {
            return Err(Error::from_message(unsafe {
                cstr_to_string(error_msg, "next_term error")
            }));
        }
        if term.is_null() {
            Ok(None)
        } else {
            Ok(Some(Term::from_raw(term)))
        }
    }

    /// Return `true` if the parser has finished reading all input.
    pub fn done(&self) -> bool {
        unsafe { parser_done(self.inner) }
    }
}

impl Drop for InputParser<'_> {
    fn drop(&mut self) {
        unsafe { parser_delete(self.inner) }
    }
}
