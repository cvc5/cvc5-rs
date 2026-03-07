use cvc5_sys::*;
use std::ffi::CString;

use crate::{Op, Sort, Term};

/// Manages creation of sorts, terms, and operators.
///
/// A `TermManager` can be shared between multiple [`Solver`](crate::Solver) instances.
pub struct TermManager {
    pub(crate) inner: *mut Cvc5TermManager,
}

impl TermManager {
    /// Create a new term manager.
    pub fn new() -> Self {
        Self {
            inner: unsafe { cvc5_term_manager_new() },
        }
    }

    // ── Sort creation ──────────────────────────────────────────────

    /// Get the Boolean sort.
    pub fn boolean_sort(&self) -> Sort {
        Sort::from_raw(unsafe { cvc5_get_boolean_sort(self.inner) })
    }

    /// Get the Integer sort.
    pub fn integer_sort(&self) -> Sort {
        Sort::from_raw(unsafe { cvc5_get_integer_sort(self.inner) })
    }

    /// Get the Real sort.
    pub fn real_sort(&self) -> Sort {
        Sort::from_raw(unsafe { cvc5_get_real_sort(self.inner) })
    }

    /// Get the String sort.
    pub fn string_sort(&self) -> Sort {
        Sort::from_raw(unsafe { cvc5_get_string_sort(self.inner) })
    }

    /// Get the RegExp sort.
    pub fn regexp_sort(&self) -> Sort {
        Sort::from_raw(unsafe { cvc5_get_regexp_sort(self.inner) })
    }

    /// Get the rounding mode sort.
    pub fn rm_sort(&self) -> Sort {
        Sort::from_raw(unsafe { cvc5_get_rm_sort(self.inner) })
    }

    /// Create a bit-vector sort of the given width.
    pub fn mk_bv_sort(&self, size: u32) -> Sort {
        Sort::from_raw(unsafe { cvc5_mk_bv_sort(self.inner, size) })
    }

    /// Create an array sort.
    pub fn mk_array_sort(&self, index: Sort, elem: Sort) -> Sort {
        Sort::from_raw(unsafe { cvc5_mk_array_sort(self.inner, index.inner, elem.inner) })
    }

    /// Create a set sort.
    pub fn mk_set_sort(&self, elem: Sort) -> Sort {
        Sort::from_raw(unsafe { cvc5_mk_set_sort(self.inner, elem.inner) })
    }

    /// Create a sequence sort.
    pub fn mk_sequence_sort(&self, elem: Sort) -> Sort {
        Sort::from_raw(unsafe { cvc5_mk_sequence_sort(self.inner, elem.inner) })
    }

    /// Create a function sort.
    pub fn mk_fun_sort(&self, domain: &[Sort], codomain: Sort) -> Sort {
        let raw: Vec<Cvc5Sort> = domain.iter().map(|s| s.inner).collect();
        Sort::from_raw(unsafe {
            cvc5_mk_fun_sort(self.inner, raw.len(), raw.as_ptr(), codomain.inner)
        })
    }

    /// Create a floating-point sort.
    pub fn mk_fp_sort(&self, exp: u32, sig: u32) -> Sort {
        Sort::from_raw(unsafe { cvc5_mk_fp_sort(self.inner, exp, sig) })
    }

    /// Create a tuple sort.
    pub fn mk_tuple_sort(&self, sorts: &[Sort]) -> Sort {
        let raw: Vec<Cvc5Sort> = sorts.iter().map(|s| s.inner).collect();
        Sort::from_raw(unsafe { cvc5_mk_tuple_sort(self.inner, raw.len(), raw.as_ptr()) })
    }

    /// Create an uninterpreted sort.
    pub fn mk_uninterpreted_sort(&self, name: &str) -> Sort {
        let c = CString::new(name).unwrap();
        Sort::from_raw(unsafe { cvc5_mk_uninterpreted_sort(self.inner, c.as_ptr()) })
    }

    // ── Term creation ──────────────────────────────────────────────

    /// Create a free constant (declared variable) of the given sort.
    pub fn mk_const(&self, sort: Sort, name: &str) -> Term {
        let c = CString::new(name).unwrap();
        Term::from_raw(unsafe { cvc5_mk_const(self.inner, sort.inner, c.as_ptr()) })
    }

    /// Create a bound variable (for use in quantifiers/lambdas).
    pub fn mk_var(&self, sort: Sort, name: &str) -> Term {
        let c = CString::new(name).unwrap();
        Term::from_raw(unsafe { cvc5_mk_var(self.inner, sort.inner, c.as_ptr()) })
    }

