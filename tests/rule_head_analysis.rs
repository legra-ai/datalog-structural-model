//! Safety, stratification, weak acyclicity and error positions for
//! generalised rule heads.
mod support;

use datalog_structural_model::{
    ArgumentPosition, AtomSlot, HeadAtomIndex, HeadAtomPath, HeadDisjunctIndex, InventionKind,
    Literal, PredicateSymbol, Program, ProgramValidationError, Rule, RuleHead, RuleIndex, RuleSite,
    TermPath, TermRoot, VariableSymbol,
};
use support::build::{atom, conjunction, disjunction, existential, skolem, triple, var};

fn path(disjunct: usize, atom: usize) -> HeadAtomPath {
    HeadAtomPath::new(HeadDisjunctIndex::new(disjunct), HeadAtomIndex::new(atom))
}

fn validate(head: RuleHead, body: Vec<Literal>) -> Result<(), ProgramValidationError> {
    Program {
        rules: vec![Rule::new(head, body)],
    }
    .validate()
}

fn r(x: &str) -> Literal {
    Literal::Positive(atom("r", vec![var(x)]))
}

#[test]
fn existential_variables_are_exempt_from_range_restriction() {
    let head = existential(&["y"], vec![atom("s", vec![var("x"), var("y")])]);
    assert_eq!(validate(head, vec![r("x")]), Ok(()));
}

#[test]
fn existential_variables_must_be_witnesses() {
    let unused = existential(&["y", "z"], vec![atom("s", vec![var("x"), var("y")])]);
    assert_eq!(
        validate(unused, vec![r("x")]),
        Err(ProgramValidationError::UnusedExistentialVariable {
            rule: RuleIndex::new(0),
            variable: VariableSymbol::new("z"),
        })
    );
    let bound = existential(&["x"], vec![atom("s", vec![var("x")])]);
    let error = validate(bound, vec![r("x")]).unwrap_err();
    assert_eq!(
        error,
        ProgramValidationError::ExistentialVariableBoundInBody {
            rule: RuleIndex::new(0),
            variable: VariableSymbol::new("x"),
        }
    );
    assert_eq!(error.code().as_str(), "existential-variable-bound-in-body");
}

#[test]
fn unsafe_head_variables_name_the_disjunct_and_atom() {
    let head = disjunction(vec![
        vec![atom("p", vec![var("x")])],
        vec![atom("q", vec![var("x")]), atom("q", vec![var("free")])],
    ]);
    let error = validate(head, vec![r("x")]).unwrap_err();
    assert_eq!(
        error,
        ProgramValidationError::UnsafeRule {
            rule: RuleIndex::new(0),
            atom: path(1, 1),
            variables: vec![VariableSymbol::new("free")],
        }
    );
    assert_eq!(
        error.to_string(),
        "[unsafe-rule] rule 0 is unsafe at head disjunct 1 atom 1; head variables not bound \
         by the body: free"
    );
}

#[test]
fn term_shape_errors_name_the_head_atom() {
    let bad = skolem("n", vec![triple(var("x"), "p", var("x"))]);
    let head = RuleHead::Conjunction(conjunction(vec![
        atom("p", vec![var("x")]),
        atom("q", vec![var("x"), bad]),
    ]));
    assert_eq!(
        validate(head, vec![r("x")]),
        Err(ProgramValidationError::InvalidSkolemArguments {
            rule: RuleIndex::new(0),
            site: RuleSite::Head(path(0, 1)),
            term: TermPath::new(TermRoot::Slot(AtomSlot::new(1))),
        })
    );
}

#[test]
fn falsum_rules_are_safe_and_produce_nothing() {
    let program = Program::try_new(vec![
        Rule::new(
            RuleHead::Falsum,
            vec![r("x"), Literal::Negative(atom("ok", vec![var("x")]))],
        ),
        Rule::new(atom("ok", vec![var("x")]), vec![r("x")]),
    ])
    .unwrap();
    let strata = program.stratify().unwrap();
    assert_eq!(strata.stratum_of_rule(RuleIndex::new(0)), Some(1));
    assert_eq!(strata.stratum_of_rule(RuleIndex::new(1)), Some(0));
}

