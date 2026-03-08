use cvc5_sys::*;
use std::fmt;

use crate::{Op, Sort};

/// A cvc5 term (expression).
pub struct Term {
    pub(crate) inner: Cvc5Term,
}

impl Clone for Term {
    fn clone(&self) -> Self { Self { inner: unsafe { cvc5_term_copy(self.inner) } } }
}

impl Drop for Term {
    fn drop(&mut self) { unsafe { cvc5_term_release(self.inner) } }
}

impl Term {
    pub(crate) fn from_raw(raw: Cvc5Term) -> Self {
        Self { inner: raw }
    }

    pub fn kind(&self) -> Cvc5Kind { unsafe { cvc5_term_get_kind(self.inner) } }

    /// Increment the internal reference count. Returns a copy of this term.
    pub fn copy(&self) -> Term { Term::from_raw(unsafe { cvc5_term_copy(self.inner) }) }
    /// Decrement the internal reference count.
    pub fn release(self) { unsafe { cvc5_term_release(self.inner) } }
    /// Check disequality (equivalent to `!=`).
    pub fn is_disequal(&self, other: &Term) -> bool { unsafe { cvc5_term_is_disequal(self.inner, other.inner) } }

    pub fn sort(&self) -> Sort { Sort::from_raw(unsafe { cvc5_term_get_sort(self.inner) }) }
    pub fn id(&self) -> u64 { unsafe { cvc5_term_get_id(self.inner) } }
    pub fn num_children(&self) -> usize { unsafe { cvc5_term_get_num_children(self.inner) } }
    pub fn child(&self, index: usize) -> Term { Term::from_raw(unsafe { cvc5_term_get_child(self.inner, index) }) }
    pub fn has_symbol(&self) -> bool { unsafe { cvc5_term_has_symbol(self.inner) } }

    pub fn symbol(&self) -> &str {
        unsafe {
            let s = cvc5_term_get_symbol(self.inner);
            std::ffi::CStr::from_ptr(s).to_str().unwrap_or("")
        }
    }

    pub fn has_op(&self) -> bool { unsafe { cvc5_term_has_op(self.inner) } }
    pub fn op(&self) -> Op { Op::from_raw(unsafe { cvc5_term_get_op(self.inner) }) }

    pub fn substitute_term(&self, t: Term, replacement: Term) -> Term {
        Term::from_raw(unsafe { cvc5_term_substitute_term(self.inner, t.inner, replacement.inner) })
    }

    pub fn substitute_terms(&self, terms: &[Term], replacements: &[Term]) -> Term {
        let t: Vec<Cvc5Term> = terms.iter().map(|t| t.inner).collect();
        let r: Vec<Cvc5Term> = replacements.iter().map(|t| t.inner).collect();
        Term::from_raw(unsafe { cvc5_term_substitute_terms(self.inner, t.len(), t.as_ptr(), r.as_ptr()) })
    }

    pub fn real_or_integer_value_sign(&self) -> i32 { unsafe { cvc5_term_get_real_or_integer_value_sign(self.inner) } }

    pub fn is_boolean_value(&self) -> bool { unsafe { cvc5_term_is_boolean_value(self.inner) } }
    pub fn boolean_value(&self) -> bool { unsafe { cvc5_term_get_boolean_value(self.inner) } }

    pub fn is_int32_value(&self) -> bool { unsafe { cvc5_term_is_int32_value(self.inner) } }
    pub fn int32_value(&self) -> i32 { unsafe { cvc5_term_get_int32_value(self.inner) } }

    pub fn is_uint32_value(&self) -> bool { unsafe { cvc5_term_is_uint32_value(self.inner) } }
    pub fn uint32_value(&self) -> u32 { unsafe { cvc5_term_get_uint32_value(self.inner) } }

    pub fn is_int64_value(&self) -> bool { unsafe { cvc5_term_is_int64_value(self.inner) } }
    pub fn int64_value(&self) -> i64 { unsafe { cvc5_term_get_int64_value(self.inner) } }

    pub fn is_uint64_value(&self) -> bool { unsafe { cvc5_term_is_uint64_value(self.inner) } }
    pub fn uint64_value(&self) -> u64 { unsafe { cvc5_term_get_uint64_value(self.inner) } }

