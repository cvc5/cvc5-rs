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
    let mut solver = Solver::new(&tm);
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
    let mut solver = Solver::new(&tm);
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
    let mut solver = Solver::new(&tm);
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
    let mut solver = Solver::new(&tm);
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
    let mut solver = Solver::new(&tm);

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
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_LIA").unwrap();
    assert!(!cvc5::has_error());
    assert_eq!(cvc5::last_error(), None);
}
