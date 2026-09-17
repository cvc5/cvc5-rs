//! Tests pinning down what the `'tm` lifetimes on [`cvc5::Term`], [`cvc5::Sort`]
//! and friends actually buy.
//!
//! These wrappers hold a borrowed pointer into the owning object's allocation
//! table and do *not* keep that owner alive: `cvc5_term_manager_delete` frees
//! every term and sort the manager allocated, and `cvc5_delete` frees every
//! result, proof and grammar the solver allocated. The lifetimes are the only
//! thing stopping a wrapper from outliving its owner.
//!
//! The negative cases — a `Term` outliving its `TermManager` — cannot be written
//! here, because they do not compile. They live as `compile_fail,E0597` doctests
//! on `Term` and `Sort` so the compiler checks them on every `cargo test`.
//! Removing `'tm` would make those doctests start compiling, which is the signal
//! that the safety property has been lost.

use cvc5::{Kind, Solver, TermManager};

/// Terms and sorts may be freely moved and stored while the manager is alive.
#[test]
fn wrappers_live_as_long_as_their_manager() {
    let tm = TermManager::new();

    let int = tm.integer_sort();
    let terms: Vec<_> = (0..8).map(|i| tm.mk_integer(i)).collect();
    let sorts: Vec<_> = (0..4).map(|_| int.clone()).collect();

    // Held across many further allocations, then still usable.
    for i in 0..64 {
        let _ = tm.mk_integer(i);
    }
    assert_eq!(terms.len(), 8);
    assert_eq!(terms[3].int64_value().unwrap(), 3);
    assert!(sorts.iter().all(|s| s.is_integer()));
}

/// A term outlives an inner scope as long as the manager encloses it — the
/// common shape the lifetime has to permit.
#[test]
fn term_outlives_inner_scope_under_live_manager() {
    let tm = TermManager::new();
    let t = {
        let zero = tm.mk_integer(0);
        let x = tm.mk_const(tm.integer_sort(), "x").unwrap();
        tm.mk_term(Kind::Gt, &[x, zero]).unwrap()
    };
    assert_eq!(t.kind(), Kind::Gt);
}

/// `TermManager` is reference counted, and terms from two handles to the same
/// manager coexist.
///
/// Note what this does *not* buy: `'tm` borrows the specific `TermManager`
/// binding, not the shared `Rc`, so a handle cannot be dropped while terms made
/// from it are alive (that is an `E0505`). The reference counting keeps the C
/// allocation alive; the lifetime is still what makes the borrow sound.
#[test]
fn cloned_manager_handles_coexist() {
    let tm = TermManager::new();
    let tm2 = tm.clone();
    let a = tm.mk_true();
    let b = tm2.mk_false();
    assert!(a.boolean_value().unwrap());
    assert!(!b.boolean_value().unwrap());
    assert_eq!(tm2.mk_integer(7).int64_value().unwrap(), 7);
}

/// Terms survive the `Solver` that observed them: they belong to the manager,
/// not the solver.
#[test]
fn terms_outlive_the_solver() {
    let tm = TermManager::new();
    let x = tm.mk_const(tm.integer_sort(), "x").unwrap();
    {
        let mut solver = Solver::new(&tm);
        solver.set_logic("QF_LIA").unwrap();
        let zero = tm.mk_integer(0);
        solver
            .assert_formula(tm.mk_term(Kind::Gt, &[x.clone(), zero]).unwrap())
            .unwrap();
        assert!(solver.check_sat().unwrap().is_sat());
    }
    assert_eq!(x.kind(), Kind::Constant);
    assert_eq!(x.symbol().unwrap(), "x");
}

/// Solver-owned objects (`SatResult`, `SynthResult`, `Proof`, `Grammar`) live in
/// the solver's allocation table, not the term manager's, so they borrow the
/// solver. They remain usable for as long as it is alive.
#[test]
fn solver_owned_objects_live_as_long_as_their_solver() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_LIA").unwrap();

    let x = tm.mk_const(tm.integer_sort(), "x").unwrap();
    let zero = tm.mk_integer(0);
    solver
        .assert_formula(tm.mk_term(Kind::Gt, &[x, zero]).unwrap())
        .unwrap();

    // Held across further solver use, then still readable.
    let first = solver.check_sat().unwrap();
    let second = solver.check_sat().unwrap();
    assert!(first.is_sat() && second.is_sat());
}

/// A `Grammar` borrows the solver, so methods that consume one take `&self`.
/// This is the shape that used to be impossible to express.
#[test]
fn grammar_usable_while_held() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_option("sygus", "true").unwrap();

    let int = tm.integer_sort();
    let x = tm.mk_var(int.clone(), "x").unwrap();
    let start = tm.mk_var(int.clone(), "start").unwrap();
    let mut g = solver
        .mk_grammar(std::slice::from_ref(&x), std::slice::from_ref(&start))
        .unwrap();
    g.add_rule(start.clone(), tm.mk_integer(0)).unwrap();

    let f = solver.synth_fun_with_grammar("f", &[x], int, &g).unwrap();
    assert!(!format!("{f}").is_empty());
}
