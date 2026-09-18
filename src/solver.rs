use cvc5_sys::*;
use std::ffi::CString;
use std::fmt;

use crate::error::Result;
use crate::ffi::{checked, cstr_or_empty, cstr_to_string, non_null, raw_slice, wrap};
use crate::{
    DatatypeConstructorDecl, Grammar, Proof, SatResult, Sort, Statistics, SynthResult, Term,
    TermManager,
};

const ERROR_NOT_UTF8: &str = "Not UTF-8";

/// The value of an option, decoded by type.
///
/// Owns its strings: cvc5 returns them from `static thread_local` buffers that
/// the next `cvc5_get_option_info` call on the same thread overwrites, so they
/// are copied out immediately.
#[derive(Clone, Debug, PartialEq)]
pub enum OptionInfoKind {
    Void,
    Bool {
        default: bool,
        current: bool,
    },
    String {
        default: String,
        current: String,
    },
    Int64 {
        default: i64,
        current: i64,
        min: Option<i64>,
        max: Option<i64>,
    },
    UInt64 {
        default: u64,
        current: u64,
        min: Option<u64>,
        max: Option<u64>,
    },
    Double {
        default: f64,
        current: f64,
        min: Option<f64>,
        max: Option<f64>,
    },
    Mode {
        default: String,
        current: String,
        modes: Vec<String>,
    },
}

/// Detailed information about a solver option.
///
/// Fully owned, so it carries no lifetime and cannot be invalidated by a later
/// [`Solver::get_option_info`] call.
#[derive(Clone, Debug, PartialEq)]
pub struct OptionInfo {
    name: String,
    aliases: Vec<String>,
    no_supports: Vec<String>,
    is_set_by_user: bool,
    category: OptionCategory,
    kind: OptionInfoKind,
    rendered: String,
}

impl OptionInfo {
    /// Copy everything out of cvc5's thread-local buffers.
    ///
    /// `cvc5_get_option_info` `memset`s its out-struct to zero before filling
    /// it, so on failure every pointer is NULL and every count is zero; the
    /// helpers below map that to `""` and empty vectors.
    fn from_raw(raw: &cvc5_sys::OptionInfo) -> Self {
        use cvc5_sys::OptionInfoKind as K;
        let kind = match raw.kind {
            K::Void => OptionInfoKind::Void,
            K::Bool => OptionInfoKind::Bool {
                default: raw.info_bool.dflt,
                current: raw.info_bool.cur,
            },
            K::Str => OptionInfoKind::String {
                default: unsafe { opt_string(raw.info_str.dflt) },
                current: unsafe { opt_string(raw.info_str.cur) },
            },
            K::Int64 => OptionInfoKind::Int64 {
                default: raw.info_int.dflt,
                current: raw.info_int.cur,
                min: raw.info_int.has_min.then_some(raw.info_int.min),
                max: raw.info_int.has_max.then_some(raw.info_int.max),
            },
            K::Uint64 => OptionInfoKind::UInt64 {
                default: raw.info_uint.dflt,
                current: raw.info_uint.cur,
                min: raw.info_uint.has_min.then_some(raw.info_uint.min),
                max: raw.info_uint.has_max.then_some(raw.info_uint.max),
            },
            K::Double => OptionInfoKind::Double {
                default: raw.info_double.dflt,
                current: raw.info_double.cur,
                min: raw.info_double.has_min.then_some(raw.info_double.min),
                max: raw.info_double.has_max.then_some(raw.info_double.max),
            },
            K::Modes => OptionInfoKind::Mode {
                default: unsafe { opt_string(raw.info_mode.dflt) },
                current: unsafe { opt_string(raw.info_mode.cur) },
                modes: unsafe { raw_slice(raw.info_mode.modes, raw.info_mode.num_modes) }
                    .iter()
                    .map(|&p| unsafe { opt_string(p) })
                    .collect(),
            },
        };
        Self {
            name: unsafe { opt_string(raw.name) },
            aliases: unsafe { raw_slice(raw.aliases, raw.num_aliases) }
                .iter()
                .map(|&p| unsafe { opt_string(p) })
                .collect(),
            no_supports: unsafe { raw_slice(raw.no_supports, raw.num_no_supports) }
                .iter()
                .map(|&p| unsafe { opt_string(p) })
                .collect(),
            is_set_by_user: raw.is_set_by_user,
            category: raw.category,
            kind,
            rendered: unsafe { opt_string(option_info_to_string(raw)) },
        }
    }

    /// The option's value, decoded by type.
    pub fn kind(&self) -> &OptionInfoKind {
        &self.kind
    }
    /// The option's category.
    pub fn category(&self) -> OptionCategory {
        self.category
    }
    /// The option's primary name.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Whether the option was explicitly set by the user.
    pub fn is_set_by_user(&self) -> bool {
        self.is_set_by_user
    }
    /// Alternative names for this option.
    pub fn aliases(&self) -> &[String] {
        &self.aliases
    }
    /// Names this option is *not* known by (cvc5 reports these for diagnostics).
    pub fn no_supports(&self) -> &[String] {
        &self.no_supports
    }
}

