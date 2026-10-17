//! Context-sensitive safety is checked after decoding the complete program.
use datalog_structural_model::{
    Atom, BindExpr, BuiltinFilter, ConstantSymbol, DatalogProfile, Literal, PredicateSymbol,
    Program, Rule, Term, VariableSymbol,
};
fn atom(name: &str) -> Atom {
    Atom {
        predicate: PredicateSymbol::new(name),
        terms: vec![Term::Variable(VariableSymbol::new("x"))],
    }
}
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
fn stratified_negation_survives_both_encodings() {
    let p = Program::try_new(vec![
        Rule::try_new(
            atom("result"),
            vec![
                Literal::Positive(atom("input")),
                Literal::Negative(atom("excluded")),
            ],
        )
        .unwrap(),
    ])
    .unwrap();
    p.validate_for(&DatalogProfile::stratified()).unwrap();
    round_trip(&p);
}
#[test]
fn cross_rule_negation_cycle_is_rejected_after_decoding() {
    let p = Program {
        rules: vec![
            Rule::new(
                atom("p"),
                vec![
                    Literal::Positive(atom("input")),
                    Literal::Negative(atom("q")),
                ],
            ),
            Rule::positive(atom("q"), vec![atom("p")]),
        ],
    };
    assert!(Program::try_new(p.rules.clone()).is_err());
    assert!(serde_json::from_str::<Program>(&serde_json::to_string(&p).unwrap()).is_err());
    assert!(postcard::from_bytes::<Program>(&postcard::to_allocvec(&p).unwrap()).is_err());
    let positive = Program::try_new(vec![
        Rule::positive(atom("p"), vec![atom("q")]),
        Rule::positive(atom("q"), vec![atom("p")]),
    ])
    .unwrap();
    positive.validate_for(&DatalogProfile::positive()).unwrap();
    round_trip(&positive);
}
#[test]
fn omitted_optional_arguments_survive_both_encodings() {
    let term = Term::Constant(ConstantSymbol::new("exact"));
    let expr = Box::new(BindExpr::Term(term.clone()));
    round_trip(&BuiltinFilter::RegexMatch {
        value: term.clone(),
        pattern: term,
        flags: None,
    });
    round_trip(&BindExpr::SubStr {
        value: expr.clone(),
        start: expr.clone(),
        len: None,
    });
    round_trip(&BindExpr::Replace {
        value: expr.clone(),
        pattern: expr.clone(),
        replacement: expr,
        flags: None,
    });
}
#[test]
fn nested_unknown_semantics_are_not_discarded() {
    assert!(
        serde_json::from_str::<Literal>(
            r#"{"Bind":{"var":"out","expr":"Now","unknownMeaning":true}}"#
        )
        .is_err()
    );
    assert!(
        serde_json::from_str::<DatalogProfile>(
            r#"{"capabilities":[],"equality_mode":"Default","unknownMeaning":true}"#
        )
        .is_err()
    );
}

#[test]
fn decoded_program_errors_keep_the_original_rule_index() {
    let p = Program {
        rules: vec![
            Rule::positive(atom("valid"), vec![atom("input")]),
            Rule::new(atom("unbound"), vec![]),
        ],
    };
    let json_error =
        serde_json::from_str::<Program>(&serde_json::to_string(&p).unwrap()).unwrap_err();
    assert!(json_error.to_string().contains("index 1"), "{json_error}");
    // Binary decoding rejects the same invalid program.
    assert!(postcard::from_bytes::<Program>(&postcard::to_allocvec(&p).unwrap()).is_err());
}
