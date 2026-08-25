//! Regression tests for the NULL guards in `src/ffi.rs`.
//!
//! The cvc5 C API returns `NULL` from its array getters whenever the result is
//! empty. Nine of them do so deterministically — they end in
//! `return *size > 0 ? res.data() : nullptr;` — and the rest return
//! `res.data()` on a `static thread_local std::vector`, which is `NULL` until
//! that thread has seen a non-empty result and non-`NULL` afterwards.
//!
//! The tests are in three groups, because they do not all prove the same thing:
//!
//! 1. **Live NULL through `ffi::raw_slice`.** Verified by instrumenting
//!    `raw_slice` to log null arrivals and running each test in isolation; the
//!    observed count is noted on each test. Replacing the guard with a bare
//!    `slice::from_raw_parts` makes every one of them abort with "requires the
//!    pointer to be aligned and non-null".
//! 2. **Live NULL through a different guard** — a documented "null term"
//!    result, which must surface as `None`.
//! 3. **Positive paths**, so the guards cannot silently start returning empty.
//!
//! What group 1 does *not* claim: the previous
//! `(0..size).map(|i| *ptr.add(i))` formulation also survived these cases,
//! because an empty range never touches the pointer. The guard's value is that
//! the invariant is now explicit, that it holds where the length does *not*
//! come from the same place as the pointer (`mk_dt_sorts`,
//! `get_synth_solutions`, and every out-param getter), and that it covers cvc5
//! post-1.3.4, where any failing call returns NULL.

use cvc5::{Kind, LearnedLitType, Solver, TermManager};

// ── Group 1: live NULL through ffi::raw_slice ───────────────────────────────

/// `cvc5_sort_tuple_get_element_sorts` — deterministic NULL when empty.
/// Observed: 1 NULL arrival.
#[test]
fn empty_tuple_sort_element_sorts() {
    let tm = TermManager::new();
    let unit = tm.mk_tuple_sort(&[]);
    assert_eq!(unit.tuple_element_sorts().len(), 0);
}

/// `cvc5_term_get_tuple_value` — `res.data()` on an empty thread-local vector.
/// Observed: 1 NULL arrival.
#[test]
fn empty_tuple_value() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_UFDT");
    solver.set_option("produce-models", "true");

    let unit = tm.mk_tuple(&[]);
    let c = tm.mk_const(tm.mk_tuple_sort(&[]), "u");
    solver.assert_formula(tm.mk_term(Kind::Equal, &[c.clone(), unit]));
    assert!(solver.check_sat().is_sat());

    let v = solver.get_value(c);
    assert!(v.is_tuple_value());
    assert_eq!(v.tuple_value().len(), 0);
}

/// `cvc5_term_get_set_value` — empty set literal.
/// Observed: 1 NULL arrival.
#[test]
fn empty_set_value() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_UFLIAFS");
    solver.set_option("produce-models", "true");

    let set_sort = tm.mk_set_sort(tm.integer_sort());
    let empty = tm.mk_empty_set(set_sort.clone());
    let c = tm.mk_const(set_sort, "s");
    solver.assert_formula(tm.mk_term(Kind::Equal, &[c.clone(), empty]));
    assert!(solver.check_sat().is_sat());

    let v = solver.get_value(c);
    assert!(v.is_set_value());
    assert_eq!(v.set_value().len(), 0);
}

/// `cvc5_get_values` with an empty request list.
/// Observed: 1 NULL arrival.
#[test]
fn get_values_empty_slice() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_LIA");
    solver.set_option("produce-models", "true");
    assert!(solver.check_sat().is_sat());
    assert_eq!(solver.get_values(&[]).len(), 0);
}

/// `cvc5_get_sygus_constraints` / `_assumptions` — deterministic NULL when none
/// have been added. Observed: 2 NULL arrivals.
#[test]
fn empty_sygus_constraints_and_assumptions() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_option("sygus", "true");
    assert_eq!(solver.get_sygus_constraints().len(), 0);
    assert_eq!(solver.get_sygus_assumptions().len(), 0);
}

/// `cvc5_get_assertions` — `res.data()` on an empty thread-local vector.
/// Observed: 1 NULL arrival.
#[test]
fn no_assertions() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_LIA");
    assert_eq!(solver.get_assertions().len(), 0);
}