/// Copy a `const char*` out of a [`cvc5_sys::OptionInfo`], mapping `NULL` to `""`.
///
/// # Safety
///
/// If non-NULL, `ptr` must point to a NUL-terminated string valid for the
/// duration of the call.
unsafe fn opt_string(ptr: *const std::os::raw::c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    unsafe { std::ffi::CStr::from_ptr(ptr) }
        .to_str()
        .expect(ERROR_NOT_UTF8)
        .to_owned()
}

impl fmt::Display for OptionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rendered)
    }
}

/// A cvc5 solver instance.
///
/// `Cvc5` takes its own reference to the [`TermManager`] it is built from, so a
/// `Solver` may outlive that binding and needs no lifetime of its own.
pub struct Solver {
    pub(crate) inner: *mut cvc5_sys::Solver,
}

impl Solver {
    /// Create a new solver instance from the given term manager.
    pub fn new(tm: &TermManager) -> Self {
        Self {
            inner: non_null(unsafe { new(tm.ptr()) }, "Solver"),
        }
    }

    // ── Configuration ──────────────────────────────────────────────

    /// Set the logic for this solver (e.g. `"QF_LIA"`).
    ///
    /// Fails if the logic is already set or is not a recognized logic.
    pub fn set_logic(&self, logic: &str) -> Result<()> {
        let c = CString::new(logic).unwrap();
        unsafe { set_logic(self.inner, c.as_ptr()) };
        checked((), "set_logic")
    }

    /// Return `true` if the logic has been set.
    pub fn is_logic_set(&self) -> bool {
        unsafe { is_logic_set(self.inner) }
    }

    /// Get the currently set logic as a string.
    ///
    /// Fails if no logic has been set.
    pub fn get_logic(&self) -> Result<String> {
        let p = unsafe { get_logic(self.inner) };
        let p = checked(p, "get_logic")?;
        Ok(unsafe { cstr_or_empty(p) }.to_owned())
    }

    /// Set a solver option (e.g. `"produce-models"`, `"true"`).
    ///
    /// Fails on an unrecognized option, an invalid value, or if the option
    /// cannot be set at this point in the session.
    pub fn set_option(&self, option: &str, value: &str) -> Result<()> {
        let o = CString::new(option).unwrap();
        let v = CString::new(value).unwrap();
        unsafe { set_option(self.inner, o.as_ptr(), v.as_ptr()) };
        checked((), "set_option")
    }

    /// Get the current value of a solver option.
    ///
    /// Fails on an unrecognized option.
    pub fn get_option(&self, option: &str) -> Result<String> {
        let o = CString::new(option).unwrap();
        let p = unsafe { get_option(self.inner, o.as_ptr()) };
        let p = checked(p, "get_option")?;
        Ok(unsafe { cstr_or_empty(p) }.to_owned())
    }

    /// Get the list of all option names.
    pub fn get_option_names(&self) -> Vec<String> {
        let mut size = 0usize;
        let ptr = unsafe { get_option_names(self.inner, &mut size) };
        unsafe { raw_slice(ptr, size) }
            .iter()
            .map(|&p| unsafe { cstr_to_string(p, "Solver::get_option_names entry") })
            .collect()
    }

    /// Set solver information (SMT-LIB `set-info`).
    pub fn set_info(&self, keyword: &str, value: &str) -> Result<()> {
        let k = CString::new(keyword).unwrap();
        let v = CString::new(value).unwrap();
        unsafe { set_info(self.inner, k.as_ptr(), v.as_ptr()) };
        checked((), "set_info")
    }

    /// Get solver information (SMT-LIB `get-info`).
    pub fn get_info(&self, flag: &str) -> Result<String> {
        let f = CString::new(flag).unwrap();
        let p = unsafe { get_info(self.inner, f.as_ptr()) };
        let p = checked(p, "get_info")?;
        Ok(unsafe { cstr_or_empty(p) }.to_owned())
    }

    // ── Assertions & checking ──────────────────────────────────────

    /// Assert a formula to the solver.
    pub fn assert_formula(&self, term: Term) -> Result<()> {
        unsafe { assert_formula(self.inner, term.inner) };
        checked((), "assert_formula")
    }

    /// Check satisfiability of the current assertions.
    ///
    /// Fails on a second query unless incremental solving is enabled.
    pub fn check_sat(&self) -> Result<SatResult> {
        let raw = unsafe { check_sat(self.inner) };
        let raw = checked(raw, "check_sat")?;
        Ok(SatResult::from_raw(raw))
    }

