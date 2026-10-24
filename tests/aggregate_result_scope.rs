//! Aggregate results must be fresh relative to prior exported variables.

use datalog_structural_model::{
    AggregateBody, Aggregator, Atom, Literal, PredicateSymbol, Program, ProgramValidationError,
    Rule, Term, VariableSymbol,
};

fn variable(name: &str) -> VariableSymbol {
    VariableSymbol::new(name)
}

fn atom(predicate: &str, names: &[&str]) -> Atom {
    Atom {
        predicate: PredicateSymbol::new(predicate),
        terms: names
            .iter()
            .map(|name| Term::Variable(variable(name)))
            .collect(),
    }
}

fn aggregate(result: &str, groups: &[&str]) -> Literal {
    Literal::Aggregate(AggregateBody {
        result_var: variable(result),
        aggregator: Aggregator::Count,
        input_var: variable("value"),
        group_by: groups.iter().map(|name| variable(name)).collect(),
        inner_body: vec![Literal::Positive(atom("input", &["group", "value"]))],
    })
}

fn program(result: &str) -> Program {
    Program {
        rules: vec![Rule::new(
            atom("output", &[result]),
            vec![aggregate("total", &["group"]), aggregate(result, &[])],
        )],
    }
}

#[test]
fn aggregate_result_cannot_overwrite_a_prior_grouping_export() {
    let program = program("group");
    assert_eq!(
        program.validate(),
        Err(ProgramValidationError::UnsafeAggregate {
            rule_index: 0,
            variables: vec![variable("group")],
        })
    );
    assert!(!program.rules[0].is_safe());
}

#[test]
fn checked_loading_rejects_aggregate_result_overwriting_a_grouping_export() {
    let program = program("group");
    let json = serde_json::to_string(&program).unwrap();
    assert!(serde_json::from_str::<Program>(&json).is_err());
    let bytes = postcard::to_allocvec(&program).unwrap();
    assert!(postcard::from_bytes::<Program>(&bytes).is_err());
}

#[test]
fn prior_inner_only_variables_do_not_prevent_a_fresh_aggregate_result() {
    let program = program("value");
    assert_eq!(program.validate(), Ok(()));
    assert!(program.rules[0].is_safe());
    let json = serde_json::to_string(&program).unwrap();
    assert_eq!(serde_json::from_str::<Program>(&json).unwrap(), program);
    let bytes = postcard::to_allocvec(&program).unwrap();
    assert_eq!(postcard::from_bytes::<Program>(&bytes).unwrap(), program);
}
