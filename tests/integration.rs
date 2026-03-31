use cvc5::{Kind, Solver, TermManager};

/// Helper: create a TermManager + Solver pair with common setup.
macro_rules! setup {
    ($tm:ident, $solver:ident, $logic:expr) => {
        let $tm = TermManager::new();
        let mut $solver = Solver::new(&$tm);
        $solver.set_logic($logic);
        $solver.set_option("produce-models", "true");
    };
}

// ── QF_LIA: linear integer arithmetic ──────────────────────────────

#[test]
fn qf_lia_sat() {
    setup!(tm, solver, "QF_LIA");

    let int = tm.integer_sort();
    let x = tm.mk_const(int.clone(), "x");
    let y = tm.mk_const(int, "y");

    // x > 0
    let zero = tm.mk_integer(0);
    let x_gt_0 = tm.mk_term(Kind::Gt, &[x.clone(), zero]);
    solver.assert_formula(x_gt_0);

    // y = x + 1
    let one = tm.mk_integer(1);
    let x_plus_1 = tm.mk_term(Kind::Add, &[x.clone(), one]);
    let y_eq = tm.mk_term(Kind::Equal, &[y.clone(), x_plus_1]);
    solver.assert_formula(y_eq);

    let result = solver.check_sat();
    assert!(result.is_sat());

    let x_val = solver.get_value(x);
    let y_val = solver.get_value(y);
    assert!(x_val.is_int32_value());
    assert!(y_val.is_int32_value());
    assert_eq!(y_val.int32_value(), x_val.int32_value() + 1);
}

#[test]
fn qf_lia_unsat() {
    setup!(tm, solver, "QF_LIA");

    let int = tm.integer_sort();
    let x = tm.mk_const(int, "x");
    let zero = tm.mk_integer(0);
    let one = tm.mk_integer(1);

    // x < 0 AND x > 0
    let lt = tm.mk_term(Kind::Lt, &[x.clone(), zero]);
    let gt = tm.mk_term(Kind::Gt, &[x, one]);
    solver.assert_formula(lt);
    solver.assert_formula(gt);

    // x < 0 AND x > 1 is unsat
    let result = solver.check_sat();
    assert!(result.is_unsat());
}

// ── QF_BV: bit-vectors ─────────────────────────────────────────────

#[test]
fn qf_bv_sat() {
    setup!(tm, solver, "QF_BV");

    let bv8 = tm.mk_bv_sort(8);
    let x = tm.mk_const(bv8, "x");
    let ff = tm.mk_bv(8, 0xFF);

    // x & 0xFF = x  (tautology for 8-bit, but let's also constrain x != 0)
    let and_term = tm.mk_term(Kind::BitvectorAnd, &[x.clone(), ff]);
    let eq = tm.mk_term(Kind::Equal, &[and_term, x.clone()]);
    solver.assert_formula(eq);

    let zero = tm.mk_bv(8, 0);
    let neq = tm.mk_term(Kind::Distinct, &[x.clone(), zero]);
    solver.assert_formula(neq);

    assert!(solver.check_sat().is_sat());
    let x_val = solver.get_value(x);
    assert!(x_val.is_bv_value());
    assert_ne!(x_val.bv_value(10), "0");
}

// ── QF_LRA: linear real arithmetic ─────────────────────────────────

#[test]
fn qf_lra_sat() {
    setup!(tm, solver, "QF_LRA");

    let real = tm.real_sort();
    let x = tm.mk_const(real, "x");
    let half = tm.mk_real_from_rational(1, 2);
    let eq = tm.mk_term(Kind::Equal, &[x.clone(), half]);
    solver.assert_formula(eq);

    assert!(solver.check_sat().is_sat());
    let val = solver.get_value(x);
    assert!(val.is_real_value());
    let (num, den) = val.real32_value();
    assert_eq!(num, 1);
    assert_eq!(den, 2);
}

// ── Boolean logic ──────────────────────────────────────────────────

#[test]
fn boolean_sat() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_UF");
    solver.set_option("produce-models", "true");

    let bool_sort = tm.boolean_sort();
    let a = tm.mk_const(bool_sort.clone(), "a");
    let b = tm.mk_const(bool_sort, "b");

    // a OR b
    let or_term = tm.mk_term(Kind::Or, &[a.clone(), b.clone()]);
    solver.assert_formula(or_term);

    // NOT (a AND b)
    let and_term = tm.mk_term(Kind::And, &[a.clone(), b.clone()]);
    let not_and = tm.mk_term(Kind::Not, &[and_term]);
    solver.assert_formula(not_and);

    assert!(solver.check_sat().is_sat());

    let a_val = solver.get_value(a).boolean_value();
    let b_val = solver.get_value(b).boolean_value();
    // Exactly one must be true
    assert!(a_val ^ b_val);
}

// ── Push / Pop scoping ─────────────────────────────────────────────

#[test]
fn push_pop() {
    setup!(tm, solver, "QF_LIA");

    let int = tm.integer_sort();
    let x = tm.mk_const(int, "x");
    let zero = tm.mk_integer(0);

    let x_gt_0 = tm.mk_term(Kind::Gt, &[x.clone(), zero.clone()]);
    solver.assert_formula(x_gt_0);

    solver.push(1);
    // Add contradictory constraint in inner scope
    let x_lt_0 = tm.mk_term(Kind::Lt, &[x.clone(), zero]);
    solver.assert_formula(x_lt_0);
    assert!(solver.check_sat().is_unsat());

    solver.pop(1);
    // After pop, only x > 0 remains — should be sat
    assert!(solver.check_sat().is_sat());
}

// ── check_sat_assuming ─────────────────────────────────────────────

#[test]
fn check_sat_assuming() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_option("produce-unsat-assumptions", "true");
    solver.set_logic("QF_UF");

    let bool_sort = tm.boolean_sort();
    let p = tm.mk_const(bool_sort.clone(), "p");
    let q = tm.mk_const(bool_sort, "q");

    // Assert p => q
    let imp = tm.mk_term(Kind::Implies, &[p.clone(), q.clone()]);
    solver.assert_formula(imp);

    // Assume p AND NOT q — should be unsat
    let not_q = tm.mk_term(Kind::Not, &[q]);
    let result = solver.check_sat_assuming(&[p.clone(), not_q.clone()]);
    assert!(result.is_unsat());
    let unsat_assumptions = solver.get_unsat_assumptions();
    assert_eq!(unsat_assumptions, vec![p.clone(), not_q.clone()]);
}

// ── Unsat core ─────────────────────────────────────────────────────

#[test]
fn unsat_core() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_UF");
    solver.set_option("produce-unsat-cores", "true");

    let bool_sort = tm.boolean_sort();
    let a = tm.mk_const(bool_sort.clone(), "a");
    let not_a = tm.mk_term(Kind::Not, std::slice::from_ref(&a));

    solver.assert_formula(a);
    solver.assert_formula(not_a);

    assert!(solver.check_sat().is_unsat());
    let core = solver.get_unsat_core();
    assert!(!core.is_empty());
}

// ── Arrays ─────────────────────────────────────────────────────────

#[test]
fn qf_alia_arrays() {
    setup!(tm, solver, "QF_ALIA");

    let int = tm.integer_sort();
    let arr_sort = tm.mk_array_sort(int.clone(), int.clone());
    let a = tm.mk_const(arr_sort, "a");
    let idx = tm.mk_integer(0);
    let val = tm.mk_integer(42);

    // store(a, 0, 42)
    let store = tm.mk_term(Kind::Store, &[a.clone(), idx.clone(), val.clone()]);
    // select(store(a, 0, 42), 0) = 42
    let sel = tm.mk_term(Kind::Select, &[store, idx]);
    let eq = tm.mk_term(Kind::Equal, &[sel, val]);
    solver.assert_formula(eq);

    assert!(solver.check_sat().is_sat());
}

// ── Datatypes ──────────────────────────────────────────────────────

#[test]
fn simple_datatype() {
    setup!(tm, solver, "ALL");

    // Declare: Color = { red | green | blue }
    let red = tm.mk_dt_cons_decl("red");
    let green = tm.mk_dt_cons_decl("green");
    let blue = tm.mk_dt_cons_decl("blue");

    let color_sort = solver.declare_dt("Color", &[red, green, blue]);
    let dt = color_sort.datatype();

    assert_eq!(dt.num_constructors(), 3);
    assert_eq!(dt.name(), "Color");

    let c = tm.mk_const(color_sort, "c");
    let red_term = tm.mk_term(Kind::ApplyConstructor, &[dt.constructor(0).term()]);
    let eq = tm.mk_term(Kind::Equal, &[c, red_term]);
    solver.assert_formula(eq);

    assert!(solver.check_sat().is_sat());
}

// ── Solver configuration ───────────────────────────────────────────

#[test]
fn solver_config() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_LIA");

    assert!(solver.is_logic_set());
    assert_eq!(solver.get_logic(), "QF_LIA");

    solver.set_option("produce-models", "true");
    assert_eq!(solver.get_option("produce-models"), "true");

    let version = solver.version();
    assert!(!version.is_empty());
}

// ── Result Display ─────────────────────────────────────────────────

#[test]
fn result_display() {
    setup!(tm, solver, "QF_LIA");
    let result = solver.check_sat();
    let s = format!("{result}");
    assert!(s == "sat" || s == "unsat" || s == "unknown");
}

#[test]
fn result_full_api() {
    // sat result
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_LIA");
    let sat = solver.check_sat();
    assert!(sat.is_sat());
    assert!(!sat.is_unsat());
    assert!(!sat.is_unknown());
    assert!(!sat.is_null());

    // copy, clone, eq, hash, debug
    let sat2 = sat.copy();
    assert_eq!(sat, sat2);
    assert!(!sat.is_disequal(&sat2));
    let sat3 = sat.clone();
    assert_eq!(sat, sat3);
    let dbg = format!("{sat:?}");
    assert!(!dbg.is_empty());
    let mut set = std::collections::HashSet::new();
    set.insert(sat2);

    // unsat result — compare with sat
    let tm2 = TermManager::new();
    let mut solver2 = Solver::new(&tm2);
    solver2.set_logic("QF_LIA");
    let b = tm2.boolean_sort();
    let a = tm2.mk_const(b.clone(), "a");
    let not_a = tm2.mk_term(Kind::Not, std::slice::from_ref(&a));
    solver2.assert_formula(a);
    solver2.assert_formula(not_a);
    let unsat = solver2.check_sat();
    assert!(unsat.is_unsat());
    assert!(sat.is_disequal(&unsat));
    assert_ne!(sat, unsat);

    // unknown result
    let tm3 = TermManager::new();
    let mut solver3 = Solver::new(&tm3);
    solver3.set_logic("QF_NIA");
    solver3.set_option("tlimit-per", "1");
    let int = tm3.integer_sort();
    let x = tm3.mk_const(int.clone(), "x");
    let y = tm3.mk_const(int.clone(), "y");
    let z = tm3.mk_const(int, "z");
    // Fermat-like: x^3 + y^3 = z^3, x,y,z > 1 — likely times out
    let two = tm3.mk_integer(2);
    let x3 = tm3.mk_term(Kind::Pow, &[x.clone(), tm3.mk_integer(3)]);
    let y3 = tm3.mk_term(Kind::Pow, &[y.clone(), tm3.mk_integer(3)]);
    let z3 = tm3.mk_term(Kind::Pow, &[z.clone(), tm3.mk_integer(3)]);
    let sum = tm3.mk_term(Kind::Add, &[x3, y3]);
    solver3.assert_formula(tm3.mk_term(Kind::Equal, &[sum, z3]));
    solver3.assert_formula(tm3.mk_term(Kind::Gt, &[x, two.clone()]));
    solver3.assert_formula(tm3.mk_term(Kind::Gt, &[y, two.clone()]));
    solver3.assert_formula(tm3.mk_term(Kind::Gt, &[z, two]));
    let unk = solver3.check_sat();
    if unk.is_unknown() {
        let expl = unk.unknown_explanation();
        let expl_dbg = format!("{expl:?}");
        assert!(!expl_dbg.is_empty());
    }
}

