//! Public predicate strata must preserve every dependency constraint.

use datalog_structural_model::{
    Atom, ConstantSymbol, Literal, PredicateSymbol, Program, Rule, Term,
};

fn atom(predicate: &str, value: &str) -> Atom {
    Atom {
        predicate: PredicateSymbol::new(predicate),
        terms: vec![Term::Constant(ConstantSymbol::new(value))],
    }
}

fn producer_rules() -> Vec<Rule> {
    vec![
        Rule::positive(atom("p", "a"), vec![]),
        Rule::new(
            atom("p", "b"),
            vec![Literal::Negative(atom("excluded", "b"))],
        ),
    ]
}

#[test]
fn negative_consumer_follows_every_context_of_its_predicate() {
    let mut rules = producer_rules();
    rules.push(Rule::new(
        atom("q", "a"),
        vec![Literal::Negative(atom("p", "a"))],
    ));
    let schedule = Program::try_new(rules).unwrap().stratify().unwrap();
    assert!(
        schedule.stratum_of(&PredicateSymbol::new("q")).unwrap()
            > schedule.stratum_of(&PredicateSymbol::new("p")).unwrap()
    );
}

#[test]
fn positive_consumer_cannot_precede_its_predicate_producer() {
    let mut rules = producer_rules();
    rules.push(Rule::positive(atom("q", "a"), vec![atom("p", "a")]));
    let schedule = Program::try_new(rules).unwrap().stratify().unwrap();
    assert!(
        schedule.stratum_of(&PredicateSymbol::new("q")).unwrap()
            >= schedule.stratum_of(&PredicateSymbol::new("p")).unwrap()
    );
}

#[test]
fn a_negation_cycle_between_contexts_cannot_use_a_predicate_schedule() {
    let program = Program {
        rules: vec![
            Rule::positive(atom("p", "a"), vec![]),
            Rule::new(atom("p", "b"), vec![Literal::Negative(atom("p", "a"))]),
        ],
    };
    assert!(program.validate().is_err());
    assert!(program.stratify().is_err());
    let json = serde_json::to_string(&program).unwrap();
    assert!(serde_json::from_str::<Program>(&json).is_err());
    let bytes = postcard::to_allocvec(&program).unwrap();
    assert!(postcard::from_bytes::<Program>(&bytes).is_err());
}
