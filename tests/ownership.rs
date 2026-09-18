//! Ownership tests: every wrapper outlives the owner that allocated it.
//!
//! Before cvc5 1.4.0 these programs could not even be written. `Term`, `Sort`,
//! `Stat`, `Command` and friends carried a lifetime tying them to their arena,
//! because `cvc5_term_manager_delete` freed every object the manager had
//! allocated. Every test below was a `compile_fail,E0597` case.
//!
//! 1.4.0 reference counts the arenas: deleting a term manager only drops the
//! caller's handle, and the arena is freed once its last object is released. The
//! wrappers own those references (`Drop` calls the matching `cvc5_*_release`),
//! so the lifetimes are gone and these are ordinary runtime tests.
//!
//! Each test deliberately *uses* the object after its owner is out of scope. If
//! the reference counting regressed, these would read freed memory rather than
//! fail an assertion, so they double as the payload for the AddressSanitizer /
//! LeakSanitizer run.
//!
//! Under LSan the only leak these produce is cvc5's own process-wide
//! `PreprocessingPassRegistry` singleton (a constant 3144 bytes in 41
//! allocations, allocated on the first `assert_formula`). One case is
//! deliberately avoided: calling `Proof::children` on a proof whose solver has
//! already been dropped leaks that child, because `cvc5_proof_t::export_proof`
//! returns a solver-owned reference while attached but a caller-owned one once
//! detached, and the C API gives no way to tell which. See `Proof::children`.

use cvc5::{InputParser, Kind, Solver, SymbolManager, TermManager};

// ── Term-manager-owned objects outlive the TermManager ─────────────────────

#[test]
fn terms_and_sorts_outlive_the_term_manager() {
    let (t, s) = {
        let tm = TermManager::new();
        let x = tm.mk_const(tm.integer_sort(), "x").unwrap();
        let zero = tm.mk_integer(0);
        (tm.mk_term(Kind::Gt, &[x, zero]).unwrap(), tm.integer_sort())
    };
    assert_eq!(t.kind(), Kind::Gt);
    assert_eq!(t.num_children(), 2);
    assert!(s.is_integer());
    // Deriving further objects from a detached term still works.
    assert!(t.child(0).unwrap().sort().is_integer());
}

#[test]
fn ops_and_datatypes_outlive_the_term_manager() {
    let (op, sort) = {
        let tm = TermManager::new();
        let op = tm
            .mk_op(Kind::BitvectorExtract, &[3, 0])
            .expect("extract op");
        let mut decl = tm.mk_dt_decl("Color", false);
        for name in ["red", "green"] {
            let c = tm.mk_dt_cons_decl(name);
            decl.add_constructor(&c).unwrap();
        }
        (op, tm.mk_dt_sort(&decl).unwrap())
    };
    assert_eq!(op.num_indices(), 2);
    let dt = sort.datatype().unwrap();
    assert_eq!(dt.num_constructors(), 2);
    // Datatype -> constructor -> term, all after the manager handle is gone.
    assert_eq!(dt.constructor(0).unwrap().name(), "red");
}

#[test]
fn statistics_outlive_the_term_manager() {
    let (stats, one) = {
        let tm = TermManager::new();
        let solver = Solver::new(&tm);
        solver.set_option("stats", "true").unwrap();
        solver.set_logic("QF_LIA").unwrap();
        assert!(solver.check_sat().unwrap().is_sat());
        let stats = solver.get_statistics();
        stats.iter_init(true, true);
        let one = stats.iter_next().unwrap();
        (stats, one)
    };
    assert!(!one.0.is_empty());
    let _ = format!("{}", one.1);
    // The iterator is still usable on the detached handle.
    stats.iter_init(true, true);
    assert!(stats.iter_has_next().unwrap());
}

// ── Solver-owned objects outlive the Solver ───────────────────────────────

#[test]
fn sat_result_outlives_the_solver() {
    let tm = TermManager::new();
    let r = {
        let solver = Solver::new(&tm);
        solver.set_logic("QF_LIA").unwrap();
        let x = tm.mk_const(tm.integer_sort(), "x").unwrap();
        let zero = tm.mk_integer(0);
        solver
            .assert_formula(tm.mk_term(Kind::Gt, &[x, zero]).unwrap())
            .unwrap();
        solver.check_sat().unwrap()
    };
    assert!(r.is_sat());
    assert!(!r.is_unsat());
    assert_eq!(format!("{r}"), "sat");
    // Cloning a detached result still adjusts the same refcount.
    assert!(r.clone().is_sat());
}