// ── Multiple solvers ───────────────────────────────────────────────

#[test]
fn multiple_solvers() {
    let tm = TermManager::new();
    let mut s1 = Solver::new(&tm);
    let mut s2 = Solver::new(&tm);

    s1.set_logic("QF_LIA");
    s2.set_logic("QF_LIA");

    let int = tm.integer_sort();
    let x = tm.mk_const(int, "x");
    let zero = tm.mk_integer(0);

    let gt = tm.mk_term(Kind::Gt, &[x.clone(), zero.clone()]);
    let lt = tm.mk_term(Kind::Lt, &[x, zero]);

    s1.assert_formula(gt);
    s2.assert_formula(lt);

    assert!(s1.check_sat().is_sat());
    assert!(s2.check_sat().is_sat());
}

// ── Op (indexed operators) ─────────────────────────────────────────

#[test]
fn op_bv_extract() {
    let tm = TermManager::new();
    let op = tm.mk_op(Kind::BitvectorExtract, &[3, 1]);
    assert_eq!(op.kind(), Kind::BitvectorExtract);
    assert!(op.is_indexed());
    assert_eq!(op.num_indices(), 2);
    assert_eq!(format!("{op}"), "(_ extract 3 1)");

    let op2 = op.copy();
    assert_eq!(op, op2);
    assert!(!op.is_disequal(&op2));

    let op3 = tm.mk_op(Kind::BitvectorExtract, &[7, 4]);
    assert!(op.is_disequal(&op3));

    // Use op to build a term
    let bv8 = tm.mk_bv_sort(8);
    let x = tm.mk_const(bv8, "x");
    let ext = tm.mk_term_from_op(op, &[x]);
    assert!(ext.has_op());
    assert_eq!(ext.op().kind(), Kind::BitvectorExtract);
    let dbg = format!("{:?}", tm.mk_op(Kind::BitvectorExtract, &[3, 1]));
    assert!(!dbg.is_empty());
    // hash
    let mut set = std::collections::HashSet::new();
    set.insert(op3);
    assert_eq!(set.len(), 1);
}

// ── Proof ──────────────────────────────────────────────────────────

#[test]
fn proof_basic() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_UF");
    solver.set_option("produce-proofs", "true");

    let b = tm.boolean_sort();
    let a = tm.mk_const(b.clone(), "a");
    let not_a = tm.mk_term(Kind::Not, std::slice::from_ref(&a));
    solver.assert_formula(a);
    solver.assert_formula(not_a);
    assert!(solver.check_sat().is_unsat());

    let proofs = solver.get_proof(cvc5_sys::Cvc5ProofComponent::Full);
    assert!(!proofs.is_empty());

    let p = &proofs[0];
    let rule = p.rule();
    assert!(!format!("{rule:?}").is_empty());
    let result = p.result();
    assert!(result.sort().is_boolean());
    let children = p.children();
    for child in &children {
        assert!(child.result().sort().is_boolean());
    }
    let arguments = p.arguments();
    for arg in &arguments {
        assert!(arg.id() > 0);
    }
    let p2 = p.copy();
    assert_eq!(p, &p2);
    assert!(!p.is_disequal(&p2));
    let dbg = format!("{:?}", p);
    assert!(!dbg.is_empty());
    // hash
    let mut set = std::collections::HashSet::new();
    set.insert(p.copy());
    assert_eq!(set.len(), 1);
}

// ── Statistics ─────────────────────────────────────────────────────

#[test]
fn statistics_basic() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_LIA");
    let int = tm.integer_sort();
    let x = tm.mk_const(int, "x");
    let zero = tm.mk_integer(0);
    let gt = tm.mk_term(Kind::Gt, &[x, zero]);
    solver.assert_formula(gt);
    solver.check_sat();

    let stats = solver.get_statistics();
    let display = format!("{stats}");
    assert!(!display.is_empty());
    let debug = format!("{stats:?}");
    assert!(!debug.is_empty());

    // iterate — after check_sat there must be at least one stat
    stats.iter_init(false, false);
    assert!(stats.iter_has_next());
    let (name, stat) = stats.iter_next();
    assert!(!name.is_empty());
    // iter_init(false, false) = skip internal, skip default → only changed non-internal stats
    assert!(!stat.is_internal());
    assert!(!stat.is_default());
    let stat_display = format!("{stat}");
    assert!(!stat_display.is_empty());
    let stat_debug = format!("{stat:?}");
    assert!(!stat_debug.is_empty());
    // at least one type predicate must be true
    assert!(stat.is_int() || stat.is_double() || stat.is_string() || stat.is_histogram());
}

// ── SynthResult ────────────────────────────────────────────────────

#[test]
fn synth_result_basic() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("LIA");
    solver.set_option("sygus", "true");

    let int = tm.integer_sort();
    let x = tm.mk_var(int.clone(), "x");
    let f = solver.synth_fun("f", std::slice::from_ref(&x), int.clone());

    let zero = tm.mk_integer(0);
    let fx = tm.mk_term(Kind::ApplyUf, &[f.clone(), x.clone()]);
    let ge = tm.mk_term(Kind::Geq, &[fx, zero]);
    solver.add_sygus_constraint(ge);

    let sr = solver.check_synth();
    assert!(!sr.is_null());
    assert!(sr.has_solution());
    assert!(!sr.has_no_solution());
    assert!(!sr.is_unknown());

    let sr2 = sr.copy();
    assert_eq!(sr, sr2);
    assert!(!sr.is_disequal(&sr2));
    let display = format!("{sr}");
    assert!(!display.is_empty());
    let debug = format!("{sr:?}");
    assert!(!debug.is_empty());
    // hash
    let mut set = std::collections::HashSet::new();
    std::hash::Hash::hash(&sr, &mut std::collections::hash_map::DefaultHasher::new());
    set.insert(());

    let sol = solver.get_synth_solution(f);
    // solution is a term (e.g. a lambda body); verify it's non-null
    assert!(!format!("{sol}").is_empty());
}

// ── Grammar ────────────────────────────────────────────────────────

#[test]
fn grammar_basic() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("LIA");
    solver.set_option("sygus", "true");

    let int = tm.integer_sort();
    let x = tm.mk_var(int.clone(), "x");
    let start = tm.mk_var(int.clone(), "start");

    let mut g = solver.mk_grammar(std::slice::from_ref(&x), std::slice::from_ref(&start));
    g.add_rule(start.clone(), tm.mk_integer(0));
    g.add_rules(start.clone(), &[tm.mk_integer(1), x.clone()]);
    g.add_any_constant(start.clone());
    g.add_any_variable(start.clone());

    let g2 = g.copy();
    assert_eq!(g, g2);
    assert!(!g.is_disequal(&g2));
    let display = format!("{g}");
    assert!(!display.is_empty());
    let debug = format!("{g:?}");
    assert!(!debug.is_empty());
    let mut set = std::collections::HashSet::new();
    std::hash::Hash::hash(&g, &mut std::collections::hash_map::DefaultHasher::new());
    set.insert(());

    let f = solver.synth_fun_with_grammar("f", &[x], int, &g);
    let sr = solver.check_synth();
    assert!(sr.has_solution());
    let sol = solver.get_synth_solution(f);
    assert!(!format!("{sol}").is_empty());
}

// ── Sort: type predicates ──────────────────────────────────────────

#[test]
fn sort_type_predicates() {
    let tm = TermManager::new();

    assert!(tm.boolean_sort().is_boolean());
    assert!(!tm.boolean_sort().is_integer());

    assert!(tm.integer_sort().is_integer());
    assert!(!tm.integer_sort().is_real());

    assert!(tm.real_sort().is_real());
    assert!(tm.string_sort().is_string());
    assert!(tm.regexp_sort().is_regexp());
    assert!(tm.rm_sort().is_rm());

    let bv32 = tm.mk_bv_sort(32);
    assert!(bv32.is_bv());
    assert!(!bv32.is_integer());

    let fp32 = tm.mk_fp_sort(8, 24);
    assert!(fp32.is_fp());

    let arr = tm.mk_array_sort(tm.integer_sort(), tm.boolean_sort());
    assert!(arr.is_array());
    assert!(!arr.is_set());

    let set = tm.mk_set_sort(tm.integer_sort());
    assert!(set.is_set());

    let bag = tm.mk_bag_sort(tm.integer_sort());
    assert!(bag.is_bag());

    let seq = tm.mk_sequence_sort(tm.integer_sort());
    assert!(seq.is_sequence());

    let tup = tm.mk_tuple_sort(&[tm.integer_sort(), tm.boolean_sort()]);
    assert!(tup.is_tuple());

    let nullable = tm.mk_nullable_sort(tm.integer_sort());
    assert!(nullable.is_nullable());

    let ff = tm.mk_ff_sort("7", 10);
    assert!(ff.is_ff());

    let fun = tm.mk_fun_sort(&[tm.integer_sort()], tm.boolean_sort());
    assert!(fun.is_fun());

    let pred = tm.mk_predicate_sort(&[tm.integer_sort()]);
    assert!(pred.is_predicate());
}

// ── Sort: bit-vector size ──────────────────────────────────────────

#[test]
fn sort_bv_size() {
    let tm = TermManager::new();
    assert_eq!(tm.mk_bv_sort(1).bv_size(), 1);
    assert_eq!(tm.mk_bv_sort(64).bv_size(), 64);
}

// ── Sort: floating-point sizes ─────────────────────────────────────

#[test]
fn sort_fp_sizes() {
    let tm = TermManager::new();
    let fp = tm.mk_fp_sort(8, 24);
    assert_eq!(fp.fp_exponent_size(), 8);
    assert_eq!(fp.fp_significand_size(), 24);
}

// ── Sort: finite field size ────────────────────────────────────────

#[test]
fn sort_ff_size() {
    let tm = TermManager::new();
    let ff = tm.mk_ff_sort("7", 10);
    assert_eq!(ff.ff_size(), "7");
}

// ── Sort: array accessors ──────────────────────────────────────────

#[test]
fn sort_array_accessors() {
    let tm = TermManager::new();
    let idx = tm.integer_sort();
    let elem = tm.boolean_sort();
    let arr = tm.mk_array_sort(idx.clone(), elem.clone());
    assert_eq!(arr.array_index_sort(), idx);
    assert_eq!(arr.array_element_sort(), elem);
}

// ── Sort: set/bag/sequence element sort ────────────────────────────

#[test]
fn sort_collection_element_sorts() {
    let tm = TermManager::new();
    let int = tm.integer_sort();

    assert_eq!(tm.mk_set_sort(int.clone()).set_element_sort(), int);
    assert_eq!(tm.mk_bag_sort(int.clone()).bag_element_sort(), int);
    assert_eq!(
        tm.mk_sequence_sort(int.clone()).sequence_element_sort(),
        int
    );
}

// ── Sort: tuple accessors ──────────────────────────────────────────

