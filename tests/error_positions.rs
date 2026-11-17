//! Errors name the exact rule, literal, term path, and offending edge.
use datalog_structural_model::{
    AggregateBody, Aggregator, Atom, AtomSlot, BuiltinFilter, ConstantSymbol, ExpressionTermIndex,
    Literal, LiteralIndex, LiteralPath, PredicateSymbol, Program, ProgramValidationError, Rule,
    RuleIndex, RuleSite, StrictDependency, StrictDependencyKind, Term, TermPath, TermRoot,
    UnstratifiableCycle, VariableSymbol,
};

fn var(name: &str) -> Term {
    Term::Variable(VariableSymbol::new(name))
}

fn atom(predicate: &str, terms: Vec<Term>) -> Atom {
    Atom {
        predicate: PredicateSymbol::new(predicate),
        terms,
    }
}

fn skolem() -> Term {
    Term::Skolem {
        name: ConstantSymbol::new("mint"),
        args: vec![var("x")],
    }
}

fn lit(index: usize) -> LiteralPath {
    LiteralPath::top(LiteralIndex::new(index))
}

#[test]
fn skolem_nested_in_a_triple_term_reports_the_term_path() {
    let quoted = Term::TripleTerm(Box::new(atom("knows", vec![var("x"), skolem()])));
    let program = Program {
        rules: vec![
            Rule::positive(atom("ok", vec![var("x")]), vec![atom("p", vec![var("x")])]),
            Rule::positive(
                atom("h", vec![var("x")]),
                vec![atom("p", vec![var("x")]), atom("q", vec![var("x"), quoted])],
            ),
        ],
    };
    assert_eq!(
        program.validate(),
        Err(ProgramValidationError::SkolemInBodyAtom {
            rule: RuleIndex::new(1),
            literal: lit(1),
            term: TermPath::new(TermRoot::Slot(AtomSlot::new(1))).child(AtomSlot::new(1)),
        })
    );
}

#[test]
fn skolem_in_a_filter_reports_the_expression_term() {
    let program = Program {
        rules: vec![Rule::new(
            atom("h", vec![var("x")]),
            vec![
                Literal::Positive(atom("p", vec![var("x")])),
                Literal::Filter(BuiltinFilter::Eq(var("x"), skolem())),
            ],
        )],
    };
    let error = program.validate().unwrap_err();
    assert_eq!(
        error,
        ProgramValidationError::SkolemInBodyAtom {
            rule: RuleIndex::new(0),
            literal: lit(1),
            term: TermPath::new(TermRoot::ExpressionTerm(ExpressionTermIndex::new(1))),
        }
    );
    assert_eq!(
        error.to_string(),
        "[skolem-in-body] rule 0, body literal 1, expression term 1: SKOLEM term in a body \
         literal; SKOLEM is only allowed in the head"
    );
}

#[test]
fn invalid_skolem_in_the_head_reports_head_site() {
    let bad = Term::Skolem {
        name: ConstantSymbol::new("mint"),
        args: vec![skolem()],
    };
    let program = Program {
        rules: vec![Rule::positive(
            atom("h", vec![var("x"), bad]),
            vec![atom("p", vec![var("x")])],
        )],
    };
    assert_eq!(
        program.validate(),
        Err(ProgramValidationError::InvalidSkolemArguments {
            rule: RuleIndex::new(0),
            site: RuleSite::Head,
            term: TermPath::new(TermRoot::Slot(AtomSlot::new(1))),
        })
    );
}

#[test]
fn negation_cycle_names_rules_and_the_offending_edge() {
    // 0: p(x) :- base(x).   1: q(x) :- base(x), not p(x).   2: p(x) :- q(x).
    let program = Program {
        rules: vec![
            Rule::positive(
                atom("p", vec![var("x")]),
                vec![atom("base", vec![var("x")])],
            ),
            Rule::new(
                atom("q", vec![var("x")]),
                vec![
                    Literal::Positive(atom("base", vec![var("x")])),
                    Literal::Negative(atom("p", vec![var("x")])),
                ],
            ),
            Rule::positive(atom("p", vec![var("x")]), vec![atom("q", vec![var("x")])]),
        ],
    };
    let error = program.validate().unwrap_err();
    assert_eq!(
        error,
        ProgramValidationError::UnstratifiableCycle(UnstratifiableCycle {
            rules: vec![RuleIndex::new(1), RuleIndex::new(2)],
            predicates: vec![PredicateSymbol::new("p"), PredicateSymbol::new("q")],
            edge: StrictDependency {
                producer: RuleIndex::new(2),
                consumer: RuleIndex::new(1),
                literal: lit(1),
                kind: StrictDependencyKind::Negation,
            },
        })
    );
    assert_eq!(
        error.to_string(),
        "[unstratifiable-cycle] program contains a recursive cycle through negation among \
         rules 1, 2 (predicates p, q); offending negation edge from rule 2 into rule 1, body \
         literal 1"
    );
}

#[test]
fn aggregation_cycle_names_the_inner_literal() {
    // 0: total(n) :- AGG COUNT(x) { total(x) }.
    let program = Program {
        rules: vec![Rule::new(
            atom("total", vec![var("n")]),
            vec![Literal::Aggregate(AggregateBody {
                result_var: VariableSymbol::new("n"),
                aggregator: Aggregator::Count,
                input_var: VariableSymbol::new("x"),
                group_by: vec![],
                inner_body: vec![Literal::Positive(atom("total", vec![var("x")]))],
            })],
        )],
    };
    let Err(ProgramValidationError::UnstratifiableCycle(cycle)) = program.validate() else {
        panic!("aggregation over itself must be rejected");
    };
    assert_eq!(cycle.rules, vec![RuleIndex::new(0)]);
    assert_eq!(
        cycle.edge,
        StrictDependency {
            producer: RuleIndex::new(0),
            consumer: RuleIndex::new(0),
            literal: lit(0).inner(LiteralIndex::new(0)),
            kind: StrictDependencyKind::Aggregation,
        }
    );
}
