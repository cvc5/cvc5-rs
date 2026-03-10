use cvc5_rs::{Kind, Solver, TermManager};

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
    let x_gt_0 = tm.mk_term(Kind::CVC5_KIND_GT, &[x.clone(), zero]);
    solver.assert_formula(x_gt_0);

    // y = x + 1
    let one = tm.mk_integer(1);
    let x_plus_1 = tm.mk_term(Kind::CVC5_KIND_ADD, &[x.clone(), one]);
    let y_eq = tm.mk_term(Kind::CVC5_KIND_EQUAL, &[y.clone(), x_plus_1]);
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
    let lt = tm.mk_term(Kind::CVC5_KIND_LT, &[x.clone(), zero]);
    let gt = tm.mk_term(Kind::CVC5_KIND_GT, &[x, one]);
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
    let and_term = tm.mk_term(Kind::CVC5_KIND_BITVECTOR_AND, &[x.clone(), ff]);
    let eq = tm.mk_term(Kind::CVC5_KIND_EQUAL, &[and_term, x.clone()]);
    solver.assert_formula(eq);

    let zero = tm.mk_bv(8, 0);
    let neq = tm.mk_term(Kind::CVC5_KIND_DISTINCT, &[x.clone(), zero]);
    solver.assert_formula(neq);

    assert!(solver.check_sat().is_sat());
}

// ── QF_LRA: linear real arithmetic ─────────────────────────────────

#[test]
fn qf_lra_sat() {
    setup!(tm, solver, "QF_LRA");

    let real = tm.real_sort();
    let x = tm.mk_const(real, "x");
    let half = tm.mk_real_from_rational(1, 2);
    let eq = tm.mk_term(Kind::CVC5_KIND_EQUAL, &[x.clone(), half]);
    solver.assert_formula(eq);

    assert!(solver.check_sat().is_sat());
    let val = solver.get_value(x);
    assert!(val.is_real_value());
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
    let or_term = tm.mk_term(Kind::CVC5_KIND_OR, &[a.clone(), b.clone()]);
    solver.assert_formula(or_term);

    // NOT (a AND b)
    let and_term = tm.mk_term(Kind::CVC5_KIND_AND, &[a.clone(), b.clone()]);
    let not_and = tm.mk_term(Kind::CVC5_KIND_NOT, &[and_term]);
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

    let x_gt_0 = tm.mk_term(Kind::CVC5_KIND_GT, &[x.clone(), zero.clone()]);
    solver.assert_formula(x_gt_0);

    solver.push(1);
    // Add contradictory constraint in inner scope
    let x_lt_0 = tm.mk_term(Kind::CVC5_KIND_LT, &[x.clone(), zero]);
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
    solver.set_logic("QF_UF");

    let bool_sort = tm.boolean_sort();
    let p = tm.mk_const(bool_sort.clone(), "p");
    let q = tm.mk_const(bool_sort, "q");

    // Assert p => q
    let imp = tm.mk_term(Kind::CVC5_KIND_IMPLIES, &[p.clone(), q.clone()]);
    solver.assert_formula(imp);

    // Assume p AND NOT q — should be unsat
    let not_q = tm.mk_term(Kind::CVC5_KIND_NOT, &[q]);
    let result = solver.check_sat_assuming(&[p, not_q]);
    assert!(result.is_unsat());
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
    let not_a = tm.mk_term(Kind::CVC5_KIND_NOT, &[a.clone()]);

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
    let store = tm.mk_term(Kind::CVC5_KIND_STORE, &[a.clone(), idx.clone(), val.clone()]);
    // select(store(a, 0, 42), 0) = 42
    let sel = tm.mk_term(Kind::CVC5_KIND_SELECT, &[store, idx]);
    let eq = tm.mk_term(Kind::CVC5_KIND_EQUAL, &[sel, val]);
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
    let red_term = tm.mk_term(Kind::CVC5_KIND_APPLY_CONSTRUCTOR, &[dt.constructor(0).term()]);
    let eq = tm.mk_term(Kind::CVC5_KIND_EQUAL, &[c, red_term]);
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

    let gt = tm.mk_term(Kind::CVC5_KIND_GT, &[x.clone(), zero.clone()]);
    let lt = tm.mk_term(Kind::CVC5_KIND_LT, &[x, zero]);

    s1.assert_formula(gt);
    s2.assert_formula(lt);

    assert!(s1.check_sat().is_sat());
    assert!(s2.check_sat().is_sat());
}

