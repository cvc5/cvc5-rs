use cvc5_sys::*;
use std::fmt;

/// The result of a satisfiability check.
pub struct Result {
    pub(crate) inner: Cvc5Result,
}

impl Clone for Result {
    fn clone(&self) -> Self {
        Self {
            inner: unsafe { cvc5_result_copy(self.inner) },
        }
    }
}

impl Drop for Result {
    fn drop(&mut self) {
        unsafe { cvc5_result_release(self.inner) }
    }
}

impl Result {
    pub(crate) fn from_raw(raw: Cvc5Result) -> Self {
        Self { inner: raw }
    }

    /// Return `true` if this is a null (uninitialized) result.
    pub fn is_null(&self) -> bool {
        unsafe { cvc5_result_is_null(self.inner) }
    }

    /// Create a copy of this result (increments the internal reference count).
    pub fn copy(&self) -> Result {
        Result::from_raw(unsafe { cvc5_result_copy(self.inner) })
    }

    /// Check disequality with another result.
    pub fn is_disequal(&self, other: &Result) -> bool {
        unsafe { cvc5_result_is_disequal(self.inner, other.inner) }
    }

    /// Return `true` if the query was satisfiable.
    pub fn is_sat(&self) -> bool {
        unsafe { cvc5_result_is_sat(self.inner) }
    }

    /// Return `true` if the query was unsatisfiable.
    pub fn is_unsat(&self) -> bool {
        unsafe { cvc5_result_is_unsat(self.inner) }
    }

    /// Return `true` if the result is unknown.
    pub fn is_unknown(&self) -> bool {
        unsafe { cvc5_result_is_unknown(self.inner) }
    }

    /// Get the explanation for an unknown result.
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

impl PartialEq for Result {
    fn eq(&self, other: &Self) -> bool {
        unsafe { cvc5_result_is_equal(self.inner, other.inner) }
    }
}

impl Eq for Result {}

impl std::hash::Hash for Result {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { cvc5_result_hash(self.inner) }.hash(state);
    }
}
