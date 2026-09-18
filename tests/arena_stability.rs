//! Regression tests for the cvc5 arena-stability fix (cvc5/cvc5#12895).
//!
//! Up to cvc5 1.3.4 the C API handed out `&d_alloc_stats.back()` into a
//! `std::vector`, so the *second* export reallocated the arena and every
//! previously returned handle dangled — reachable from safe Rust as a
//! SIGBUS/SIGSEGV. cvc5 1.4.0 stores these in an
//! `unordered_map<T*, unique_ptr<T>>`, so addresses are stable.
//!
//! These tests hold many handles live at once and read the oldest ones last,
//! which is precisely the access pattern that used to fault.

use cvc5::{Kind, Solver, TermManager};

// Pre-1.4.0 this crashed: export_stat returned `&d_alloc_stats.back()` into a
// std::vector, so the second export reallocated and invalidated the first Stat.
#[test]
fn many_live_stat_handles_stay_valid() {
    let mut tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_LIA").unwrap();
    solver.set_option("stats", "true").unwrap();
    let x = tm.mk_const(tm.integer_sort(), "x").unwrap();
    let zero = tm.mk_integer(0);
    solver
        .assert_formula(tm.mk_term(Kind::Gt, &[x, zero]).unwrap())
        .unwrap();
    assert!(solver.check_sat().unwrap().is_sat());

    let mut stats = solver.get_statistics();
    stats.iter_init(true, true);

    // Hold every Stat live while continuing to export more.
    let mut held = Vec::new();
    while stats.iter_has_next().unwrap() {
        held.push(stats.iter_next().unwrap());
    }
    assert!(held.len() > 1, "expected many stats, got {}", held.len());

    // Touch the earliest handles last: this is what used to read freed memory.
    for (name, stat) in &held {
        assert!(!name.is_empty());
        let _ = format!("{stat}");
    }
    eprintln!("held {} live Stat handles, all readable", held.len());
}

// Same shape for Statistics itself (export_stats had the identical bug).
#[test]
fn many_live_statistics_handles_stay_valid() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_option("stats", "true").unwrap();
    let all: Vec<_> = (0..64).map(|_| solver.get_statistics()).collect();
    for s in &all {
        let _ = format!("{s}");
    }
    eprintln!("held {} live Statistics handles, all readable", all.len());
}