    /// Check satisfiability under the given assumptions.
    ///
    /// Fails on a second query unless incremental solving is enabled.
    pub fn check_sat_assuming(&self, assumptions: &[Term]) -> Result<SatResult> {
        let raw: Vec<cvc5_sys::Term> = assumptions.iter().map(|t| t.inner).collect();
        let res = unsafe { check_sat_assuming(self.inner, raw.len(), raw.as_ptr()) };
        let res = checked(res, "check_sat_assuming")?;
        Ok(SatResult::from_raw(res))
    }

    /// Get the list of asserted formulas.
    pub fn get_assertions(&self) -> Vec<Term> {
        let mut size = 0usize;
        let ptr = unsafe { get_assertions(self.inner, &mut size) };
        unsafe { raw_slice(ptr, size) }
            .iter()
            .map(|&p| Term::from_raw(p))
            .collect()
    }

    // ── Simplification ─────────────────────────────────────────────

    /// Simplify a term. If `apply_subs` is true, apply learned substitutions.
    pub fn simplify(&self, term: Term, apply_subs: bool) -> Result<Term> {
        let raw = unsafe { simplify(self.inner, term.inner, apply_subs) };
        wrap(raw, "simplify")
    }

    // ── Model queries ──────────────────────────────────────────────

    /// Get the value of a term in the current model.
    ///
    /// Fails unless model generation is enabled and the solver is in a SAT
    /// state.
    pub fn get_value(&self, term: Term) -> Result<Term> {
        let raw = unsafe { get_value(self.inner, term.inner) };
        let raw = checked(raw, "get_value")?;
        Ok(Term::from_raw(raw))
    }

    /// Get the values of multiple terms in the current model.
    ///
    /// Fails unless model generation is enabled and the solver is in a SAT
    /// state.
    pub fn get_values(&self, terms: &[Term]) -> Result<Vec<Term>> {
        let raw: Vec<cvc5_sys::Term> = terms.iter().map(|t| t.inner).collect();
        let mut rsize = 0usize;
        let ptr = unsafe { get_values(self.inner, raw.len(), raw.as_ptr(), &mut rsize) };
        let ptr = checked(ptr, "get_values")?;
        Ok(unsafe { raw_slice(ptr, rsize) }
            .iter()
            .map(|&p| Term::from_raw(p))
            .collect())
    }

    /// Get the domain elements of an uninterpreted sort in the current model.
    ///
    /// Fails unless model generation is enabled and the solver is in a SAT
    /// state.
    pub fn get_model_domain_elements(&self, sort: Sort) -> Result<Vec<Term>> {
        let mut size = 0usize;
        let ptr = unsafe { get_model_domain_elements(self.inner, sort.inner, &mut size) };
        let ptr = checked(ptr, "get_model_domain_elements")?;
        Ok(unsafe { raw_slice(ptr, size) }
            .iter()
            .map(|&p| Term::from_raw(p))
            .collect())
    }

    /// Return `true` if the given variable is a model core symbol.
    pub fn is_model_core_symbol(&self, v: Term) -> Result<bool> {
        let v = unsafe { is_model_core_symbol(self.inner, v.inner) };
        checked(v, "is_model_core_symbol")
    }

    /// Get a string representation of the current model.
    ///
    /// Fails unless model generation is enabled and the solver is in a SAT
    /// state.
    pub fn get_model(&self, sorts: &[Sort], consts: &[Term]) -> Result<String> {
        let rs: Vec<cvc5_sys::Sort> = sorts.iter().map(|s| s.inner).collect();
        let rt: Vec<cvc5_sys::Term> = consts.iter().map(|t| t.inner).collect();
        let p = unsafe { get_model(self.inner, rs.len(), rs.as_ptr(), rt.len(), rt.as_ptr()) };
        let p = checked(p, "get_model")?;
        Ok(unsafe { cstr_or_empty(p) }.to_owned())
    }

    /// Block the current model using the given mode.
    ///
    /// Fails unless model generation is enabled.
    pub fn block_model(&self, mode: cvc5_sys::BlockModelsMode) -> Result<()> {
        unsafe { block_model(self.inner, mode) };
        checked((), "block_model")
    }

    /// Block the current model values for the given terms.
    pub fn block_model_values(&self, terms: &[Term]) -> Result<()> {
        let raw: Vec<cvc5_sys::Term> = terms.iter().map(|t| t.inner).collect();
        unsafe { block_model_values(self.inner, raw.len(), raw.as_ptr()) };
        checked((), "block_model_values")
    }

    // ── Declarations ───────────────────────────────────────────────

    /// Declare a function (SMT-LIB `declare-fun`).
    pub fn declare_fun(&self, name: &str, domain: &[Sort], codomain: Sort) -> Result<Term> {
        let c = CString::new(name).unwrap();
        let raw: Vec<cvc5_sys::Sort> = domain.iter().map(|s| s.inner).collect();
        let raw = unsafe {
            declare_fun(
                self.inner,
                c.as_ptr(),
                raw.len(),
                raw.as_ptr(),
                codomain.inner,
                true,
            )
        };
        wrap(raw, "declare_fun")
    }

