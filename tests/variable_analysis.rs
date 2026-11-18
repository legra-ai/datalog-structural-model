//! Recursive variable analysis is shared by compilers and independent evaluators.
use datalog_structural_model::{Atom, ConstantSymbol, PredicateSymbol, Term, VariableSymbol};
use std::collections::BTreeSet;

fn variable(name: &str) -> Term {
    Term::Variable(VariableSymbol::new(name))
}

fn atom(terms: Vec<Term>) -> Atom {
    Atom {
        predicate: PredicateSymbol::new("relation"),
        terms,
    }
}

#[test]
fn recursive_variables_are_deduplicated_and_sorted_including_skolem_arguments() {
    let nested = Term::try_triple_term(atom(vec![variable("z"), variable("a")])).unwrap();
    let skolem = Term::try_skolem(
        ConstantSymbol::new("namespace-is-not-a-variable"),
        vec![variable("m"), variable("a")],
    )
    .unwrap();
    let value = atom(vec![nested, skolem, variable("z")]);
    let expected = BTreeSet::from([
        VariableSymbol::new("a"),
        VariableSymbol::new("m"),
        VariableSymbol::new("z"),
    ]);
    assert_eq!(value.variables(), expected);
    let term = Term::try_triple_term(value.clone()).unwrap();
    assert_eq!(term.variables(), expected);
    let json: Vec<Term> =
        serde_json::from_str(&serde_json::to_string(&vec![term.clone()]).unwrap()).unwrap();
    let binary: Vec<Term> =
        postcard::from_bytes(&postcard::to_allocvec(&vec![term]).unwrap()).unwrap();
    assert_eq!(json[0].variables(), expected);
    assert_eq!(binary[0].variables(), expected);
}

#[test]
fn constants_empty_atoms_and_ground_structures_have_no_variables() {
    let constant = Term::Constant(ConstantSymbol::new("opaque:?not-a-variable"));
    assert!(constant.variables().is_empty());
    assert!(atom(vec![]).variables().is_empty());
    let ground = Term::try_triple_term(atom(vec![constant])).unwrap();
    assert!(ground.variables().is_empty());
}

#[test]
fn borrowed_visitors_preserve_occurrence_order_without_cloning_symbols() {
    let value = atom(vec![
        variable("z"),
        Term::try_triple_term(atom(vec![variable("a"), variable("z")])).unwrap(),
        Term::try_skolem(ConstantSymbol::new("label"), vec![variable("m")]).unwrap(),
    ]);
    let mut seen: Vec<&VariableSymbol> = Vec::new();
    value.visit_variables(&mut |variable| seen.push(variable));
    assert_eq!(
        seen.iter().map(|v| v.as_str()).collect::<Vec<_>>(),
        ["z", "a", "z", "m"]
    );
    let term = Term::try_triple_term(value).unwrap();
    let mut distinct: BTreeSet<&VariableSymbol> = BTreeSet::new();
    term.visit_variables(&mut |variable| {
        distinct.insert(variable);
    });
    assert_eq!(
        distinct.into_iter().map(|v| v.as_str()).collect::<Vec<_>>(),
        ["a", "m", "z"]
    );
}
