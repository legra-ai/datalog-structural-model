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
    assert_eq!(strat.strata.len(), 1);
    assert_eq!(strat.rule_stratum, vec![0, 0]);
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
    let strat = stratify(&program).expect("program is stratified");
    // EDB inputs are already closed before rule stratum zero.
    assert_eq!(strat.rule_stratum, vec![0]);
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
    let strat = stratify(&program).expect("stratifiable");
    assert_eq!(strat.rule_stratum, vec![0, 0]);
}

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
fn positive_quad_rules_share_one_stratum() {
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
    assert_eq!(strat.rule_stratum, vec![0, 0]);
}

#[test]
fn positive_fixed_graph_dependency_is_not_strict() {
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
    assert_eq!(strat.rule_stratum, vec![0, 0]);
}

#[test]
fn body_atom_with_specific_graph_does_not_depend_on_other_literal_producers() {
    let g_a = "urn:example:G_a";
    let g_b = "urn:example:G_b";
    let program = Program {
        rules: vec![
            Rule::positive(
                atom_with_graph("producer", vec![var("s")], constant(g_a)),
                vec![atom_with_graph("input", vec![var("s")], var("g"))],
            ),
            Rule::new(
                atom_with_graph("producer", vec![var("s")], constant(g_b)),
                vec![
                    Literal::Positive(atom_with_graph("input", vec![var("s")], var("g"))),
                    Literal::Negative(atom_with_graph("exclusion", vec![var("s")], var("g"))),
                ],
            ),
            Rule::positive(
                atom_with_graph("consumer", vec![var("s")], var("g")),
                vec![atom_with_graph("producer", vec![var("s")], constant(g_a))],
            ),
            Rule::positive(
                atom_with_graph("exclusion", vec![constant("alice")], constant(g_b)),
                vec![],
            ),
        ],
    };
    let strat = stratify(&program).expect("stratifiable");
    assert_eq!(strat.rule_stratum, vec![0, 1, 0, 0]);
}
