use cvc5_sys::*;
use std::ffi::CString;

use crate::{DatatypeConstructorDecl, DatatypeDecl, Op, Sort, Term};

/// Manages creation of sorts, terms, and operators.
pub struct TermManager {
    pub(crate) inner: *mut Cvc5TermManager,
}

impl TermManager {
    pub fn new() -> Self {
        Self { inner: unsafe { cvc5_term_manager_new() } }
    }

    /// Release all managed references.
    pub fn release(&mut self) {
        unsafe { cvc5_term_manager_release(self.inner) }
    }

    // ── Sort creation ──────────────────────────────────────────────

    pub fn boolean_sort(&self) -> Sort { Sort::from_raw(unsafe { cvc5_get_boolean_sort(self.inner) }) }
    pub fn integer_sort(&self) -> Sort { Sort::from_raw(unsafe { cvc5_get_integer_sort(self.inner) }) }
    pub fn real_sort(&self) -> Sort { Sort::from_raw(unsafe { cvc5_get_real_sort(self.inner) }) }
    pub fn string_sort(&self) -> Sort { Sort::from_raw(unsafe { cvc5_get_string_sort(self.inner) }) }
    pub fn regexp_sort(&self) -> Sort { Sort::from_raw(unsafe { cvc5_get_regexp_sort(self.inner) }) }
    pub fn rm_sort(&self) -> Sort { Sort::from_raw(unsafe { cvc5_get_rm_sort(self.inner) }) }

    pub fn mk_array_sort(&self, index: Sort, elem: Sort) -> Sort {
        Sort::from_raw(unsafe { cvc5_mk_array_sort(self.inner, index.inner, elem.inner) })
    }

    pub fn mk_bv_sort(&self, size: u32) -> Sort {
        Sort::from_raw(unsafe { cvc5_mk_bv_sort(self.inner, size) })
    }

    pub fn mk_fp_sort(&self, exp: u32, sig: u32) -> Sort {
        Sort::from_raw(unsafe { cvc5_mk_fp_sort(self.inner, exp, sig) })
    }

    pub fn mk_ff_sort(&self, size: &str, base: u32) -> Sort {
        let c = CString::new(size).unwrap();
        Sort::from_raw(unsafe { cvc5_mk_ff_sort(self.inner, c.as_ptr(), base) })
    }

    pub fn mk_dt_sort(&self, decl: &DatatypeDecl) -> Sort {
        Sort::from_raw(unsafe { cvc5_mk_dt_sort(self.inner, decl.inner) })
    }

    pub fn mk_dt_sorts(&self, decls: &[DatatypeDecl]) -> Vec<Sort> {
        let raw: Vec<Cvc5DatatypeDecl> = decls.iter().map(|d| d.inner).collect();
        let ptr = unsafe { cvc5_mk_dt_sorts(self.inner, raw.len(), raw.as_ptr()) };
        (0..decls.len()).map(|i| Sort::from_raw(unsafe { *ptr.add(i) })).collect()
    }

    pub fn mk_fun_sort(&self, domain: &[Sort], codomain: Sort) -> Sort {
        let raw: Vec<Cvc5Sort> = domain.iter().map(|s| s.inner).collect();
        Sort::from_raw(unsafe { cvc5_mk_fun_sort(self.inner, raw.len(), raw.as_ptr(), codomain.inner) })
    }

    pub fn mk_param_sort(&self, symbol: &str) -> Sort {
        let c = CString::new(symbol).unwrap();
        Sort::from_raw(unsafe { cvc5_mk_param_sort(self.inner, c.as_ptr()) })
    }

    pub fn mk_predicate_sort(&self, sorts: &[Sort]) -> Sort {
        let raw: Vec<Cvc5Sort> = sorts.iter().map(|s| s.inner).collect();
        Sort::from_raw(unsafe { cvc5_mk_predicate_sort(self.inner, raw.len(), raw.as_ptr()) })
    }

    pub fn mk_record_sort(&self, names: &[&str], sorts: &[Sort]) -> Sort {
        let cnames: Vec<CString> = names.iter().map(|n| CString::new(*n).unwrap()).collect();
        let mut ptrs: Vec<*const std::ffi::c_char> = cnames.iter().map(|c| c.as_ptr()).collect();
        let raw: Vec<Cvc5Sort> = sorts.iter().map(|s| s.inner).collect();
        Sort::from_raw(unsafe { cvc5_mk_record_sort(self.inner, raw.len(), ptrs.as_mut_ptr(), raw.as_ptr()) })
    }

    pub fn mk_set_sort(&self, elem: Sort) -> Sort {
        Sort::from_raw(unsafe { cvc5_mk_set_sort(self.inner, elem.inner) })
    }

