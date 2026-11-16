//! Three-slot RDF adapter terms retain dynamic predicates through checked transport.
use datalog_structural_model::{
    Atom, ConstantSymbol, DatalogCapabilities, DatalogFeature, DatalogProfile, EqualityMode,
    Literal, PredicateSymbol, ProfileCheckError, Program, Rule, Term, VariableSymbol,
};
fn v(s: &str) -> Term {
    Term::Variable(VariableSymbol::new(s))
}
fn c(s: &str) -> Term {
    Term::Constant(ConstantSymbol::new(s))
}
fn a(p: &str, t: Vec<Term>) -> Atom {
    Atom {
        predicate: PredicateSymbol::new(p),
        terms: t,
    }
}
fn program() -> Program {
    let triple = Term::TripleTerm(Box::new(a(
        "urn:rdf-reasoning:triple",
        vec![v("s"), v("p"), v("o")],
    )));
    let head = a(
        "urn:rdf-reasoning:triple",
        vec![
            triple,
            c("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
            c("http://www.w3.org/2000/01/rdf-schema#Proposition"),
        ],
    );
    let body = [("s", "Resource"), ("p", "Property"), ("o", "Resource")]
        .into_iter()
        .map(|(x, k)| Literal::Positive(a("urn:rdf-reasoning:triple", vec![v(x), c("type"), c(k)])))
        .collect();
    Program::try_new(vec![Rule::try_new(head, body).unwrap()]).unwrap()
}
#[test]
fn dynamic_predicate_tripleterm_is_checked_and_transportable() {
    let p = program();
    assert!(
        p.required_capabilities()
            .contains(&DatalogFeature::TripleTerms)
    );
    assert!(matches!(
        p.validate_for(&DatalogProfile::positive()),
        Err(ProfileCheckError::Unsupported(_))
    ));
    let profile = DatalogProfile::new(
        DatalogCapabilities::positive().with(DatalogFeature::TripleTerms),
        EqualityMode::Default,
    );
    p.validate_for(&profile).unwrap();
    let nested = vec![p];
    assert_eq!(
        serde_json::from_str::<Vec<Program>>(&serde_json::to_string(&nested).unwrap()).unwrap(),
        nested
    );
    assert_eq!(
        postcard::from_bytes::<Vec<Program>>(&postcard::to_allocvec(&nested).unwrap()).unwrap(),
        nested
    );
}
#[test]
fn nested_predicate_slot_must_be_bound() {
    let mut p = program();
    if let Term::TripleTerm(atom) = &mut p.rules[0].head.terms[0] {
        atom.terms[1] = v("unbound_predicate");
    } else {
        panic!("fixture shape");
    }
    assert!(p.validate().is_err());
    assert!(
        serde_json::from_str::<Vec<Program>>(&serde_json::to_string(&vec![p.clone()]).unwrap())
            .is_err()
    );
    assert!(
        postcard::from_bytes::<Vec<Program>>(&postcard::to_allocvec(&vec![p]).unwrap()).is_err()
    );
}
