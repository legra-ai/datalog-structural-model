//! Generalised rule heads: forms, non-empty collections, serialization,
//! digests and profile gating.
mod support;

use datalog_structural_model::{
    DatalogCapabilities, DatalogFeature, DatalogProfile, EmptyHeadCollection, EqualityMode,
    ExistentialVariables, HeadCollectionKind, HeadConjunction, HeadDisjunction, HeadForm,
    ProfileCheckError, Program, ProgramDigest, Rule, RuleHead, RuleIndex,
};
use support::build::{atom, conjunction, disjunction, existential, rule, var};

fn p(x: &str) -> datalog_structural_model::Atom {
    atom("p", vec![var(x)])
}

fn q(x: &str) -> datalog_structural_model::Atom {
    atom("q", vec![var(x)])
}

/// One rule of every head form over the body `r(x)`.
fn every_form() -> Vec<(HeadForm, RuleHead)> {
    vec![
        (HeadForm::Atomic, RuleHead::Atom(p("x"))),
        (
            HeadForm::Conjunctive,
            RuleHead::Conjunction(conjunction(vec![p("x"), q("x")])),
        ),
        (
            HeadForm::Existential,
            existential(&["y"], vec![atom("s", vec![var("x"), var("y")])]),
        ),
        (
            HeadForm::Disjunctive,
            disjunction(vec![vec![p("x")], vec![q("x")]]),
        ),
        (HeadForm::Inconsistency, RuleHead::Falsum),
    ]
}

fn program_with(head: RuleHead) -> Program {
    Program::try_new(vec![Rule::positive(head, vec![atom("r", vec![var("x")])])]).unwrap()
}

#[test]
fn head_form_is_a_pure_function_with_fixed_precedence() {
    for (form, head) in every_form() {
        assert_eq!(head.form(), form, "{head:?}");
        assert_eq!(program_with(head).rules[0].head_form(), form);
    }
    // A one-atom conjunction is atomic and a one-disjunct disjunction
    // classifies by its conjunction.
    assert_eq!(
        RuleHead::Conjunction(conjunction(vec![p("x")])).form(),
        HeadForm::Atomic
    );
    assert_eq!(
        disjunction(vec![vec![p("x"), q("x")]]).form(),
        HeadForm::Conjunctive
    );
    assert_eq!(disjunction(vec![vec![p("x")]]).form(), HeadForm::Atomic);
    assert_eq!(
        HeadForm::ALL
            .iter()
            .map(|form| form.as_str())
            .collect::<Vec<_>>(),
        [
            "atomic",
            "conjunctive",
            "existential",
            "disjunctive",
            "inconsistency"
        ]
    );
}

#[test]
fn head_collections_cannot_be_empty() {
    let kinds = [
        HeadConjunction::try_new(vec![]).unwrap_err().kind(),
        HeadDisjunction::try_new(vec![]).unwrap_err().kind(),
        ExistentialVariables::try_new([]).unwrap_err().kind(),
    ];
    assert_eq!(
        kinds,
        [
            HeadCollectionKind::Conjunction,
            HeadCollectionKind::Disjunction,
            HeadCollectionKind::ExistentialVariables,
        ]
    );
    let error = HeadConjunction::try_new(vec![]).unwrap_err();
    assert_eq!(error.code().as_str(), "empty-head-collection");
    assert_eq!(
        error.code(),
        EmptyHeadCollection::CODE_EMPTY_HEAD_COLLECTION
    );
    for json in [
        r#"{"Conjunction":[]}"#,
        r#"{"Disjunction":[]}"#,
        r#"{"Existential":{"variables":[],"conjunction":[{"predicate":"p","terms":[]}]}}"#,
        r#"{"Existential":{"variables":["y"],"conjunction":[]}}"#,
    ] {
        assert!(serde_json::from_str::<RuleHead>(json).is_err(), "{json}");
    }
}

#[test]
fn existential_variables_are_a_canonical_set() {
    let variables = ExistentialVariables::try_new(
        ["b", "a", "b"].map(datalog_structural_model::VariableSymbol::new),
    )
    .unwrap();
    let names: Vec<_> = variables.variables().iter().map(|v| v.as_str()).collect();
    assert_eq!(names, ["a", "b"]);
}

#[test]
fn every_form_round_trips_through_json_and_postcard() {
    for (_, head) in every_form() {
        let program = program_with(head);
        let json = serde_json::to_string(&program).unwrap();
        let bytes = postcard::to_allocvec(&program).unwrap();
        assert_eq!(serde_json::from_str::<Program>(&json).unwrap(), program);
        assert_eq!(postcard::from_bytes::<Program>(&bytes).unwrap(), program);
    }
    assert_eq!(
        serde_json::to_string(&RuleHead::Falsum).unwrap(),
        r#""Falsum""#
    );
}

#[test]
fn falsum_is_distinct_from_every_other_head() {
    let falsum = program_with(RuleHead::Falsum);
    for (form, head) in every_form() {
        if form == HeadForm::Inconsistency {
            continue;
        }
        let other = program_with(head);
        assert_ne!(falsum, other);
        assert_ne!(falsum.digest(), other.digest());
        assert_ne!(
            serde_json::to_string(&falsum).unwrap(),
            serde_json::to_string(&other).unwrap()
        );
    }
}

