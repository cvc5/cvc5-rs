//! Regression tests for `Term::substitute_terms` and `Sort::substitute_sorts`
//! with argument slices of different lengths.
//!
//! The C API takes a single `size` for both arrays and reads `size` elements
//! from each, so the wrappers must reject mismatched lengths before calling it.
//! The C++ API they mirror rejects them too ("expected vectors of the same
//! arity in substitute").
//!
//! An empty `replacements` makes cvc5 read past the end of the Rust buffer and
//! segfault, so that case runs in a child process: the crash then fails only
//! that test instead of killing the whole test binary. A shorter but non-empty
//! `replacements` is the same out-of-bounds read, but is not tested because
//! the adjacent heap word is often NULL, which cvc5 reports as an error, so the
//! test would pass by luck.

use cvc5::{Kind, Sort, Term, TermManager};
use std::process::Command;

const CHILD_ENV: &str = "CVC5_SUBSTITUTE_CHILD";

fn int_consts(tm: &mut TermManager, names: &[&str]) -> Vec<Term> {
    names
        .iter()
        .map(|n| {
            let int = tm.integer_sort();
            tm.mk_const(int, n).unwrap()
        })
        .collect()
}

fn sum(tm: &mut TermManager, terms: &[Term]) -> Term {
    tm.mk_term(Kind::Add, terms).unwrap()
}

fn params(tm: &mut TermManager, names: &[&str]) -> Vec<Sort> {
    names.iter().map(|n| tm.mk_param_sort(n)).collect()
}

/// `(Array T U)` over the given parameter sorts.
fn array_of(tm: &mut TermManager, ps: &[Sort]) -> Sort {
    tm.mk_array_sort(ps[0].clone(), ps[1].clone()).unwrap()
}

/// Re-run `child` (a test in this binary) in its own process and require that
/// it exited normally and passed.
fn in_child(child: &str) {
    let status = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", child, "--nocapture", "--test-threads=1"])
        .env(CHILD_ENV, "1")
        .status()
        .unwrap();
    assert!(
        status.code().is_some(),
        "{child} was killed by a signal ({status}): cvc5 read past the end of `replacements`"
    );
    assert!(status.success(), "{child} failed ({status})");
}

fn is_child() -> bool {
    std::env::var_os(CHILD_ENV).is_some()
}

// ── Term::substitute_terms ──────────────────────────────────────────────────

#[test]
fn substitute_terms_rejects_extra_replacements() {
    let mut tm = TermManager::new();
    let v = int_consts(&mut tm, &["x", "y", "z"]);
    let t = sum(&mut tm, &[v[0].clone(), v[1].clone()]);
    // Before the fix the extra replacement is silently dropped and this is Ok.
    let r = t.substitute_terms(&[v[0].clone()], &[v[1].clone(), v[2].clone()]);
    assert!(r.is_err(), "mismatched lengths accepted: {r:?}");
}

#[test]
fn substitute_terms_rejects_missing_replacements() {
    in_child("child_substitute_terms_empty_replacements");
}

#[test]
fn child_substitute_terms_empty_replacements() {
    if !is_child() {
        return;
    }
    let mut tm = TermManager::new();
    let v = int_consts(&mut tm, &["x", "y"]);
    let t = sum(&mut tm, &v);
    assert!(t.substitute_terms(&[v[0].clone()], &[]).is_err());
}

#[test]
fn substitute_terms_equal_lengths_still_work() {
    let mut tm = TermManager::new();
    let v = int_consts(&mut tm, &["x", "y", "z"]);
    let t = sum(&mut tm, &[v[0].clone(), v[1].clone()]);
    let r = t
        .substitute_terms(&[v[0].clone(), v[1].clone()], &[v[2].clone(), v[2].clone()])
        .unwrap();
    assert_eq!(r, sum(&mut tm, &[v[2].clone(), v[2].clone()]));
    assert_eq!(t.substitute_terms(&[], &[]).unwrap(), t);
}

// ── Sort::substitute_sorts ──────────────────────────────────────────────────

#[test]
fn substitute_sorts_rejects_extra_replacements() {
    let mut tm = TermManager::new();
    let ps = params(&mut tm, &["T", "U"]);
    let arr = array_of(&mut tm, &ps);
    let r = arr.substitute_sorts(&[ps[0].clone()], &[tm.integer_sort(), tm.boolean_sort()]);
    assert!(r.is_err(), "mismatched lengths accepted: {r:?}");
}

#[test]
fn substitute_sorts_rejects_missing_replacements() {
    in_child("child_substitute_sorts_empty_replacements");
}

#[test]
fn child_substitute_sorts_empty_replacements() {
    if !is_child() {
        return;
    }
    let mut tm = TermManager::new();
    let ps = params(&mut tm, &["T", "U"]);
    let arr = array_of(&mut tm, &ps);
    assert!(arr.substitute_sorts(&[ps[0].clone()], &[]).is_err());
}
