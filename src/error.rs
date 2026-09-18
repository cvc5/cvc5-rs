//! Access to cvc5's thread-local error state.
//!
//! cvc5's C API does not return error codes. When a call fails it records the
//! reason in thread-local state and returns a default value — `NULL`, `false`,
//! or `0` — then carries on. These bindings turn the memory-unsafe half of that
//! into panics (see the `from_raw` constructors), but the *reason* a call failed
//! is only available from the error state, which this module exposes.
//!
//! ```no_run
//! use cvc5::{Solver, TermManager};
//!
//! let mut tm = TermManager::new();
//! let mut solver = Solver::new(&tm);
//! solver.set_logic("QF_LIA").unwrap();
//!
//! let names = solver.get_option_names();
//! if let Some(msg) = cvc5::last_error() {
//!     eprintln!("cvc5 call failed: {msg}");
//! }
//! # let _ = names;
//! ```
//!
//! # Lifetime of the state
//!
//! The state is **per-thread** and reflects only the *most recent* cvc5 call on
//! that thread: every guarded C API function clears it on entry. Read it
//! immediately after the call you care about — any intervening cvc5 call, on the
//! same thread, overwrites it.
//!
//! [`has_error`] and [`last_error`] are themselves queries and do not disturb
//! the state, so calling them is safe after a failure.

use std::ffi::CStr;
use std::fmt;

/// A failed cvc5 call, carrying the message cvc5 recorded for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    message: String,
}

impl Error {
    /// The message cvc5 recorded for the failure.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Build an [`Error`] from a message cvc5 reported out-of-band.
    ///
    /// The parser is the one part of the C API that does not use the
    /// thread-local error state: `cvc5_parser_next_command` and
    /// `cvc5_parser_next_term` write the parse error to a `const char**`
    /// out-param instead.
    pub(crate) fn from_message(message: String) -> Self {
        Self { message }
    }

    /// Build an [`Error`] from the current thread-local error state, falling
    /// back to a generic message if cvc5 recorded none.
    pub(crate) fn from_state(what: &str) -> Self {
        Self {
            message: last_error().unwrap_or_else(|| {
                format!("cvc5 reported no message for the failed `{what}` call")
            }),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

/// Result of a fallible cvc5 call.
///
/// Shadows [`std::result::Result`] within this crate's public API; the
/// satisfiability verdict of a `check-sat` query is [`crate::SatResult`].
pub type Result<T> = std::result::Result<T, Error>;

/// Returns `true` if the most recent cvc5 call on this thread failed.
///
/// Cheaper than [`last_error`], which copies the message.
///
/// The state is per-thread and reflects only the *most recent* cvc5 call on
/// that thread: every cvc5 call clears it on entry. Read it immediately after
/// the call you care about. This function is itself a query and does not
/// disturb the state.
pub fn has_error() -> bool {
    unsafe { cvc5_sys::has_error() }
}

/// The reason the most recent cvc5 call on this thread failed, or `None` if it
/// succeeded.
///
/// The state is per-thread and reflects only the *most recent* cvc5 call on
/// that thread: every cvc5 call clears it on entry. Read it immediately after
/// the call you care about. This function is itself a query and does not
/// disturb the state.
pub fn last_error() -> Option<String> {
    if !has_error() {
        return None;
    }
    let ptr = unsafe { cvc5_sys::get_error_message() };
    if ptr.is_null() {
        return Some(String::new());
    }
    Some(
        unsafe { CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned(),
    )
}

/// Clear this thread's error state, so [`has_error`] returns `false` and
/// [`last_error`] returns `None` until the next failure.
///
/// Rarely needed: every guarded cvc5 call already clears the state on entry.
pub fn clear_error() {
    unsafe { cvc5_sys::reset_error() }
}
