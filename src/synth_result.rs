use cvc5_sys::*;
use std::fmt;

/// The result of a synthesis query (SyGuS).
pub struct SynthResult {
    pub(crate) inner: Cvc5SynthResult,
}

impl Clone for SynthResult {
    fn clone(&self) -> Self {
        Self {
            inner: unsafe { cvc5_synth_result_copy(self.inner) },
        }
    }
}

impl Drop for SynthResult {
    fn drop(&mut self) {
        unsafe { cvc5_synth_result_release(self.inner) }
    }
}

impl SynthResult {
    pub(crate) fn from_raw(raw: Cvc5SynthResult) -> Self {
        Self { inner: raw }
    }

    /// Return `true` if this is a null (uninitialized) synthesis result.
    pub fn is_null(&self) -> bool {
        unsafe { cvc5_synth_result_is_null(self.inner) }
    }

    /// Create a copy of this synthesis result (increments the internal reference count).
    pub fn copy(&self) -> SynthResult {
        SynthResult::from_raw(unsafe { cvc5_synth_result_copy(self.inner) })
    }

    /// Check disequality with another synthesis result.
    pub fn is_disequal(&self, other: &SynthResult) -> bool {
        unsafe { cvc5_synth_result_is_disequal(self.inner, other.inner) }
    }

    /// Return `true` if a solution was found.
    pub fn has_solution(&self) -> bool {
        unsafe { cvc5_synth_result_has_solution(self.inner) }
    }

    /// Return `true` if it was determined that no solution exists.
    pub fn has_no_solution(&self) -> bool {
        unsafe { cvc5_synth_result_has_no_solution(self.inner) }
    }

    /// Return `true` if the synthesis result is unknown.
    pub fn is_unknown(&self) -> bool {
        unsafe { cvc5_synth_result_is_unknown(self.inner) }
    }
}

impl fmt::Display for SynthResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { cvc5_synth_result_to_string(self.inner) };
        let cs = unsafe { std::ffi::CStr::from_ptr(s) };
        write!(f, "{}", cs.to_string_lossy())
    }
}

impl PartialEq for SynthResult {
    fn eq(&self, other: &Self) -> bool {
        unsafe { cvc5_synth_result_is_equal(self.inner, other.inner) }
    }
}

impl Eq for SynthResult {}

impl std::hash::Hash for SynthResult {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { cvc5_synth_result_hash(self.inner) }.hash(state);
    }
}

impl fmt::Debug for SynthResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SynthResult({self})")
    }
}
