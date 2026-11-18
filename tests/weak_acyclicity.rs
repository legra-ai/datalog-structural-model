//! Weak acyclicity of the position graph on hand-written programs.
mod support;

use datalog_structural_model::{
    ArgumentPosition, AtomSlot, HeadAtomPath, InventionKind, LawVersion, Literal, LiteralIndex,
    LiteralPath, PredicateSymbol, Program, Rule, RuleIndex, RuleSite, TerminationCriterion,
};
use support::build::{atom, constant, increment, rule, skolem, triple, var};

fn program(rules: Vec<Rule>) -> Program {
    Program { rules }
}

fn head(predicate: &str, slot: usize, rule: usize) -> ArgumentPosition {
    ArgumentPosition::new(
        PredicateSymbol::new(predicate),
        AtomSlot::new(slot),
        RuleIndex::new(rule),
        RuleSite::Head(HeadAtomPath::atomic()),
    )
}

fn body(predicate: &str, slot: usize, rule: usize, literal: usize) -> ArgumentPosition {
    ArgumentPosition::new(
        PredicateSymbol::new(predicate),
        AtomSlot::new(slot),
        RuleIndex::new(rule),
        RuleSite::Body(LiteralPath::top(LiteralIndex::new(literal))),
    )
}

#[test]
fn criterion_is_labelled_and_versioned() {
    let criterion = TerminationCriterion::WeaklyAcyclic;
    assert_eq!(criterion.label(), "weakly-acyclic");
    assert_eq!(criterion.law_version(), LawVersion::new(1));
    assert_eq!(criterion.to_string(), "weakly-acyclic v1");
}

#[test]
fn recursion_without_invention_is_weakly_acyclic() {
    let certificate = program(vec![
        rule(
            atom("ancestor", vec![var("x"), var("y")]),
            vec![atom("parent", vec![var("x"), var("y")])],
        ),
        rule(
            atom("ancestor", vec![var("x"), var("z")]),
            vec![
                atom("parent", vec![var("x"), var("y")]),
                atom("ancestor", vec![var("y"), var("z")]),
            ],
        ),
    ])
    .check_weak_acyclicity()
    .unwrap();
    assert_eq!(certificate.criterion(), TerminationCriterion::WeaklyAcyclic);
    assert_eq!(certificate.special_edges(), 0);
    assert_eq!(certificate.skolem_depth(), 0);
    assert_eq!(certificate.deepest(), None);
}

#[test]
fn recursive_skolem_is_reported_with_typed_positions() {
    let violation = program(vec![rule(
        atom("node", vec![skolem("next", vec![var("x")])]),
        vec![atom("node", vec![var("x")])],
    )])
    .check_weak_acyclicity()
    .unwrap_err();
    assert_eq!(violation.code().as_str(), "not-weakly-acyclic");
    assert_eq!(violation.criterion(), TerminationCriterion::WeaklyAcyclic);
    let edge = violation.edge();
    assert_eq!(edge.kind, InventionKind::Skolem);
    assert_eq!(edge.from, body("node", 0, 0, 0));
    assert_eq!(edge.to, head("node", 0, 0));
    assert_eq!(
        violation.component(),
        [head("node", 0, 0), body("node", 0, 0, 0)]
    );
}

#[test]
fn non_recursive_skolem_chain_reports_its_depth() {
    let certificate = program(vec![
        rule(
            atom("b", vec![skolem("b", vec![var("x")])]),
            vec![atom("a", vec![var("x")])],
        ),
        rule(
            atom("c", vec![skolem("c", vec![var("y")])]),
            vec![atom("b", vec![var("y")])],
        ),
    ])
    .check_weak_acyclicity()
    .unwrap();
    assert_eq!(certificate.special_edges(), 2);
    assert_eq!(certificate.skolem_depth(), 2);
    assert_eq!(certificate.deepest(), Some(&head("c", 0, 1)));
}

#[test]
fn cycle_through_an_ordinary_rule_is_detected() {
    let violation = program(vec![
        rule(
            atom("b", vec![skolem("b", vec![var("x")])]),
            vec![atom("a", vec![var("x")])],
        ),
        rule(atom("a", vec![var("y")]), vec![atom("b", vec![var("y")])]),
    ])
    .check_weak_acyclicity()
    .unwrap_err();
    assert_eq!(violation.edge().from, body("a", 0, 0, 0));
    assert_eq!(violation.edge().to, head("b", 0, 0));
    assert_eq!(violation.component().len(), 4);
}

#[test]
fn constants_that_keep_patterns_apart_break_the_cycle() {
    let refined = |tag: &str| {
        program(vec![rule(
            atom("p", vec![skolem("p", vec![var("x")]), constant(tag)]),
            vec![atom("p", vec![var("x"), constant("seed")])],
        )])
    };
    assert_eq!(
        refined("derived")
            .check_weak_acyclicity()
            .unwrap()
            .skolem_depth(),
        1
    );
    assert!(refined("seed").check_weak_acyclicity().is_err());
}

#[test]
fn recursive_triple_term_construction_is_invention() {
    let violation = program(vec![rule(
        atom("p", vec![triple(var("x"), "wraps", var("x"))]),
        vec![atom("p", vec![var("x")])],
    )])
    .check_weak_acyclicity()
    .unwrap_err();
    assert_eq!(violation.edge().kind, InventionKind::TripleTerm);
}

#[test]
fn recursive_bind_is_invention() {
    let violation = program(vec![Rule::new(
        atom("n", vec![var("y")]),
        vec![
            Literal::Positive(atom("n", vec![var("x")])),
            increment("x", "y"),
        ],
    )])
    .check_weak_acyclicity()
    .unwrap_err();
    assert_eq!(violation.edge().kind, InventionKind::Bind);
    assert_eq!(violation.edge().from, body("n", 0, 0, 0));
}

#[test]
fn negated_atoms_bind_nothing() {
    let certificate = program(vec![Rule::new(
        atom("p", vec![var("x")]),
        vec![
            Literal::Positive(atom("q", vec![var("x")])),
            Literal::Negative(atom("p", vec![skolem("s", vec![var("x")])])),
        ],
    )])
    .check_weak_acyclicity()
    .unwrap();
    assert_eq!(certificate.special_edges(), 0);
}

#[test]
fn analysis_cost_follows_program_size() {
    // bounded: a fixed 2048-rule chain, no data relation.
    let rules = (1..=2048)
        .map(|index| {
            rule(
                atom(&format!("p{index}"), vec![skolem("s", vec![var("x")])]),
                vec![atom(&format!("p{}", index - 1), vec![var("x")])],
            )
        })
        .collect();
    let certificate = program(rules).check_weak_acyclicity().unwrap();
    assert_eq!(certificate.skolem_depth(), 2048);
    assert_eq!(certificate.positions(), 4096);
}