#[test]
fn proof_outlives_the_solver() {
    let tm = TermManager::new();
    let (proofs, children) = {
        let solver = Solver::new(&tm);
        solver.set_logic("QF_UF").unwrap();
        solver.set_option("produce-proofs", "true").unwrap();
        let b = tm.boolean_sort();
        let p = tm.mk_const(b.clone(), "p").unwrap();
        solver.assert_formula(p.clone()).unwrap();
        solver
            .assert_formula(tm.mk_term(Kind::Not, &[p]).unwrap())
            .unwrap();
        assert!(solver.check_sat().unwrap().is_unsat());
        let proofs = solver.get_proof(cvc5_sys::ProofComponent::Full).unwrap();
        // Taken while the solver is alive; see `Proof::children` for why the
        // detached call is only leak-free in this direction.
        let children = proofs[0].children();
        (proofs, children)
    };
    assert!(!proofs.is_empty());
    // `Proof::result` exports a Term through the proof's own term-manager
    // reference, which is the path that needs `d_tm` to still be alive.
    let _ = proofs[0].result();
    let _ = proofs[0].rule();
    for c in &children {
        let _ = c.result();
    }
}

#[test]
fn synth_result_and_grammar_outlive_the_solver() {
    let tm = TermManager::new();
    let (sr, g) = {
        let solver = Solver::new(&tm);
        solver.set_option("sygus", "true").unwrap();
        let int = tm.integer_sort();
        let start = tm.mk_var(int.clone(), "start").unwrap();
        let mut g = solver
            .mk_grammar(&[], std::slice::from_ref(&start))
            .unwrap();
        g.add_rule(start, tm.mk_integer(0)).unwrap();
        let f = solver.synth_fun_with_grammar("f", &[], int, &g).unwrap();
        solver
            .add_sygus_constraint(tm.mk_term(Kind::Equal, &[f.clone(), f]).unwrap())
            .unwrap();
        (solver.check_synth().unwrap(), g)
    };
    let _ = format!("{sr}");
    let _ = format!("{g}");
    assert!(!g.is_disequal(&g));
}

// ── Parser-owned commands outlive the InputParser ─────────────────────────

#[test]
fn commands_outlive_the_parser() {
    let tm = TermManager::new();
    let solver = Solver::new(&tm);
    let sm = SymbolManager::new(&tm);

    let cmds: Vec<_> = {
        let mut parser = InputParser::new(&solver, &sm);
        parser
            .set_str_input(
                cvc5_sys::InputLanguage::SmtLib26,
                "(set-logic QF_LIA)(declare-const a Int)(declare-const b Int)",
                "in",
            )
            .unwrap();
        // Collecting commands was impossible before: each one used to be a
        // pointer into a std::vector that the next parse could reallocate.
        let mut v = Vec::new();
        while let Ok(Some(c)) = parser.next_command() {
            v.push(c);
        }
        v
    };
    assert_eq!(cmds.len(), 3);
    assert_eq!(cmds[0].name(), "set-logic");
    // Invoking after the parser is gone: the command holds the parser alive.
    for c in &cmds {
        c.invoke(&solver, &sm).unwrap();
    }
}

// ── The owners themselves ─────────────────────────────────────────────────

#[test]
fn solver_and_symbol_manager_outlive_the_term_manager() {
    let (solver, sm) = {
        let tm = TermManager::new();
        (Solver::new(&tm), SymbolManager::new(&tm))
    };
    solver.set_logic("QF_LIA").unwrap();
    assert!(solver.check_sat().unwrap().is_sat());
    assert!(!sm.is_logic_set());
}

/// Drop order is unconstrained: the arena survives until its last object goes,
/// whichever order that happens in.
#[test]
fn arena_survives_arbitrary_drop_order() {
    let tm = TermManager::new();
    let a = tm.mk_true();
    let b = tm.mk_false();
    drop(a);
    drop(tm);
    assert!(!b.boolean_value().unwrap());
    let c = b.clone();
    drop(b);
    assert!(!c.boolean_value().unwrap());
}

/// Churn: if releasing an object failed to free the arena, this would grow
/// without bound. Paired with the leak-sanitizer run, it pins the other
/// direction — that objects are actually released, not merely kept alive.
#[test]
fn arenas_are_reclaimed_across_many_generations() {
    for _ in 0..200 {
        let tm = TermManager::new();
        let terms: Vec<_> = (0..50).map(|i| tm.mk_integer(i)).collect();
        drop(tm);
        assert_eq!(terms[49].int64_value().unwrap(), 49);
    }
}