/// `cvc5_get_learned_literals` — empty when nothing was learned.
/// Observed: 1 NULL arrival.
#[test]
fn empty_learned_literals() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_LIA");
    solver.set_option("produce-learned-literals", "true");
    assert!(solver.check_sat().is_sat());
    assert_eq!(solver.get_learned_literals(LearnedLitType::Input).len(), 0);
}

/// `cvc5_get_option_info` `memset`s its out-struct to zero before filling it, so
/// an option with no aliases leaves `aliases` NULL and `num_aliases` 0.
/// Observed: 2 NULL arrivals (`aliases` and `no_supports`).
#[test]
fn option_info_without_aliases() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    let info = solver.get_option_info("incremental");
    assert_eq!(info.name().as_ref(), "incremental");
    assert_eq!(info.aliases().len(), 0);
    assert_eq!(info.no_supports().len(), 0);
    let _ = info.kind();
}

// ── Group 2: live NULL through a different guard ────────────────────────────

/// `cvc5_get_interpolant` returns the *null term* when no interpolant exists — a
/// documented outcome that must surface as `None` rather than being wrapped.
/// Guarded in `Solver::get_interpolant`, not in `raw_slice`.
#[test]
fn absent_interpolant_is_none() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_LIA");
    solver.set_option("produce-interpolants", "true");
    solver.set_option("incremental", "true");

    let x = tm.mk_const(tm.integer_sort(), "x");
    let zero = tm.mk_integer(0);
    solver.assert_formula(tm.mk_term(Kind::Gt, &[x.clone(), zero.clone()]));

    // `x < 0` does not follow from `x > 0`, so no interpolant exists and cvc5
    // hands back the null term.
    let conj = tm.mk_term(Kind::Lt, &[x, zero]);
    assert!(solver.get_interpolant(conj).is_none());
}

// ── Group 3: positive paths ────────────────────────────────────────────────

/// An empty `terms` slice must not reach `cvc5_get_synth_solutions`: the C++ API
/// rejects an empty vector outright, which on cvc5 <= 1.3.4 terminates the
/// process. No NULL is involved here — the C call never happens.
#[test]
fn synth_solutions_empty_input_is_short_circuited() {
    let tm = TermManager::new();
    let solver = Solver::new(&tm);
    assert_eq!(solver.get_synth_solutions(&[]).len(), 0);
}

/// A symbol-bearing sort/term still reports its symbol after the guard change.
///
/// The NULL path of `cvc5_{sort,term}_get_symbol` is *not* reachable on cvc5
/// <= 1.3.4: a symbol-less argument fails `CVC5_API_CHECK(has_symbol)` and
/// terminates the process before returning. That guard is for post-1.3.4.
#[test]
fn symbol_roundtrip() {
    let tm = TermManager::new();
    let x = tm.mk_const(tm.integer_sort(), "x");
    assert!(x.has_symbol());
    assert_eq!(x.symbol(), "x");

    let u = tm.mk_uninterpreted_sort("U");
    assert!(u.has_symbol());
    assert_eq!(u.symbol(), "U");
}

/// `cvc5_get_option_names` returns a `const char**`; guarding it must not have
/// broken the (non-empty) success path.
#[test]
fn option_names_still_populated() {
    let tm = TermManager::new();
    let solver = Solver::new(&tm);
    let names = solver.get_option_names();
    assert!(names.len() > 100, "got {} option names", names.len());
    assert!(names.iter().any(|n| n == "incremental"));
}

/// A non-empty array getter still round-trips, so `raw_slice` is not silently
/// swallowing results.
#[test]
fn non_empty_array_getter_still_works() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_LIA");

    let x = tm.mk_const(tm.integer_sort(), "x");
    let zero = tm.mk_integer(0);
    solver.assert_formula(tm.mk_term(Kind::Gt, &[x.clone(), zero.clone()]));
    solver.assert_formula(tm.mk_term(Kind::Lt, &[x, tm.mk_integer(10)]));
    assert_eq!(solver.get_assertions().len(), 2);

    let tup = tm.mk_tuple_sort(&[tm.integer_sort(), tm.boolean_sort()]);
    assert_eq!(tup.tuple_element_sorts().len(), 2);
}