// ── Op (indexed operators) ─────────────────────────────────────────

#[test]
fn op_bv_extract() {
    let tm = TermManager::new();
    let op = tm.mk_op(Kind::CVC5_KIND_BITVECTOR_EXTRACT, &[3, 1]);
    assert_eq!(op.kind(), Kind::CVC5_KIND_BITVECTOR_EXTRACT);
    assert!(op.is_indexed());
    assert_eq!(op.num_indices(), 2);
    assert_eq!(format!("{op}"), "(_ extract 3 1)");

    let op2 = op.copy();
    assert_eq!(op, op2);
    assert!(!op.is_disequal(&op2));

    let op3 = tm.mk_op(Kind::CVC5_KIND_BITVECTOR_EXTRACT, &[7, 4]);
    assert!(op.is_disequal(&op3));

    // Use op to build a term
    let bv8 = tm.mk_bv_sort(8);
    let x = tm.mk_const(bv8, "x");
    let ext = tm.mk_term_from_op(op, &[x]);
    assert!(ext.has_op());
    let _ = ext.op();
    let _ = format!("{:?}", tm.mk_op(Kind::CVC5_KIND_BITVECTOR_EXTRACT, &[3, 1]));
    // hash
    let mut set = std::collections::HashSet::new();
    set.insert(op3);
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
    let not_a = tm.mk_term(Kind::CVC5_KIND_NOT, &[a.clone()]);
    solver.assert_formula(a);
    solver.assert_formula(not_a);
    assert!(solver.check_sat().is_unsat());

    let proofs = solver.get_proof(cvc5_sys::Cvc5ProofComponent::CVC5_PROOF_COMPONENT_FULL);
    assert!(!proofs.is_empty());

    let p = &proofs[0];
    let _ = p.rule();
    let _ = p.result();
    let _ = p.children();
    let _ = p.arguments();
    let p2 = p.copy();
    assert_eq!(p, &p2);
    assert!(!p.is_disequal(&p2));
    let _ = format!("{:?}", p);
    // hash
    let mut set = std::collections::HashSet::new();
    set.insert(p.copy());
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
    let gt = tm.mk_term(Kind::CVC5_KIND_GT, &[x, zero]);
    solver.assert_formula(gt);
    solver.check_sat();

    let stats = solver.get_statistics();
    let _ = format!("{stats}");
    let _ = format!("{stats:?}");

    // iterate
    stats.iter_init(false, false);
    if stats.iter_has_next() {
        let (name, stat) = stats.iter_next();
        assert!(!name.is_empty());
        let _ = stat.is_internal();
        let _ = stat.is_default();
        let _ = format!("{stat}");
        let _ = format!("{stat:?}");
        // type checks
        if stat.is_int() { let _ = stat.get_int(); }
        if stat.is_double() { let _ = stat.get_double(); }
        if stat.is_string() { let _ = stat.get_string(); }
        if stat.is_histogram() { let _ = stat.get_histogram(); }
    }
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
    let f = solver.synth_fun("f", &[x.clone()], int.clone());

    let zero = tm.mk_integer(0);
    let fx = tm.mk_term(Kind::CVC5_KIND_APPLY_UF, &[f.clone(), x.clone()]);
    let ge = tm.mk_term(Kind::CVC5_KIND_GEQ, &[fx, zero]);
    solver.add_sygus_constraint(ge);

    let sr = solver.check_synth();
    assert!(!sr.is_null());
    assert!(sr.has_solution());
    assert!(!sr.has_no_solution());
    assert!(!sr.is_unknown());

    let sr2 = sr.copy();
    assert_eq!(sr, sr2);
    assert!(!sr.is_disequal(&sr2));
    let _ = format!("{sr}");
    let _ = format!("{sr:?}");
    // hash
    let mut set = std::collections::HashSet::new();
    set.insert(std::hash::Hash::hash(&sr, &mut std::collections::hash_map::DefaultHasher::new()));

    let _ = solver.get_synth_solution(f);
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

    let mut g = solver.mk_grammar(&[x.clone()], &[start.clone()]);
    g.add_rule(start.clone(), tm.mk_integer(0));
    g.add_rules(start.clone(), &[tm.mk_integer(1), x.clone()]);
    g.add_any_constant(start.clone());
    g.add_any_variable(start.clone());

    let g2 = g.copy();
    assert_eq!(g, g2);
    assert!(!g.is_disequal(&g2));
    let _ = format!("{g}");
    let _ = format!("{g:?}");
    let mut set = std::collections::HashSet::new();
    set.insert(std::hash::Hash::hash(&g, &mut std::collections::hash_map::DefaultHasher::new()));

    let f = solver.synth_fun_with_grammar("f", &[x], int, &g);
    let sr = solver.check_synth();
    assert!(sr.has_solution());
    let _ = solver.get_synth_solution(f);
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
    assert_eq!(tm.mk_sequence_sort(int.clone()).sequence_element_sort(), int);
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
    assert_eq!(tm.boolean_sort().kind(), cvc5_sys::Cvc5SortKind::CVC5_SORT_KIND_BOOLEAN_SORT);
    assert_eq!(tm.integer_sort().kind(), cvc5_sys::Cvc5SortKind::CVC5_SORT_KIND_INTEGER_SORT);
    assert_eq!(tm.real_sort().kind(), cvc5_sys::Cvc5SortKind::CVC5_SORT_KIND_REAL_SORT);
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
    let rec = tm.mk_record_sort(
        &["x", "y"],
        &[tm.integer_sort(), tm.boolean_sort()],
    );
    assert!(rec.is_record());
    assert!(rec.is_dt()); // records are datatypes internally
}