/// Frozen digests of the generalised head forms, under the unchanged
/// `program-digest/v1` domain: atomic heads encode exactly as before.
const HEAD_DIGESTS: [(HeadForm, &str); 5] = [
    (
        HeadForm::Atomic,
        "e205fc03d7ab0dfbaa333bf651cd334514f262d51d4b18e3c17413cc4cd43076",
    ),
    (
        HeadForm::Conjunctive,
        "388da84428236a16a94936503cc152d883d93488caaf2b12ac031ee65da554d9",
    ),
    (
        HeadForm::Existential,
        "97e6f49ff82140e1fca1842e7033463dff2e77fb6c0e441324a698942056ea72",
    ),
    (
        HeadForm::Disjunctive,
        "0e1393d2261e284318395e33e1d68a99b6045679dcd7ffcc0814170672812ec9",
    ),
    (
        HeadForm::Inconsistency,
        "d9699ab438fbe73f76edab356da57aedf82d8d1a3c50eb38fa8656e48cadb044",
    ),
];

#[test]
fn head_form_digests_are_frozen() {
    for ((form, head), (pinned, expected)) in every_form().into_iter().zip(HEAD_DIGESTS) {
        assert_eq!(form, pinned);
        assert_eq!(program_with(head).digest().to_string(), expected, "{form}");
    }
}

#[test]
fn authored_shape_is_part_of_the_digest() {
    let digests: Vec<ProgramDigest> = [
        RuleHead::Atom(p("x")),
        RuleHead::Conjunction(conjunction(vec![p("x")])),
        disjunction(vec![vec![p("x")]]),
        disjunction(vec![vec![p("x")], vec![q("x")]]),
        disjunction(vec![vec![p("x"), q("x")]]),
        RuleHead::Conjunction(conjunction(vec![p("x"), q("x")])),
        RuleHead::Conjunction(conjunction(vec![q("x"), p("x")])),
    ]
    .into_iter()
    .map(|head| program_with(head).digest())
    .collect();
    for (index, digest) in digests.iter().enumerate() {
        assert!(!digests[index + 1..].contains(digest), "{index}");
    }
}

#[test]
fn plain_profiles_refuse_non_atomic_heads_by_name() {
    for profile in [DatalogProfile::positive(), DatalogProfile::stratified()] {
        for (form, head) in every_form() {
            let program = Program::try_new(vec![
                rule(atom("r", vec![var("x")]), vec![atom("e", vec![var("x")])]),
                Rule::positive(head, vec![atom("r", vec![var("x")])]),
            ])
            .unwrap();
            let result = program.validate_for(&profile);
            if form == HeadForm::Atomic {
                assert_eq!(result, Ok(()));
                continue;
            }
            let error = result.unwrap_err();
            assert_eq!(
                error,
                ProfileCheckError::UnsupportedHeadForm {
                    rule: RuleIndex::new(1),
                    form,
                }
            );
            assert_eq!(error.code().as_str(), "unsupported-head-form");
            assert_eq!(error.code(), ProfileCheckError::CODE_UNSUPPORTED_HEAD_FORM);
            assert_eq!(
                error.to_string(),
                format!(
                    "[unsupported-head-form] rule 1 has a {form} head, which the profile does not \
                     accept"
                )
            );
        }
    }
}

#[test]
fn head_form_capabilities_admit_their_forms() {
    for (form, head) in every_form() {
        let program = program_with(head);
        let Some(feature) = form.required_feature() else {
            continue;
        };
        assert!(program.required_capabilities().contains(&feature));
        let profile = DatalogProfile::new(
            DatalogCapabilities::positive().with(feature),
            EqualityMode::Default,
        );
        assert_eq!(program.validate_for(&profile), Ok(()), "{form}");
        assert!(DatalogCapabilities::all_model_features().contains(&feature));
    }
    assert_eq!(
        [
            HeadForm::Conjunctive,
            HeadForm::Existential,
            HeadForm::Disjunctive,
            HeadForm::Inconsistency
        ]
        .map(HeadForm::required_feature),
        [
            Some(DatalogFeature::ConjunctiveHeads),
            Some(DatalogFeature::ExistentialHeads),
            Some(DatalogFeature::DisjunctiveHeads),
            Some(DatalogFeature::InconsistencyConstraints),
        ]
    );
}

#[test]
fn head_form_is_checked_before_other_missing_capabilities() {
    let program = Program::try_new(vec![Rule::new(
        RuleHead::Falsum,
        vec![
            datalog_structural_model::Literal::Positive(atom("r", vec![var("x")])),
            datalog_structural_model::Literal::Negative(atom("s", vec![var("x")])),
        ],
    )])
    .unwrap();
    assert_eq!(
        program.validate_for(&DatalogProfile::positive()),
        Err(ProfileCheckError::UnsupportedHeadForm {
            rule: RuleIndex::new(0),
            form: HeadForm::Inconsistency,
        })
    );
}
