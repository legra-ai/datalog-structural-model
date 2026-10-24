use super::{StratificationError, stratify};
use crate::ir::{
    Atom, ConstantSymbol, Literal, PredicateSymbol, Program, Rule, Term, VariableSymbol,
};

fn var(name: &str) -> Term {
    Term::Variable(VariableSymbol::new(name))
}

fn atom(predicate: &str, terms: Vec<Term>) -> Atom {
    Atom {
        predicate: PredicateSymbol::new(predicate),
        terms,
    }
}

fn level(program: &Program, name: &str) -> usize {
    program
        .rules
        .iter()
        .enumerate()
        .filter(|(_, rule)| rule.head.predicate.as_str() == name)
        .map(|(index, _)| program.stratify().unwrap().stratum_of_rule(index).unwrap() + 1)
        .max()
        .unwrap_or(0)
}

#[test]
fn positive_program_is_single_stratum() {
    // ancestor(X,Y) :- parent(X,Y).
    // ancestor(X,Z) :- parent(X,Y), ancestor(Y,Z).
    let program = Program {
        rules: vec![
            Rule::positive(
                atom("ancestor", vec![var("x"), var("y")]),
                vec![atom("parent", vec![var("x"), var("y")])],
            ),
            Rule::positive(
                atom("ancestor", vec![var("x"), var("z")]),
                vec![
                    atom("parent", vec![var("x"), var("y")]),
                    atom("ancestor", vec![var("y"), var("z")]),
                ],
            ),
        ],
    };
    let strat = stratify(&program).expect("positive program is stratifiable");
    // parent and ancestor both in stratum 0 (no negation).
    assert_eq!(strat.strata.len(), 1);
    assert_eq!(level(&program, "parent"), 0);
    assert_eq!(level(&program, "ancestor"), 1);
}

#[test]
fn simple_stratified_negation() {
    // bachelor(X) :- person(X), not married(X).
    let program = Program {
        rules: vec![Rule::new(
            atom("bachelor", vec![var("x")]),
            vec![
                Literal::Positive(atom("person", vec![var("x")])),
                Literal::Negative(atom("married", vec![var("x")])),
            ],
        )],
    };
    stratify(&program).expect("program is stratified");
    // married is in stratum 0, bachelor must be in stratum ≥ 1.
    let married_s = level(&program, "married");
    let bachelor_s = level(&program, "bachelor");
    assert!(bachelor_s > married_s);
}

#[test]
fn recursion_through_negation_is_rejected() {
    // p(X) :- q(X), not p(X).
    let program = Program {
        rules: vec![Rule::new(
            atom("p", vec![var("x")]),
            vec![
                Literal::Positive(atom("q", vec![var("x")])),
                Literal::Negative(atom("p", vec![var("x")])),
            ],
        )],
    };
    let err = stratify(&program).expect_err("unstratifiable");
    assert!(matches!(
        err,
        StratificationError::UnstratifiableCycle { .. }
    ));
}

#[test]
fn mutual_recursion_through_negation_is_rejected() {
    // a(X) :- b(X), not b(X').
    // b(X) :- a(X).
    // The SCC {a, b} contains a negation edge.
    let program = Program {
        rules: vec![
            Rule::new(
                atom("a", vec![var("x")]),
                vec![
                    Literal::Positive(atom("b", vec![var("x")])),
                    Literal::Negative(atom("b", vec![var("x")])),
                ],
            ),
            Rule::positive(atom("b", vec![var("x")]), vec![atom("a", vec![var("x")])]),
        ],
    };
    let err = stratify(&program).expect_err("unstratifiable");
    assert!(matches!(
        err,
        StratificationError::UnstratifiableCycle { .. }
    ));
}

#[test]
fn chained_negation_strata() {
    // level1(X) :- base(X), not excluded(X).
    // level2(X) :- level1(X), not more_excluded(X).
    let program = Program {
        rules: vec![
            Rule::new(
                atom("level1", vec![var("x")]),
                vec![
                    Literal::Positive(atom("base", vec![var("x")])),
                    Literal::Negative(atom("excluded", vec![var("x")])),
                ],
            ),
            Rule::new(
                atom("level2", vec![var("x")]),
                vec![
                    Literal::Positive(atom("level1", vec![var("x")])),
                    Literal::Negative(atom("more_excluded", vec![var("x")])),
                ],
            ),
        ],
    };
    stratify(&program).expect("stratifiable");
    let base_s = level(&program, "base");
    let excluded_s = level(&program, "excluded");
    let more_excluded_s = level(&program, "more_excluded");
    let level1_s = level(&program, "level1");
    let level2_s = level(&program, "level2");
    // Both negated predicates sit in stratum 0 alongside
    // `base`; `level1` must be > `excluded`, and `level2`
    // must be > `more_excluded` *and* ≥ `level1` (positive
    // edge — no stratum bump needed). Minimum valid
    // stratification: base/excluded/more_excluded = 0,
    // level1 = 1, level2 = 1.
    assert_eq!(base_s, 0);
    assert_eq!(excluded_s, 0);
    assert_eq!(more_excluded_s, 0);
    assert!(level1_s > excluded_s);
    assert!(level2_s >= level1_s);
    assert!(level2_s > more_excluded_s);
}