    /// Declare an uninterpreted sort (SMT-LIB `declare-sort`).
    pub fn declare_sort(&self, name: &str, arity: u32) -> Sort {
        let c = CString::new(name).unwrap();
        Sort::from_raw(unsafe { declare_sort(self.inner, c.as_ptr(), arity, true) })
    }

    /// Declare a datatype from constructor declarations.
    pub fn declare_dt(&self, symbol: &str, ctors: &[DatatypeConstructorDecl]) -> Result<Sort> {
        let c = CString::new(symbol).unwrap();
        let raw: Vec<cvc5_sys::DatatypeConstructorDecl> = ctors.iter().map(|d| d.inner).collect();
        let raw = unsafe { declare_dt(self.inner, c.as_ptr(), raw.len(), raw.as_ptr()) };
        wrap(raw, "declare_dt")
    }

    // ── Definitions ────────────────────────────────────────────────

    /// Define a function (SMT-LIB `define-fun`).
    pub fn define_fun(
        &self,
        symbol: &str,
        vars: &[Term],
        sort: Sort,
        term: Term,
        global: bool,
    ) -> Result<Term> {
        let c = CString::new(symbol).unwrap();
        let raw: Vec<cvc5_sys::Term> = vars.iter().map(|t| t.inner).collect();
        let raw = unsafe {
            define_fun(
                self.inner,
                c.as_ptr(),
                raw.len(),
                raw.as_ptr(),
                sort.inner,
                term.inner,
                global,
            )
        };
        wrap(raw, "define_fun")
    }

    /// Define a recursive function (SMT-LIB `define-fun-rec`).
    pub fn define_fun_rec(
        &self,
        symbol: &str,
        vars: &[Term],
        sort: Sort,
        term: Term,
        global: bool,
    ) -> Result<Term> {
        let c = CString::new(symbol).unwrap();
        let raw: Vec<cvc5_sys::Term> = vars.iter().map(|t| t.inner).collect();
        let raw = unsafe {
            define_fun_rec(
                self.inner,
                c.as_ptr(),
                raw.len(),
                raw.as_ptr(),
                sort.inner,
                term.inner,
                global,
            )
        };
        wrap(raw, "define_fun_rec")
    }

    /// Define a recursive function from a previously declared constant.
    pub fn define_fun_rec_from_const(
        &self,
        fun: Term,
        vars: &[Term],
        term: Term,
        global: bool,
    ) -> Result<Term> {
        let raw: Vec<cvc5_sys::Term> = vars.iter().map(|t| t.inner).collect();
        let raw = unsafe {
            define_fun_rec_from_const(
                self.inner,
                fun.inner,
                raw.len(),
                raw.as_ptr(),
                term.inner,
                global,
            )
        };
        wrap(raw, "define_fun_rec_from_const")
    }

    // ── Scope management ───────────────────────────────────────────

    /// Push `n` assertion scope levels.
    pub fn push(&self, n: u32) -> Result<()> {
        unsafe { push(self.inner, n) };
        checked((), "push")
    }
    /// Pop `n` assertion scope levels.
    pub fn pop(&self, n: u32) -> Result<()> {
        unsafe { pop(self.inner, n) };
        checked((), "pop")
    }
    /// Remove all assertions and reset the scope.
    pub fn reset_assertions(&self) {
        unsafe { reset_assertions(self.inner) }
    }

    // ── Unsat core / assumptions ───────────────────────────────────

    /// Get the unsat core (subset of assertions that are unsatisfiable).
    pub fn get_unsat_core(&self) -> Result<Vec<Term>> {
        let mut size = 0usize;
        let ptr = unsafe { get_unsat_core(self.inner, &mut size) };
        let ptr = checked(ptr, "get_unsat_core")?;
        Ok(unsafe { raw_slice(ptr, size) }
            .iter()
            .map(|&p| Term::from_raw(p))
            .collect())
    }

    /// Get the lemmas used in the unsat core.
    pub fn get_unsat_core_lemmas(&self) -> Result<Vec<Term>> {
        let mut size = 0usize;
        let ptr = unsafe { get_unsat_core_lemmas(self.inner, &mut size) };
        let ptr = checked(ptr, "get_unsat_core_lemmas")?;
        Ok(unsafe { raw_slice(ptr, size) }
            .iter()
            .map(|&p| Term::from_raw(p))
            .collect())
    }

    /// Get the unsat assumptions (subset of assumptions from `check_sat_assuming`).
    pub fn get_unsat_assumptions(&self) -> Result<Vec<Term>> {
        let mut size = 0usize;
        let ptr = unsafe { get_unsat_assumptions(self.inner, &mut size) };
        let ptr = checked(ptr, "get_unsat_assumptions")?;
        Ok(unsafe { raw_slice(ptr, size) }
            .iter()
            .map(|&p| Term::from_raw(p))
            .collect())
    }

    // ── Proofs ─────────────────────────────────────────────────────

