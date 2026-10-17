//! Portable validation reports preserve typed failure details.
use datalog_structural_model::{
    PredicateSymbol, ProgramValidationError, StratificationError, TermShapeError, VariableSymbol,
};
fn round_trip<T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug>(
    value: &T,
) {
    assert_eq!(
        &serde_json::from_str::<T>(&serde_json::to_string(value).unwrap()).unwrap(),
        value
    );
    assert_eq!(
        &postcard::from_bytes::<T>(&postcard::to_allocvec(value).unwrap()).unwrap(),
        value
    );
}
#[test]
fn every_program_failure_round_trips() {
    let vars = vec![VariableSymbol::new("missing")];
    for error in [
        ProgramValidationError::UnsafeRule {
            rule_index: 3,
            variables: vars.clone(),
        },
        ProgramValidationError::UnsafeNegation {
            rule_index: 3,
            variables: vars.clone(),
        },
        ProgramValidationError::UnsafeFilter {
            rule_index: 3,
            variables: vars.clone(),
        },
        ProgramValidationError::UnsafeBind {
            rule_index: 3,
            variables: vars.clone(),
        },
        ProgramValidationError::UnsafeAggregate {
            rule_index: 3,
            variables: vars,
        },
        ProgramValidationError::NestedAggregateInInnerBody { rule_index: 3 },
        ProgramValidationError::UnstratifiableNegation {
            predicates: vec![PredicateSymbol::new("cycle")],
        },
        ProgramValidationError::NegationUnderEqualityMode { rule_index: 3 },
        ProgramValidationError::AggregationUnderEqualityMode { rule_index: 3 },
        ProgramValidationError::SkolemInBodyAtom { rule_index: 3 },
        ProgramValidationError::TripleTermNestingExceeded {
            rule_index: 3,
            depth: 5,
            limit: 4,
        },
        ProgramValidationError::InvalidSkolemArguments { rule_index: 3 },
    ] {
        round_trip(&error);
    }
}
#[test]
fn structural_and_stratification_errors_round_trip() {
    round_trip(&TermShapeError::InvalidSkolemArguments);
    round_trip(&TermShapeError::TripleTermNestingExceeded { depth: 5, limit: 4 });
    round_trip(&StratificationError::UnstratifiableCycle {
        predicates: vec![PredicateSymbol::new("cycle")],
    });
}