    pub fn mk_bag_sort(&self, elem: Sort) -> Sort {
        Sort::from_raw(unsafe { cvc5_mk_bag_sort(self.inner, elem.inner) })
    }

    pub fn mk_sequence_sort(&self, elem: Sort) -> Sort {
        Sort::from_raw(unsafe { cvc5_mk_sequence_sort(self.inner, elem.inner) })
    }

    pub fn mk_abstract_sort(&self, k: Cvc5SortKind) -> Sort {
        Sort::from_raw(unsafe { cvc5_mk_abstract_sort(self.inner, k) })
    }

    pub fn mk_uninterpreted_sort(&self, name: &str) -> Sort {
        let c = CString::new(name).unwrap();
        Sort::from_raw(unsafe { cvc5_mk_uninterpreted_sort(self.inner, c.as_ptr()) })
    }

    pub fn mk_unresolved_dt_sort(&self, symbol: &str, arity: usize) -> Sort {
        let c = CString::new(symbol).unwrap();
        Sort::from_raw(unsafe { cvc5_mk_unresolved_dt_sort(self.inner, c.as_ptr(), arity) })
    }

    pub fn mk_uninterpreted_sort_constructor_sort(&self, arity: usize, symbol: &str) -> Sort {
        let c = CString::new(symbol).unwrap();
        Sort::from_raw(unsafe { cvc5_mk_uninterpreted_sort_constructor_sort(self.inner, arity, c.as_ptr()) })
    }

    pub fn mk_tuple_sort(&self, sorts: &[Sort]) -> Sort {
        let raw: Vec<Cvc5Sort> = sorts.iter().map(|s| s.inner).collect();
        Sort::from_raw(unsafe { cvc5_mk_tuple_sort(self.inner, raw.len(), raw.as_ptr()) })
    }

    pub fn mk_nullable_sort(&self, sort: Sort) -> Sort {
        Sort::from_raw(unsafe { cvc5_mk_nullable_sort(self.inner, sort.inner) })
    }

    // ── Operator creation ──────────────────────────────────────────

    pub fn mk_op(&self, kind: Cvc5Kind, indices: &[u32]) -> Op {
        Op::from_raw(unsafe { cvc5_mk_op(self.inner, kind, indices.len(), indices.as_ptr()) })
    }

    pub fn mk_op_from_str(&self, kind: Cvc5Kind, arg: &str) -> Op {
        let c = CString::new(arg).unwrap();
        Op::from_raw(unsafe { cvc5_mk_op_from_str(self.inner, kind, c.as_ptr()) })
    }

    // ── Term creation ──────────────────────────────────────────────

    pub fn mk_term(&self, kind: Cvc5Kind, children: &[Term]) -> Term {
        let raw: Vec<Cvc5Term> = children.iter().map(|t| t.inner).collect();
        Term::from_raw(unsafe { cvc5_mk_term(self.inner, kind, raw.len(), raw.as_ptr()) })
    }

    pub fn mk_term_from_op(&self, op: Op, children: &[Term]) -> Term {
        let raw: Vec<Cvc5Term> = children.iter().map(|t| t.inner).collect();
        Term::from_raw(unsafe { cvc5_mk_term_from_op(self.inner, op.inner, raw.len(), raw.as_ptr()) })
    }

    pub fn mk_tuple(&self, terms: &[Term]) -> Term {
        let raw: Vec<Cvc5Term> = terms.iter().map(|t| t.inner).collect();
        Term::from_raw(unsafe { cvc5_mk_tuple(self.inner, raw.len(), raw.as_ptr()) })
    }

    pub fn mk_nullable_some(&self, term: Term) -> Term {
        Term::from_raw(unsafe { cvc5_mk_nullable_some(self.inner, term.inner) })
    }

    pub fn mk_nullable_val(&self, term: Term) -> Term {
        Term::from_raw(unsafe { cvc5_mk_nullable_val(self.inner, term.inner) })
    }

    pub fn mk_nullable_is_null(&self, term: Term) -> Term {
        Term::from_raw(unsafe { cvc5_mk_nullable_is_null(self.inner, term.inner) })
    }

    pub fn mk_nullable_is_some(&self, term: Term) -> Term {
        Term::from_raw(unsafe { cvc5_mk_nullable_is_some(self.inner, term.inner) })
    }

    pub fn mk_nullable_null(&self, sort: Sort) -> Term {
        Term::from_raw(unsafe { cvc5_mk_nullable_null(self.inner, sort.inner) })
    }