#[test]
fn every_head_atom_is_a_producer() {
    // `p(x) ∧ q(x) :- r(x)` and `p(x) ∨ q(x) :- r(x)` both feed `not q(x)`.
    for head in [
        RuleHead::Conjunction(conjunction(vec![
            atom("p", vec![var("x")]),
            atom("q", vec![var("x")]),
        ])),
        disjunction(vec![
            vec![atom("p", vec![var("x")])],
            vec![atom("q", vec![var("x")])],
        ]),
    ] {
        let program = Program::try_new(vec![
            Rule::new(head.clone(), vec![r("x")]),
            Rule::new(
                atom("s", vec![var("x")]),
                vec![r("x"), Literal::Negative(atom("q", vec![var("x")]))],
            ),
        ])
        .unwrap();
        let strata = program.stratify().unwrap();
        assert_eq!(strata.stratum_of_rule(RuleIndex::new(1)), Some(1));
        // Closing the loop through the second head atom is a cycle through
        // negation.
        let cyclic = Program {
            rules: vec![
                Rule::new(
                    head,
                    vec![r("x"), Literal::Negative(atom("s", vec![var("x")]))],
                ),
                program.rules[1].clone(),
            ],
        };
        let Err(ProgramValidationError::UnstratifiableCycle(cycle)) = cyclic.validate() else {
            panic!("expected a cycle through negation");
        };
        assert!(cycle.predicates.contains(&PredicateSymbol::new("q")));
        assert!(cycle.predicates.contains(&PredicateSymbol::new("p")));
    }
}

fn head_position(predicate: &str, slot: usize, at: HeadAtomPath) -> ArgumentPosition {
    ArgumentPosition::new(
        PredicateSymbol::new(predicate),
        AtomSlot::new(slot),
        RuleIndex::new(0),
        RuleSite::Head(at),
    )
}

#[test]
fn recursive_existential_heads_are_not_weakly_acyclic() {
    // person(x) → ∃y. parent(x, y) ∧ person(y): the classic infinite chase.
    let head = existential(
        &["y"],
        vec![
            atom("parent", vec![var("x"), var("y")]),
            atom("person", vec![var("y")]),
        ],
    );
    let program = Program::try_new(vec![Rule::positive(
        head,
        vec![atom("person", vec![var("x")])],
    )])
    .unwrap();
    let violation = program.check_weak_acyclicity().unwrap_err();
    assert_eq!(violation.edge().kind, InventionKind::Existential);
    assert_eq!(violation.edge().to, head_position("person", 0, path(0, 1)));
    assert!(violation.to_string().contains("existential edge"));
}

#[test]
fn non_recursive_existential_heads_are_weakly_acyclic() {
    let head = existential(&["y"], vec![atom("parent", vec![var("x"), var("y")])]);
    let program = Program::try_new(vec![Rule::positive(
        head,
        vec![atom("person", vec![var("x")])],
    )])
    .unwrap();
    let certificate = program.check_weak_acyclicity().unwrap();
    assert_eq!(certificate.skolem_depth(), 1);
    assert_eq!(
        certificate.deepest(),
        Some(&head_position("parent", 1, path(0, 0)))
    );
}

#[test]
fn disjunct_atoms_feed_the_position_graph() {
    // The second disjunct carries a Skolem value back into its own body.
    let head = disjunction(vec![
        vec![atom("done", vec![var("x")])],
        vec![atom("node", vec![skolem("n", vec![var("x")])])],
    ]);
    let program = Program::try_new(vec![Rule::positive(
        head,
        vec![atom("node", vec![var("x")])],
    )])
    .unwrap();
    let violation = program.check_weak_acyclicity().unwrap_err();
    assert_eq!(violation.edge().kind, InventionKind::Skolem);
    assert_eq!(violation.edge().to, head_position("node", 0, path(1, 0)));
}
