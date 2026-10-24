//! Tuple partitions of one relation can have different rule strata.

use datalog_structural_model::{
    Atom, ConstantSymbol, Literal, PredicateSymbol, Program, Rule, Term,
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
}
