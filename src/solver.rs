use cvc5_sys::*;
use std::ffi::CString;
use std::marker::PhantomData;

use crate::{Result, Sort, Term, TermManager};

/// A cvc5 solver instance.
///
/// The solver borrows a [`TermManager`] for the duration of its lifetime.
pub struct Solver<'tm> {
    inner: *mut Cvc5,
    _tm: PhantomData<&'tm TermManager>,
}

impl<'tm> Solver<'tm> {
    /// Create a new solver using the given term manager.
    pub fn new(tm: &'tm TermManager) -> Self {
        Self {
            inner: unsafe { cvc5_new(tm.inner) },
            _tm: PhantomData,
        }
    }

    // ── Configuration ──────────────────────────────────────────────

    /// Set the logic for this solver.
    pub fn set_logic(&mut self, logic: &str) {
        let c = CString::new(logic).unwrap();
        unsafe { cvc5_set_logic(self.inner, c.as_ptr()) }
    }

    /// Set a solver option.
    pub fn set_option(&mut self, option: &str, value: &str) {
        let o = CString::new(option).unwrap();
        let v = CString::new(value).unwrap();
        unsafe { cvc5_set_option(self.inner, o.as_ptr(), v.as_ptr()) }
    }

    /// Get the value of a solver option.
    pub fn get_option(&self, option: &str) -> String {
        let o = CString::new(option).unwrap();
        unsafe {
            let s = cvc5_get_option(self.inner, o.as_ptr());
            std::ffi::CStr::from_ptr(s)
                .to_string_lossy()
                .into_owned()
        }
    }

    /// Check if a logic has been set.
    pub fn is_logic_set(&self) -> bool {
        unsafe { cvc5_is_logic_set(self.inner) }
    }

    // ── Assertions & checking ──────────────────────────────────────

    /// Assert a formula.
    pub fn assert_formula(&mut self, term: Term) {
        unsafe { cvc5_assert_formula(self.inner, term.inner) }
    }

    /// Check satisfiability.
    pub fn check_sat(&mut self) -> Result {
        Result::from_raw(unsafe { cvc5_check_sat(self.inner) })
    }

    /// Check satisfiability under assumptions.
    pub fn check_sat_assuming(&mut self, assumptions: &[Term]) -> Result {
        let raw: Vec<Cvc5Term> = assumptions.iter().map(|t| t.inner).collect();
        Result::from_raw(unsafe {
            cvc5_check_sat_assuming(self.inner, raw.len(), raw.as_ptr())
        })
    }

    // ── Model queries ──────────────────────────────────────────────

    /// Get the value of a term in the current model.
    pub fn get_value(&self, term: Term) -> Term {
        Term::from_raw(unsafe { cvc5_get_value(self.inner, term.inner) })
    }

    /// Get the values of multiple terms in the current model.
    pub fn get_values(&self, terms: &[Term]) -> Vec<Term> {
        let raw: Vec<Cvc5Term> = terms.iter().map(|t| t.inner).collect();
        let mut rsize = 0usize;
        let ptr =
            unsafe { cvc5_get_values(self.inner, raw.len(), raw.as_ptr(), &mut rsize) };
        (0..rsize)
            .map(|i| Term::from_raw(unsafe { *ptr.add(i) }))
            .collect()
    }

    // ── Declarations ───────────────────────────────────────────────

    /// Declare an uninterpreted function.
    pub fn declare_fun(
        &mut self,
        name: &str,
        domain: &[Sort],
        codomain: Sort,
    ) -> Term {
        let c = CString::new(name).unwrap();
        let raw: Vec<Cvc5Sort> = domain.iter().map(|s| s.inner).collect();
        Term::from_raw(unsafe {
            cvc5_declare_fun(
                self.inner,
                c.as_ptr(),
                raw.len(),
                raw.as_ptr(),
                codomain.inner,
                true,
            )
        })
    }

    /// Declare an uninterpreted sort.
    pub fn declare_sort(&mut self, name: &str, arity: u32) -> Sort {
        let c = CString::new(name).unwrap();
        Sort::from_raw(unsafe { cvc5_declare_sort(self.inner, c.as_ptr(), arity, true) })
    }

    // ── Scope management ───────────────────────────────────────────

    /// Push assertion scope(s).
    pub fn push(&mut self, n: u32) {
        unsafe { cvc5_push(self.inner, n) }
    }

    /// Pop assertion scope(s).
    pub fn pop(&mut self, n: u32) {
        unsafe { cvc5_pop(self.inner, n) }
    }

    /// Reset all assertions.
    pub fn reset_assertions(&mut self) {
        unsafe { cvc5_reset_assertions(self.inner) }
    }

    // ── Unsat core ─────────────────────────────────────────────────

    /// Get the unsat core (requires `produce-unsat-cores` option).
    pub fn get_unsat_core(&self) -> Vec<Term> {
        let mut size = 0usize;
        let ptr = unsafe { cvc5_get_unsat_core(self.inner, &mut size) };
        (0..size)
            .map(|i| Term::from_raw(unsafe { *ptr.add(i) }))
            .collect()
    }

    // ── Simplification ─────────────────────────────────────────────

    /// Simplify a term.
    pub fn simplify(&self, term: Term) -> Term {
        Term::from_raw(unsafe { cvc5_simplify(self.inner, term.inner, true) })
    }

    // ── Info ────────────────────────────────────────────────────────

    /// Get solver version string.
    pub fn version(&self) -> String {
        unsafe {
            let s = cvc5_get_version(self.inner);
            std::ffi::CStr::from_ptr(s)
                .to_string_lossy()
                .into_owned()
        }
    }
}

impl Drop for Solver<'_> {
    fn drop(&mut self) {
        unsafe { cvc5_delete(self.inner) }
    }
}

// SAFETY: A Solver is not Sync but can be sent between threads.
unsafe impl Send for Solver<'_> {}