    /// Get the proof of unsatisfiability.
    pub fn get_proof(&self, c: cvc5_sys::ProofComponent) -> Result<Vec<Proof>> {
        let mut size = 0usize;
        let ptr = unsafe { get_proof(self.inner, c, &mut size) };
        let ptr = checked(ptr, "get_proof")?;
        Ok(unsafe { raw_slice(ptr, size) }
            .iter()
            .map(|&p| Proof::from_raw(p))
            .collect())
    }

    /// Convert a proof to a string in the given format.
    pub fn proof_to_string(
        &self,
        proof: Proof,
        format: cvc5_sys::ProofFormat,
        assertions: &[Term],
        names: &[&str],
    ) -> String {
        let rt: Vec<cvc5_sys::Term> = assertions.iter().map(|t| t.inner).collect();
        let cnames: Vec<CString> = names.iter().map(|n| CString::new(*n).unwrap()).collect();
        let mut ptrs: Vec<*const std::ffi::c_char> = cnames.iter().map(|c| c.as_ptr()).collect();
        unsafe {
            std::ffi::CStr::from_ptr(proof_to_string(
                self.inner,
                proof.inner,
                format,
                rt.len(),
                rt.as_ptr(),
                ptrs.as_mut_ptr(),
            ))
            .to_string_lossy()
            .into_owned()
        }
    }

    // ── Learned literals / difficulty ──────────────────────────────

    /// Get the learned literals of the given type.
    pub fn get_learned_literals(&self, lit_type: cvc5_sys::LearnedLitType) -> Result<Vec<Term>> {
        let mut size = 0usize;
        let ptr = unsafe { get_learned_literals(self.inner, lit_type, &mut size) };
        let ptr = checked(ptr, "get_learned_literals")?;
        Ok(unsafe { raw_slice(ptr, size) }
            .iter()
            .map(|&p| Term::from_raw(p))
            .collect())
    }

    /// Get the difficulty of each assertion as `(inputs, values)` pairs.
    pub fn get_difficulty(&self) -> Result<(Vec<Term>, Vec<Term>)> {
        let mut size = 0usize;
        let mut inputs: *mut cvc5_sys::Term = std::ptr::null_mut();
        let mut values: *mut cvc5_sys::Term = std::ptr::null_mut();
        unsafe { get_difficulty(self.inner, &mut size, &mut inputs, &mut values) };
        checked((), "get_difficulty")?;
        let i = unsafe { raw_slice(inputs, size) }
            .iter()
            .map(|&p| Term::from_raw(p))
            .collect();
        let v = unsafe { raw_slice(values, size) }
            .iter()
            .map(|&p| Term::from_raw(p))
            .collect();
        Ok((i, v))
    }

    // ── Timeout core ───────────────────────────────────────────────

    /// Get a timeout core: a minimal subset of assertions causing a timeout.
    pub fn get_timeout_core(&self) -> Result<(SatResult, Vec<Term>)> {
        let mut result: cvc5_sys::Result = std::ptr::null_mut();
        let mut size = 0usize;
        let ptr = unsafe { get_timeout_core(self.inner, &mut result, &mut size) };
        let ptr = checked(ptr, "get_timeout_core")?;
        let terms = unsafe { raw_slice(ptr, size) }
            .iter()
            .map(|&p| Term::from_raw(p))
            .collect();
        Ok((SatResult::from_raw(result), terms))
    }

    /// Get a timeout core under the given assumptions.
    pub fn get_timeout_core_assuming(
        &self,
        assumptions: &[Term],
    ) -> Result<(SatResult, Vec<Term>)> {
        let raw: Vec<cvc5_sys::Term> = assumptions.iter().map(|t| t.inner).collect();
        let mut result: cvc5_sys::Result = std::ptr::null_mut();
        let mut rsize = 0usize;
        let ptr = unsafe {
            get_timeout_core_assuming(self.inner, raw.len(), raw.as_ptr(), &mut result, &mut rsize)
        };
        let ptr = checked(ptr, "get_timeout_core_assuming")?;
        let terms = unsafe { raw_slice(ptr, rsize) }
            .iter()
            .map(|&p| Term::from_raw(p))
            .collect();
        Ok((SatResult::from_raw(result), terms))
    }

    // ── Quantifier elimination ─────────────────────────────────────

    /// Perform quantifier elimination on the given formula.
    pub fn get_quantifier_elimination(&self, q: Term) -> Result<Term> {
        let raw = unsafe { get_quantifier_elimination(self.inner, q.inner) };
        wrap(raw, "get_quantifier_elimination")
    }

    /// Perform partial quantifier elimination, returning a single disjunct.
    pub fn get_quantifier_elimination_disjunct(&self, q: Term) -> Result<Term> {
        let raw = unsafe { get_quantifier_elimination_disjunct(self.inner, q.inner) };
        wrap(raw, "get_quantifier_elimination_disjunct")
    }

    // ── Separation logic ───────────────────────────────────────────