#[test]
fn sort_tuple_accessors() {
    let tm = TermManager::new();
    let int = tm.integer_sort();
    let bool_s = tm.boolean_sort();
    let tup = tm.mk_tuple_sort(&[int.clone(), bool_s.clone()]);
    assert_eq!(tup.tuple_length(), 2);
    let elems = tup.tuple_element_sorts();
    assert_eq!(elems.len(), 2);
    assert_eq!(elems[0], int);
    assert_eq!(elems[1], bool_s);
}

// ── Sort: nullable element sort ────────────────────────────────────

#[test]
fn sort_nullable_element() {
    let tm = TermManager::new();
    let int = tm.integer_sort();
    let nullable = tm.mk_nullable_sort(int.clone());
    assert_eq!(nullable.nullable_element_sort(), int);
}

// ── Sort: function sort accessors ──────────────────────────────────

#[test]
fn sort_fun_accessors() {
    let tm = TermManager::new();
    let int = tm.integer_sort();
    let bool_s = tm.boolean_sort();
    let fun = tm.mk_fun_sort(&[int.clone(), int.clone()], bool_s.clone());
    assert_eq!(fun.fun_arity(), 2);
    let dom = fun.fun_domain();
    assert_eq!(dom.len(), 2);
    assert_eq!(dom[0], int);
    assert_eq!(fun.fun_codomain(), bool_s);
}

// ── Sort: datatype sort accessors ──────────────────────────────────

#[test]
fn sort_dt_accessors() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("ALL");

    // Pair(fst: Int, snd: Bool)
    let mut cons = tm.mk_dt_cons_decl("Pair");
    cons.add_selector("fst", tm.integer_sort());
    cons.add_selector("snd", tm.boolean_sort());
    let pair_sort = solver.declare_dt("Pair", &[cons]);

    assert!(pair_sort.is_dt());
    assert_eq!(pair_sort.dt_arity(), 0);

    let dt = pair_sort.datatype();
    let ctor = dt.constructor(0);
    let ctor_sort = ctor.term().sort();
    assert!(ctor_sort.is_dt_constructor());
    assert_eq!(ctor_sort.dt_constructor_arity(), 2);
    assert_eq!(ctor_sort.dt_constructor_codomain(), pair_sort);
    let dom = ctor_sort.dt_constructor_domain();
    assert_eq!(dom.len(), 2);
    assert_eq!(dom[0], tm.integer_sort());
    assert_eq!(dom[1], tm.boolean_sort());

    // selector sort
    let sel = ctor.selector(0);
    let sel_sort = sel.term().sort();
    assert!(sel_sort.is_dt_selector());
    assert_eq!(sel_sort.dt_selector_domain(), pair_sort);
    assert_eq!(sel_sort.dt_selector_codomain(), tm.integer_sort());

    // tester sort
    let tester_sort = ctor.tester_term().sort();
    assert!(tester_sort.is_dt_tester());
    assert_eq!(tester_sort.dt_tester_domain(), pair_sort);
    assert!(tester_sort.dt_tester_codomain().is_boolean());
}

// ── Sort: uninterpreted sort ───────────────────────────────────────

#[test]
fn sort_uninterpreted() {
    let tm = TermManager::new();
    let u = tm.mk_uninterpreted_sort("U");
    assert!(u.is_uninterpreted_sort());
    assert!(u.has_symbol());
    assert_eq!(u.symbol(), "U");
    assert!(!u.is_integer());
}

// ── Sort: uninterpreted sort constructor ───────────────────────────

#[test]
fn sort_uninterpreted_sort_constructor() {
    let tm = TermManager::new();
    let usc = tm.mk_uninterpreted_sort_constructor_sort(2, "List");
    assert!(usc.is_uninterpreted_sort_constructor());
    assert_eq!(usc.uninterpreted_sort_constructor_arity(), 2);

    let inst = usc.instantiate(&[tm.integer_sort(), tm.boolean_sort()]);
    assert!(inst.is_instantiated());
    assert_eq!(inst.uninterpreted_sort_constructor(), usc);
    let params = inst.instantiated_parameters();
    assert_eq!(params.len(), 2);
    assert_eq!(params[0], tm.integer_sort());
}

// ── Sort: copy, equality, disequality ──────────────────────────────

#[test]
fn sort_copy_eq_diseq() {
    let tm = TermManager::new();
    let int = tm.integer_sort();
    let int2 = int.copy();
    assert_eq!(int, int2);
    assert!(!int.is_disequal(&int2));

    let bool_s = tm.boolean_sort();
    assert_ne!(int, bool_s);
    assert!(int.is_disequal(&bool_s));
}

// ── Sort: Display, Debug ───────────────────────────────────────────

#[test]
fn sort_display_debug() {
    let tm = TermManager::new();
    let int = tm.integer_sort();
    let s = format!("{int}");
    assert!(!s.is_empty());
    let d = format!("{int:?}");
    assert!(d.starts_with("Sort("));
}

// ── Sort: Hash ─────────────────────────────────────────────────────

#[test]
fn sort_hash() {
    let tm = TermManager::new();
    let mut set = std::collections::HashSet::new();
    set.insert(tm.integer_sort());
    set.insert(tm.integer_sort());
    set.insert(tm.boolean_sort());
    assert_eq!(set.len(), 2);
}

// ── Sort: Ord ──────────────────────────────────────────────────────

#[test]
fn sort_ord() {
    let tm = TermManager::new();
    let int = tm.integer_sort();
    let bool_s = tm.boolean_sort();
    // Just verify Ord doesn't panic and is consistent
    let cmp1 = int.cmp(&bool_s);
    let cmp2 = bool_s.cmp(&int);
    assert_eq!(cmp1, cmp2.reverse());
    assert_eq!(int.cmp(&int.copy()), std::cmp::Ordering::Equal);
}

// ── Sort: kind ─────────────────────────────────────────────────────

#[test]
fn sort_kind() {
    let tm = TermManager::new();
    assert_eq!(
        tm.boolean_sort().kind(),
        cvc5_sys::Cvc5SortKind::BooleanSort
    );
    assert_eq!(
        tm.integer_sort().kind(),
        cvc5_sys::Cvc5SortKind::IntegerSort
    );
    assert_eq!(tm.real_sort().kind(), cvc5_sys::Cvc5SortKind::RealSort);
}

// ── Sort: substitute ───────────────────────────────────────────────

#[test]
fn sort_substitute() {
    let tm = TermManager::new();
    let p = tm.mk_param_sort("T");
    let arr = tm.mk_array_sort(p.clone(), p.clone());
    let subst = arr.substitute(p.clone(), tm.integer_sort());
    assert!(subst.is_array());
    assert_eq!(subst.array_index_sort(), tm.integer_sort());
    assert_eq!(subst.array_element_sort(), tm.integer_sort());
}

// ── Sort: substitute_sorts (multiple) ──────────────────────────────

#[test]
fn sort_substitute_sorts() {
    let tm = TermManager::new();
    let t = tm.mk_param_sort("T");
    let u = tm.mk_param_sort("U");
    let arr = tm.mk_array_sort(t.clone(), u.clone());
    let subst = arr.substitute_sorts(&[t, u], &[tm.integer_sort(), tm.boolean_sort()]);
    assert_eq!(subst.array_index_sort(), tm.integer_sort());
    assert_eq!(subst.array_element_sort(), tm.boolean_sort());
}

// ── Sort: record sort ──────────────────────────────────────────────

#[test]
fn sort_record() {
    let tm = TermManager::new();
    let rec = tm.mk_record_sort(&["x", "y"], &[tm.integer_sort(), tm.boolean_sort()]);
    assert!(rec.is_record());
    assert!(rec.is_dt()); // records are datatypes internally
}

// ── Sort: abstract sort ────────────────────────────────────────────

#[test]
fn sort_abstract() {
    let tm = TermManager::new();
    let abs = tm.mk_abstract_sort(cvc5_sys::Cvc5SortKind::BitvectorSort);
    assert!(abs.is_abstract());
    assert_eq!(abs.abstract_kind(), cvc5_sys::Cvc5SortKind::BitvectorSort);
}

// ── Sort: Clone trait ──────────────────────────────────────────────

#[test]
fn sort_clone() {
    let tm = TermManager::new();
    let int = tm.integer_sort();
    let cloned = int.clone();
    assert_eq!(int, cloned);
}

// ── Sort: has_symbol for built-in sorts ────────────────────────────

#[test]
fn sort_builtin_no_symbol() {
    let tm = TermManager::new();
    // Built-in sorts like Int/Bool don't have user-given symbols
    assert!(!tm.integer_sort().has_symbol());
    assert!(!tm.boolean_sort().has_symbol());
}

// ── Datatype: properties on a simple enum ──────────────────────────

#[test]
fn dt_properties() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("ALL");

    let color_sort = solver.declare_dt(
        "Color",
        &[
            tm.mk_dt_cons_decl("red"),
            tm.mk_dt_cons_decl("green"),
            tm.mk_dt_cons_decl("blue"),
        ],
    );
    let dt = color_sort.datatype();

    assert!(!dt.is_parametric());
    assert!(!dt.is_codatatype());
    assert!(!dt.is_tuple());
    assert!(!dt.is_record());
    assert!(dt.is_finite());
    assert!(dt.is_well_founded());
}

// ── Datatype: copy, eq, hash, display, debug ───────────────────────

#[test]
fn dt_copy_eq_hash_display() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("ALL");

    let sort = solver.declare_dt("Unit", &[tm.mk_dt_cons_decl("unit")]);
    let dt = sort.datatype();
    let dt2 = dt.copy();
    assert_eq!(dt, dt2);

    let s = format!("{dt}");
    assert!(!s.is_empty());
    let d = format!("{dt:?}");
    assert!(d.starts_with("Datatype("));

    let mut set = std::collections::HashSet::new();
    set.insert(dt.clone());
    set.insert(dt2);
    assert_eq!(set.len(), 1);
}

// ── Datatype: constructor_by_name ──────────────────────────────────

#[test]
fn dt_constructor_by_name() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("ALL");

    let sort = solver.declare_dt("AB", &[tm.mk_dt_cons_decl("A"), tm.mk_dt_cons_decl("B")]);
    let dt = sort.datatype();
    let a = dt.constructor_by_name("A");
    assert_eq!(a.name(), "A");
    let b = dt.constructor_by_name("B");
    assert_eq!(b.name(), "B");
}

// ── Datatype: selector lookup by name on Datatype ──────────────────

#[test]
fn dt_selector_by_name_on_datatype() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("ALL");

    let mut cons = tm.mk_dt_cons_decl("Wrap");
    cons.add_selector("val", tm.integer_sort());
    let sort = solver.declare_dt("Wrapper", &[cons]);
    let dt = sort.datatype();

    let sel = dt.selector("val");
    assert_eq!(sel.name(), "val");
    assert_eq!(sel.codomain_sort(), tm.integer_sort());
}

// ── DatatypeConstructor: full API ──────────────────────────────────

#[test]
fn dt_constructor_api() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("ALL");

    let mut cons = tm.mk_dt_cons_decl("Pair");
    cons.add_selector("fst", tm.integer_sort());
    cons.add_selector("snd", tm.boolean_sort());
    let sort = solver.declare_dt("Pair", &[cons]);
    let dt = sort.datatype();
    let ctor = dt.constructor(0);

    assert_eq!(ctor.name(), "Pair");
    assert_eq!(ctor.num_selectors(), 2);

    // selector by index
    let fst = ctor.selector(0);
    assert_eq!(fst.name(), "fst");

    // selector by name
    let snd = ctor.selector_by_name("snd");
    assert_eq!(snd.name(), "snd");

    // tester
    let tester = ctor.tester_term();
    assert!(tester.sort().is_dt_tester());

    // term
    let ctor_term = ctor.term();
    assert!(ctor_term.sort().is_dt_constructor());

    // copy, eq, hash
    let ctor2 = ctor.clone();
    assert_eq!(ctor, ctor2);
    let mut set = std::collections::HashSet::new();
    set.insert(ctor.clone());
    set.insert(ctor2);
    assert_eq!(set.len(), 1);

    // display, debug
    let s = format!("{ctor}");
    assert!(!s.is_empty());
    let d = format!("{ctor:?}");
    assert!(d.starts_with("DatatypeConstructor("));
}

