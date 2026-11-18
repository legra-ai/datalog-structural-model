//! Typed validation limits: profile-owned defaults, enforced by
//! `validate_for`, and limit errors that report the limit and the value.
mod support;

use datalog_structural_model::{
    DatalogCapabilities, DatalogFeature, DatalogProfile, EqualityMode, Literal, ProfileCheckError,
    Program, ProgramValidationError, Rule, RuleIndex, ValidationLimits,
};
use support::build::{atom, rule, skolem, triple, var};

fn skolem_profile(limits: ValidationLimits) -> DatalogProfile {
    DatalogProfile::new(
        DatalogCapabilities::positive().with(DatalogFeature::SkolemGeneration),
        EqualityMode::Default,
    )
    .with_limits(limits)
}

fn invalid(error: ProfileCheckError) -> ProgramValidationError {
    let ProfileCheckError::InvalidProgram(error) = error else {
        panic!("expected an invalid program, got {error:?}")
    };
    error
}

#[test]
fn profiles_own_the_standard_limits() {
    let standard = ValidationLimits::standard();
    assert_eq!(ValidationLimits::default(), standard);
    assert_eq!(standard.triple_term_nesting(), 4);
    assert_eq!(DatalogProfile::positive().limits(), &standard);
    assert_eq!(DatalogProfile::stratified().limits(), &standard);
    let tight = standard.with_rules(1);
    assert_eq!(
        DatalogProfile::positive().with_limits(tight).limits(),
        &tight
    );
}

#[test]
fn rule_count_limit_reports_count_and_limit() {
    let fact = rule(atom("p", vec![]), vec![]);
    let error = Program {
        rules: vec![fact.clone(), fact.clone(), fact],
    }
    .validate_for(
        &DatalogProfile::positive().with_limits(ValidationLimits::standard().with_rules(2)),
    )
    .unwrap_err();
    assert_eq!(
        invalid(error),
        ProgramValidationError::RuleCountExceeded { count: 3, limit: 2 }
    );
}

#[test]
fn body_length_limit_counts_every_literal() {
    let body = (0..3).map(|_| atom("q", vec![var("x")])).collect();
    let error = Program {
        rules: vec![rule(atom("p", vec![var("x")]), body)],
    }
    .validate_for(
        &DatalogProfile::positive().with_limits(ValidationLimits::standard().with_body_literals(2)),
    )
    .unwrap_err();
    let error = invalid(error);
    assert_eq!(error.code().as_str(), "body-length-exceeded");
    assert_eq!(
        error,
        ProgramValidationError::BodyLengthExceeded {
            rule: RuleIndex::new(0),
            length: 3,
            limit: 2
        }
    );
    assert!(error.to_string().contains("3 literals; limit is 2"));
}

#[test]
fn nesting_limit_comes_from_the_profile() {
    let nested = triple(triple(var("x"), "r", var("x")), "r", var("x"));
    let program = Program {
        rules: vec![Rule::new(
            atom("p", vec![nested]),
            vec![Literal::Positive(atom("q", vec![var("x")]))],
        )],
    };
    let profile = DatalogProfile::new(
        DatalogCapabilities::positive().with(DatalogFeature::TripleTerms),
        EqualityMode::Default,
    );
    program.validate_for(&profile).unwrap();
    let error = program
        .validate_for(
            &profile.with_limits(ValidationLimits::standard().with_triple_term_nesting(1)),
        )
        .unwrap_err();
    assert!(matches!(
        invalid(error),
        ProgramValidationError::TripleTermNestingExceeded {
            depth: 2,
            limit: 1,
            ..
        }
    ));
}

#[test]
fn skolem_depth_limit_uses_the_termination_certificate() {
    let chain = Program {
        rules: vec![
            rule(
                atom("b", vec![skolem("b", vec![var("x")])]),
                vec![atom("a", vec![var("x")])],
            ),
            rule(
                atom("c", vec![skolem("c", vec![var("y")])]),
                vec![atom("b", vec![var("y")])],
            ),
        ],
    };
    chain
        .validate_for(&skolem_profile(ValidationLimits::standard()))
        .unwrap();
    let error = invalid(
        chain
            .validate_for(&skolem_profile(
                ValidationLimits::standard().with_skolem_depth(1),
            ))
            .unwrap_err(),
    );
    let ProgramValidationError::SkolemDepthExceeded {
        position,
        depth,
        limit,
    } = &error
    else {
        panic!("{error:?}")
    };
    assert_eq!((*depth, *limit), (2, 1));
    assert_eq!(position.rule, RuleIndex::new(1));
    assert_eq!(error.code().as_str(), "skolem-depth-exceeded");
}

#[test]
fn validate_for_requires_weak_acyclicity() {
    let recursive = Program {
        rules: vec![rule(
            atom("node", vec![skolem("next", vec![var("x")])]),
            vec![atom("node", vec![var("x")])],
        )],
    };
    recursive.validate().unwrap();
    let error = recursive
        .validate_for(&skolem_profile(ValidationLimits::standard()))
        .unwrap_err();
    assert_eq!(error.code().as_str(), "termination-uncertified");
    let ProfileCheckError::TerminationUncertified(violation) = error else {
        unreachable!()
    };
    assert_eq!(violation.code().as_str(), "not-weakly-acyclic");
}
