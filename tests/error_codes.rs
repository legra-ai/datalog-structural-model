//! Census of stable error codes. Every variant is pinned here; renaming a
//! variant must not change its code, and a new variant fails to compile
//! until it is added to the census.
mod support;

use std::collections::BTreeSet;

use datalog_structural_model::{
    ErrorCode, ProfileCheckError, ProgramValidationError, StratificationError, TermShapeError,
    TerminationViolation,
};
use support::errors::{cycle, every_program_error, termination_violation};

/// Exhaustive on purpose: no wildcard arm.
fn pinned_program_code(error: &ProgramValidationError) -> &'static str {
    match error {
        ProgramValidationError::UnsafeRule { .. } => "unsafe-rule",
        ProgramValidationError::UnsafeNegation { .. } => "unsafe-negation",
        ProgramValidationError::UnsafeFilter { .. } => "unsafe-filter",
        ProgramValidationError::UnsafeBind { .. } => "unsafe-bind",
        ProgramValidationError::UnsafeAggregate { .. } => "unsafe-aggregate",
        ProgramValidationError::NestedAggregateInInnerBody { .. } => "nested-aggregate",
        ProgramValidationError::UnstratifiableCycle(_) => "unstratifiable-cycle",
        ProgramValidationError::NegationUnderEqualityMode { .. } => "negation-under-equality-mode",
        ProgramValidationError::AggregationUnderEqualityMode { .. } => {
            "aggregation-under-equality-mode"
        }
        ProgramValidationError::SkolemInBodyAtom { .. } => "skolem-in-body",
        ProgramValidationError::TripleTermNestingExceeded { .. } => "triple-term-nesting-exceeded",
        ProgramValidationError::InvalidSkolemArguments { .. } => "invalid-skolem-arguments",
        ProgramValidationError::RuleCountExceeded { .. } => "rule-count-exceeded",
        ProgramValidationError::BodyLengthExceeded { .. } => "body-length-exceeded",
        ProgramValidationError::SkolemDepthExceeded { .. } => "skolem-depth-exceeded",
    }
}

fn assert_kebab(code: ErrorCode) {
    let text = code.as_str();
    assert!(!text.is_empty() && !text.starts_with('-') && !text.ends_with('-'));
    assert!(
        text.chars().all(|c| c.is_ascii_lowercase() || c == '-'),
        "{text} is not kebab-case"
    );
    assert!(!text.contains("--"), "{text}");
}

#[test]
fn program_validation_codes_are_pinned_and_unique() {
    let errors = every_program_error();
    let mut seen = BTreeSet::new();
    for error in &errors {
        assert_eq!(
            error.code().as_str(),
            pinned_program_code(error),
            "{error:?}"
        );
        assert_kebab(error.code());
        assert!(seen.insert(error.code()), "duplicate code {}", error.code());
        assert!(
            error
                .to_string()
                .starts_with(&format!("[{}] ", error.code()))
        );
    }
    assert_eq!(seen.len(), 15);
}

#[test]
fn associated_constants_match_codes() {
    assert_eq!(
        ProgramValidationError::CODE_UNSAFE_RULE.as_str(),
        "unsafe-rule"
    );
    assert_eq!(
        StratificationError::CODE_UNSTRATIFIABLE_CYCLE,
        ProgramValidationError::CODE_UNSTRATIFIABLE_CYCLE
    );
    assert_eq!(
        TermShapeError::CODE_TRIPLE_TERM_NESTING_EXCEEDED,
        ProgramValidationError::CODE_TRIPLE_TERM_NESTING_EXCEEDED
    );
}

#[test]
fn other_error_codes_are_pinned() {
    let stratification = StratificationError::UnstratifiableCycle(cycle());
    assert_eq!(stratification.code().as_str(), "unstratifiable-cycle");
    for (error, code) in [
        (
            TermShapeError::TripleTermNestingExceeded { depth: 5, limit: 4 },
            "triple-term-nesting-exceeded",
        ),
        (
            TermShapeError::InvalidSkolemArguments,
            "invalid-skolem-arguments",
        ),
    ] {
        assert_eq!(error.code().as_str(), code);
        assert_kebab(error.code());
    }
    let invalid = ProfileCheckError::InvalidProgram(every_program_error().remove(0));
    assert_eq!(invalid.code().as_str(), "invalid-program");
    assert_eq!(
        ProfileCheckError::CODE_UNSUPPORTED_CAPABILITIES.as_str(),
        "unsupported-capabilities"
    );
    let uncertified = ProfileCheckError::TerminationUncertified(termination_violation());
    assert_eq!(uncertified.code().as_str(), "termination-uncertified");
    assert_kebab(uncertified.code());
}

#[test]
fn termination_violation_code_is_pinned() {
    let violation = termination_violation();
    assert_eq!(violation.code().as_str(), "not-weakly-acyclic");
    assert_eq!(
        violation.code(),
        TerminationViolation::CODE_NOT_WEAKLY_ACYCLIC
    );
    assert_kebab(violation.code());
    assert!(violation.to_string().starts_with("[not-weakly-acyclic] "));
}
