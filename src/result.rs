use crate::ffi::non_null;
use cvc5_sys::*;
use std::fmt;

/// The result of a satisfiability check.
///
/// # Ownership
///
/// A result is allocated by the [`Solver`](crate::Solver) rather than the
/// [`TermManager`](crate::TermManager), and the solver is not reference counted.
/// This wrapper therefore takes a reference of its own, which makes `~Cvc5`
/// *detach* the result instead of freeing it — so a `SatResult` outlives the
/// solver that produced it:
///
/// ```
/// use cvc5::{Solver, TermManager};
/// let mut tm = TermManager::new();
/// let r = {
///     let mut solver = Solver::new(&tm);
///     solver.check_sat().unwrap()
/// };
/// assert!(r.is_sat());
/// ```
pub struct SatResult {
    pub(crate) inner: cvc5_sys::Result,
}

impl Clone for SatResult {
    fn clone(&self) -> Self {
        Self::from_raw(self.inner)
    }
}

impl Drop for SatResult {
    fn drop(&mut self) {
        unsafe { result_release(self.inner) }
    }
}

impl SatResult {
    /// Wrap a raw result, taking a reference of our own.
    ///
    /// The single reference on an exported result belongs to the
    /// [`Solver`](crate::Solver) that made it, and `~Cvc5` drops that reference.
    /// Holding one ourselves is what lets this outlive the solver: the C object
    /// detaches (its solver back-pointer is nulled) instead of being freed.
    pub(crate) fn from_raw(raw: cvc5_sys::Result) -> Self {
        let inner = non_null(raw, "SatResult");
        unsafe { result_copy(inner) };
        Self { inner }
    }

    /// Return `true` if this is a null (uninitialized) result.
    pub fn is_null(&self) -> bool {
        unsafe { result_is_null(self.inner) }
    }

    /// Create a copy of this result (increments the internal reference count).
    pub fn copy(&self) -> SatResult {
        SatResult::from_raw(self.inner)
    }

    /// Check disequality with another result.
    pub fn is_disequal(&self, other: &SatResult) -> bool {
        unsafe { result_is_disequal(self.inner, other.inner) }
    }

    /// Return `true` if the query was satisfiable.
    pub fn is_sat(&self) -> bool {
        unsafe { result_is_sat(self.inner) }
    }

    /// Return `true` if the query was unsatisfiable.
    pub fn is_unsat(&self) -> bool {
        unsafe { result_is_unsat(self.inner) }
    }

    /// Return `true` if the result is unknown.
    pub fn is_unknown(&self) -> bool {
        unsafe { result_is_unknown(self.inner) }
    }

    /// Get the explanation for an unknown result.
    pub fn unknown_explanation(&self) -> cvc5_sys::UnknownExplanation {
        unsafe { result_get_unknown_explanation(self.inner) }
    }
}

impl fmt::Display for SatResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { result_to_string(self.inner) };
        let cs = unsafe { std::ffi::CStr::from_ptr(s) };
        write!(f, "{}", cs.to_string_lossy())
    }
}

impl fmt::Debug for SatResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SatResult({self})")
    }
}

impl PartialEq for SatResult {
    fn eq(&self, other: &Self) -> bool {
        unsafe { result_is_equal(self.inner, other.inner) }
    }
}

impl Eq for SatResult {}

impl std::hash::Hash for SatResult {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { result_hash(self.inner) }.hash(state);
    }
}
