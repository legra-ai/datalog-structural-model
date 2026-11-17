//! Aggregate safety and scoping.

use super::support::*;

/// extension — a well-formed aggregate body literal whose
/// `input_var` and `group_by` are bound by the inner body and
/// whose `result_var` is fresh passes validation.
#[test]
fn safe_aggregate_passes_validation() {
    // total(?dept, ?total) :- AGG ?sal SUM(?sal) GROUP BY ?dept
    //                          WHERE { salary(?e, ?dept, ?sal) }
    let program = Program {
        rules: vec![Rule::new(
            atom("total", vec![var("dept"), var("total")]),
            vec![Literal::Aggregate(agg_body(
                "total",
                Aggregator::Sum,
                "sal",
                vec!["dept"],
                vec![atom("salary", vec![var("e"), var("dept"), var("sal")])],
            ))],
        )],
    };
    assert!(program.validate().is_ok());
}

/// extension — `input_var` not produced by the inner body is an
/// `UnsafeAggregate` failure.
#[test]
fn unsafe_aggregate_unbound_input_fails_validation() {
    let program = Program {
        rules: vec![Rule::new(
            atom("total", vec![var("dept"), var("total")]),
            vec![Literal::Aggregate(agg_body(
                "total",
                Aggregator::Sum,
                "missing_sal", // <-- not in inner body
                vec!["dept"],
                vec![atom("salary", vec![var("e"), var("dept"), var("sal")])],
            ))],
        )],
    };
    let err = program.validate().unwrap_err();
    let ProgramValidationError::UnsafeAggregate {
        rule,
        literal,
        variables,
    } = err
    else {
        panic!("expected UnsafeAggregate, got {err:?}");
    };
    assert_eq!(rule, rule0());
    assert_eq!(literal, lit(0));
    assert!(
        variables.contains(&VariableSymbol::new("missing_sal")),
        "expected `missing_sal` in offending list, got {variables:?}"
    );
}

/// extension — a `group_by` key not bound by the inner body is an
/// `UnsafeAggregate` failure.
#[test]
fn unsafe_aggregate_unbound_group_key_fails_validation() {
    let program = Program {
        rules: vec![Rule::new(
            atom("total", vec![var("region"), var("total")]),
            vec![Literal::Aggregate(agg_body(
                "total",
                Aggregator::Sum,
                "sal",
                vec!["region"], // <-- not in inner body
                vec![atom("salary", vec![var("e"), var("dept"), var("sal")])],
            ))],
        )],
    };
    let err = program.validate().unwrap_err();
    let ProgramValidationError::UnsafeAggregate { variables, .. } = err else {
        panic!("expected UnsafeAggregate, got {err:?}");
    };
    assert!(
        variables.contains(&VariableSymbol::new("region")),
        "expected `region` in offending list, got {variables:?}"
    );
}

/// extension — `result_var` colliding with a variable bound by a
/// positive body atom is an `UnsafeAggregate` failure.
#[test]
fn unsafe_aggregate_result_var_collides_fails_validation() {
    let program = Program {
        rules: vec![Rule::new(
            atom("h", vec![var("dept"), var("total")]),
            vec![
                Literal::Positive(atom("anchor", vec![var("total")])),
                Literal::Aggregate(agg_body(
                    "total", // <-- collides with anchor's binding
                    Aggregator::Sum,
                    "sal",
                    vec!["dept"],
                    vec![atom("salary", vec![var("e"), var("dept"), var("sal")])],
                )),
            ],
        )],
    };
    let err = program.validate().unwrap_err();
    let ProgramValidationError::UnsafeAggregate { variables, .. } = err else {
        panic!("expected UnsafeAggregate, got {err:?}");
    };
    assert!(
        variables.contains(&VariableSymbol::new("total")),
        "expected `total` in offending list, got {variables:?}"
    );
}

/// extension — an aggregate's exported variables (`result_var` +
/// `group_by` keys) satisfy outer-rule head safety. The head
/// references variables only the aggregate exports; the rule
/// passes validation.
#[test]
fn aggregate_exports_satisfy_head_safety() {
    let program = Program {
        rules: vec![Rule::new(
            atom("total", vec![var("dept"), var("total")]),
            vec![Literal::Aggregate(agg_body(
                "total",
                Aggregator::Count,
                "sal",
                vec!["dept"],
                vec![atom("salary", vec![var("e"), var("dept"), var("sal")])],
            ))],
        )],
    };
    assert!(program.validate().is_ok());
}

/// extension — a rule whose head references the inner-only variable
/// of an aggregate is rejected as unsafe-head. Inner variables
/// do not leak past the aggregate boundary.
#[test]
fn aggregate_inner_only_variable_does_not_leak_to_head() {
    let program = Program {
        rules: vec![Rule::new(
            atom("h", vec![var("e"), var("total")]),
            vec![Literal::Aggregate(agg_body(
                "total",
                Aggregator::Count,
                "sal",
                vec!["dept"],
                vec![atom("salary", vec![var("e"), var("dept"), var("sal")])],
            ))],
        )],
    };
    let err = program.validate().unwrap_err();
    let ProgramValidationError::UnsafeRule { variables, .. } = err else {
        panic!("expected UnsafeRule, got {err:?}");
    };
    assert!(
        variables.contains(&VariableSymbol::new("e")),
        "inner-only `e` must not satisfy head safety; got {variables:?}"
    );
}

/// extension — recursion through an aggregate body literal is
/// rejected the same way negation cycles are: the aggregate's
/// inner predicate must sit in a strictly lower stratum than
/// the rule's head.
#[test]
fn recursion_through_aggregate_is_rejected() {
    // r(?d, ?t) :- AGG ?sal COUNT(?sal) GROUP BY ?d WHERE { r(?d, ?sal) }
    let program = Program {
        rules: vec![Rule::new(
            atom("r", vec![var("d"), var("t")]),
            vec![Literal::Aggregate(agg_body(
                "t",
                Aggregator::Count,
                "sal",
                vec!["d"],
                vec![atom("r", vec![var("d"), var("sal")])],
            ))],
        )],
    };
    let err = program.validate().unwrap_err();
    let ProgramValidationError::UnstratifiableCycle(cycle) = err else {
        panic!("expected UnstratifiableCycle, got {err:?}");
    };
    let predicates = cycle.predicates;
    assert!(
        predicates.iter().any(|p| p.as_str() == "r"),
        "expected `r` in unstratifiable cycle, got {predicates:?}"
    );
}
