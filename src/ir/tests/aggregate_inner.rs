//! Aggregate inner-body rules and nesting.

use super::support::*;

/// extension — an aggregate whose inner body carries an inner
/// negation alongside an inner positive atom validates, as long
/// as every variable named by the negated atom is bound by some
/// inner positive atom or `BIND` (the inner-scope range
/// restriction).
#[test]
fn safe_aggregate_inner_negation_passes_validation() {
    // total_unflagged(?dept, ?total) :- AGG ?sal SUM(?sal)
    //   GROUP BY ?dept
    //   WHERE { salary(?e, ?dept, ?sal), NOT flagged(?e) }
    let program = Program {
        rules: vec![Rule::new(
            atom("total_unflagged", vec![var("dept"), var("total")]),
            vec![Literal::Aggregate(agg_body_with_inner(
                "total",
                Aggregator::Sum,
                "sal",
                vec!["dept"],
                vec![
                    Literal::Positive(atom("salary", vec![var("e"), var("dept"), var("sal")])),
                    Literal::Negative(atom("flagged", vec![var("e")])),
                ],
            ))],
        )],
    };
    assert!(program.validate().is_ok());
}

/// extension — an inner negation that names a variable no inner
/// positive atom or `BIND` binds is rejected as `UnsafeAggregate`.
/// Mirrors the outer-rule `UnsafeNegation` check applied to the
/// inner scope.
#[test]
fn unsafe_aggregate_inner_negation_unbound_var_fails_validation() {
    let program = Program {
        rules: vec![Rule::new(
            atom("h", vec![var("dept"), var("total")]),
            vec![Literal::Aggregate(agg_body_with_inner(
                "total",
                Aggregator::Sum,
                "sal",
                vec!["dept"],
                vec![
                    Literal::Positive(atom("salary", vec![var("e"), var("dept"), var("sal")])),
                    // `m` is not bound by any inner positive atom or BIND.
                    Literal::Negative(atom("flagged", vec![var("m")])),
                ],
            ))],
        )],
    };
    let err = program.validate().unwrap_err();
    let ProgramValidationError::UnsafeAggregate { variables, .. } = err else {
        panic!("expected UnsafeAggregate, got {err:?}");
    };
    assert!(
        variables.contains(&VariableSymbol::new("m")),
        "expected unbound `m` in offending list, got {variables:?}"
    );
}

/// extension — an inner BIND whose output variable feeds the
/// aggregate's `input_var` or a group key is safe and validates.
/// Exercises the inner-scope `BIND` safety path.
#[test]
fn safe_aggregate_inner_bind_feeds_input_var() {
    // total_bonus(?dept, ?total) :- AGG ?bonus SUM(?bonus)
    //   GROUP BY ?dept
    //   WHERE { salary(?e, ?dept, ?sal), BIND(?sal + ?sal AS ?bonus) }
    let program = Program {
        rules: vec![Rule::new(
            atom("total_bonus", vec![var("dept"), var("total")]),
            vec![Literal::Aggregate(agg_body_with_inner(
                "total",
                Aggregator::Sum,
                "bonus",
                vec!["dept"],
                vec![
                    Literal::Positive(atom("salary", vec![var("e"), var("dept"), var("sal")])),
                    Literal::Bind {
                        var: VariableSymbol::new("bonus"),
                        expr: BindExpr::Add(
                            Box::new(BindExpr::Term(var("sal"))),
                            Box::new(BindExpr::Term(var("sal"))),
                        ),
                    },
                ],
            ))],
        )],
    };
    assert!(program.validate().is_ok());
}

/// extension — an inner filter referencing a variable bound by an
/// inner positive atom validates. Mirrors the outer-rule
/// `UnsafeFilter` rules applied to the inner scope.
#[test]
fn safe_aggregate_inner_filter_passes_validation() {
    let program = Program {
        rules: vec![Rule::new(
            atom("high_total", vec![var("dept"), var("total")]),
            vec![Literal::Aggregate(agg_body_with_inner(
                "total",
                Aggregator::Sum,
                "sal",
                vec!["dept"],
                vec![
                    Literal::Positive(atom("salary", vec![var("e"), var("dept"), var("sal")])),
                    Literal::Filter(BuiltinFilter::Gt(var("sal"), constant("0"))),
                ],
            ))],
        )],
    };
    assert!(program.validate().is_ok());
}

/// extension — a nested `Literal::Aggregate` inside another
/// aggregate's `inner_body` surfaces as
/// [`ProgramValidationError::NestedAggregateInInnerBody`].
#[test]
fn nested_aggregate_in_inner_body_is_rejected() {
    let inner_agg = AggregateBody {
        result_var: VariableSymbol::new("inner_total"),
        aggregator: Aggregator::Sum,
        input_var: VariableSymbol::new("sal"),
        group_by: vec![VariableSymbol::new("dept")],
        inner_body: vec![Literal::Positive(atom(
            "salary",
            vec![var("e"), var("dept"), var("sal")],
        ))],
    };
    let outer_agg = AggregateBody {
        result_var: VariableSymbol::new("outer_total"),
        aggregator: Aggregator::Sum,
        input_var: VariableSymbol::new("inner_total"),
        group_by: vec![],
        inner_body: vec![Literal::Aggregate(inner_agg)],
    };
    let program = Program {
        rules: vec![Rule::new(
            atom("grand_total", vec![var("outer_total")]),
            vec![Literal::Aggregate(outer_agg)],
        )],
    };
    let err = program.validate().unwrap_err();
    assert_eq!(
        err,
        ProgramValidationError::NestedAggregateInInnerBody {
            rule: rule0(),
            literal: lit(0).inner(LiteralIndex::new(0)),
        }
    );
}

/// extension — `Aggregator::name` returns the canonical uppercase
/// rendering for every operator.
#[test]
fn aggregator_name_renders_canonical_strings() {
    assert_eq!(Aggregator::Count.name(), "COUNT");
    assert_eq!(Aggregator::Sum.name(), "SUM");
    assert_eq!(Aggregator::Min.name(), "MIN");
    assert_eq!(Aggregator::Max.name(), "MAX");
    assert_eq!(Aggregator::Avg.name(), "AVG");
}

/// extension — `AggregateBody::exported_variables` exposes
/// `result_var` plus every `group_by` key; inner-only variables
/// stay scoped.
#[test]
fn aggregate_exported_variables_match_outer_scope() {
    let agg = agg_body(
        "t",
        Aggregator::Sum,
        "sal", // inner-only
        vec!["dept"],
        vec![atom("salary", vec![var("e"), var("dept"), var("sal")])],
    );
    let exported = agg.exported_variables();
    let expected: BTreeSet<VariableSymbol> = ["t", "dept"]
        .iter()
        .copied()
        .map(VariableSymbol::new)
        .collect();
    assert_eq!(exported, expected);
}