    /// Declare the heap sorts for separation logic.
    pub fn declare_sep_heap(&self, loc: Sort, data: Sort) -> Result<()> {
        unsafe { declare_sep_heap(self.inner, loc.inner, data.inner) };
        checked((), "declare_sep_heap")
    }

    /// Get the separation logic heap term.
    pub fn get_value_sep_heap(&self) -> Result<Term> {
        let raw = unsafe { get_value_sep_heap(self.inner) };
        wrap(raw, "get_value_sep_heap")
    }

    /// Get the separation logic nil term.
    pub fn get_value_sep_nil(&self) -> Result<Term> {
        let raw = unsafe { get_value_sep_nil(self.inner) };
        wrap(raw, "get_value_sep_nil")
    }

    // ── Pools ──────────────────────────────────────────────────────

    /// Declare a term pool with the given initial values.
    pub fn declare_pool(&self, symbol: &str, sort: Sort, init_value: &[Term]) -> Result<Term> {
        let c = CString::new(symbol).unwrap();
        let raw: Vec<cvc5_sys::Term> = init_value.iter().map(|t| t.inner).collect();
        let raw =
            unsafe { declare_pool(self.inner, c.as_ptr(), sort.inner, raw.len(), raw.as_ptr()) };
        wrap(raw, "declare_pool")
    }

    // ── Interpolation ──────────────────────────────────────────────

    /// Compute an interpolant for the given conjecture.
    ///
    /// Returns `None` if no interpolant exists.
    pub fn get_interpolant(&self, conj: Term) -> Result<Option<Term>> {
        let raw = unsafe { get_interpolant(self.inner, conj.inner) };
        let raw = checked(raw, "get_interpolant")?;
        Ok((!raw.is_null()).then(|| Term::from_raw(raw)))
    }

    /// Compute an interpolant constrained by the given grammar.
    ///
    /// Returns `None` if no interpolant exists.
    pub fn get_interpolant_with_grammar(
        &self,
        conj: Term,
        grammar: &Grammar,
    ) -> Result<Option<Term>> {
        let raw = unsafe { get_interpolant_with_grammar(self.inner, conj.inner, grammar.inner) };
        let raw = checked(raw, "get_interpolant_with_grammar")?;
        Ok((!raw.is_null()).then(|| Term::from_raw(raw)))
    }

    /// Get the next interpolant (after a previous `get_interpolant` call).
    ///
    /// Returns `None` if no further interpolant can be found.
    pub fn get_interpolant_next(&self) -> Result<Option<Term>> {
        let raw = unsafe { get_interpolant_next(self.inner) };
        let raw = checked(raw, "get_interpolant_next")?;
        Ok((!raw.is_null()).then(|| Term::from_raw(raw)))
    }

    // ── Abduction ──────────────────────────────────────────────────

    /// Compute an abduct for the given conjecture.
    ///
    /// Returns `None` if no abduct can be found.
    pub fn get_abduct(&self, conj: Term) -> Result<Option<Term>> {
        let raw = unsafe { get_abduct(self.inner, conj.inner) };
        let raw = checked(raw, "get_abduct")?;
        Ok((!raw.is_null()).then(|| Term::from_raw(raw)))
    }

    /// Compute an abduct constrained by the given grammar.
    ///
    /// Returns `None` if no abduct can be found.
    pub fn get_abduct_with_grammar(&self, conj: Term, grammar: &Grammar) -> Result<Option<Term>> {
        let raw = unsafe { get_abduct_with_grammar(self.inner, conj.inner, grammar.inner) };
        let raw = checked(raw, "get_abduct_with_grammar")?;
        Ok((!raw.is_null()).then(|| Term::from_raw(raw)))
    }

    /// Get the next abduct (after a previous `get_abduct` call).
    ///
    /// Returns `None` if no further abduct can be found.
    pub fn get_abduct_next(&self) -> Result<Option<Term>> {
        let raw = unsafe { get_abduct_next(self.inner) };
        let raw = checked(raw, "get_abduct_next")?;
        Ok((!raw.is_null()).then(|| Term::from_raw(raw)))
    }

    // ── Instantiations ─────────────────────────────────────────────

    /// Get a string representation of all quantifier instantiations.
    pub fn get_instantiations(&self) -> Result<String> {
        let p = unsafe { get_instantiations(self.inner) };
        let p = checked(p, "get_instantiations")?;
        Ok(unsafe { cstr_or_empty(p) }.to_owned())
    }

    // ── SyGuS ──────────────────────────────────────────────────────

    /// Declare a SyGuS variable.
    pub fn declare_sygus_var(&self, symbol: &str, sort: Sort) -> Result<Term> {
        let c = CString::new(symbol).unwrap();
        let raw = unsafe { declare_sygus_var(self.inner, c.as_ptr(), sort.inner) };
        wrap(raw, "declare_sygus_var")
    }

