//! Tuple partitions of one relation can have different rule strata.

use datalog_structural_model::{
    AggregateBody, Aggregator, Atom, ConstantSymbol, Literal, PredicateSymbol, Program, Rule, Term,
    VariableSymbol,
};

fn atom(value: &str) -> Atom {
    Atom {
        predicate: PredicateSymbol::new("p"),
        terms: vec![Term::Constant(ConstantSymbol::new(value))],
    }
}

#[test]
fn disjoint_tuple_patterns_allow_an_acyclic_negative_dependency() {
    let program = Program {
        rules: vec![
            Rule::new(atom("b"), vec![Literal::Negative(atom("a"))]),
            Rule::positive(atom("a"), vec![]),
        ],
    };
    assert!(program.validate().is_ok());
    let schedule = program.stratify().unwrap();
    assert_eq!(schedule.rule_stratum, vec![1, 0]);
    assert_eq!(
        schedule.strata[0].rules.iter().copied().collect::<Vec<_>>(),
        vec![1]
    );
    assert_eq!(
        schedule.strata[1].rules.iter().copied().collect::<Vec<_>>(),
        vec![0]
    );
    assert_eq!(schedule.stratum_of_rule(2), None);
    let json = serde_json::to_string(&schedule).unwrap();
    assert_eq!(
        serde_json::from_str::<datalog_structural_model::Stratification>(&json).unwrap(),
        schedule
    );
    let bytes = postcard::to_allocvec(&schedule).unwrap();
    assert_eq!(
        postcard::from_bytes::<datalog_structural_model::Stratification>(&bytes).unwrap(),
        schedule
    );
}

#[test]
fn reordering_rules_preserves_dependency_order() {
    for rules in [
        vec![
            Rule::positive(atom("a"), vec![]),
            Rule::new(atom("b"), vec![Literal::Negative(atom("a"))]),
        ],
        vec![
            Rule::new(atom("b"), vec![Literal::Negative(atom("a"))]),
            Rule::positive(atom("a"), vec![]),
        ],
    ] {
        let program = Program::try_new(rules).unwrap();
        let schedule = program.stratify().unwrap();
        for (index, rule) in program.rules.iter().enumerate() {
            let expected = usize::from(rule.head.terms == atom("b").terms);
            assert_eq!(schedule.stratum_of_rule(index), Some(expected));
        }
    }
}

#[test]
fn empty_program_has_no_scheduled_rules() {
    let schedule = Program::try_new(vec![]).unwrap().stratify().unwrap();
    assert!(schedule.strata.is_empty());
    assert!(schedule.rule_stratum.is_empty());
}

#[test]
fn aggregate_waits_for_each_matching_tuple_producer() {
    let variable = |name| Term::Variable(VariableSymbol::new(name));
    let tuple = |kind, value| Atom {
        predicate: PredicateSymbol::new("p"),
        terms: vec![Term::Constant(ConstantSymbol::new(kind)), value],
    };
    let program = Program::try_new(vec![
        Rule::new(
            tuple("result", variable("count")),
            vec![Literal::Aggregate(AggregateBody {
                result_var: VariableSymbol::new("count"),
                aggregator: Aggregator::Count,
                input_var: VariableSymbol::new("value"),
                group_by: vec![],
                inner_body: vec![Literal::Positive(tuple("input", variable("value")))],
            })],
        ),
        Rule::positive(
            tuple("input", Term::Constant(ConstantSymbol::new("a"))),
            vec![],
        ),
        Rule::positive(
            tuple("input", Term::Constant(ConstantSymbol::new("b"))),
            vec![],
        ),
    ])
    .unwrap();
    assert_eq!(program.stratify().unwrap().rule_stratum, vec![1, 0, 0]);
}
