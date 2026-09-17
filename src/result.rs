use crate::ffi::non_null;
use cvc5_sys::*;
use std::fmt;
use std::marker::PhantomData;

/// The result of a satisfiability check.
///
/// # Lifetime
///
/// The lifetime parameter is bound to the [`Solver`](crate::Solver) that produced
/// this result, not to the [`TermManager`](crate::TermManager): results live in
/// the solver's own allocation table (`Cvc5::d_alloc_results`), which
/// `cvc5_delete` frees. So this does not compile:
///
/// ```compile_fail,E0597
/// use cvc5::{Solver, TermManager};
/// let tm = TermManager::new();
/// let r = {
///     let solver = Solver::new(&tm);
///     solver.check_sat().unwrap()
/// };
/// println!("{}", r.is_sat());
/// ```
pub struct SatResult<'tm> {
    pub(crate) inner: cvc5_sys::Result,
    pub(crate) _phantom: PhantomData<&'tm ()>,
}

impl Clone for SatResult<'_> {
    fn clone(&self) -> Self {
        Self::from_raw(unsafe { result_copy(self.inner) })
    }
}

impl Drop for SatResult<'_> {
    fn drop(&mut self) {
        unsafe { result_release(self.inner) }
    }
}

impl<'tm> SatResult<'tm> {
    pub(crate) fn from_raw(raw: cvc5_sys::Result) -> Self {
        Self {
            inner: non_null(raw, "SatResult"),
            _phantom: PhantomData,
        }
    }

    /// Return `true` if this is a null (uninitialized) result.
    pub fn is_null(&self) -> bool {
        unsafe { result_is_null(self.inner) }
    }

    /// Create a copy of this result (increments the internal reference count).
    pub fn copy(&self) -> SatResult<'tm> {
        SatResult::from_raw(unsafe { result_copy(self.inner) })
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

impl fmt::Display for SatResult<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { result_to_string(self.inner) };
        let cs = unsafe { std::ffi::CStr::from_ptr(s) };
        write!(f, "{}", cs.to_string_lossy())
    }
}

impl fmt::Debug for SatResult<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SatResult({self})")
    }
}

impl PartialEq for SatResult<'_> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { result_is_equal(self.inner, other.inner) }
    }
}

impl Eq for SatResult<'_> {}

impl std::hash::Hash for SatResult<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { result_hash(self.inner) }.hash(state);
    }
}
