use crate::ffi::non_null;
use cvc5_sys::*;
use std::fmt;

/// The result of a synthesis query (SyGuS).
pub struct SynthResult {
    pub(crate) inner: cvc5_sys::SynthResult,
}

impl Clone for SynthResult {
    fn clone(&self) -> Self {
        Self::from_raw(self.inner)
    }
}

impl Drop for SynthResult {
    fn drop(&mut self) {
        unsafe { synth_result_release(self.inner) }
    }
}

impl SynthResult {
    /// Wrap a raw synthesis result, taking a reference of our own.
    ///
    /// The single reference on an exported synthesis result belongs to the
    /// [`Solver`](crate::Solver) that made it, and `~Cvc5` drops that reference.
    /// Holding one ourselves is what lets this outlive the solver: the C object
    /// detaches (its solver back-pointer is nulled) instead of being freed.
    pub(crate) fn from_raw(raw: cvc5_sys::SynthResult) -> Self {
        let inner = non_null(raw, "SynthResult");
        unsafe { synth_result_copy(inner) };
        Self { inner }
    }

    /// Return `true` if this is a null (uninitialized) synthesis result.
    pub fn is_null(&self) -> bool {
        unsafe { synth_result_is_null(self.inner) }
    }

    /// Create a copy of this synthesis result (increments the internal reference count).
    pub fn copy(&self) -> SynthResult {
        SynthResult::from_raw(self.inner)
    }

    /// Check disequality with another synthesis result.
    pub fn is_disequal(&self, other: &SynthResult) -> bool {
        unsafe { synth_result_is_disequal(self.inner, other.inner) }
    }

    /// Return `true` if a solution was found.
    pub fn has_solution(&self) -> bool {
        unsafe { synth_result_has_solution(self.inner) }
    }

    /// Return `true` if it was determined that no solution exists.
    pub fn has_no_solution(&self) -> bool {
        unsafe { synth_result_has_no_solution(self.inner) }
    }

    /// Return `true` if the synthesis result is unknown.
    pub fn is_unknown(&self) -> bool {
        unsafe { synth_result_is_unknown(self.inner) }
    }
}

impl fmt::Display for SynthResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { synth_result_to_string(self.inner) };
        let cs = unsafe { std::ffi::CStr::from_ptr(s) };
        write!(f, "{}", cs.to_string_lossy())
    }
}

impl PartialEq for SynthResult {
    fn eq(&self, other: &Self) -> bool {
        unsafe { synth_result_is_equal(self.inner, other.inner) }
    }
}

impl Eq for SynthResult {}

impl std::hash::Hash for SynthResult {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { synth_result_hash(self.inner) }.hash(state);
    }
}

impl fmt::Debug for SynthResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SynthResult({self})")
    }
}
