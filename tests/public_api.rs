//! The published package exposes the shared Datalog structural model.
use datalog_structural_model::{
    Atom, ConstantSymbol, Literal, PredicateSymbol, Program, Rule, Term, VariableSymbol,
};

#[test]
fn renamed_package_validates_a_recursive_rule_program() {
    assert_eq!(env!("CARGO_PKG_NAME"), "datalog-structural-model");
    let x = Term::Variable(VariableSymbol::new("x"));
    let edge = Atom {
        predicate: PredicateSymbol::new("edge"),
        terms: vec![x.clone(), Term::Constant(ConstantSymbol::new("target"))],
    };
    let rule = Rule {
        head: edge.clone(),
        body: vec![Literal::Positive(edge)],
    };
    let program = Program { rules: vec![rule] };
    program.validate().unwrap();
}