// ── DatatypeSelector: full API ─────────────────────────────────────

#[test]
fn dt_selector_api() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("ALL");

    let mut cons = tm.mk_dt_cons_decl("Box");
    cons.add_selector("contents", tm.integer_sort());
    let sort = solver.declare_dt("Box", &[cons]);
    let sel = sort.datatype().constructor(0).selector(0);

    assert_eq!(sel.name(), "contents");
    assert_eq!(sel.codomain_sort(), tm.integer_sort());

    // selector term
    let sel_term = sel.term();
    assert!(sel_term.sort().is_dt_selector());

    // updater term
    let upd = sel.updater_term();
    assert!(upd.sort().is_dt_updater());

    // copy, eq, hash
    let sel2 = sel.copy();
    assert_eq!(sel, sel2);
    let mut set = std::collections::HashSet::new();
    set.insert(sel.clone());
    set.insert(sel2);
    assert_eq!(set.len(), 1);

    // display, debug
    let s = format!("{sel}");
    assert!(!s.is_empty());
    let d = format!("{sel:?}");
    assert!(d.starts_with("DatatypeSelector("));
}

// ── DatatypeDecl: manual construction ──────────────────────────────

#[test]
fn dt_decl_manual() {
    let tm = TermManager::new();

    let mut decl = tm.mk_dt_decl("Maybe", false);
    assert_eq!(decl.name(), "Maybe");
    assert!(!decl.is_parametric());
    assert!(!decl.is_resolved());
    assert_eq!(decl.num_constructors(), 0);

    let nothing = tm.mk_dt_cons_decl("Nothing");
    decl.add_constructor(&nothing);
    assert_eq!(decl.num_constructors(), 1);

    let mut just = tm.mk_dt_cons_decl("Just");
    just.add_selector("val", tm.integer_sort());
    decl.add_constructor(&just);
    assert_eq!(decl.num_constructors(), 2);

    // copy, eq, hash
    let decl2 = decl.copy();
    assert_eq!(decl, decl2);
    let mut set = std::collections::HashSet::new();
    set.insert(decl.clone());
    set.insert(decl2);
    assert_eq!(set.len(), 1);

    // display, debug
    let s = format!("{decl}");
    assert!(!s.is_empty());
    let d = format!("{decl:?}");
    assert!(d.starts_with("DatatypeDecl("));

    // resolve via mk_dt_sort
    let sort = tm.mk_dt_sort(&decl);
    assert!(sort.is_dt());
    assert!(decl.is_resolved());
    let dt = sort.datatype();
    assert_eq!(dt.num_constructors(), 2);
    assert!(dt.is_well_founded());
}

// ── DatatypeConstructorDecl: display, eq, hash ─────────────────────

#[test]
fn dt_cons_decl_traits() {
    let tm = TermManager::new();
    let c1 = tm.mk_dt_cons_decl("Foo");
    let c2 = c1.clone();
    assert_eq!(c1, c2);

    let mut set = std::collections::HashSet::new();
    set.insert(c1.clone());
    set.insert(c2);
    assert_eq!(set.len(), 1);

    let s = format!("{c1}");
    assert!(!s.is_empty());
    let d = format!("{c1:?}");
    assert!(d.starts_with("DatatypeConstructorDecl("));
}

// ── Recursive datatype (self-referencing selector) ─────────────────

#[test]
fn dt_recursive_self_selector() {
    let tm = TermManager::new();

    let nil = tm.mk_dt_cons_decl("Nil");
    let mut cons = tm.mk_dt_cons_decl("Cons");
    cons.add_selector("head", tm.integer_sort());
    cons.add_selector_self("tail");

    let mut decl = tm.mk_dt_decl("IntList", false);
    decl.add_constructor(&nil);
    decl.add_constructor(&cons);

    let sort = tm.mk_dt_sort(&decl);
    let dt = sort.datatype();
    assert_eq!(dt.num_constructors(), 2);
    assert!(!dt.is_finite()); // recursive list over Int is not finite
    assert!(dt.is_well_founded());

    // tail selector codomain is IntList itself
    let cons_ctor = dt.constructor_by_name("Cons");
    let tail_sel = cons_ctor.selector_by_name("tail");
    assert_eq!(tail_sel.codomain_sort(), sort);
}

// ── Mutually recursive datatypes via mk_dt_sorts ──────────────────

#[test]
fn dt_mutual_recursion() {
    let tm = TermManager::new();

    // Tree = Leaf(Int) | Node(Forest)
    // Forest = Empty | Cons(Tree, Forest)
    let forest_unres = tm.mk_unresolved_dt_sort("Forest", 0);
    let tree_unres = tm.mk_unresolved_dt_sort("Tree", 0);

    let mut leaf = tm.mk_dt_cons_decl("Leaf");
    leaf.add_selector("val", tm.integer_sort());
    let mut node = tm.mk_dt_cons_decl("Node");
    node.add_selector("children", forest_unres.clone());
    let mut tree_decl = tm.mk_dt_decl("Tree", false);
    tree_decl.add_constructor(&leaf);
    tree_decl.add_constructor(&node);

    let empty = tm.mk_dt_cons_decl("Empty");
    let mut fcons = tm.mk_dt_cons_decl("FCons");
    fcons.add_selector("head", tree_unres);
    fcons.add_selector_unresolved("tail", "Forest");
    let mut forest_decl = tm.mk_dt_decl("Forest", false);
    forest_decl.add_constructor(&empty);
    forest_decl.add_constructor(&fcons);

    let sorts = tm.mk_dt_sorts(&[tree_decl, forest_decl]);
    assert_eq!(sorts.len(), 2);
    assert!(sorts[0].is_dt());
    assert!(sorts[1].is_dt());
    assert_eq!(sorts[0].datatype().name(), "Tree");
    assert_eq!(sorts[1].datatype().name(), "Forest");
}

// ── Parametric datatype ────────────────────────────────────────────

#[test]
fn dt_parametric() {
    let tm = TermManager::new();

    let t = tm.mk_param_sort("T");
    let mut decl = tm.mk_dt_decl_with_params("Opt", std::slice::from_ref(&t), false);
    assert!(decl.is_parametric());

    let none = tm.mk_dt_cons_decl("None");
    let mut some = tm.mk_dt_cons_decl("Some");
    some.add_selector("val", t.clone());
    decl.add_constructor(&none);
    decl.add_constructor(&some);

    let sort = tm.mk_dt_sort(&decl);
    let dt = sort.datatype();
    assert!(dt.is_parametric());
    let params = dt.parameters();
    assert_eq!(params.len(), 1);

    // instantiate with Int
    let inst = sort.instantiate(&[tm.integer_sort()]);
    assert!(inst.is_dt());
    let inst_dt = inst.datatype();
    assert_eq!(inst_dt.num_constructors(), 2);

    // instantiated_term on constructor of the *uninstantiated* sort
    let some_ctor = dt.constructor_by_name("Some");
    let inst_term = some_ctor.instantiated_term(inst.clone());
    assert!(inst_term.sort().is_dt_constructor());
}

// ── Tuple datatype (built-in) ──────────────────────────────────────

#[test]
fn dt_tuple() {
    let tm = TermManager::new();
    let tup_sort = tm.mk_tuple_sort(&[tm.integer_sort(), tm.boolean_sort()]);
    let dt = tup_sort.datatype();
    assert!(dt.is_tuple());
    assert!(!dt.is_record());
    assert_eq!(dt.num_constructors(), 1);
}

// ── Record datatype (built-in) ─────────────────────────────────────

#[test]
fn dt_record() {
    let tm = TermManager::new();
    let rec_sort = tm.mk_record_sort(&["x", "y"], &[tm.integer_sort(), tm.boolean_sort()]);
    let dt = rec_sort.datatype();
    assert!(dt.is_record());
    assert_eq!(dt.num_constructors(), 1);
    let ctor = dt.constructor(0);
    assert_eq!(ctor.num_selectors(), 2);
}

// ── Datatype used in solving ───────────────────────────────────────

#[test]
fn dt_solving_with_selectors() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("ALL");
    solver.set_option("produce-models", "true");

    let mut cons = tm.mk_dt_cons_decl("Pair");
    cons.add_selector("fst", tm.integer_sort());
    cons.add_selector("snd", tm.integer_sort());
    let sort = solver.declare_dt("Pair", &[cons]);
    let dt = sort.datatype();

    let ctor = dt.constructor(0);
    let fst_sel = ctor.selector_by_name("fst");
    let snd_sel = ctor.selector_by_name("snd");

    // Build: p = Pair(3, 7)
    let three = tm.mk_integer(3);
    let seven = tm.mk_integer(7);
    let pair_val = tm.mk_term(
        Kind::ApplyConstructor,
        &[ctor.term(), three.clone(), seven.clone()],
    );
    let p = tm.mk_const(sort, "p");
    solver.assert_formula(tm.mk_term(Kind::Equal, &[p.clone(), pair_val]));

    assert!(solver.check_sat().is_sat());

    // select fst(p) == 3
    let fst_app = tm.mk_term(Kind::ApplySelector, &[fst_sel.term(), p.clone()]);
    let fst_val = solver.get_value(fst_app);
    assert_eq!(fst_val.int32_value(), 3);

    // select snd(p) == 7
    let snd_app = tm.mk_term(Kind::ApplySelector, &[snd_sel.term(), p.clone()]);
    let snd_val = solver.get_value(snd_app);
    assert_eq!(snd_val.int32_value(), 7);

    // tester: isPair(p) == true
    let tester = tm.mk_term(Kind::ApplyTester, &[ctor.tester_term(), p]);
    let tester_val = solver.get_value(tester);
    assert!(tester_val.boolean_value());
}

// ── Term: kind, sort, id ───────────────────────────────────────────

#[test]
fn term_kind_sort_id() {
    let tm = TermManager::new();
    let x = tm.mk_const(tm.integer_sort(), "x");
    assert_eq!(x.kind(), Kind::Constant);
    assert!(x.sort().is_integer());
    assert!(x.id() > 0);

    let zero = tm.mk_integer(0);
    assert_eq!(zero.kind(), Kind::ConstInteger);

    let gt = tm.mk_term(Kind::Gt, &[x, zero]);
    assert_eq!(gt.kind(), Kind::Gt);
    assert!(gt.sort().is_boolean());
}

// ── Term: num_children, child ──────────────────────────────────────

#[test]
fn term_children() {
    let tm = TermManager::new();
    let x = tm.mk_const(tm.integer_sort(), "x");
    let y = tm.mk_const(tm.integer_sort(), "y");
    let add = tm.mk_term(Kind::Add, &[x.clone(), y.clone()]);
    assert_eq!(add.num_children(), 2);
    assert_eq!(add.child(0), x);
    assert_eq!(add.child(1), y);
}

// ── Term: has_symbol, symbol ───────────────────────────────────────

#[test]
fn term_symbol() {
    let tm = TermManager::new();
    let x = tm.mk_const(tm.integer_sort(), "x");
    assert!(x.has_symbol());
    assert_eq!(x.symbol(), "x");

    let zero = tm.mk_integer(0);
    assert!(!zero.has_symbol());
}

