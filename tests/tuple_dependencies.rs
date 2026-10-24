//! Rule scheduling checked against finite ground-pattern intersections.

use datalog_structural_model::{
    Atom, ConstantSymbol, Literal, PredicateSymbol, Program, Rule, Term, VariableSymbol,
};
use std::collections::BTreeSet;

fn constant(value: &str) -> Term {
    Term::Constant(ConstantSymbol::new(value))
}
fn variable(value: &str) -> Term {
    Term::Variable(VariableSymbol::new(value))
}
fn atom(predicate: &str, terms: Vec<Term>) -> Atom {
    Atom {
        predicate: PredicateSymbol::new(predicate),
        terms,
    }
}
fn quoted(predicate: &str, terms: Vec<Term>) -> Term {
    Term::TripleTerm(Box::new(atom(predicate, terms)))
}

fn has_dependency(head: Vec<Term>, body: Vec<Term>) -> bool {
    let program = Program {
        rules: vec![
            Rule::positive(atom("p", head), vec![]),
            Rule::new(
                atom("out", vec![]),
                vec![Literal::Negative(atom("p", body))],
            ),
        ],
    };
    // Unbound heads are intentional: stratify analyzes patterns independently
    // of the range-safety validation exercised by Program::try_new tests.
    program.stratify().unwrap().stratum_of_rule(1) == Some(1)
}

fn flat_extension(pattern: &[Term]) -> BTreeSet<Vec<&'static str>> {
    let mut extension = BTreeSet::new();
    for x in ["a", "b"] {
        for y in ["a", "b"] {
            extension.insert(
                pattern
                    .iter()
                    .map(|term| match term {
                        Term::Constant(value) if value.as_str() == "a" => "a",
                        Term::Constant(_) => "b",
                        Term::Variable(value) if value.as_str() == "x" => x,
                        Term::Variable(_) => y,
                        _ => unreachable!("flat fixture"),
                    })
                    .collect(),
            );
        }
    }
    extension
}

#[test]
fn all_flat_two_slot_patterns_match_ground_intersection() {
    let terms = [constant("a"), constant("b"), variable("x"), variable("y")];
    let patterns: Vec<_> = terms
        .iter()
        .flat_map(|a| terms.iter().map(move |b| vec![a.clone(), b.clone()]))
        .collect();
    for head in &patterns {
        for body in &patterns {
            let expected = !flat_extension(head).is_disjoint(&flat_extension(body));
            assert_eq!(
                has_dependency(head.clone(), body.clone()),
                expected,
                "head={head:?}, body={body:?}"
            );
        }
    }
}

#[test]
fn every_position_and_repeated_variable_constrains_dependencies() {
    assert!(!has_dependency(
        vec![variable("x"), variable("x")],
        vec![constant("a"), constant("b")]
    ));
    assert!(!has_dependency(
        vec![constant("a"), constant("b")],
        vec![variable("x"), variable("x")]
    ));
    assert!(has_dependency(
        vec![variable("x"), constant("a")],
        vec![constant("b"), variable("x")]
    ));
    assert!(!has_dependency(
        vec![constant("a")],
        vec![constant("a"), constant("b")]
    ));
}

#[test]
fn quoted_term_structure_and_variable_aliases_constrain_dependencies() {
    let q = |terms| quoted("quoted", terms);
    assert!(has_dependency(
        vec![q(vec![variable("x"), constant("a")])],
        vec![q(vec![constant("b"), variable("x")])]
    ));
    assert!(!has_dependency(
        vec![q(vec![variable("x"), variable("x")])],
        vec![q(vec![constant("a"), constant("b")])]
    ));
    assert!(!has_dependency(
        vec![q(vec![constant("a")])],
        vec![quoted("other", vec![constant("a")])]
    ));
    assert!(!has_dependency(
        vec![q(vec![constant("a")])],
        vec![constant("a")]
    ));
    // x = quoted(y), y = quoted(x) has no finite ground solution.
    assert!(!has_dependency(
        vec![variable("x"), q(vec![variable("x")])],
        vec![q(vec![variable("y")]), variable("y")]
    ));
}

#[test]
fn skolem_output_dependencies_are_conservative() {
    let head = || {
        vec![Term::Skolem {
            name: ConstantSymbol::new("key"),
            args: vec![constant("a")],
        }]
    };
    assert!(has_dependency(
        head(),
        vec![constant("opaque-runtime-value")]
    ));
    assert!(has_dependency(head(), vec![variable("value")]));
}