    pub fn is_integer_value(&self) -> bool { unsafe { cvc5_term_is_integer_value(self.inner) } }
    pub fn integer_value(&self) -> String {
        unsafe { std::ffi::CStr::from_ptr(cvc5_term_get_integer_value(self.inner)).to_string_lossy().into_owned() }
    }

    pub fn is_string_value(&self) -> bool { unsafe { cvc5_term_is_string_value(self.inner) } }
    pub fn string_value(&self) -> Vec<char32_t> {
        let ptr = unsafe { cvc5_term_get_string_value(self.inner) };
        let mut v = Vec::new();
        let mut i = 0;
        loop {
            let c = unsafe { *ptr.add(i) };
            if c == 0 { break; }
            v.push(c as char32_t);
            i += 1;
        }
        v
    }
    pub fn u32string_value(&self) -> Vec<char32_t> {
        let ptr = unsafe { cvc5_term_get_u32string_value(self.inner) };
        let mut v = Vec::new();
        let mut i = 0;
        loop {
            let c = unsafe { *ptr.add(i) };
            if c == 0 { break; }
            v.push(c);
            i += 1;
        }
        v
    }

    pub fn is_real32_value(&self) -> bool { unsafe { cvc5_term_is_real32_value(self.inner) } }
    pub fn real32_value(&self) -> (i32, u32) {
        let (mut num, mut den) = (0i32, 0u32);
        unsafe { cvc5_term_get_real32_value(self.inner, &mut num, &mut den) };
        (num, den)
    }

    pub fn is_real64_value(&self) -> bool { unsafe { cvc5_term_is_real64_value(self.inner) } }
    pub fn real64_value(&self) -> (i64, u64) {
        let (mut num, mut den) = (0i64, 0u64);
        unsafe { cvc5_term_get_real64_value(self.inner, &mut num, &mut den) };
        (num, den)
    }

    pub fn is_real_value(&self) -> bool { unsafe { cvc5_term_is_real_value(self.inner) } }
    pub fn real_value(&self) -> String {
        unsafe { std::ffi::CStr::from_ptr(cvc5_term_get_real_value(self.inner)).to_string_lossy().into_owned() }
    }

    pub fn is_const_array(&self) -> bool { unsafe { cvc5_term_is_const_array(self.inner) } }
    pub fn const_array_base(&self) -> Term { Term::from_raw(unsafe { cvc5_term_get_const_array_base(self.inner) }) }

    pub fn is_bv_value(&self) -> bool { unsafe { cvc5_term_is_bv_value(self.inner) } }
    pub fn bv_value(&self, base: u32) -> String {
        unsafe { std::ffi::CStr::from_ptr(cvc5_term_get_bv_value(self.inner, base)).to_string_lossy().into_owned() }
    }

    pub fn is_ff_value(&self) -> bool { unsafe { cvc5_term_is_ff_value(self.inner) } }
    pub fn ff_value(&self) -> String {
        unsafe { std::ffi::CStr::from_ptr(cvc5_term_get_ff_value(self.inner)).to_string_lossy().into_owned() }
    }

    pub fn is_uninterpreted_sort_value(&self) -> bool { unsafe { cvc5_term_is_uninterpreted_sort_value(self.inner) } }
    pub fn uninterpreted_sort_value(&self) -> String {
        unsafe { std::ffi::CStr::from_ptr(cvc5_term_get_uninterpreted_sort_value(self.inner)).to_string_lossy().into_owned() }
    }

    pub fn is_tuple_value(&self) -> bool { unsafe { cvc5_term_is_tuple_value(self.inner) } }
    pub fn tuple_value(&self) -> Vec<Term> {
        let mut size = 0usize;
        let ptr = unsafe { cvc5_term_get_tuple_value(self.inner, &mut size) };
        (0..size).map(|i| Term::from_raw(unsafe { *ptr.add(i) })).collect()
    }

    pub fn is_rm_value(&self) -> bool { unsafe { cvc5_term_is_rm_value(self.inner) } }
    pub fn rm_value(&self) -> Cvc5RoundingMode { unsafe { cvc5_term_get_rm_value(self.inner) } }

