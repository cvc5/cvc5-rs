//! Safe wrappers for the cvc5 parser API.
//!
//! This module is only available when the `parser` feature is enabled.
//!
//! # Example
//!
//! ```rust
//! use cvc5::{TermManager, Solver, InputParser, SymbolManager, InputLanguage};
//!
//! let tm = TermManager::new();
//! let solver = Solver::new(&tm);
//! let sm = SymbolManager::new(&tm);
//!
//! let mut parser = InputParser::new(&solver, &sm);
//! parser.set_str_input(
//!     InputLanguage::SmtLib26,
//!     "(set-logic QF_LIA)(declare-const x Int)(assert (> x 0))(check-sat)",
//!     "example",
//! );
//!
//! while !parser.done() {
//!     // SAFETY: each command is used before the next is parsed.
//!     match unsafe { parser.next_command() } {
//!         Ok(Some(cmd)) => { cmd.invoke(parser.get_solver(), &sm); }
//!         Ok(None) => break,
//!         Err(e) => panic!("parse error: {e}"),
//!     }
//! }
//! ```

use cvc5_sys::InputLanguage;
use cvc5_sys::parser::*;
use std::ffi::CString;
use std::fmt;

use crate::{Solver, Sort, Term, TermManager};

/// Manages symbols for the parser.
///
/// Internally tracks a symbol table and meta-information from SMT-LIB inputs
/// (named assertions, declared functions/sorts, etc.).
///
/// Borrows the [`TermManager`] it was created from, and can be shared with an
/// [`InputParser`] by reference so that parsed commands update the same table.
pub struct SymbolManager<'tm> {
    inner: *mut cvc5_sys::parser::SymbolManager,
    tm: &'tm TermManager,
}

impl Drop for SymbolManager<'_> {
    fn drop(&mut self) {
        unsafe { symbol_manager_delete(self.inner) };
    }
}

impl<'tm> SymbolManager<'tm> {
    /// Create a new symbol manager associated with the given term manager.
    pub fn new(tm: &'tm TermManager) -> Self {
        Self {
            inner: crate::ffi::non_null(unsafe { symbol_manager_new(tm.ptr()) }, "SymbolManager"),
            tm,
        }
    }

    pub(crate) fn ptr(&self) -> *mut cvc5_sys::parser::SymbolManager {
        self.inner
    }

    /// Return the underlying term manager.
    pub fn term_manager(&self) -> &'tm TermManager {
        self.tm
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
    pub fn get_logic(&self) -> &str {
        unsafe { crate::ffi::cstr_or_empty(sm_get_logic(self.ptr())) }
    }

