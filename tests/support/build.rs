//! Terse program builders shared by termination tests.
use datalog_structural_model::{
    Atom, BindExpr, ConstantSymbol, Literal, PredicateSymbol, Rule, Term, VariableSymbol,
};

pub fn var(name: &str) -> Term {
    Term::Variable(VariableSymbol::new(name))
}

pub fn constant(name: &str) -> Term {
    Term::Constant(ConstantSymbol::new(name))
}

pub fn skolem(name: &str, args: Vec<Term>) -> Term {
    Term::Skolem {
        name: ConstantSymbol::new(name),
        args,
    }
}

pub fn triple(subject: Term, predicate: &str, object: Term) -> Term {
    Term::TripleTerm(Box::new(atom(predicate, vec![subject, object])))
}

pub fn atom(predicate: &str, terms: Vec<Term>) -> Atom {
    Atom {
        predicate: PredicateSymbol::new(predicate),
        terms,
    }
}

pub fn rule(head: Atom, body: Vec<Atom>) -> Rule {
    Rule::positive(head, body)
}

pub fn increment(input: &str, output: &str) -> Literal {
    Literal::Bind {
        var: VariableSymbol::new(output),
        expr: BindExpr::Add(
            Box::new(BindExpr::Term(var(input))),
            Box::new(BindExpr::Term(constant("1"))),
        ),
    }
}
