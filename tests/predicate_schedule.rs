//! Rule strata preserve dependencies without coupling disjoint tuple patterns.

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
fn negative_consumer_follows_its_matching_producer() {
    let mut rules = producer_rules();
    rules.push(Rule::new(
        atom("q", "a"),
        vec![Literal::Negative(atom("p", "a"))],
    ));
    let schedule = Program::try_new(rules).unwrap().stratify().unwrap();
    assert!(schedule.stratum_of_rule(2).unwrap() > schedule.stratum_of_rule(0).unwrap());
}

#[test]
fn positive_consumer_cannot_precede_its_predicate_producer() {
    let mut rules = producer_rules();
    rules.push(Rule::positive(atom("q", "a"), vec![atom("p", "a")]));
    let schedule = Program::try_new(rules).unwrap().stratify().unwrap();
    assert!(schedule.stratum_of_rule(2).unwrap() >= schedule.stratum_of_rule(0).unwrap());
}

#[test]
fn acyclic_negation_between_tuple_partitions_round_trips() {
    let program = Program {
        rules: vec![
            Rule::positive(atom("p", "a"), vec![]),
            Rule::new(atom("p", "b"), vec![Literal::Negative(atom("p", "a"))]),
        ],
    };
    assert!(program.validate().is_ok());
    assert_eq!(program.stratify().unwrap().rule_stratum, vec![0, 1]);
    let json = serde_json::to_string(&program).unwrap();
    assert!(serde_json::from_str::<Program>(&json).unwrap() == program);
    let bytes = postcard::to_allocvec(&program).unwrap();
    assert!(postcard::from_bytes::<Program>(&bytes).unwrap() == program);
}