// ── Term: has_op, op ───────────────────────────────────────────────

#[test]
fn term_op() {
    let tm = TermManager::new();
    let bv8 = tm.mk_bv_sort(8);
    let x = tm.mk_const(bv8, "x");
    let op = tm.mk_op(Kind::BitvectorExtract, &[3, 0]);
    let ext = tm.mk_term_from_op(op.clone(), &[x]);
    assert!(ext.has_op());
    assert_eq!(ext.op(), op);

    // a plain constant has no op
    let c = tm.mk_integer(42);
    assert!(!c.has_op());
}

// ── Term: copy, eq, disequal, clone ────────────────────────────────

#[test]
fn term_copy_eq_diseq() {
    let tm = TermManager::new();
    let x = tm.mk_const(tm.integer_sort(), "x");
    let x2 = x.copy();
    assert_eq!(x, x2);
    assert!(!x.is_disequal(&x2));

    let y = tm.mk_const(tm.integer_sort(), "y");
    assert_ne!(x, y);
    assert!(x.is_disequal(&y));

    let cloned = x.clone();
    assert_eq!(x, cloned);
}

// ── Term: Display, Debug ───────────────────────────────────────────

#[test]
fn term_display_debug() {
    let tm = TermManager::new();
    let x = tm.mk_const(tm.integer_sort(), "x");
    let s = format!("{x}");
    assert!(s.contains("x"));
    let d = format!("{x:?}");
    assert!(d.starts_with("Term("));
}

// ── Term: Hash ─────────────────────────────────────────────────────

#[test]
fn term_hash() {
    let tm = TermManager::new();
    let x = tm.mk_const(tm.integer_sort(), "x");
    let mut set = std::collections::HashSet::new();
    set.insert(x.clone());
    set.insert(x.copy());
    assert_eq!(set.len(), 1);

    let y = tm.mk_const(tm.integer_sort(), "y");
    set.insert(y);
    assert_eq!(set.len(), 2);
}

// ── Term: Ord ──────────────────────────────────────────────────────

#[test]
fn term_ord() {
    let tm = TermManager::new();
    let x = tm.mk_const(tm.integer_sort(), "x");
    let y = tm.mk_const(tm.integer_sort(), "y");
    let cmp1 = x.cmp(&y);
    let cmp2 = y.cmp(&x);
    assert_eq!(cmp1, cmp2.reverse());
    assert_eq!(x.cmp(&x.copy()), std::cmp::Ordering::Equal);
}

// ── Term: boolean value ────────────────────────────────────────────

#[test]
fn term_boolean_value() {
    let tm = TermManager::new();
    let t = tm.mk_true();
    assert!(t.is_boolean_value());
    assert!(t.boolean_value());

    let f = tm.mk_false();
    assert!(f.is_boolean_value());
    assert!(!f.boolean_value());

    let b = tm.mk_boolean(true);
    assert!(b.boolean_value());
}

// ── Term: integer values (i32, u32, i64, u64, string) ──────────────

#[test]
fn term_integer_values() {
    let tm = TermManager::new();

    let pos = tm.mk_integer(42);
    assert!(pos.is_int32_value());
    assert_eq!(pos.int32_value(), 42);
    assert!(pos.is_uint32_value());
    assert_eq!(pos.uint32_value(), 42);
    assert!(pos.is_int64_value());
    assert_eq!(pos.int64_value(), 42);
    assert!(pos.is_uint64_value());
    assert_eq!(pos.uint64_value(), 42);
    assert!(pos.is_integer_value());
    assert_eq!(pos.integer_value(), "42");

    let neg = tm.mk_integer(-7);
    assert!(neg.is_int32_value());
    assert_eq!(neg.int32_value(), -7);
    assert!(!neg.is_uint32_value()); // negative can't be uint
    assert!(neg.is_int64_value());
    assert_eq!(neg.int64_value(), -7);

    // sign
    assert_eq!(pos.real_or_integer_value_sign(), 1);
    assert_eq!(neg.real_or_integer_value_sign(), -1);
    assert_eq!(tm.mk_integer(0).real_or_integer_value_sign(), 0);
}

// ── Term: integer from string ──────────────────────────────────────

#[test]
fn term_integer_from_str() {
    let tm = TermManager::new();
    let big = tm.mk_integer_from_str("999999999999999999");
    assert!(big.is_integer_value());
    assert_eq!(big.integer_value(), "999999999999999999");
}

// ── Term: real values ──────────────────────────────────────────────

#[test]
fn term_real_values() {
    let tm = TermManager::new();

    let half = tm.mk_real_from_rational(1, 2);
    assert!(half.is_real_value());
    let rv = half.real_value();
    assert!(rv.contains("1") && rv.contains("2"));

    assert!(half.is_real32_value());
    let (num, den) = half.real32_value();
    assert_eq!(num, 1);
    assert_eq!(den, 2);

    assert!(half.is_real64_value());
    let (num64, den64) = half.real64_value();
    assert_eq!(num64, 1);
    assert_eq!(den64, 2);

    let from_str = tm.mk_real_from_str("3/4");
    assert!(from_str.is_real_value());

    let from_int = tm.mk_real(5);
    assert!(from_int.is_real_value());
}

// ── Term: bit-vector value ─────────────────────────────────────────

#[test]
fn term_bv_value() {
    let tm = TermManager::new();
    let bv = tm.mk_bv(8, 0xAB);
    assert!(bv.is_bv_value());
    assert_eq!(bv.bv_value(10), "171"); // 0xAB = 171
    assert_eq!(bv.bv_value(16), "ab");
    assert_eq!(bv.bv_value(2), "10101011");

    let from_str = tm.mk_bv_from_str(8, "ff", 16);
    assert!(from_str.is_bv_value());
    assert_eq!(from_str.bv_value(10), "255");
}

// ── Term: string value ─────────────────────────────────────────────

#[test]
fn term_string_value() {
    let tm = TermManager::new();
    let s = tm.mk_string("hello", false);
    assert!(s.is_string_value());
    let chars = s.u32string_value();
    assert_eq!(chars.len(), 5);
    assert_eq!(chars[0], b'h' as u32);
}

// ── Term: floating-point special values ────────────────────────────

#[test]
fn term_fp_special_values() {
    let tm = TermManager::new();

    let pos_inf = tm.mk_fp_pos_inf(8, 24);
    assert!(pos_inf.is_fp_pos_inf());
    assert!(pos_inf.is_fp_value());
    assert!(!pos_inf.is_fp_nan());

    let neg_inf = tm.mk_fp_neg_inf(8, 24);
    assert!(neg_inf.is_fp_neg_inf());

    let nan = tm.mk_fp_nan(8, 24);
    assert!(nan.is_fp_nan());

    let pos_zero = tm.mk_fp_pos_zero(8, 24);
    assert!(pos_zero.is_fp_pos_zero());

    let neg_zero = tm.mk_fp_neg_zero(8, 24);
    assert!(neg_zero.is_fp_neg_zero());
}

// ── Term: floating-point value decomposition ───────────────────────

#[test]
fn term_fp_value() {
    let tm = TermManager::new();
    let pos_zero = tm.mk_fp_pos_zero(8, 24);
    assert!(pos_zero.is_fp_value());
    let (ew, sw, bv) = pos_zero.fp_value();
    assert_eq!(ew, 8);
    assert_eq!(sw, 24);
    assert!(bv.is_bv_value());
}

// ── Term: rounding mode value ──────────────────────────────────────

#[test]
fn term_rm_value() {
    let tm = TermManager::new();
    let rne = tm.mk_rm(cvc5_sys::Cvc5RoundingMode::RoundNearestTiesToEven);
    assert!(rne.is_rm_value());
    assert_eq!(
        rne.rm_value(),
        cvc5_sys::Cvc5RoundingMode::RoundNearestTiesToEven
    );
}

// ── Term: const array ──────────────────────────────────────────────

#[test]
fn term_const_array() {
    let tm = TermManager::new();
    let arr_sort = tm.mk_array_sort(tm.integer_sort(), tm.integer_sort());
    let zero = tm.mk_integer(0);
    let ca = tm.mk_const_array(arr_sort, zero.clone());
    assert!(ca.is_const_array());
    assert_eq!(ca.const_array_base(), zero);
}

// ── Term: finite field value ───────────────────────────────────────

#[test]
fn term_ff_value() {
    let tm = TermManager::new();
    let ff_sort = tm.mk_ff_sort("7", 10);
    let elem = tm.mk_ff_elem("3", ff_sort, 10);
    assert!(elem.is_ff_value());
    let v = elem.ff_value();
    assert!(v.contains("3"));
}

// ── Term: tuple value ──────────────────────────────────────────────

#[test]
fn term_tuple_value() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("ALL");
    solver.set_option("produce-models", "true");

    let tup = tm.mk_tuple(&[tm.mk_integer(1), tm.mk_true()]);
    let t = tm.mk_const(
        tm.mk_tuple_sort(&[tm.integer_sort(), tm.boolean_sort()]),
        "t",
    );
    solver.assert_formula(tm.mk_term(Kind::Equal, &[t.clone(), tup]));
    assert!(solver.check_sat().is_sat());

    let val = solver.get_value(t);
    assert!(val.is_tuple_value());
    let elems = val.tuple_value();
    assert_eq!(elems.len(), 2);
}

// ── Term: set value ────────────────────────────────────────────────

#[test]
fn term_set_value() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("ALL");
    solver.set_option("produce-models", "true");

    let int = tm.integer_sort();
    let set_sort = tm.mk_set_sort(int.clone());
    let s = tm.mk_const(set_sort.clone(), "s");

    // s = {1, 2}
    let one = tm.mk_integer(1);
    let two = tm.mk_integer(2);
    let s1 = tm.mk_term(Kind::SetSingleton, &[one]);
    let s2 = tm.mk_term(Kind::SetSingleton, &[two]);
    let union = tm.mk_term(Kind::SetUnion, &[s1, s2]);
    solver.assert_formula(tm.mk_term(Kind::Equal, &[s.clone(), union]));
    assert!(solver.check_sat().is_sat());

    let val = solver.get_value(s);
    assert!(val.is_set_value());
    let elems = val.set_value();
    assert_eq!(elems.len(), 2);
}

// ── Term: sequence value ───────────────────────────────────────────

#[test]
fn term_sequence_value() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("ALL");
    solver.set_option("produce-models", "true");

    let int = tm.integer_sort();
    let seq_sort = tm.mk_sequence_sort(int.clone());
    let s = tm.mk_const(seq_sort, "s");

    let one = tm.mk_integer(1);
    let unit = tm.mk_term(Kind::SeqUnit, &[one]);
    solver.assert_formula(tm.mk_term(Kind::Equal, &[s.clone(), unit]));
    assert!(solver.check_sat().is_sat());

    let val = solver.get_value(s);
    assert!(val.is_sequence_value());
    let elems = val.sequence_value();
    assert_eq!(elems.len(), 1);
}

// ── Term: substitute_term ──────────────────────────────────────────

#[test]
fn term_substitute_term() {
    let tm = TermManager::new();
    let x = tm.mk_const(tm.integer_sort(), "x");
    let y = tm.mk_const(tm.integer_sort(), "y");
    let zero = tm.mk_integer(0);

    // x > 0, substitute x -> y
    let gt = tm.mk_term(Kind::Gt, &[x.clone(), zero.clone()]);
    let subst = gt.substitute_term(x, y.clone());
    // result should be y > 0
    assert_eq!(subst.child(0), y);
    assert_eq!(subst.child(1), zero);
}