// ── context-aware: graph-aware dependency-graph nodes ──────────────

fn constant(value: &str) -> Term {
    Term::Constant(ConstantSymbol::new(value))
}

fn atom_with_graph(predicate: &str, terms: Vec<Term>, graph: Term) -> Atom {
    let mut all_terms = terms;
    all_terms.push(graph);
    Atom {
        predicate: PredicateSymbol::new(predicate),
        terms: all_terms,
    }
}

#[test]
fn quad_shape_rules_stratify_like_predicate_only_when_head_class_is_uniform() {
    // Every rule's head ends in a graph variable
    // (`InheritFromBody {0}` default in the reasoner) — every
    // node should collapse to (pred, Inherit), so the strata
    // come out identical to the predicate-only algorithm.
    //
    // p(?s, ?g) :- q(?s, ?g).
    // q(?s, ?g) :- r(?s, ?g).
    let program = Program {
        rules: vec![
            Rule::positive(
                atom_with_graph("p", vec![var("s")], var("g")),
                vec![atom_with_graph("q", vec![var("s")], var("g"))],
            ),
            Rule::positive(
                atom_with_graph("q", vec![var("s")], var("g")),
                vec![atom_with_graph("r", vec![var("s")], var("g"))],
            ),
        ],
    };
    let strat = stratify(&program).expect("stratifiable");
    assert_eq!(strat.strata.len(), 1);
    assert_eq!(level(&program, "p"), 1);
    assert_eq!(level(&program, "q"), 1);
    assert_eq!(level(&program, "r"), 0);
}

#[test]
fn fixed_graph_producer_and_consumer_share_one_node_per_predicate() {
    // Producer rule pins its head to `urn:G_inferred`:
    //   inferred(?s, <urn:G_inferred>) :- input(?s, ?g).
    // Consumer rule reads from `urn:G_inferred`:
    //   downstream(?s, ?g) :- inferred(?s, <urn:G_inferred>).
    //
    // The dependency-graph node for `inferred` is
    // `(inferred, Literal(urn:G_inferred))`, distinct from the
    // `(inferred, Inherit)` node that a variable-graph producer
    // would create. Stratify must still produce a sensible
    // per-predicate stratum.
    let g_inferred = "urn:example:G_inferred";
    let program = Program {
        rules: vec![
            Rule::positive(
                atom_with_graph("inferred", vec![var("s")], constant(g_inferred)),
                vec![atom_with_graph("input", vec![var("s")], var("g"))],
            ),
            Rule::positive(
                atom_with_graph("downstream", vec![var("s")], var("g")),
                vec![atom_with_graph(
                    "inferred",
                    vec![var("s")],
                    constant(g_inferred),
                )],
            ),
        ],
    };
    let strat = stratify(&program).expect("stratifiable");
    let input_s = level(&program, "input");
    let inferred_s = level(&program, "inferred");
    let downstream_s = level(&program, "downstream");
    assert!(inferred_s >= input_s);
    assert!(downstream_s >= inferred_s);
    // Positive program — no negation, single stratum.
    assert_eq!(strat.strata.len(), 1);
}

#[test]
fn body_atom_with_specific_graph_does_not_depend_on_other_literal_producers() {
    // Two producers writing into two different fixed graphs.
    // A consumer that reads only from `G_a` must depend on the
    // `G_a` producer but **not** on the `G_b` producer.
    //
    //   producer_a(?s, <urn:G_a>) :- input(?s, ?g).
    //   producer_b(?s, <urn:G_b>) :- input(?s, ?g), not exclusion(?s, ?g).
    //   consumer(?s, ?g)          :- producer_a(?s, <urn:G_a>).
    //
    // The negated edge through `exclusion` bumps
    // `producer_b`'s stratum but, because the `consumer` only
    // depends on `producer_a`'s (Literal G_a) node, the
    // consumer should stay in the same stratum as `producer_a`.
    let g_a = "urn:example:G_a";
    let g_b = "urn:example:G_b";
    let program = Program {
        rules: vec![
            Rule::positive(
                atom_with_graph("producer_a", vec![var("s")], constant(g_a)),
                vec![atom_with_graph("input", vec![var("s")], var("g"))],
            ),
            Rule::new(
                atom_with_graph("producer_b", vec![var("s")], constant(g_b)),
                vec![
                    Literal::Positive(atom_with_graph("input", vec![var("s")], var("g"))),
                    Literal::Negative(atom_with_graph("exclusion", vec![var("s")], var("g"))),
                ],
            ),
            Rule::positive(
                atom_with_graph("consumer", vec![var("s")], var("g")),
                vec![atom_with_graph("producer_a", vec![var("s")], constant(g_a))],
            ),
        ],
    };
    stratify(&program).expect("stratifiable");
    let producer_a_s = level(&program, "producer_a");
    let producer_b_s = level(&program, "producer_b");
    let consumer_s = level(&program, "consumer");
    let exclusion_s = level(&program, "exclusion");
    // producer_b sits above exclusion (negated edge).
    assert!(producer_b_s > exclusion_s);
    // consumer rides on producer_a only — it does NOT inherit
    // producer_b's higher stratum, because its body atom names a
    // graph constant that matches producer_a's head node alone.
    assert_eq!(consumer_s, producer_a_s);
}