    /// Create an n-ary term of the given kind.
    pub fn mk_term(&self, kind: Cvc5Kind, children: &[Term]) -> Term {
        let raw: Vec<Cvc5Term> = children.iter().map(|t| t.inner).collect();
        Term::from_raw(unsafe { cvc5_mk_term(self.inner, kind, raw.len(), raw.as_ptr()) })
    }

    /// Create an n-ary term from an operator.
    pub fn mk_term_from_op(&self, op: Op, children: &[Term]) -> Term {
        let raw: Vec<Cvc5Term> = children.iter().map(|t| t.inner).collect();
        Term::from_raw(unsafe {
            cvc5_mk_term_from_op(self.inner, op.inner, raw.len(), raw.as_ptr())
        })
    }

    /// Create the Boolean `true` constant.
    pub fn mk_true(&self) -> Term {
        Term::from_raw(unsafe { cvc5_mk_true(self.inner) })
    }

    /// Create the Boolean `false` constant.
    pub fn mk_false(&self) -> Term {
        Term::from_raw(unsafe { cvc5_mk_false(self.inner) })
    }

    /// Create a Boolean constant.
    pub fn mk_boolean(&self, val: bool) -> Term {
        Term::from_raw(unsafe { cvc5_mk_boolean(self.inner, val) })
    }

    /// Create an integer constant from an `i64`.
    pub fn mk_integer(&self, val: i64) -> Term {
        Term::from_raw(unsafe { cvc5_mk_integer_int64(self.inner, val) })
    }

    /// Create an integer constant from a string.
    pub fn mk_integer_from_str(&self, s: &str) -> Term {
        let c = CString::new(s).unwrap();
        Term::from_raw(unsafe { cvc5_mk_integer(self.inner, c.as_ptr()) })
    }

    /// Create a real constant from an `i64`.
    pub fn mk_real(&self, val: i64) -> Term {
        Term::from_raw(unsafe { cvc5_mk_real_int64(self.inner, val) })
    }

    /// Create a real constant from numerator and denominator.
    pub fn mk_real_from_rational(&self, num: i64, den: i64) -> Term {
        Term::from_raw(unsafe { cvc5_mk_real_num_den(self.inner, num, den) })
    }

    /// Create a bit-vector constant from a `u64`.
    pub fn mk_bv(&self, size: u32, val: u64) -> Term {
        Term::from_raw(unsafe { cvc5_mk_bv_uint64(self.inner, size, val) })
    }

    /// Create a bit-vector constant from a string in the given base (2, 10, or 16).
    pub fn mk_bv_from_str(&self, size: u32, s: &str, base: u32) -> Term {
        let c = CString::new(s).unwrap();
        Term::from_raw(unsafe { cvc5_mk_bv(self.inner, size, c.as_ptr(), base) })
    }

    /// Create a string constant.
    pub fn mk_string(&self, s: &str, use_esc_seq: bool) -> Term {
        let c = CString::new(s).unwrap();
        Term::from_raw(unsafe { cvc5_mk_string(self.inner, c.as_ptr(), use_esc_seq) })
    }

    /// Create a constant array.
    pub fn mk_const_array(&self, sort: Sort, val: Term) -> Term {
        Term::from_raw(unsafe { cvc5_mk_const_array(self.inner, sort.inner, val.inner) })
    }

    /// Create an empty set of the given sort.
    pub fn mk_empty_set(&self, sort: Sort) -> Term {
        Term::from_raw(unsafe { cvc5_mk_empty_set(self.inner, sort.inner) })
    }

    /// Create a universe set of the given sort.
    pub fn mk_universe_set(&self, sort: Sort) -> Term {
        Term::from_raw(unsafe { cvc5_mk_universe_set(self.inner, sort.inner) })
    }

    /// Create a tuple term.
    pub fn mk_tuple(&self, terms: &[Term]) -> Term {
        let raw: Vec<Cvc5Term> = terms.iter().map(|t| t.inner).collect();
        Term::from_raw(unsafe { cvc5_mk_tuple(self.inner, raw.len(), raw.as_ptr()) })
    }

    // ── Operator creation ──────────────────────────────────────────

    /// Create an indexed operator.
    pub fn mk_op(&self, kind: Cvc5Kind, indices: &[u32]) -> Op {
        Op::from_raw(unsafe { cvc5_mk_op(self.inner, kind, indices.len(), indices.as_ptr()) })
    }
}

impl Default for TermManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TermManager {
    fn drop(&mut self) {
        unsafe { cvc5_term_manager_delete(self.inner) }
    }
}

// SAFETY: The cvc5 TermManager is thread-safe for creation operations.
unsafe impl Send for TermManager {}
