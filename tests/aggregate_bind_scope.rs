//! Aggregate exports participate in subsequent BIND safety checks.

use datalog_structural_model::{
    AggregateBody, Aggregator, Atom, BindExpr, Literal, PredicateSymbol, Program,
    ProgramValidationError, Rule, Term, VariableSymbol,
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

fn aggregate() -> Literal {
    Literal::Aggregate(AggregateBody {
        result_var: variable("total"),
        aggregator: Aggregator::Sum,
        input_var: variable("value"),
        group_by: vec![variable("group")],
        inner_body: vec![Literal::Positive(atom("input", &["group", "value"]))],
    })
}

fn bind(output: &str, input: &str) -> Literal {
    Literal::Bind {
        var: variable(output),
        expr: BindExpr::Term(Term::Variable(variable(input))),
    }
}

fn program(body: Vec<Literal>) -> Program {
    Program {
        rules: vec![Rule::new(atom("output", &["copy"]), body)],
    }
}

#[test]
fn bind_can_consume_prior_aggregate_exports() {
    for input in ["total", "group"] {
        let program = program(vec![aggregate(), bind("copy", input)]);
        assert_eq!(program.validate(), Ok(()), "export: {input}");
        assert!(program.rules[0].is_safe());
    }
}

#[test]
fn aggregate_then_bind_survives_checked_loading() {
    let program = program(vec![aggregate(), bind("copy", "total")]);
    let json = serde_json::to_string(&program).unwrap();
    assert_eq!(serde_json::from_str::<Program>(&json).unwrap(), program);
    let bytes = postcard::to_allocvec(&program).unwrap();
    assert_eq!(postcard::from_bytes::<Program>(&bytes).unwrap(), program);
}

#[test]
fn bind_cannot_consume_future_aggregate_exports_or_inner_only_variables() {
    for (body, unbound) in [
        (vec![bind("copy", "total"), aggregate()], "total"),
        (vec![aggregate(), bind("copy", "value")], "value"),
    ] {
        assert_eq!(
            program(body).validate(),
            Err(ProgramValidationError::UnsafeBind {
                rule_index: 0,
                variables: vec![variable(unbound)],
            })
        );
    }
}

#[test]
fn bind_cannot_overwrite_an_exported_grouping_variable() {
    assert_eq!(
        program(vec![
            aggregate(),
            bind("group", "total"),
            bind("copy", "group")
        ])
        .validate(),
        Err(ProgramValidationError::UnsafeBind {
            rule_index: 0,
            variables: vec![variable("group")],
        })
    );
}