// ── Term: substitute_terms (multiple) ──────────────────────────────

#[test]
fn term_substitute_terms() {
    let tm = TermManager::new();
    let x = tm.mk_const(tm.integer_sort(), "x");
    let y = tm.mk_const(tm.integer_sort(), "y");
    let a = tm.mk_const(tm.integer_sort(), "a");
    let b = tm.mk_const(tm.integer_sort(), "b");

    let add = tm.mk_term(Kind::Add, &[x.clone(), y.clone()]);
    let subst = add.substitute_terms(&[x, y], &[a.clone(), b.clone()]);
    assert_eq!(subst.child(0), a);
    assert_eq!(subst.child(1), b);
}

// ── Term: uninterpreted sort value ─────────────────────────────────

#[test]
fn term_uninterpreted_sort_value() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_UF");
    solver.set_option("produce-models", "true");

    let u = tm.mk_uninterpreted_sort("U");
    let x = tm.mk_const(u, "x");
    assert!(solver.check_sat().is_sat());

    let val = solver.get_value(x);
    assert!(val.is_uninterpreted_sort_value());
    let s = val.uninterpreted_sort_value();
    assert!(!s.is_empty());
}

// ── Term: cardinality constraint ───────────────────────────────────

#[test]
fn term_cardinality_constraint() {
    let tm = TermManager::new();
    let u = tm.mk_uninterpreted_sort("U");
    let cc = tm.mk_cardinality_constraint(u.clone(), 3);
    assert!(cc.is_cardinality_constraint());
    let (sort, upper) = cc.cardinality_constraint();
    assert_eq!(sort, u);
    assert_eq!(upper, 3);
}

// ── Term: empty set / bag / sequence ───────────────────────────────

#[test]
fn term_empty_collections() {
    let tm = TermManager::new();
    let int = tm.integer_sort();

    let es = tm.mk_empty_set(tm.mk_set_sort(int.clone()));
    assert!(es.sort().is_set());

    let eb = tm.mk_empty_bag(tm.mk_bag_sort(int.clone()));
    assert!(eb.sort().is_bag());

    let eseq = tm.mk_empty_sequence(int);
    assert!(eseq.sort().is_sequence());
}

// ── Term: regexp constants ─────────────────────────────────────────

#[test]
fn term_regexp_constants() {
    let tm = TermManager::new();
    let all = tm.mk_regexp_all();
    assert!(all.sort().is_regexp());
    let allchar = tm.mk_regexp_allchar();
    assert!(allchar.sort().is_regexp());
    let none = tm.mk_regexp_none();
    assert!(none.sort().is_regexp());
}

// ── Term: universe set ─────────────────────────────────────────────

#[test]
fn term_universe_set() {
    let tm = TermManager::new();
    let us = tm.mk_universe_set(tm.mk_set_sort(tm.integer_sort()));
    assert!(us.sort().is_set());
}

// ── Term: pi ───────────────────────────────────────────────────────

#[test]
fn term_pi() {
    let tm = TermManager::new();
    let pi = tm.mk_pi();
    assert!(pi.sort().is_real());
}

// ── Term: skolem ───────────────────────────────────────────────────

#[test]
fn term_skolem() {
    let tm = TermManager::new();
    let id = cvc5_sys::Cvc5SkolemId::Purify;
    let n = tm.get_num_idxs_for_skolem_id(id);
    assert!(n > 0);

    let x = tm.mk_const(tm.integer_sort(), "x");
    let sk = tm.mk_skolem(id, &[x]);
    assert!(sk.is_skolem());
    assert_eq!(sk.skolem_id(), id);
    let indices = sk.skolem_indices();
    assert_eq!(indices.len(), n);
}

// ── Term: nullable ─────────────────────────────────────────────────

#[test]
fn term_nullable() {
    let tm = TermManager::new();
    let int = tm.integer_sort();
    let ns = tm.mk_nullable_sort(int.clone());

    let some = tm.mk_nullable_some(tm.mk_integer(42));
    assert!(some.sort().is_nullable());

    let null = tm.mk_nullable_null(ns);
    assert!(null.sort().is_nullable());

    let is_null = tm.mk_nullable_is_null(some.clone());
    assert!(is_null.sort().is_boolean());

    let is_some = tm.mk_nullable_is_some(some.clone());
    assert!(is_some.sort().is_boolean());

    let val = tm.mk_nullable_val(some);
    assert!(val.sort().is_integer());
}

// ── Term: fp from ieee ─────────────────────────────────────────────

#[test]
fn term_fp_from_ieee() {
    let tm = TermManager::new();
    let sign = tm.mk_bv(1, 0);
    let exp = tm.mk_bv(8, 0);
    let sig = tm.mk_bv(23, 0);
    let fp = tm.mk_fp_from_ieee(sign, exp, sig);
    assert!(fp.sort().is_fp());
    assert!(fp.is_fp_pos_zero());
}

// ── Term: mk_fp from bv ───────────────────────────────────────────

#[test]
fn term_fp_from_bv() {
    let tm = TermManager::new();
    let bv = tm.mk_bv(32, 0);
    let fp = tm.mk_fp(8, 24, bv);
    assert!(fp.sort().is_fp());
}

// ── TermManager: Default trait ─────────────────────────────────────

#[test]
fn tm_default() {
    let tm: TermManager = Default::default();
    // just verify it works like new()
    assert!(tm.boolean_sort().is_boolean());
}

// ── TermManager: mk_op_from_str ────────────────────────────────────

#[test]
fn tm_mk_op_from_str() {
    let tm = TermManager::new();
    let op = tm.mk_op_from_str(Kind::Divisible, "3");
    assert_eq!(op.kind(), Kind::Divisible);
    assert!(op.is_indexed());
    assert_eq!(op.num_indices(), 1);

    // use it in a term
    let x = tm.mk_const(tm.integer_sort(), "x");
    let div = tm.mk_term_from_op(op, &[x]);
    assert!(div.sort().is_boolean());
}

// ── TermManager: mk_sep_emp, mk_sep_nil ───────────────────────────

#[test]
fn tm_sep_terms() {
    let tm = TermManager::new();
    let emp = tm.mk_sep_emp();
    assert!(emp.sort().is_boolean());

    let nil = tm.mk_sep_nil(tm.integer_sort());
    assert!(nil.sort().is_integer());
}

// ── TermManager: mk_string_from_char32 ─────────────────────────────

#[test]
fn tm_mk_string_from_char32() {
    let tm = TermManager::new();
    // "AB" as null-terminated char32 array
    let chars: Vec<u32> = vec![0x41, 0x42, 0];
    let s = tm.mk_string_from_char32(&chars);
    assert!(s.is_string_value());
    let vals = s.u32string_value();
    assert_eq!(vals.len(), 2);
    assert_eq!(vals[0], 0x41);
    assert_eq!(vals[1], 0x42);
}

// ── TermManager: mk_nullable_lift ──────────────────────────────────

#[test]
fn tm_mk_nullable_lift() {
    let tm = TermManager::new();
    let a = tm.mk_nullable_some(tm.mk_integer(1));
    let b = tm.mk_nullable_some(tm.mk_integer(2));
    let lifted = tm.mk_nullable_lift(Kind::Add, &[a, b]);
    assert!(lifted.sort().is_nullable());
}

// ── TermManager: mk_var (bound variable) ──────────────────────────

#[test]
fn tm_mk_var() {
    let tm = TermManager::new();
    let v = tm.mk_var(tm.integer_sort(), "v");
    assert_eq!(v.kind(), Kind::Variable);
    assert!(v.has_symbol());
    assert_eq!(v.symbol(), "v");
    assert!(v.sort().is_integer());
}

// ── TermManager: mk_const ─────────────────────────────────────────

#[test]
fn tm_mk_const() {
    let tm = TermManager::new();
    let c = tm.mk_const(tm.boolean_sort(), "p");
    assert_eq!(c.kind(), Kind::Constant);
    assert!(c.has_symbol());
    assert_eq!(c.symbol(), "p");
    assert!(c.sort().is_boolean());
}

// ── TermManager: print_stats_safe ──────────────────────────────────

#[test]
fn tm_print_stats_safe() {
    let tm = TermManager::new();
    tm.print_stats_safe(2);
    // tm still usable after printing stats
    assert!(tm.boolean_sort().is_boolean());
}

// ── TermManager: mk_string with escape sequences ──────────────────

#[test]
fn tm_mk_string_escape() {
    let tm = TermManager::new();
    let s1 = tm.mk_string("hello", false);
    let s2 = tm.mk_string("hello", true);
    // both should produce string values
    assert!(s1.is_string_value());
    assert!(s2.is_string_value());
}

// ── TermManager: mk_bv bases ──────────────────────────────────────

#[test]
fn tm_mk_bv_from_str_bases() {
    let tm = TermManager::new();
    let from_bin = tm.mk_bv_from_str(8, "11111111", 2);
    let from_dec = tm.mk_bv_from_str(8, "255", 10);
    let from_hex = tm.mk_bv_from_str(8, "ff", 16);
    assert_eq!(from_bin.bv_value(10), "255");
    assert_eq!(from_dec.bv_value(10), "255");
    assert_eq!(from_hex.bv_value(10), "255");
}

// ── TermManager: all rounding modes ────────────────────────────────

#[test]
fn tm_all_rounding_modes() {
    use cvc5_sys::Cvc5RoundingMode::*;
    let tm = TermManager::new();
    for rm in [
        RoundNearestTiesToEven,
        RoundTowardPositive,
        RoundTowardNegative,
        RoundTowardZero,
        RoundNearestTiesToAway,
    ] {
        let t = tm.mk_rm(rm);
        assert!(t.is_rm_value());
        assert_eq!(t.rm_value(), rm);
    }
}

// ══════════════════════════════════════════════════════════════════
// Additional solver.rs coverage tests
// ══════════════════════════════════════════════════════════════════

// ── set_info / get_info ────────────────────────────────────────────

#[test]
fn solver_set_get_info() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_info("source", "test-suite");
    let info = solver.get_info("name");
    assert!(!info.is_empty());
}

// ── get_option_names ───────────────────────────────────────────────

#[test]
fn solver_get_option_names() {
    let tm = TermManager::new();
    let solver = Solver::new(&tm);
    let names = solver.get_option_names();
    assert!(!names.is_empty());
    assert!(names.iter().any(|n| n == "produce-models"));
}

// ── get_option_info / option_info_to_string ────────────────────────

#[test]
fn solver_option_info() {
    let tm = TermManager::new();
    let solver = Solver::new(&tm);
    let info = solver.get_option_info("produce-models");
    let s = Solver::option_info_to_string(&info);
    assert!(!s.is_empty());
}

// ── simplify ───────────────────────────────────────────────────────

#[test]
fn solver_simplify() {
    setup!(tm, solver, "QF_LIA");
    let t = tm.mk_term(Kind::And, &[tm.mk_true(), tm.mk_true()]);
    let simplified = solver.simplify(t, false);
    assert!(simplified.is_boolean_value());
    assert!(simplified.boolean_value());
}

// ── get_assertions ─────────────────────────────────────────────────

#[test]
fn solver_get_assertions() {
    setup!(tm, solver, "QF_LIA");
    let int = tm.integer_sort();
    let x = tm.mk_const(int, "x");
    let zero = tm.mk_integer(0);
    let gt = tm.mk_term(Kind::Gt, &[x, zero]);
    solver.assert_formula(gt.clone());
    let assertions = solver.get_assertions();
    assert_eq!(assertions.len(), 1);
    assert_eq!(assertions[0], gt);
}