    /// Get the sorts declared via `declare-sort` commands.
    ///
    /// These are the sorts printed as part of a `get-model` response.
    pub fn get_declared_sorts(&self) -> Vec<Sort<'tm>> {
        let mut size = 0usize;
        let ptr = unsafe { sm_get_declared_sorts(self.ptr(), &mut size) };
        unsafe { crate::ffi::raw_slice(ptr, size) }
            .iter()
            .map(|&p| Sort::from_raw(p))
            .collect()
    }

    /// Get the terms declared via `declare-fun` and `declare-const` commands.
    ///
    /// These are the terms printed in a `get-model` response.
    pub fn get_declared_terms(&self) -> Vec<Term<'tm>> {
        let mut size = 0usize;
        let ptr = unsafe { sm_get_declared_terms(self.ptr(), &mut size) };
        unsafe { crate::ffi::raw_slice(ptr, size) }
            .iter()
            .map(|&p| Term::from_raw(p))
            .collect()
    }

    /// Get terms that have been given names via the `:named` attribute.
    ///
    /// Returns a list of `(term, name)` pairs.
    pub fn get_named_terms(&self) -> Vec<(Term<'tm>, String)> {
        let mut size = 0usize;
        let mut terms: *mut cvc5_sys::Term = std::ptr::null_mut();
        let mut names: *mut *const std::os::raw::c_char = std::ptr::null_mut();
        unsafe { sm_get_named_terms(self.ptr(), &mut size, &mut terms, &mut names) };
        let terms = unsafe { crate::ffi::raw_slice(terms, size) };
        let names = unsafe { crate::ffi::raw_slice(names, size) };
        terms
            .iter()
            .zip(names)
            .map(|(&t, &n)| {
                let t = Term::from_raw(t);
                let n =
                    unsafe { crate::ffi::cstr_to_string(n, "SymbolManager::get_named_terms name") };
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
/// The lifetime parameter is bound to the [`InputParser`] that produced this
/// command: commands live in `Cvc5InputParser::d_alloc_cmds`, which
/// `cvc5_parser_delete` frees.
pub struct Command<'p> {
    pub(crate) inner: cvc5_sys::parser::Command,
    pub(crate) _phantom: std::marker::PhantomData<&'p ()>,
}

impl<'p> Command<'p> {
    pub(crate) fn from_raw(raw: cvc5_sys::parser::Command) -> Self {
        Self {
            inner: crate::ffi::non_null(raw, "Command"),
            _phantom: std::marker::PhantomData,
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
    pub fn invoke(&self, solver: &Solver<'_>, sm: &SymbolManager) -> String {
        unsafe {
            std::ffi::CStr::from_ptr(cmd_invoke(self.inner, solver.inner, sm.ptr()))
                .to_string_lossy()
                .into_owned()
        }
    }

    /// Get the name of this command (e.g. `"assert"`, `"check-sat"`).
    pub fn name(&self) -> &str {
        unsafe { crate::ffi::cstr_or_empty(cmd_get_name(self.inner)) }
    }
}

impl fmt::Display for Command<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { cmd_to_string(self.inner) };
        let cs = unsafe { std::ffi::CStr::from_ptr(s) };
        write!(f, "{}", cs.to_string_lossy())
    }
}

impl fmt::Debug for Command<'_> {
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
/// # Lifetimes
///
/// `'s` is the borrow of the [`Solver`] and [`SymbolManager`] this parser reads
/// and writes; both must outlive it. `'tm` is the term manager they belong to.
/// So a parser cannot escape the scope of the solver it was built from:
///
/// ```compile_fail,E0597
/// use cvc5::{InputParser, Solver, SymbolManager, TermManager};
/// let tm = TermManager::new();
/// let sm = SymbolManager::new(&tm);
/// let parser = {
///     let solver = Solver::new(&tm);
///     InputParser::new(&solver, &sm)
/// };
/// let _ = parser.done();
/// ```
pub struct InputParser<'s, 'tm> {
    inner: *mut cvc5_sys::parser::InputParser,
    solver: &'s Solver<'tm>,
    sm: &'s SymbolManager<'tm>,
}

impl<'s, 'tm> InputParser<'s, 'tm> {
    /// Create a new input parser.
    ///
    /// Both the solver and the symbol manager are borrowed, so both must outlive
    /// the parser. If both have their logic set, the logics must be the same.
    pub fn new(solver: &'s Solver<'tm>, sm: &'s SymbolManager<'tm>) -> Self {
        Self {
            inner: crate::ffi::non_null(
                unsafe { parser_new(solver.inner, sm.ptr()) },
                "InputParser",
            ),
            solver,
            sm,
        }
    }

    /// Return the solver associated with this parser.
    pub fn get_solver(&self) -> &'s Solver<'tm> {
        self.solver
    }

    /// Get the symbol manager associated with this parser.
    pub fn get_symbol_manager(&self) -> &'s SymbolManager<'tm> {
        self.sm
    }

    /// Configure a file as the input source.
    ///
    /// - `lang` — the input language (e.g.
    ///   [`SmtLib26`](cvc5_sys::InputLanguage::SmtLib26)).
    /// - `filename` — path to the file.
    pub fn set_file_input(&mut self, lang: InputLanguage, filename: &str) {
        let f = CString::new(filename).unwrap();
        unsafe { parser_set_file_input(self.inner, lang, f.as_ptr()) }
    }

    /// Configure a concrete string as the input source.
    ///
    /// - `lang` — the input language.
    /// - `input` — the input string to parse.
    /// - `name` — a name used in error messages (e.g. `"<stdin>"`).
    pub fn set_str_input(&mut self, lang: InputLanguage, input: &str, name: &str) {
        let i = CString::new(input).unwrap();
        let n = CString::new(name).unwrap();
        unsafe { parser_set_str_input(self.inner, lang, i.as_ptr(), n.as_ptr()) }
    }

    /// Configure incremental string input mode.
    ///
    /// After calling this, feed input with
    /// [`append_inc_str_input`](InputParser::append_inc_str_input).
    ///
    /// - `lang` — the input language.
    /// - `name` — a name used in error messages.
    pub fn set_inc_str_input(&mut self, lang: InputLanguage, name: &str) {
        let n = CString::new(name).unwrap();
        unsafe { parser_set_inc_str_input(self.inner, lang, n.as_ptr()) }
    }

    /// Append a string to the incremental input stream.
    ///
    /// Must be called after [`set_inc_str_input`](InputParser::set_inc_str_input).
    pub fn append_inc_str_input(&mut self, input: &str) {
        let i = CString::new(input).unwrap();
        unsafe { parser_append_inc_str_input(self.inner, i.as_ptr()) }
    }

    /// Parse and return the next command.
    ///
    /// Returns:
    /// - `Ok(Some(cmd))` — a successfully parsed command.
    /// - `Ok(None)` — no more commands (end of input).
    /// - `Err(msg)` — a parse error with the error message.
    ///
    /// If no logic has been set, the first command that requires one will
    /// initialize the logic to `"ALL"`.
    /// # Safety
    ///
    /// The returned [`Command`] points into a `std::vector` owned by the parser,
    /// and the C API hands out a pointer to its last element. Each further call
    /// to `next_command` may grow that vector and invalidate every `Command`
    /// returned earlier — the second call is already enough. Commands therefore
    /// cannot be collected; invoke or inspect each one before parsing the next.
    ///
    /// Tracked upstream as cvc5/cvc5#12898; this becomes safe once those arenas
    /// use `std::deque`.
    pub unsafe fn next_command<'p>(&'p self) -> std::result::Result<Option<Command<'p>>, String> {
        let mut error_msg: *const std::os::raw::c_char = std::ptr::null();
        let cmd = unsafe { parser_next_command(self.inner, &mut error_msg) };
        if !error_msg.is_null() {
            let msg = unsafe { std::ffi::CStr::from_ptr(error_msg) }
                .to_string_lossy()
                .into_owned();
            return Err(msg);
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
    /// - `Err(msg)` — a parse error with the error message.
    ///
    /// The logic must be set before calling this method.
    pub fn next_term(&mut self) -> std::result::Result<Option<Term<'tm>>, String> {
        let mut error_msg: *const std::os::raw::c_char = std::ptr::null();
        let term = unsafe { parser_next_term(self.inner, &mut error_msg) };
        if !error_msg.is_null() {
            let msg = unsafe { std::ffi::CStr::from_ptr(error_msg) }
                .to_string_lossy()
                .into_owned();
            return Err(msg);
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

impl Drop for InputParser<'_, '_> {
    fn drop(&mut self) {
        unsafe { parser_delete(self.inner) }
    }
}
