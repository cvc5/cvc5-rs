//! Tests for cvc5's error state and the `Result`-returning API.
//!
//! cvc5 >= the PR #12724 snapshot records failures in thread-local state and
//! returns a default value instead of calling `exit(EXIT_FAILURE)`. These tests
//! drive real failures and assert that the reason reaches the caller.

use cvc5::{Kind, Solver, TermManager};

/// A second `check_sat` without incremental mode is rejected by cvc5
/// (`api/cpp/cvc5.cpp`: "cannot make multiple queries unless incremental
/// solving is enabled"). It must arrive as `Err`, not a panic or an abort.
#[test]
fn second_check_sat_without_incremental_is_err() {
    let tm = TermManager::new();
    let solver = Solver::new(&tm);
    solver.set_logic("QF_LIA").unwrap();
    solver.set_option("incremental", "false").unwrap();

    assert!(solver.check_sat().unwrap().is_sat());

    let err = solver.check_sat().expect_err("second query must fail");
    assert!(
        err.message().contains("incremental"),
        "unexpected message: {err}"
    );
}

/// `get_value` without model production enabled fails rather than returning a
/// bogus term.
#[test]
fn get_value_without_models_is_err() {
    let tm = TermManager::new();
    let solver = Solver::new(&tm);
    solver.set_logic("QF_LIA").unwrap();
    solver.set_option("produce-models", "false").unwrap();

    let x = tm.mk_const(tm.integer_sort(), "x").unwrap();
    let zero = tm.mk_integer(0);
    solver
        .assert_formula(tm.mk_term(Kind::Gt, &[x.clone(), zero]).unwrap())
        .unwrap();
    assert!(solver.check_sat().unwrap().is_sat());

    let err = solver.get_value(x).expect_err("models are disabled");
    assert!(!err.message().is_empty());
}

/// An unrecognized option name fails on both the setter and the getter.
#[test]
fn unknown_option_is_err() {
    let tm = TermManager::new();
    let solver = Solver::new(&tm);
    assert!(solver.set_option("no-such-option-here", "true").is_err());
    assert!(solver.get_option("no-such-option-here").is_err());
}

/// `get_logic` before any logic has been set fails.
#[test]
fn get_logic_before_set_is_err() {
    let tm = TermManager::new();
    let solver = Solver::new(&tm);
    assert!(!solver.is_logic_set());
    assert!(solver.get_logic().is_err());
}

/// The unsat-core getters fail when the option is off, instead of silently
/// returning an empty vector.
#[test]
fn unsat_core_without_option_is_err() {
    let tm = TermManager::new();
    let solver = Solver::new(&tm);
    solver.set_logic("QF_LIA").unwrap();

    let x = tm.mk_const(tm.integer_sort(), "x").unwrap();
    let zero = tm.mk_integer(0);
    solver
        .assert_formula(tm.mk_term(Kind::Gt, &[x.clone(), zero.clone()]).unwrap())
        .unwrap();
    solver
        .assert_formula(tm.mk_term(Kind::Lt, &[x, zero]).unwrap())
        .unwrap();
    assert!(solver.check_sat().unwrap().is_unsat());

    assert!(solver.get_unsat_core().is_err());
}

/// The thread-local state is observable directly, and cleared on demand.
#[test]
fn error_state_is_queryable_and_clearable() {
    let tm = TermManager::new();
    let solver = Solver::new(&tm);

    assert!(solver.set_option("definitely-not-an-option", "1").is_err());
    assert!(cvc5::has_error());
    let msg = cvc5::last_error().expect("a message was recorded");
    assert!(!msg.is_empty());

    cvc5::clear_error();
    assert!(!cvc5::has_error());
    assert_eq!(cvc5::last_error(), None);
}

/// A successful call leaves the state clean.
#[test]
fn success_leaves_no_error() {
    let tm = TermManager::new();
    let solver = Solver::new(&tm);
    solver.set_logic("QF_LIA").unwrap();
    assert!(!cvc5::has_error());
    assert_eq!(cvc5::last_error(), None);
}

// ── Failure paths for the tags added by the necessity audit ────────────────
//
// Each of these was previously an infallible signature. The audit found a
// reachable `CVC5_API_CHECK` behind it; these drive that check so the `Result`
// cannot be removed again without a test failing.

/// `Solver::getOutput` converts an `OptionException` from an unknown tag into
/// `CVC5ApiException("invalid output tag ...")`.
#[test]
fn unknown_output_tag_is_err() {
    let tm = TermManager::new();
    let solver = Solver::new(&tm);
    assert!(solver.is_output_on("definitely-not-a-tag").is_err());
    let err = solver
        .get_output("definitely-not-a-tag", "/dev/null")
        .expect_err("unknown output tag");
    assert!(!err.message().is_empty());
    // A real tag still works.
    assert!(!solver.is_output_on("inst").unwrap());
}

/// `Term::operator[]` checks `index < getNumChildren()`.
#[test]
fn term_child_out_of_bounds_is_err() {
    let tm = TermManager::new();
    let x = tm.mk_const(tm.integer_sort(), "x").unwrap();
    let zero = tm.mk_integer(0);
    let gt = tm.mk_term(Kind::Gt, &[x, zero]).unwrap();
    assert_eq!(gt.num_children(), 2);
    assert!(gt.child(0).is_ok());
    assert!(gt.child(1).is_ok());
    assert!(gt.child(2).is_err(), "index 2 is out of bounds");
    assert!(gt.child(usize::MAX).is_err());
}

/// The statistics iterator reports "iterator not initialized" until `iter_init`.
#[test]
fn statistics_iterator_requires_init() {
    let tm = TermManager::new();
    let solver = Solver::new(&tm);
    solver.set_option("stats", "true").unwrap();
    solver.set_logic("QF_LIA").unwrap();
    assert!(solver.check_sat().unwrap().is_sat());

    let stats = solver.get_statistics();
    // Before iter_init: both iterator entry points fail.
    assert!(stats.iter_has_next().is_err());
    assert!(stats.iter_next().is_err());
    assert!(stats.iter_next_stat().is_err());

    stats.iter_init(true, true);
    assert!(stats.iter_has_next().unwrap());
    assert!(stats.iter_next().is_ok());
}

/// A name that no constructor carries is a reachable failure, which is why the
/// `*_by_name` lookups are fallible.
#[test]
fn datatype_lookup_by_unknown_name_is_err() {
    let tm = TermManager::new();
    let mut decl = tm.mk_dt_decl("Color", false);
    let c = tm.mk_dt_cons_decl("red");
    decl.add_constructor(&c).unwrap();
    let sort = tm.mk_dt_sort(&decl).unwrap();
    let dt = sort.datatype().unwrap();

    assert!(dt.constructor_by_name("red").is_ok());
    assert!(dt.constructor_by_name("chartreuse").is_err());
    assert!(dt.selector("nope").is_err());
}
