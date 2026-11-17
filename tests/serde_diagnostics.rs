//! Portable validation reports preserve typed failure details.
mod support;

use datalog_structural_model::{StratificationError, TermShapeError};
use support::errors::{cycle, every_program_error};

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
    for error in every_program_error() {
        round_trip(&error);
    }
}

#[test]
fn structural_and_stratification_errors_round_trip() {
    round_trip(&TermShapeError::InvalidSkolemArguments);
    round_trip(&TermShapeError::TripleTermNestingExceeded { depth: 5, limit: 4 });
    round_trip(&StratificationError::UnstratifiableCycle(cycle()));
}
