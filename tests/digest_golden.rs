//! Frozen `ProgramDigest` and `ContractDigest` values. A digest is a published
//! identity: if one of these assertions fails, the identity changed. Never
//! update a constant to make it pass; a deliberate encoding change takes a new
//! versioned domain tag and a new set of constants.
mod support;

use std::path::PathBuf;

use datalog_structural_model::{
    BindExpr, BuiltinFilter, ContractDigest, EqualityMode, Literal, Program, ProgramDigest, Rule,
    TerminationCriterion, ValidationContract, ValidationLimits, VariableSymbol,
};
use support::build::{atom, constant, var};

/// Digests of the serialized corpus under `tests/golden`.
const CORPUS: [(&str, &str); 6] = [
    (
        "aggregate",
        "d79ac24cbd125b06545ff224b7208dc879883145b82a35b61c375e38a7e19687",
    ),
    (
        "ancestor",
        "b24da3aaaa34983891fd96f84424b7ecc75353412006f0b437000c5525a2962c",
    ),
    (
        "empty",
        "37437adce0c337d2e00bccc414d67f006e67f927a86cf604dea36c47fa52a7f2",
    ),
    (
        "filter_and_bind",
        "d3e86440d8ca1c0a6e1e7951a18414b7874283222612a11244020b89580546cd",
    ),
    (
        "stratified_negation",
        "72e2e1b6d089c1fed39b6741ce41095fe6e7fc04a523ddef12a0936ff809ede1",
    ),
    (
        "triple_terms_and_skolem",
        "0171ecf23e5b5fc46f7360c7e737ce4e9b1d1a4874c96490bb79b049cc6c4437",
    ),
];

const OPERATORS: &str = "91f141e35f80ec2fa2f309dc6be2c414478f1dd4ef525810ec96514dffc4f7c2";
const CONTRACT_STANDARD: &str = "0c7c332d28d37eaaeb510a188380423890e8b0036695bef65b55cce9554463ee";
const CONTRACT_SAME_AS_TIGHT: &str =
    "589248efe60c57c4ca0c47530551a2691293b803c64213578ac81158a1ef7cbb";

fn golden_program(name: &str) -> Program {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join(format!("{name}.json"));
    let text = std::fs::read_to_string(&path).unwrap();
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn golden_postcard(name: &str) -> Program {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join(format!("{name}.postcard"));
    postcard::from_bytes(&std::fs::read(path).unwrap()).unwrap()
}

fn term(name: &str) -> Box<BindExpr> {
    Box::new(BindExpr::Term(var(name)))
}

/// Exercises the optional and n-ary filter and expression encodings.
fn operators() -> Program {
    Program::try_new(vec![Rule::new(
        atom("out", vec![var("n"), var("joined"), var("part")]),
        vec![
            Literal::Positive(atom("name", vec![var("n")])),
            Literal::Filter(BuiltinFilter::RegexMatch {
                value: var("n"),
                pattern: constant("^a"),
                flags: Some(constant("i")),
            }),
            Literal::Filter(BuiltinFilter::Or(
                Box::new(BuiltinFilter::IsLiteral(var("n"))),
                Box::new(BuiltinFilter::Not(Box::new(BuiltinFilter::IsBlank(var(
                    "n",
                ))))),
            )),
            Literal::Bind {
                var: VariableSymbol::new("joined"),
                expr: BindExpr::Concat(vec![*term("n"), BindExpr::Term(constant("-")), *term("n")]),
            },
            Literal::Bind {
                var: VariableSymbol::new("part"),
                expr: BindExpr::SubStr {
                    value: term("joined"),
                    start: Box::new(BindExpr::Term(constant("1"))),
                    len: None,
                },
            },
        ],
    )])
    .unwrap()
}

fn assert_frozen(name: &str, actual: impl std::fmt::Display, expected: &str) {
    let actual = actual.to_string();
    assert_eq!(
        actual, expected,
        "the published identity of `{name}` changed: a moved digest is a changed published \
         identity; introduce a new versioned domain tag instead of editing this constant"
    );
}

#[test]
fn corpus_program_digests_are_frozen() {
    for (name, expected) in CORPUS {
        let program = golden_program(name);
        assert_frozen(name, program.digest(), expected);
        assert_eq!(
            golden_postcard(name).digest(),
            program.digest(),
            "{name}: postcard"
        );
    }
}

#[test]
fn operator_program_digest_is_frozen() {
    assert_frozen("operators", operators().digest(), OPERATORS);
}

#[test]
fn contract_digests_are_frozen() {
    let program = golden_program("ancestor");
    let standard = ValidationContract::new(
        EqualityMode::Default,
        ValidationLimits::standard(),
        TerminationCriterion::WeaklyAcyclic,
    );
    let tight = ValidationContract::new(
        EqualityMode::SameAs,
        ValidationLimits::standard().with_rules(8),
        TerminationCriterion::WeaklyAcyclic,
    );
    assert_frozen(
        "contract-standard",
        standard.digest_for(&program),
        CONTRACT_STANDARD,
    );
    assert_frozen(
        "contract-same-as-tight",
        tight.digest_for(&program),
        CONTRACT_SAME_AS_TIGHT,
    );
}

#[test]
fn contract_digest_differs_from_program_digest() {
    let program = golden_program("ancestor");
    let contract = ValidationContract::new(
        EqualityMode::Default,
        ValidationLimits::standard(),
        TerminationCriterion::WeaklyAcyclic,
    );
    assert_ne!(
        contract.digest_for(&program).as_bytes(),
        program.digest().as_bytes()
    );
}

#[test]
fn digest_text_round_trips_and_rejects_bad_input() {
    let digest = golden_program("ancestor").digest();
    let text = digest.to_string();
    assert_eq!(text.len(), 64);
    assert_eq!(text.parse::<ProgramDigest>().unwrap(), digest);
    assert!(text.to_uppercase().parse::<ProgramDigest>().is_err());
    assert!(text[1..].parse::<ContractDigest>().is_err());
}

#[test]
fn term_and_literal_kinds_are_distinguished() {
    let body = atom("p", vec![var("x")]);
    let head = atom("q", vec![var("x")]);
    let positive = Program {
        rules: vec![Rule::new(
            head.clone(),
            vec![Literal::Positive(body.clone())],
        )],
    };
    let negative = Program {
        rules: vec![Rule::new(head, vec![Literal::Negative(body)])],
    };
    assert_ne!(positive.digest(), negative.digest());
    let variable = Program {
        rules: vec![Rule::positive(atom("q", vec![var("x")]), vec![])],
    };
    let constant_term = Program {
        rules: vec![Rule::positive(atom("q", vec![constant("x")]), vec![])],
    };
    assert_ne!(variable.digest(), constant_term.digest());
}
