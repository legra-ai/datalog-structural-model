//! Semantic data must round-trip without serializing cached implementation state.
use datalog_structural_model::{
    Atom, ConstantSymbol, PredicateSymbol, Program, Rule, Term, VariableSymbol,
};
use serde::{Serialize, de::DeserializeOwned};
use std::{collections::HashMap, fmt::Debug};

fn round_trip<T: Serialize + DeserializeOwned + PartialEq + Debug>(value: &T) {
    let json = serde_json::to_string(value).unwrap();
    assert_eq!(&serde_json::from_str::<T>(&json).unwrap(), value);
    let bytes = postcard::to_allocvec(value).unwrap();
    assert_eq!(&postcard::from_bytes::<T>(&bytes).unwrap(), value);
}

#[test]
fn symbols_preserve_identity_and_rebuild_cached_hashes() {
    let symbol = ConstantSymbol::new("literal:exact lexical form");
    assert_eq!(
        serde_json::to_string(&symbol).unwrap(),
        "\"literal:exact lexical form\""
    );
    let decoded: ConstantSymbol =
        postcard::from_bytes(&postcard::to_allocvec(&symbol).unwrap()).unwrap();
    let mut values = HashMap::new();
    values.insert(symbol.clone(), 1);
    assert_eq!(values.get(&decoded), Some(&1));
    round_trip(&symbol);
    round_trip(&VariableSymbol::new("name"));
    round_trip(&PredicateSymbol::new("relation"));
}

fn atom(variable: &str) -> Atom {
    Atom {
        predicate: PredicateSymbol::new("p"),
        terms: vec![Term::Variable(VariableSymbol::new(variable))],
    }
}

#[test]
fn safe_program_round_trips() {
    let p = Program {
        rules: vec![Rule::positive(atom("x"), vec![atom("x")])],
    };
    round_trip(&p);
}

#[test]
fn unsafe_program_cannot_be_deserialized_in_either_format() {
    let p = Program {
        rules: vec![Rule::new(atom("unbound"), vec![])],
    };
    assert!(serde_json::from_str::<Program>(&serde_json::to_string(&p).unwrap()).is_err());
    assert!(postcard::from_bytes::<Program>(&postcard::to_allocvec(&p).unwrap()).is_err());
}

#[test]
fn standalone_unsafe_rule_cannot_be_deserialized() {
    let rule = Rule::new(atom("unbound"), vec![]);
    assert!(serde_json::from_str::<Rule>(&serde_json::to_string(&rule).unwrap()).is_err());
    assert!(postcard::from_bytes::<Rule>(&postcard::to_allocvec(&rule).unwrap()).is_err());
}

#[test]
fn unknown_semantic_fields_are_rejected() {
    assert!(serde_json::from_str::<Program>(r#"{"rules":[],"discardedMeaning":true}"#).is_err());
    assert!(
        serde_json::from_str::<Atom>(r#"{"predicate":"p","terms":[],"discardedMeaning":true}"#)
            .is_err()
    );
}