// ── get_values ─────────────────────────────────────────────────────

#[test]
fn solver_get_values() {
    setup!(tm, solver, "QF_LIA");
    let int = tm.integer_sort();
    let x = tm.mk_const(int.clone(), "x");
    let y = tm.mk_const(int, "y");
    let one = tm.mk_integer(1);
    let two = tm.mk_integer(2);
    solver.assert_formula(tm.mk_term(Kind::Equal, &[x.clone(), one]));
    solver.assert_formula(tm.mk_term(Kind::Equal, &[y.clone(), two]));
    assert!(solver.check_sat().is_sat());
    let vals = solver.get_values(&[x, y]);
    assert_eq!(vals.len(), 2);
    assert_eq!(vals[0].int32_value(), 1);
    assert_eq!(vals[1].int32_value(), 2);
}

// ── reset_assertions ───────────────────────────────────────────────

#[test]
fn solver_reset_assertions() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_option("produce-models", "true");
    solver.set_logic("QF_LIA");
    let int = tm.integer_sort();
    let x = tm.mk_const(int, "x");
    let zero = tm.mk_integer(0);
    // assert contradiction
    solver.assert_formula(tm.mk_term(Kind::Gt, &[x.clone(), zero.clone()]));
    solver.assert_formula(tm.mk_term(Kind::Lt, &[x, zero]));
    assert!(solver.check_sat().is_unsat());
    solver.reset_assertions();
    // after reset, no assertions → sat
    assert!(solver.check_sat().is_sat());
}

// ── declare_fun ────────────────────────────────────────────────────

#[test]
fn solver_declare_fun() {
    setup!(tm, solver, "QF_UFLIA");
    let int = tm.integer_sort();
    let f = solver.declare_fun("f", std::slice::from_ref(&int), int.clone());
    let x = tm.mk_const(int, "x");
    let fx = tm.mk_term(Kind::ApplyUf, &[f, x]);
    assert!(fx.sort().is_integer());
}

// ── declare_sort ───────────────────────────────────────────────────

#[test]
fn solver_declare_sort() {
    setup!(tm, solver, "QF_UF");
    let u = solver.declare_sort("U", 0);
    assert!(u.is_uninterpreted_sort());
}

// ── define_fun ─────────────────────────────────────────────────────

#[test]
fn solver_define_fun() {
    setup!(tm, solver, "QF_LIA");
    let int = tm.integer_sort();
    let x = tm.mk_var(int.clone(), "x");
    let one = tm.mk_integer(1);
    let body = tm.mk_term(Kind::Add, &[x.clone(), one]);
    let f = solver.define_fun("inc", &[x], int, body, true);
    assert!(f.sort().is_fun());
}

// ── define_fun_rec ─────────────────────────────────────────────────

#[test]
fn solver_define_fun_rec() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("UFLIA");
    solver.set_option("produce-models", "true");

    let int = tm.integer_sort();
    let x = tm.mk_var(int.clone(), "x");
    let zero = tm.mk_integer(0);
    // define f(x) = if x <= 0 then 0 else x + f(x-1)
    // For simplicity, just define f(x) = 0 recursively
    let f = solver.define_fun_rec("f", &[x], int, zero, true);
    assert!(f.sort().is_fun());
}

// ── define_fun_rec_from_const ──────────────────────────────────────

#[test]
fn solver_define_fun_rec_from_const() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("UFLIA");
    solver.set_option("produce-models", "true");

    let int = tm.integer_sort();
    let fun = solver.declare_fun("g", std::slice::from_ref(&int), int.clone());
    let x = tm.mk_var(int.clone(), "x");
    let zero = tm.mk_integer(0);
    let f = solver.define_fun_rec_from_const(fun, &[x], zero, true);
    assert!(f.sort().is_fun());
}

// ── define_funs_rec ────────────────────────────────────────────────

#[test]
fn solver_define_funs_rec() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("UFLIA");

    let int = tm.integer_sort();
    let f = solver.declare_fun("f", std::slice::from_ref(&int), int.clone());
    let g = solver.declare_fun("g", std::slice::from_ref(&int), int.clone());
    let xf = tm.mk_var(int.clone(), "xf");
    let xg = tm.mk_var(int.clone(), "xg");
    let zero = tm.mk_integer(0);
    solver.define_funs_rec(&[f, g], &[&[xf], &[xg]], &[zero.clone(), zero], true);
    // definitions accepted — verify solver recorded them
    assert_eq!(solver.get_assertions().len(), 2);
}

// ── get_model ──────────────────────────────────────────────────────

#[test]
fn solver_get_model() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_UF");
    solver.set_option("produce-models", "true");

    let u = solver.declare_sort("U", 0);
    let x = tm.mk_const(u.clone(), "x");
    assert!(solver.check_sat().is_sat());
    let model = solver.get_model(&[u], &[x]);
    assert!(!model.is_empty());
}

// ── block_model / block_model_values ───────────────────────────────

#[test]
fn solver_block_model() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_LIA");
    solver.set_option("produce-models", "true");

    let int = tm.integer_sort();
    let x = tm.mk_const(int.clone(), "x");
    let zero = tm.mk_integer(0);
    let ten = tm.mk_integer(10);
    solver.assert_formula(tm.mk_term(Kind::Geq, &[x.clone(), zero]));
    solver.assert_formula(tm.mk_term(Kind::Leq, &[x.clone(), ten]));
    assert!(solver.check_sat().is_sat());
    let v1 = solver.get_value(x.clone()).int32_value();
    solver.block_model(cvc5_sys::Cvc5BlockModelsMode::Values);
    assert!(solver.check_sat().is_sat());
    let v2 = solver.get_value(x).int32_value();
    assert_ne!(v1, v2);
}

#[test]
fn solver_block_model_values() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_LIA");
    solver.set_option("produce-models", "true");

    let int = tm.integer_sort();
    let x = tm.mk_const(int, "x");
    let zero = tm.mk_integer(0);
    let ten = tm.mk_integer(10);
    solver.assert_formula(tm.mk_term(Kind::Geq, &[x.clone(), zero]));
    solver.assert_formula(tm.mk_term(Kind::Leq, &[x.clone(), ten]));
    assert!(solver.check_sat().is_sat());
    let v1 = solver.get_value(x.clone()).int32_value();
    solver.block_model_values(std::slice::from_ref(&x));
    assert!(solver.check_sat().is_sat());
    let v2 = solver.get_value(x).int32_value();
    assert_ne!(v1, v2);
}

// ── get_model_domain_elements / is_model_core_symbol ───────────────

#[test]
fn solver_model_domain_elements() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_UF");
    solver.set_option("produce-models", "true");

    let u = solver.declare_sort("U", 0);
    let x = tm.mk_const(u.clone(), "x");
    let y = tm.mk_const(u.clone(), "y");
    solver.assert_formula(tm.mk_term(Kind::Distinct, &[x.clone(), y.clone()]));
    assert!(solver.check_sat().is_sat());
    let elems = solver.get_model_domain_elements(u);
    assert!(elems.len() >= 2);
}

#[test]
fn solver_is_model_core_symbol() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_LIA");
    solver.set_option("produce-models", "true");
    solver.set_option("model-cores", "simple");

    let int = tm.integer_sort();
    let x = tm.mk_const(int.clone(), "x");
    let y = tm.mk_const(int, "y");
    let one = tm.mk_integer(1);
    solver.assert_formula(tm.mk_term(Kind::Equal, &[x.clone(), one]));
    assert!(solver.check_sat().is_sat());
    // x is constrained, y is not — at minimum y should not be in core
    // x_in_core may vary by solver heuristics, just exercise the API
    solver.is_model_core_symbol(x);
    assert!(!solver.is_model_core_symbol(y));
}

// ── get_unsat_core_lemmas ──────────────────────────────────────────

#[test]
fn solver_unsat_core_lemmas() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_UF");
    solver.set_option("produce-unsat-cores", "true");
    solver.set_option("produce-proofs", "true");

    let b = tm.boolean_sort();
    let a = tm.mk_const(b, "a");
    let not_a = tm.mk_term(Kind::Not, std::slice::from_ref(&a));
    solver.assert_formula(a);
    solver.assert_formula(not_a);
    assert!(solver.check_sat().is_unsat());
    let lemmas = solver.get_unsat_core_lemmas();
    // lemmas is a valid vec; each element should be a boolean term
    for lemma in &lemmas {
        assert!(lemma.sort().is_boolean());
    }
}

// ── proof_to_string ────────────────────────────────────────────────

#[test]
fn solver_proof_to_string() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_UF");
    solver.set_option("produce-proofs", "true");

    let b = tm.boolean_sort();
    let a = tm.mk_const(b, "a");
    let not_a = tm.mk_term(Kind::Not, std::slice::from_ref(&a));
    solver.assert_formula(a.clone());
    solver.assert_formula(not_a.clone());
    assert!(solver.check_sat().is_unsat());

    let proofs = solver.get_proof(cvc5_sys::Cvc5ProofComponent::Full);
    assert!(!proofs.is_empty());
    let s = solver.proof_to_string(
        proofs[0].copy(),
        cvc5_sys::Cvc5ProofFormat::Default,
        &[a, not_a],
        &["a", "not_a"],
    );
    assert!(!s.is_empty());
}

// ── get_learned_literals ───────────────────────────────────────────

#[test]
fn solver_get_learned_literals() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_LIA");
    solver.set_option("produce-models", "true");
    solver.set_option("produce-learned-literals", "true");

    let int = tm.integer_sort();
    let x = tm.mk_const(int, "x");
    let zero = tm.mk_integer(0);
    solver.assert_formula(tm.mk_term(Kind::Gt, &[x, zero]));
    assert!(solver.check_sat().is_sat());
    let lits = solver.get_learned_literals(cvc5_sys::Cvc5LearnedLitType::Input);
    // returned vec may be empty but should not panic; verify it's a valid vec
    for lit in &lits {
        assert!(lit.sort().is_boolean());
    }
}

// ── get_difficulty ─────────────────────────────────────────────────

#[test]
fn solver_get_difficulty() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_LIA");
    solver.set_option("produce-difficulty", "true");

    let int = tm.integer_sort();
    let x = tm.mk_const(int, "x");
    let zero = tm.mk_integer(0);
    solver.assert_formula(tm.mk_term(Kind::Gt, &[x, zero]));
    solver.check_sat();
    let (inputs, values) = solver.get_difficulty();
    assert_eq!(inputs.len(), values.len());
}

// ── declare_pool ───────────────────────────────────────────────────

#[test]
fn solver_declare_pool() {
    setup!(tm, solver, "ALL");
    let int = tm.integer_sort();
    let one = tm.mk_integer(1);
    let two = tm.mk_integer(2);
    let pool = solver.declare_pool("p", int.clone(), &[one, two]);
    assert!(pool.sort().is_set());
}

// ── get_interpolant ────────────────────────────────────────────────

#[test]
fn solver_get_interpolant() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_LIA");
    solver.set_option("produce-interpolants", "true");

    let int = tm.integer_sort();
    let x = tm.mk_const(int.clone(), "x");
    let y = tm.mk_const(int, "y");
    let zero = tm.mk_integer(0);
    // A: x > 0 AND x = y
    solver.assert_formula(tm.mk_term(Kind::Gt, &[x.clone(), zero.clone()]));
    solver.assert_formula(tm.mk_term(Kind::Equal, &[x, y.clone()]));
    // B (conjecture): y > 0
    let conj = tm.mk_term(Kind::Gt, &[y, zero]);
    let interp = solver.get_interpolant(conj);
    assert!(interp.unwrap().sort().is_boolean());
}