    pub fn is_fp_pos_zero(&self) -> bool { unsafe { cvc5_term_is_fp_pos_zero(self.inner) } }
    pub fn is_fp_neg_zero(&self) -> bool { unsafe { cvc5_term_is_fp_neg_zero(self.inner) } }
    pub fn is_fp_pos_inf(&self) -> bool { unsafe { cvc5_term_is_fp_pos_inf(self.inner) } }
    pub fn is_fp_neg_inf(&self) -> bool { unsafe { cvc5_term_is_fp_neg_inf(self.inner) } }
    pub fn is_fp_nan(&self) -> bool { unsafe { cvc5_term_is_fp_nan(self.inner) } }
    pub fn is_fp_value(&self) -> bool { unsafe { cvc5_term_is_fp_value(self.inner) } }
    pub fn fp_value(&self) -> (u32, u32, Term) {
        let (mut ew, mut sw) = (0u32, 0u32);
        let mut val = std::ptr::null_mut();
        unsafe { cvc5_term_get_fp_value(self.inner, &mut ew, &mut sw, &mut val) };
        (ew, sw, Term::from_raw(val))
    }

    pub fn is_set_value(&self) -> bool { unsafe { cvc5_term_is_set_value(self.inner) } }
    pub fn set_value(&self) -> Vec<Term> {
        let mut size = 0usize;
        let ptr = unsafe { cvc5_term_get_set_value(self.inner, &mut size) };
        (0..size).map(|i| Term::from_raw(unsafe { *ptr.add(i) })).collect()
    }

    pub fn is_sequence_value(&self) -> bool { unsafe { cvc5_term_is_sequence_value(self.inner) } }
    pub fn sequence_value(&self) -> Vec<Term> {
        let mut size = 0usize;
        let ptr = unsafe { cvc5_term_get_sequence_value(self.inner, &mut size) };
        (0..size).map(|i| Term::from_raw(unsafe { *ptr.add(i) })).collect()
    }

    pub fn is_cardinality_constraint(&self) -> bool { unsafe { cvc5_term_is_cardinality_constraint(self.inner) } }
    pub fn cardinality_constraint(&self) -> (Sort, u32) {
        let mut sort = std::ptr::null_mut();
        let mut upper = 0u32;
        unsafe { cvc5_term_get_cardinality_constraint(self.inner, &mut sort, &mut upper) };
        (Sort::from_raw(sort), upper)
    }

    pub fn is_real_algebraic_number(&self) -> bool { unsafe { cvc5_term_is_real_algebraic_number(self.inner) } }
    pub fn real_algebraic_number_defining_polynomial(&self, v: Term) -> Term {
        Term::from_raw(unsafe { cvc5_term_get_real_algebraic_number_defining_polynomial(self.inner, v.inner) })
    }
    pub fn real_algebraic_number_lower_bound(&self) -> Term {
        Term::from_raw(unsafe { cvc5_term_get_real_algebraic_number_lower_bound(self.inner) })
    }
    pub fn real_algebraic_number_upper_bound(&self) -> Term {
        Term::from_raw(unsafe { cvc5_term_get_real_algebraic_number_upper_bound(self.inner) })
    }

    pub fn is_skolem(&self) -> bool { unsafe { cvc5_term_is_skolem(self.inner) } }
    pub fn skolem_id(&self) -> Cvc5SkolemId { unsafe { cvc5_term_get_skolem_id(self.inner) } }
    pub fn skolem_indices(&self) -> Vec<Term> {
        let mut size = 0usize;
        let ptr = unsafe { cvc5_term_get_skolem_indices(self.inner, &mut size) };
        (0..size).map(|i| Term::from_raw(unsafe { *ptr.add(i) })).collect()
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { cvc5_term_to_string(self.inner) };
        let cs = unsafe { std::ffi::CStr::from_ptr(s) };
        write!(f, "{}", cs.to_string_lossy())
    }
}

impl fmt::Debug for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Term({self})")
    }
}

impl PartialEq for Term {
    fn eq(&self, other: &Self) -> bool {
        unsafe { cvc5_term_is_equal(self.inner, other.inner) }
    }
}

impl Eq for Term {}

impl PartialOrd for Term {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Term {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let c = unsafe { cvc5_term_compare(self.inner, other.inner) };
        c.cmp(&0)
    }
}

impl std::hash::Hash for Term {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { cvc5_term_hash(self.inner) }.hash(state);
    }
}