// ── Sort: abstract sort ────────────────────────────────────────────

#[test]
fn sort_abstract() {
    let tm = TermManager::new();
    let abs = tm.mk_abstract_sort(cvc5_sys::Cvc5SortKind::CVC5_SORT_KIND_BITVECTOR_SORT);
    assert!(abs.is_abstract());
    assert_eq!(abs.abstract_kind(), cvc5_sys::Cvc5SortKind::CVC5_SORT_KIND_BITVECTOR_SORT);
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

    let sort = solver.declare_dt(
        "AB",
        &[tm.mk_dt_cons_decl("A"), tm.mk_dt_cons_decl("B")],
    );
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
    let mut decl = tm.mk_dt_decl_with_params("Opt", &[t.clone()], false);
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
        Kind::CVC5_KIND_APPLY_CONSTRUCTOR,
        &[ctor.term(), three.clone(), seven.clone()],
    );
    let p = tm.mk_const(sort, "p");
    solver.assert_formula(tm.mk_term(Kind::CVC5_KIND_EQUAL, &[p.clone(), pair_val]));

    assert!(solver.check_sat().is_sat());

    // select fst(p) == 3
    let fst_app = tm.mk_term(Kind::CVC5_KIND_APPLY_SELECTOR, &[fst_sel.term(), p.clone()]);
    let fst_val = solver.get_value(fst_app);
    assert_eq!(fst_val.int32_value(), 3);

    // select snd(p) == 7
    let snd_app = tm.mk_term(Kind::CVC5_KIND_APPLY_SELECTOR, &[snd_sel.term(), p.clone()]);
    let snd_val = solver.get_value(snd_app);
    assert_eq!(snd_val.int32_value(), 7);

    // tester: isPair(p) == true
    let tester = tm.mk_term(Kind::CVC5_KIND_APPLY_TESTER, &[ctor.tester_term(), p]);
    let tester_val = solver.get_value(tester);
    assert!(tester_val.boolean_value());
}

// ── Term: kind, sort, id ───────────────────────────────────────────

#[test]
fn term_kind_sort_id() {
    let tm = TermManager::new();
    let x = tm.mk_const(tm.integer_sort(), "x");
    assert_eq!(x.kind(), Kind::CVC5_KIND_CONSTANT);
    assert!(x.sort().is_integer());
    assert!(x.id() > 0);

    let zero = tm.mk_integer(0);
    assert_eq!(zero.kind(), Kind::CVC5_KIND_CONST_INTEGER);

    let gt = tm.mk_term(Kind::CVC5_KIND_GT, &[x, zero]);
    assert_eq!(gt.kind(), Kind::CVC5_KIND_GT);
    assert!(gt.sort().is_boolean());
}

