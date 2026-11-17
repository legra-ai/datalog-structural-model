//! Negation and aggregation under equality mode.

use super::support::*;

// ── extension — negation / aggregation under equality mode ──────────

/// extension — a program with only positive atoms validates under
/// `EqualityMode::SameAs`. The check only rejects negation and
/// aggregation.
#[test]
fn equality_mode_sameas_accepts_positive_only_program() {
    let program = Program {
        rules: vec![Rule::positive(
            atom("parent", vec![var("x"), var("y")]),
            vec![atom("has_parent", vec![var("x"), var("y")])],
        )],
    };
    assert!(
        program
            .validate_with_equality_mode(EqualityMode::SameAs)
            .is_ok()
    );
}

/// extension — a program containing a negative literal is rejected
/// under `EqualityMode::SameAs` with
/// `ProgramValidationError::NegationUnderEqualityMode`.
#[test]
fn equality_mode_sameas_rejects_negation() {
    let program = Program {
        rules: vec![Rule::new(
            atom("bachelor", vec![var("x")]),
            vec![
                Literal::Positive(atom("person", vec![var("x")])),
                Literal::Negative(atom("married", vec![var("x")])),
            ],
        )],
    };
    // Under the default mode the same program validates.
    assert!(program.validate().is_ok());
    let err = program
        .validate_with_equality_mode(EqualityMode::SameAs)
        .expect_err("expected extension rejection");
    assert_eq!(
        err,
        ProgramValidationError::NegationUnderEqualityMode {
            rule: rule0(),
            literal: lit(1),
        }
    );
}

/// extension — a program containing an aggregate body literal is
/// rejected under `EqualityMode::SameAs` with
/// `ProgramValidationError::AggregationUnderEqualityMode`.
#[test]
fn equality_mode_sameas_rejects_aggregation() {
    let program = Program {
        rules: vec![Rule::new(
            atom("total", vec![var("dept"), var("total")]),
            vec![Literal::Aggregate(AggregateBody {
                result_var: VariableSymbol::new("total"),
                aggregator: Aggregator::Sum,
                input_var: VariableSymbol::new("sal"),
                group_by: vec![VariableSymbol::new("dept")],
                inner_body: vec![Literal::Positive(atom(
                    "salary",
                    vec![var("e"), var("dept"), var("sal")],
                ))],
            })],
        )],
    };
    // Default mode accepts.
    assert!(program.validate().is_ok());
    let err = program
        .validate_with_equality_mode(EqualityMode::SameAs)
        .expect_err("expected extension rejection");
    assert_eq!(
        err,
        ProgramValidationError::AggregationUnderEqualityMode {
            rule: rule0(),
            literal: lit(0),
        }
    );
}

/// extension — `EqualityMode::Default` (the bare `validate` path)
/// permits everything the program would otherwise pass — neither
/// negation nor aggregation gets blocked when equality mode is
/// off. Regression guard against an inverted mode comparison.
#[test]
fn equality_mode_default_permits_negation_and_aggregation() {
    let program = Program {
        rules: vec![
            Rule::new(
                atom("a", vec![var("x")]),
                vec![
                    Literal::Positive(atom("p", vec![var("x")])),
                    Literal::Negative(atom("q", vec![var("x")])),
                ],
            ),
            Rule::new(
                atom("b", vec![var("dept"), var("c")]),
                vec![Literal::Aggregate(AggregateBody {
                    result_var: VariableSymbol::new("c"),
                    aggregator: Aggregator::Count,
                    input_var: VariableSymbol::new("e"),
                    group_by: vec![VariableSymbol::new("dept")],
                    inner_body: vec![Literal::Positive(atom(
                        "works_in",
                        vec![var("e"), var("dept")],
                    ))],
                })],
            ),
        ],
    };
    assert!(program.validate().is_ok());
    assert!(
        program
            .validate_with_equality_mode(EqualityMode::Default)
            .is_ok()
    );
}

/// extension — the rejection reports the first offending rule
/// index when several rules conflict, walking the rule list in
/// declaration order. Pins the diagnostic shape so external
/// tools can surface a single targeted error rather than a
/// flood.
#[test]
fn equality_mode_sameas_reports_first_offending_rule() {
    let program = Program {
        rules: vec![
            Rule::positive(atom("ok", vec![var("x")]), vec![atom("p", vec![var("x")])]),
            Rule::new(
                atom("bad", vec![var("x")]),
                vec![
                    Literal::Positive(atom("p", vec![var("x")])),
                    Literal::Negative(atom("q", vec![var("x")])),
                ],
            ),
            Rule::new(
                atom("worse", vec![var("dept"), var("c")]),
                vec![Literal::Aggregate(AggregateBody {
                    result_var: VariableSymbol::new("c"),
                    aggregator: Aggregator::Count,
                    input_var: VariableSymbol::new("e"),
                    group_by: vec![VariableSymbol::new("dept")],
                    inner_body: vec![Literal::Positive(atom(
                        "works_in",
                        vec![var("e"), var("dept")],
                    ))],
                })],
            ),
        ],
    };
    let err = program
        .validate_with_equality_mode(EqualityMode::SameAs)
        .expect_err("expected extension rejection");
    assert_eq!(
        err,
        ProgramValidationError::NegationUnderEqualityMode {
            rule: RuleIndex::new(1),
            literal: lit(1),
        }
    );
}