    pub fn mk_nullable_lift(&self, kind: Cvc5Kind, args: &[Term]) -> Term {
        let raw: Vec<Cvc5Term> = args.iter().map(|t| t.inner).collect();
        Term::from_raw(unsafe { cvc5_mk_nullable_lift(self.inner, kind, raw.len(), raw.as_ptr()) })
    }

    pub fn mk_skolem(&self, id: Cvc5SkolemId, indices: &[Term]) -> Term {
        let raw: Vec<Cvc5Term> = indices.iter().map(|t| t.inner).collect();
        Term::from_raw(unsafe { cvc5_mk_skolem(self.inner, id, raw.len(), raw.as_ptr()) })
    }

    pub fn get_num_idxs_for_skolem_id(&self, id: Cvc5SkolemId) -> usize {
        unsafe { cvc5_get_num_idxs_for_skolem_id(self.inner, id) }
    }

    // ── Constants ──────────────────────────────────────────────────

    pub fn mk_true(&self) -> Term { Term::from_raw(unsafe { cvc5_mk_true(self.inner) }) }
    pub fn mk_false(&self) -> Term { Term::from_raw(unsafe { cvc5_mk_false(self.inner) }) }
    pub fn mk_boolean(&self, val: bool) -> Term { Term::from_raw(unsafe { cvc5_mk_boolean(self.inner, val) }) }
    pub fn mk_pi(&self) -> Term { Term::from_raw(unsafe { cvc5_mk_pi(self.inner) }) }

    pub fn mk_integer(&self, val: i64) -> Term {
        Term::from_raw(unsafe { cvc5_mk_integer_int64(self.inner, val) })
    }

    pub fn mk_integer_from_str(&self, s: &str) -> Term {
        let c = CString::new(s).unwrap();
        Term::from_raw(unsafe { cvc5_mk_integer(self.inner, c.as_ptr()) })
    }

    pub fn mk_real(&self, val: i64) -> Term {
        Term::from_raw(unsafe { cvc5_mk_real_int64(self.inner, val) })
    }

    pub fn mk_real_from_str(&self, s: &str) -> Term {
        let c = CString::new(s).unwrap();
        Term::from_raw(unsafe { cvc5_mk_real(self.inner, c.as_ptr()) })
    }

    pub fn mk_real_from_rational(&self, num: i64, den: i64) -> Term {
        Term::from_raw(unsafe { cvc5_mk_real_num_den(self.inner, num, den) })
    }

    pub fn mk_regexp_all(&self) -> Term { Term::from_raw(unsafe { cvc5_mk_regexp_all(self.inner) }) }
    pub fn mk_regexp_allchar(&self) -> Term { Term::from_raw(unsafe { cvc5_mk_regexp_allchar(self.inner) }) }
    pub fn mk_regexp_none(&self) -> Term { Term::from_raw(unsafe { cvc5_mk_regexp_none(self.inner) }) }

    pub fn mk_empty_set(&self, sort: Sort) -> Term {
        Term::from_raw(unsafe { cvc5_mk_empty_set(self.inner, sort.inner) })
    }

    pub fn mk_empty_bag(&self, sort: Sort) -> Term {
        Term::from_raw(unsafe { cvc5_mk_empty_bag(self.inner, sort.inner) })
    }

    pub fn mk_sep_emp(&self) -> Term { Term::from_raw(unsafe { cvc5_mk_sep_emp(self.inner) }) }

    pub fn mk_sep_nil(&self, sort: Sort) -> Term {
        Term::from_raw(unsafe { cvc5_mk_sep_nil(self.inner, sort.inner) })
    }

    pub fn mk_string(&self, s: &str, use_esc_seq: bool) -> Term {
        let c = CString::new(s).unwrap();
        Term::from_raw(unsafe { cvc5_mk_string(self.inner, c.as_ptr(), use_esc_seq) })
    }

    pub fn mk_empty_sequence(&self, sort: Sort) -> Term {
        Term::from_raw(unsafe { cvc5_mk_empty_sequence(self.inner, sort.inner) })
    }

    pub fn mk_universe_set(&self, sort: Sort) -> Term {
        Term::from_raw(unsafe { cvc5_mk_universe_set(self.inner, sort.inner) })
    }

    pub fn mk_bv(&self, size: u32, val: u64) -> Term {
        Term::from_raw(unsafe { cvc5_mk_bv_uint64(self.inner, size, val) })
    }

    pub fn mk_bv_from_str(&self, size: u32, s: &str, base: u32) -> Term {
        let c = CString::new(s).unwrap();
        Term::from_raw(unsafe { cvc5_mk_bv(self.inner, size, c.as_ptr(), base) })
    }

