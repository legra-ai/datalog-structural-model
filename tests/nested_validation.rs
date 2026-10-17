//! All expression positions participate in structural safety validation.
use datalog_structural_model::{
    Atom, BindExpr, BuiltinFilter, ConstantSymbol, Literal, MAX_TRIPLE_TERM_NESTING,
    PredicateSymbol, Program, Rule, Term, VariableSymbol,
};
fn var(s: &str) -> Term {
    Term::Variable(VariableSymbol::new(s))
}
fn atom(term: Term) -> Atom {
    Atom {
        predicate: PredicateSymbol::new("p"),
        terms: vec![term],
    }
}
fn triple(term: Term) -> Term {
    Term::TripleTerm(Box::new(atom(term)))
}
fn program(literal: Literal) -> Program {
    Program {
        rules: vec![Rule::new(
            atom(var("bound")),
            vec![Literal::Positive(atom(var("bound"))), literal],
        )],
    }
}
#[test]
fn variables_inside_filter_triple_terms_must_be_bound() {
    assert!(
        program(Literal::Filter(BuiltinFilter::IsIri(triple(var(
            "unbound"
        )))))
        .validate()
        .is_err()
    );
}
#[test]
fn variables_inside_bind_triple_terms_must_be_bound() {
    assert!(
        program(Literal::Bind {
            var: VariableSymbol::new("result"),
            expr: BindExpr::Term(triple(var("unbound")))
        })
        .validate()
        .is_err()
    );
}
#[test]
fn triple_depth_inside_expression_trees_is_checked() {
    let mut term = Term::Constant(ConstantSymbol::new("value"));
    for _ in 0..=MAX_TRIPLE_TERM_NESTING {
        term = triple(term);
    }
    assert!(
        program(Literal::Filter(BuiltinFilter::IsIri(term.clone())))
            .validate()
            .is_err()
    );
    assert!(
        program(Literal::Bind {
            var: VariableSymbol::new("result"),
            expr: BindExpr::Term(term)
        })
        .validate()
        .is_err()
    );
}
#[test]
fn nested_skolem_arguments_are_rejected() {
    let inner = Term::Skolem {
        name: ConstantSymbol::new("inner"),
        args: vec![var("x")],
    };
    let outer = Term::Skolem {
        name: ConstantSymbol::new("outer"),
        args: vec![inner],
    };
    assert!(
        Program {
            rules: vec![Rule::positive(atom(outer), vec![atom(var("x"))])]
        }
        .validate()
        .is_err()
    );
}

#[test]
fn skolem_terms_in_filters_and_bindings_are_rejected() {
    let skolem = Term::Skolem {
        name: ConstantSymbol::new("label"),
        args: vec![],
    };
    assert!(
        program(Literal::Filter(BuiltinFilter::IsIri(skolem.clone())))
            .validate()
            .is_err()
    );
    assert!(
        program(Literal::Bind {
            var: VariableSymbol::new("out"),
            expr: BindExpr::Term(skolem)
        })
        .validate()
        .is_err()
    );
}