// ── Term: num_children, child ──────────────────────────────────────

#[test]
fn term_children() {
    let tm = TermManager::new();
    let x = tm.mk_const(tm.integer_sort(), "x");
    let y = tm.mk_const(tm.integer_sort(), "y");
    let add = tm.mk_term(Kind::CVC5_KIND_ADD, &[x.clone(), y.clone()]);
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
    let op = tm.mk_op(Kind::CVC5_KIND_BITVECTOR_EXTRACT, &[3, 0]);
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
    let rne = tm.mk_rm(cvc5_sys::Cvc5RoundingMode::CVC5_RM_ROUND_NEAREST_TIES_TO_EVEN);
    assert!(rne.is_rm_value());
    assert_eq!(
        rne.rm_value(),
        cvc5_sys::Cvc5RoundingMode::CVC5_RM_ROUND_NEAREST_TIES_TO_EVEN
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
    solver.assert_formula(tm.mk_term(Kind::CVC5_KIND_EQUAL, &[t.clone(), tup]));
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
    let s1 = tm.mk_term(Kind::CVC5_KIND_SET_SINGLETON, &[one]);
    let s2 = tm.mk_term(Kind::CVC5_KIND_SET_SINGLETON, &[two]);
    let union = tm.mk_term(Kind::CVC5_KIND_SET_UNION, &[s1, s2]);
    solver.assert_formula(tm.mk_term(Kind::CVC5_KIND_EQUAL, &[s.clone(), union]));
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
    let unit = tm.mk_term(Kind::CVC5_KIND_SEQ_UNIT, &[one]);
    solver.assert_formula(tm.mk_term(Kind::CVC5_KIND_EQUAL, &[s.clone(), unit]));
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
    let gt = tm.mk_term(Kind::CVC5_KIND_GT, &[x.clone(), zero.clone()]);
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

    let add = tm.mk_term(Kind::CVC5_KIND_ADD, &[x.clone(), y.clone()]);
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
    let id = cvc5_sys::Cvc5SkolemId::CVC5_SKOLEM_ID_PURIFY;
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
    let op = tm.mk_op_from_str(Kind::CVC5_KIND_DIVISIBLE, "3");
    assert_eq!(op.kind(), Kind::CVC5_KIND_DIVISIBLE);
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
    let lifted = tm.mk_nullable_lift(Kind::CVC5_KIND_ADD, &[a, b]);
    assert!(lifted.sort().is_nullable());
}

// ── TermManager: mk_var (bound variable) ──────────────────────────

#[test]
fn tm_mk_var() {
    let tm = TermManager::new();
    let v = tm.mk_var(tm.integer_sort(), "v");
    assert_eq!(v.kind(), Kind::CVC5_KIND_VARIABLE);
    assert!(v.has_symbol());
    assert_eq!(v.symbol(), "v");
    assert!(v.sort().is_integer());
}

// ── TermManager: mk_const ─────────────────────────────────────────

#[test]
fn tm_mk_const() {
    let tm = TermManager::new();
    let c = tm.mk_const(tm.boolean_sort(), "p");
    assert_eq!(c.kind(), Kind::CVC5_KIND_CONSTANT);
    assert!(c.has_symbol());
    assert_eq!(c.symbol(), "p");
    assert!(c.sort().is_boolean());
}

// ── TermManager: print_stats_safe ──────────────────────────────────

#[test]
fn tm_print_stats_safe() {
    let tm = TermManager::new();
    // write to /dev/null (fd obtained via open); just verify no crash
    // Use fd 2 (stderr) as a safe target
    tm.print_stats_safe(2);
}

// ── TermManager: Send ──────────────────────────────────────────────

#[test]
fn tm_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<TermManager>();
    assert_send::<cvc5_rs::Solver>();
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
        CVC5_RM_ROUND_NEAREST_TIES_TO_EVEN,
        CVC5_RM_ROUND_TOWARD_POSITIVE,
        CVC5_RM_ROUND_TOWARD_NEGATIVE,
        CVC5_RM_ROUND_TOWARD_ZERO,
        CVC5_RM_ROUND_NEAREST_TIES_TO_AWAY,
    ] {
        let t = tm.mk_rm(rm);
        assert!(t.is_rm_value());
        assert_eq!(t.rm_value(), rm);
    }
}