    /// Create a SyGuS grammar from bound variables and non-terminal symbols.
    pub fn mk_grammar(&self, bound_vars: &[Term], symbols: &[Term]) -> Result<Grammar> {
        let bv: Vec<cvc5_sys::Term> = bound_vars.iter().map(|t| t.inner).collect();
        let sy: Vec<cvc5_sys::Term> = symbols.iter().map(|t| t.inner).collect();
        let raw = unsafe { mk_grammar(self.inner, bv.len(), bv.as_ptr(), sy.len(), sy.as_ptr()) };
        wrap(raw, "mk_grammar")
    }

    /// Declare a function to synthesize (SyGuS `synth-fun`).
    pub fn synth_fun(&self, symbol: &str, bound_vars: &[Term], sort: Sort) -> Result<Term> {
        let c = CString::new(symbol).unwrap();
        let raw: Vec<cvc5_sys::Term> = bound_vars.iter().map(|t| t.inner).collect();
        let raw = unsafe { synth_fun(self.inner, c.as_ptr(), raw.len(), raw.as_ptr(), sort.inner) };
        wrap(raw, "synth_fun")
    }

    /// Declare a function to synthesize with a grammar constraint.
    pub fn synth_fun_with_grammar(
        &self,
        symbol: &str,
        bound_vars: &[Term],
        sort: Sort,
        grammar: &Grammar,
    ) -> Result<Term> {
        let c = CString::new(symbol).unwrap();
        let raw: Vec<cvc5_sys::Term> = bound_vars.iter().map(|t| t.inner).collect();
        let raw = unsafe {
            synth_fun_with_grammar(
                self.inner,
                c.as_ptr(),
                raw.len(),
                raw.as_ptr(),
                sort.inner,
                grammar.inner,
            )
        };
        wrap(raw, "synth_fun_with_grammar")
    }

    /// Add a SyGuS constraint.
    pub fn add_sygus_constraint(&self, term: Term) -> Result<()> {
        unsafe { add_sygus_constraint(self.inner, term.inner) };
        checked((), "add_sygus_constraint")
    }

    /// Get the list of SyGuS constraints.
    pub fn get_sygus_constraints(&self) -> Vec<Term> {
        let mut size = 0usize;
        let ptr = unsafe { get_sygus_constraints(self.inner, &mut size) };
        unsafe { raw_slice(ptr, size) }
            .iter()
            .map(|&p| Term::from_raw(p))
            .collect()
    }

    /// Add a SyGuS assumption.
    pub fn add_sygus_assume(&self, term: Term) -> Result<()> {
        unsafe { add_sygus_assume(self.inner, term.inner) };
        checked((), "add_sygus_assume")
    }

    /// Get the list of SyGuS assumptions.
    pub fn get_sygus_assumptions(&self) -> Vec<Term> {
        let mut size = 0usize;
        let ptr = unsafe { get_sygus_assumptions(self.inner, &mut size) };
        unsafe { raw_slice(ptr, size) }
            .iter()
            .map(|&p| Term::from_raw(p))
            .collect()
    }

    /// Add a SyGuS invariant constraint.
    pub fn add_sygus_inv_constraint(
        &self,
        inv: Term,
        pre: Term,
        trans: Term,
        post: Term,
    ) -> Result<()> {
        unsafe {
            add_sygus_inv_constraint(self.inner, inv.inner, pre.inner, trans.inner, post.inner)
        };
        checked((), "add_sygus_inv_constraint")
    }

    /// Check for a synthesis solution.
    pub fn check_synth(&self) -> Result<SynthResult> {
        let raw = unsafe { check_synth(self.inner) };
        wrap(raw, "check_synth")
    }

    /// Get the next synthesis solution.
    pub fn check_synth_next(&self) -> Result<SynthResult> {
        let raw = unsafe { check_synth_next(self.inner) };
        wrap(raw, "check_synth_next")
    }

    /// Get the synthesis solution for a given function-to-synthesize term.
    pub fn get_synth_solution(&self, term: Term) -> Result<Term> {
        let raw = unsafe { get_synth_solution(self.inner, term.inner) };
        wrap(raw, "get_synth_solution")
    }

    /// Get synthesis solutions for multiple function-to-synthesize terms.
    ///
    /// Returns an empty vector for an empty `terms` slice. The underlying C++
    /// API rejects an empty vector outright
    /// (`CVC5_API_ARG_SIZE_CHECK_EXPECTED(!terms.empty())`), which on cvc5
    /// <= 1.3.4 terminates the process, so the empty case is short-circuited
    /// here rather than forwarded.
    pub fn get_synth_solutions(&self, terms: &[Term]) -> Result<Vec<Term>> {
        if terms.is_empty() {
            return Ok(Vec::new());
        }
        let raw: Vec<cvc5_sys::Term> = terms.iter().map(|t| t.inner).collect();
        // `cvc5_get_synth_solutions` writes no out-length; on success it returns
        // exactly `terms.len()` entries (api/cpp/cvc5.cpp:8947-8960).
        let ptr = unsafe { get_synth_solutions(self.inner, raw.len(), raw.as_ptr()) };
        let ptr = checked(ptr, "get_synth_solutions")?;
        Ok(unsafe { raw_slice(ptr, terms.len()) }
            .iter()
            .map(|&p| Term::from_raw(p))
            .collect())
    }

