use cvc5_sys::*;
use std::fmt;

/// The result of a satisfiability check.
pub struct Result {
    pub(crate) inner: Cvc5Result,
}

impl Result {
    pub(crate) fn from_raw(raw: Cvc5Result) -> Self {
        Self { inner: raw }
    }

    /// True if the result is satisfiable.
    pub fn is_sat(&self) -> bool {
        unsafe { cvc5_result_is_sat(self.inner) }
    }

    /// True if the result is unsatisfiable.
    pub fn is_unsat(&self) -> bool {
        unsafe { cvc5_result_is_unsat(self.inner) }
    }

    /// True if the result is unknown.
    pub fn is_unknown(&self) -> bool {
        unsafe { cvc5_result_is_unknown(self.inner) }
    }

    /// Get the unknown explanation (only valid if `is_unknown()` is true).
    pub fn unknown_explanation(&self) -> Cvc5UnknownExplanation {
        unsafe { cvc5_result_get_unknown_explanation(self.inner) }
    }
}

impl fmt::Display for Result {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { cvc5_result_to_string(self.inner) };
        let cs = unsafe { std::ffi::CStr::from_ptr(s) };
        write!(f, "{}", cs.to_string_lossy())
    }
}

impl fmt::Debug for Result {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Result({self})")
    }
}
