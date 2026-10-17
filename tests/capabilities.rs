//! Profiles describe required language features, not a claim of engine conformance.
use datalog_structural_model::{
    Atom, BindExpr, BindingKind, BuiltinFilter, DatalogFeature, DatalogProfile, EqualityMode,
    FilterKind, Literal, PredicateSymbol, ProfileCheckError, Program, Rule, Term, VariableSymbol,
};
fn atom(name: &str) -> Atom {
    Atom {
        predicate: PredicateSymbol::new(name),
        terms: vec![Term::Variable(VariableSymbol::new("x"))],
    }
}
fn program(body: Vec<Literal>) -> Program {
    Program {
        rules: vec![Rule::new(atom("result"), body)],
    }
}
#[test]
fn positive_profile_rejects_negation_instead_of_ignoring_it() {
    let p = program(vec![
        Literal::Positive(atom("input")),
        Literal::Negative(atom("excluded")),
    ]);
    assert!(matches!(
        p.validate_for(&DatalogProfile::positive()),
        Err(ProfileCheckError::Unsupported(_))
    ));
    p.validate_for(&DatalogProfile::stratified()).unwrap();
}
#[test]
fn nested_operators_have_exact_capability_requirements() {
    let p = program(vec![
        Literal::Positive(atom("input")),
        Literal::Filter(BuiltinFilter::Not(Box::new(BuiltinFilter::IsIri(
            Term::TripleTerm(Box::new(atom("nested"))),
        )))),
        Literal::Bind {
            var: VariableSymbol::new("out"),
            expr: BindExpr::Sha256(Box::new(BindExpr::Str(Box::new(BindExpr::Term(
                Term::Variable(VariableSymbol::new("x")),
            ))))),
        },
    ]);
    let required = p.required_capabilities();
    for feature in [
        DatalogFeature::TripleTerms,
        DatalogFeature::Filter(FilterKind::Not),
        DatalogFeature::Filter(FilterKind::IsIri),
        DatalogFeature::Binding(BindingKind::Sha256),
        DatalogFeature::Binding(BindingKind::Str),
        DatalogFeature::Binding(BindingKind::Term),
    ] {
        assert!(required.contains(&feature), "{feature:?}");
    }
    assert!(p.validate_for(&DatalogProfile::positive()).is_err());
    p.validate_for(&DatalogProfile::new(required, EqualityMode::Default))
        .unwrap();
}
#[test]
fn equality_rewriting_does_not_silently_allow_stratified_negation() {
    let p = program(vec![
        Literal::Positive(atom("input")),
        Literal::Negative(atom("excluded")),
    ]);
    let profile = DatalogProfile::new(p.required_capabilities(), EqualityMode::SameAs);
    assert!(matches!(
        p.validate_for(&profile),
        Err(ProfileCheckError::InvalidProgram(_))
    ));
}
#[test]
fn profiles_round_trip_and_unknown_features_fail() {
    let p = DatalogProfile::stratified();
    assert_eq!(
        serde_json::from_str::<DatalogProfile>(&serde_json::to_string(&p).unwrap()).unwrap(),
        p
    );
    assert_eq!(
        postcard::from_bytes::<DatalogProfile>(&postcard::to_allocvec(&p).unwrap()).unwrap(),
        p
    );
    assert!(serde_json::from_str::<DatalogFeature>("\"UnknownFeature\"").is_err());
}