// ── get_abduct ─────────────────────────────────────────────────────

#[test]
fn solver_get_abduct() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_LIA");
    solver.set_option("produce-abducts", "true");

    let int = tm.integer_sort();
    let x = tm.mk_const(int, "x");
    let zero = tm.mk_integer(0);
    // conjecture: x > 0
    let conj = tm.mk_term(Kind::Gt, &[x, zero]);
    let abd = solver.get_abduct(conj);
    assert!(abd.unwrap().sort().is_boolean());
}

// ── declare_sygus_var / get_sygus_constraints / get_sygus_assumptions ──

#[test]
fn solver_sygus_var_and_queries() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("LIA");
    solver.set_option("sygus", "true");

    let int = tm.integer_sort();
    let x = solver.declare_sygus_var("x", int.clone());
    assert!(x.sort().is_integer());

    let f = solver.synth_fun("f", &[], int.clone());
    let zero = tm.mk_integer(0);
    // constraint: f() >= 0
    let ge = tm.mk_term(Kind::Geq, &[f.clone(), zero.clone()]);
    solver.add_sygus_constraint(ge);
    let constraints = solver.get_sygus_constraints();
    assert_eq!(constraints.len(), 1);

    // assumption
    let assume = tm.mk_term(Kind::Gt, &[f.clone(), zero]);
    solver.add_sygus_assume(assume);
    let assumptions = solver.get_sygus_assumptions();
    assert_eq!(assumptions.len(), 1);

    let sr = solver.check_synth();
    assert!(sr.has_solution());
}

// ── get_synth_solutions (multiple) ─────────────────────────────────

#[test]
fn solver_get_synth_solutions() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("LIA");
    solver.set_option("sygus", "true");

    let int = tm.integer_sort();
    let f = solver.synth_fun("f", &[], int.clone());
    let g = solver.synth_fun("g", &[], int.clone());
    let zero = tm.mk_integer(0);
    solver.add_sygus_constraint(tm.mk_term(Kind::Geq, &[f.clone(), zero.clone()]));
    solver.add_sygus_constraint(tm.mk_term(Kind::Geq, &[g.clone(), zero]));
    assert!(solver.check_synth().has_solution());
    let sols = solver.get_synth_solutions(&[f, g]);
    assert_eq!(sols.len(), 2);
}

// ── is_output_on ───────────────────────────────────────────────────

#[test]
fn solver_is_output_on() {
    let tm = TermManager::new();
    let solver = Solver::new(&tm);
    // by default, most output tags are off
    assert!(!solver.is_output_on("inst"));
}

// ── print_stats_safe ───────────────────────────────────────────────

#[test]
fn solver_print_stats_safe() {
    let tm = TermManager::new();
    let solver = Solver::new(&tm);
    solver.print_stats_safe(2);
    // solver still usable after printing stats
    assert!(!solver.version().is_empty());
}

// ── separation logic ──────────────────────────────────────────────

#[test]
fn solver_sep_logic() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_ALL");
    solver.set_option("produce-models", "true");
    solver.set_option("incremental", "false");

    let int = tm.integer_sort();
    solver.declare_sep_heap(int.clone(), int.clone());

    let x = tm.mk_const(int.clone(), "x");
    let one = tm.mk_integer(1);
    // x pto 1
    let pto = tm.mk_term(Kind::SepPto, &[x.clone(), one]);
    solver.assert_formula(pto);
    assert!(solver.check_sat().is_sat());

    let heap = solver.get_value_sep_heap();
    assert!(!format!("{heap}").is_empty());
    let nil = solver.get_value_sep_nil();
    assert!(!format!("{nil}").is_empty());
}

// ── get_instantiations ─────────────────────────────────────────────

#[test]
fn solver_get_instantiations() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("UFLIA");
    solver.set_option("produce-models", "true");

    let int = tm.integer_sort();
    let x = tm.mk_var(int.clone(), "x");
    let f = solver.declare_fun("f", std::slice::from_ref(&int), int.clone());
    let fx = tm.mk_term(Kind::ApplyUf, &[f.clone(), x.clone()]);
    let zero = tm.mk_integer(0);
    let ge = tm.mk_term(Kind::Geq, &[fx, zero]);
    // forall x. f(x) >= 0
    let bound = tm.mk_term(Kind::VariableList, &[x]);
    let forall = tm.mk_term(Kind::Forall, &[bound, ge]);
    solver.assert_formula(forall);

    let c = tm.mk_const(int, "c");
    let fc = tm.mk_term(Kind::ApplyUf, &[f, c]);
    let neg = tm.mk_term(Kind::Lt, &[fc, tm.mk_integer(0)]);
    solver.assert_formula(neg);

    let result = solver.check_sat();
    assert!(result.is_unsat());
    let inst = solver.get_instantiations();
    assert!(!inst.is_empty());
}

// ── find_synth_with_grammar ─────────────────────────────────────────

// Note: find_synth, find_synth_with_grammar, and find_synth_next require
// specific SyGuS setup that is difficult to test in isolation. They are
// exercised indirectly through the SyGuS workflow tests above.

// ── check_synth_next ───────────────────────────────────────────────

#[test]
fn solver_check_synth_next() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("LIA");
    solver.set_option("sygus", "true");
    solver.set_option("incremental", "true");

    let int = tm.integer_sort();
    let f = solver.synth_fun("f", &[], int.clone());
    let zero = tm.mk_integer(0);
    solver.add_sygus_constraint(tm.mk_term(Kind::Geq, &[f.clone(), zero]));
    assert!(solver.check_synth().has_solution());
    let sr2 = solver.check_synth_next();
    assert!(sr2.has_solution());
}

// ── get_interpolant_with_grammar ────────────────────────────────────

#[test]
fn solver_get_interpolant_with_grammar() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_LIA");
    solver.set_option("produce-interpolants", "true");

    let int = tm.integer_sort();
    let x = tm.mk_const(int.clone(), "x");
    let y = tm.mk_const(int, "y");
    let zero = tm.mk_integer(0);
    solver.assert_formula(tm.mk_term(Kind::Gt, &[x.clone(), zero.clone()]));
    solver.assert_formula(tm.mk_term(Kind::Equal, &[x, y.clone()]));
    let conj = tm.mk_term(Kind::Gt, &[y, zero]);

    let start = tm.mk_var(tm.boolean_sort(), "start");
    let mut g = solver.mk_grammar(&[], std::slice::from_ref(&start));
    g.add_rule(start.clone(), tm.mk_true());
    g.add_rule(start, conj.clone());

    let interp = solver.get_interpolant_with_grammar(conj, &g);
    assert!(interp.unwrap().sort().is_boolean());
}

// ── get_abduct_with_grammar ────────────────────────────────────────

#[test]
fn solver_get_abduct_with_grammar() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_LIA");
    solver.set_option("produce-abducts", "true");

    let int = tm.integer_sort();
    let x = tm.mk_const(int, "x");
    let zero = tm.mk_integer(0);
    let conj = tm.mk_term(Kind::Gt, &[x, zero]);

    let start = tm.mk_var(tm.boolean_sort(), "start");
    let mut g = solver.mk_grammar(&[], std::slice::from_ref(&start));
    g.add_rule(start.clone(), conj.clone());
    g.add_rule(start, tm.mk_true());

    let abd = solver.get_abduct_with_grammar(conj, &g);
    assert!(abd.unwrap().sort().is_boolean());
}

// ── get_quantifier_elimination ─────────────────────────────────────

#[test]
fn solver_quantifier_elimination() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("LIA");

    let int = tm.integer_sort();
    let x = tm.mk_var(int.clone(), "x");
    let y = tm.mk_const(int, "y");
    let zero = tm.mk_integer(0);

    // exists x. (x > 0 AND y = x)  =>  y > 0
    let body = tm.mk_term(
        Kind::And,
        &[
            tm.mk_term(Kind::Gt, &[x.clone(), zero]),
            tm.mk_term(Kind::Equal, &[y, x.clone()]),
        ],
    );
    let bound = tm.mk_term(Kind::VariableList, &[x]);
    let exists = tm.mk_term(Kind::Exists, &[bound, body]);

    let result = solver.get_quantifier_elimination(exists.clone());
    assert!(result.sort().is_boolean());

    let partial = solver.get_quantifier_elimination_disjunct(exists);
    assert!(partial.sort().is_boolean());
}

// ── get_output / close_output ──────────────────────────────────────

#[test]
fn solver_output_file() {
    let tm = TermManager::new();
    let solver = Solver::new(&tm);
    let path = "/tmp/cvc5_rs_test_output.txt";
    solver.get_output("inst", path);
    solver.close_output(path);
    // get_output + close_output should not panic; verify solver still usable
    assert!(!solver.version().is_empty());
    std::fs::remove_file(path).ok();
}

// ── get_timeout_core ───────────────────────────────────────────────

#[test]
fn solver_get_timeout_core() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_LIA");
    solver.set_option("produce-unsat-cores", "true");
    solver.set_option("timeout-core-timeout", "100");

    let int = tm.integer_sort();
    let x = tm.mk_const(int, "x");
    let one = tm.mk_integer(1);
    solver.assert_formula(tm.mk_term(Kind::Equal, &[x, one]));

    let (result, terms) = solver.get_timeout_core();
    // simple problem should be sat, not timeout
    assert!(result.is_sat() || result.is_unknown());
    for t in &terms {
        assert!(t.sort().is_boolean());
    }
}

// ── get_timeout_core_assuming ──────────────────────────────────────

#[test]
fn solver_get_timeout_core_assuming() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_LIA");
    solver.set_option("produce-unsat-cores", "true");
    solver.set_option("timeout-core-timeout", "100");

    let int = tm.integer_sort();
    let x = tm.mk_const(int, "x");
    let zero = tm.mk_integer(0);
    let gt = tm.mk_term(Kind::Gt, &[x, zero]);

    let (result, terms) = solver.get_timeout_core_assuming(&[gt]);
    assert!(result.is_sat() || result.is_unknown());
    for t in &terms {
        assert!(t.sort().is_boolean());
    }
}

// ── get_interpolant_next ───────────────────────────────────────────

#[test]
fn solver_get_interpolant_next() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_LIA");
    solver.set_option("produce-interpolants", "true");
    solver.set_option("incremental", "true");

    let int = tm.integer_sort();
    let x = tm.mk_const(int.clone(), "x");
    let y = tm.mk_const(int, "y");
    let zero = tm.mk_integer(0);
    solver.assert_formula(tm.mk_term(Kind::Gt, &[x.clone(), zero.clone()]));
    solver.assert_formula(tm.mk_term(Kind::Equal, &[x, y.clone()]));
    let conj = tm.mk_term(Kind::Gt, &[y, zero]);

    let interp1 = solver.get_interpolant(conj.clone());
    assert!(interp1.unwrap().sort().is_boolean());

    let interp2 = solver.get_interpolant_next();
    assert!(interp2.unwrap().sort().is_boolean());
}

// ── get_abduct_next ────────────────────────────────────────────────

#[test]
fn solver_get_abduct_next() {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_logic("QF_LIA");
    solver.set_option("produce-abducts", "true");
    solver.set_option("incremental", "true");

    let int = tm.integer_sort();
    let x = tm.mk_const(int, "x");
    let zero = tm.mk_integer(0);
    let conj = tm.mk_term(Kind::Gt, &[x, zero]);

    let abd1 = solver.get_abduct(conj.clone());
    assert!(abd1.unwrap().sort().is_boolean());

    let abd2 = solver.get_abduct_next();
    assert!(abd2.unwrap().sort().is_boolean());
}