    /// Find a synthesis target of the given type.
    ///
    /// Returns `None` if the call failed.
    pub fn find_synth(&self, target: cvc5_sys::FindSynthTarget) -> Result<Option<Term>> {
        let raw = unsafe { find_synth(self.inner, target) };
        let raw = checked(raw, "find_synth")?;
        Ok((!raw.is_null()).then(|| Term::from_raw(raw)))
    }

    /// Find a synthesis target constrained by the given grammar.
    ///
    /// Returns `None` if the call failed.
    pub fn find_synth_with_grammar(
        &self,
        target: cvc5_sys::FindSynthTarget,
        grammar: &Grammar,
    ) -> Result<Option<Term>> {
        let raw = unsafe { find_synth_with_grammar(self.inner, target, grammar.inner) };
        let raw = checked(raw, "find_synth_with_grammar")?;
        Ok((!raw.is_null()).then(|| Term::from_raw(raw)))
    }

    /// Get the next synthesis target.
    ///
    /// Returns `None` if the call failed.
    pub fn find_synth_next(&self) -> Option<Term> {
        let raw = unsafe { find_synth_next(self.inner) };
        (!raw.is_null()).then(|| Term::from_raw(raw))
    }

    // ── Mutually recursive definitions ─────────────────────────────

    /// Define mutually recursive functions.
    pub fn define_funs_rec(
        &self,
        funs: &[Term],
        vars: &[&[Term]],
        terms: &[Term],
        global: bool,
    ) -> Result<()> {
        let rf: Vec<cvc5_sys::Term> = funs.iter().map(|t| t.inner).collect();
        let mut nvars: Vec<usize> = vars.iter().map(|v| v.len()).collect();
        let raw_vars: Vec<Vec<cvc5_sys::Term>> = vars
            .iter()
            .map(|v| v.iter().map(|t| t.inner).collect())
            .collect();
        let mut var_ptrs: Vec<*const cvc5_sys::Term> =
            raw_vars.iter().map(|v| v.as_ptr()).collect();
        let rt: Vec<cvc5_sys::Term> = terms.iter().map(|t| t.inner).collect();
        unsafe {
            define_funs_rec(
                self.inner,
                rf.len(),
                rf.as_ptr(),
                nvars.as_mut_ptr(),
                var_ptrs.as_mut_ptr(),
                rt.as_ptr(),
                global,
            )
        };
        checked((), "define_funs_rec")
    }

    // ── Output ─────────────────────────────────────────────────────

    /// Redirect solver output for the given tag to a file.
    pub fn get_output(&self, tag: &str, filename: &str) {
        let t = CString::new(tag).unwrap();
        let f = CString::new(filename).unwrap();
        unsafe { get_output(self.inner, t.as_ptr(), f.as_ptr()) }
    }

    /// Close a previously opened output file.
    pub fn close_output(&self, filename: &str) {
        let f = CString::new(filename).unwrap();
        unsafe { close_output(self.inner, f.as_ptr()) }
    }

    /// Print statistics to the given file descriptor (async-signal-safe).
    pub fn print_stats_safe(&self, fd: i32) {
        unsafe { print_stats_safe(self.inner, fd) }
    }

    // ── Statistics / output ────────────────────────────────────────

    /// Return `true` if the given output tag is enabled.
    pub fn is_output_on(&self, tag: &str) -> bool {
        let c = CString::new(tag).unwrap();
        unsafe { is_output_on(self.inner, c.as_ptr()) }
    }

    /// Get the solver statistics.
    pub fn get_statistics(&self) -> Statistics {
        Statistics::from_raw(unsafe { get_statistics(self.inner) })
    }

    /**
    Get detailed information about a solver option.
     */
    pub fn get_option_info(&self, option: &str) -> Result<OptionInfo> {
        let c = CString::new(option).unwrap();
        let mut info: cvc5_sys::OptionInfo = unsafe { std::mem::zeroed() };
        unsafe { get_option_info(self.inner, c.as_ptr(), &mut info) };
        checked((), "get_option_info")?;
        Ok(OptionInfo::from_raw(&info))
    }

    // ── Plugin ─────────────────────────────────────────────────────

    /// Add a plugin to the solver.
    pub fn add_plugin(&self, plugin: &mut cvc5_sys::Plugin) {
        unsafe { add_plugin(self.inner, plugin) }
    }

    /// Get the cvc5 version string.
    pub fn version(&self) -> String {
        unsafe {
            std::ffi::CStr::from_ptr(get_version(self.inner))
                .to_string_lossy()
                .into_owned()
        }
    }
}

impl Drop for Solver {
    fn drop(&mut self) {
        unsafe { delete(self.inner) }
    }
}