    pub fn mk_ff_elem(&self, value: &str, sort: Sort, base: u32) -> Term {
        let c = CString::new(value).unwrap();
        Term::from_raw(unsafe { cvc5_mk_ff_elem(self.inner, c.as_ptr(), sort.inner, base) })
    }

    pub fn mk_const_array(&self, sort: Sort, val: Term) -> Term {
        Term::from_raw(unsafe { cvc5_mk_const_array(self.inner, sort.inner, val.inner) })
    }

    pub fn mk_fp_pos_inf(&self, exp: u32, sig: u32) -> Term {
        Term::from_raw(unsafe { cvc5_mk_fp_pos_inf(self.inner, exp, sig) })
    }

    pub fn mk_fp_neg_inf(&self, exp: u32, sig: u32) -> Term {
        Term::from_raw(unsafe { cvc5_mk_fp_neg_inf(self.inner, exp, sig) })
    }

    pub fn mk_fp_nan(&self, exp: u32, sig: u32) -> Term {
        Term::from_raw(unsafe { cvc5_mk_fp_nan(self.inner, exp, sig) })
    }

    pub fn mk_fp_pos_zero(&self, exp: u32, sig: u32) -> Term {
        Term::from_raw(unsafe { cvc5_mk_fp_pos_zero(self.inner, exp, sig) })
    }

    pub fn mk_fp_neg_zero(&self, exp: u32, sig: u32) -> Term {
        Term::from_raw(unsafe { cvc5_mk_fp_neg_zero(self.inner, exp, sig) })
    }

    pub fn mk_rm(&self, rm: Cvc5RoundingMode) -> Term {
        Term::from_raw(unsafe { cvc5_mk_rm(self.inner, rm) })
    }

    pub fn mk_fp(&self, exp: u32, sig: u32, val: Term) -> Term {
        Term::from_raw(unsafe { cvc5_mk_fp(self.inner, exp, sig, val.inner) })
    }

    pub fn mk_fp_from_ieee(&self, sign: Term, exp: Term, sig: Term) -> Term {
        Term::from_raw(unsafe { cvc5_mk_fp_from_ieee(self.inner, sign.inner, exp.inner, sig.inner) })
    }

    pub fn mk_cardinality_constraint(&self, sort: Sort, upper_bound: u32) -> Term {
        Term::from_raw(unsafe { cvc5_mk_cardinality_constraint(self.inner, sort.inner, upper_bound) })
    }

    // ── Variables ──────────────────────────────────────────────────

    pub fn mk_const(&self, sort: Sort, name: &str) -> Term {
        let c = CString::new(name).unwrap();
        Term::from_raw(unsafe { cvc5_mk_const(self.inner, sort.inner, c.as_ptr()) })
    }

    pub fn mk_var(&self, sort: Sort, name: &str) -> Term {
        let c = CString::new(name).unwrap();
        Term::from_raw(unsafe { cvc5_mk_var(self.inner, sort.inner, c.as_ptr()) })
    }

    // ── Datatype declarations ──────────────────────────────────────

    pub fn mk_dt_cons_decl(&self, name: &str) -> DatatypeConstructorDecl {
        let c = CString::new(name).unwrap();
        DatatypeConstructorDecl::from_raw(unsafe { cvc5_mk_dt_cons_decl(self.inner, c.as_ptr()) })
    }

    pub fn mk_dt_decl(&self, name: &str, is_codt: bool) -> DatatypeDecl {
        let c = CString::new(name).unwrap();
        DatatypeDecl::from_raw(unsafe { cvc5_mk_dt_decl(self.inner, c.as_ptr(), is_codt) })
    }

    pub fn mk_dt_decl_with_params(&self, name: &str, params: &[Sort], is_codt: bool) -> DatatypeDecl {
        let c = CString::new(name).unwrap();
        let raw: Vec<Cvc5Sort> = params.iter().map(|s| s.inner).collect();
        DatatypeDecl::from_raw(unsafe { cvc5_mk_dt_decl_with_params(self.inner, c.as_ptr(), raw.len(), raw.as_ptr(), is_codt) })
    }

    pub fn mk_string_from_char32(&self, s: &[char32_t]) -> Term {
        Term::from_raw(unsafe { cvc5_mk_string_from_char32(self.inner, s.as_ptr()) })
    }

    pub fn print_stats_safe(&self, fd: i32) {
        unsafe { cvc5_term_manager_print_stats_safe(self.inner, fd) }
    }
}

impl Default for TermManager {
    fn default() -> Self { Self::new() }
}

impl Drop for TermManager {
    fn drop(&mut self) {
        unsafe { cvc5_term_manager_delete(self.inner) }
    }
}

unsafe impl Send for TermManager {}
