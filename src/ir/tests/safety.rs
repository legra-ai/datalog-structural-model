//! Range restriction, negation, filter and BIND safety.

use super::support::*;

#[test]
fn safe_rule_passes_validation() {
    let program = Program {
        rules: vec![Rule::positive(
            atom("parent", vec![var("x"), var("y")]),
            vec![atom("has_parent", vec![var("x"), var("y")])],
        )],
    };
    assert!(program.validate().is_ok());
}

#[test]
fn unsafe_rule_fails_validation() {
    let program = Program {
        rules: vec![Rule::positive(
            atom("parent", vec![var("x"), var("y")]),
            vec![atom("has_parent", vec![var("x"), constant("alice")])],
        )],
    };

    let err = program.validate().expect_err("program should be invalid");
    assert_eq!(
        err,
        ProgramValidationError::UnsafeRule {
            rule: rule0(),
            atom: crate::HeadAtomPath::atomic(),
            variables: vec![VariableSymbol::new("y")],
        }
    );
}

#[test]
fn safe_negation_passes_validation() {
    // bachelor(X) :- person(X), not married(X).
    let program = Program {
        rules: vec![Rule::new(
            atom("bachelor", vec![var("x")]),
            vec![
                Literal::Positive(atom("person", vec![var("x")])),
                Literal::Negative(atom("married", vec![var("x")])),
            ],
        )],
    };
    assert!(program.validate().is_ok());
}

#[test]
fn unsafe_negation_fails_validation() {
    // bad(X) :- person(X), not knows(X, Y).  — Y is only in
    // the negation, so it is range-unrestricted.
    let program = Program {
        rules: vec![Rule::new(
            atom("bad", vec![var("x")]),
            vec![
                Literal::Positive(atom("person", vec![var("x")])),
                Literal::Negative(atom("knows", vec![var("x"), var("y")])),
            ],
        )],
    };

    let err = program.validate().expect_err("program should be invalid");
    assert_eq!(
        err,
        ProgramValidationError::UnsafeNegation {
            rule: rule0(),
            literal: lit(1),
            variables: vec![VariableSymbol::new("y")],
        }
    );
}

#[test]
fn safe_filter_passes_validation() {
    // adult(?p) :- person(?p, ?age), ?age >= 18.
    let program = Program {
        rules: vec![Rule::new(
            atom("adult", vec![var("p")]),
            vec![
                Literal::Positive(atom("person", vec![var("p"), var("age")])),
                Literal::Filter(BuiltinFilter::Ge(var("age"), constant("18"))),
            ],
        )],
    };
    assert!(program.validate().is_ok());
}

#[test]
fn unsafe_filter_fails_validation() {
    // bad(?p) :- person(?p), ?ghost = "x".  — ?ghost is only
    // in the filter, not bound by any positive body literal.
    let program = Program {
        rules: vec![Rule::new(
            atom("bad", vec![var("p")]),
            vec![
                Literal::Positive(atom("person", vec![var("p")])),
                Literal::Filter(BuiltinFilter::Eq(var("ghost"), constant("x"))),
            ],
        )],
    };
    let err = program.validate().expect_err("program should be invalid");
    assert_eq!(
        err,
        ProgramValidationError::UnsafeFilter {
            rule: rule0(),
            literal: lit(1),
            variables: vec![VariableSymbol::new("ghost")],
        }
    );
}

#[test]
fn filter_does_not_contribute_to_stratification_dependencies() {
    // Two rules over disjoint predicates linked only by a
    // filter that mentions both variables. The filter must NOT
    // pull `q` into `p`'s stratum nor introduce a cycle.
    // p(?x) :- q(?x), ?x != "skip".
    let program = Program {
        rules: vec![Rule::new(
            atom("p", vec![var("x")]),
            vec![
                Literal::Positive(atom("q", vec![var("x")])),
                Literal::Filter(BuiltinFilter::Ne(var("x"), constant("skip"))),
            ],
        )],
    };
    assert!(program.validate().is_ok());
    // The IDB predicate `p` should be in a single stratum
    // alongside `q`. Filters never bump the stratum.
    let strat = program.stratify().expect("stratifiable");
    assert!(strat.stratum_of_rule(rule0()).is_some());
}

#[test]
fn safe_bind_passes_validation() {
    // tall(?p) :- person(?p, ?h), BIND(?h - 180 AS ?over),
    //             ?over > 0.
    let program = Program {
        rules: vec![Rule::new(
            atom("tall", vec![var("p")]),
            vec![
                Literal::Positive(atom("person", vec![var("p"), var("h")])),
                Literal::Bind {
                    var: VariableSymbol::new("over"),
                    expr: BindExpr::Sub(
                        Box::new(BindExpr::Term(var("h"))),
                        Box::new(BindExpr::Term(constant("180"))),
                    ),
                },
                Literal::Filter(BuiltinFilter::Gt(var("over"), constant("0"))),
            ],
        )],
    };
    assert!(program.validate().is_ok());
}

#[test]
fn unsafe_bind_expr_unbound_variable_fails_validation() {
    // bad(?p) :- person(?p), BIND(?ghost AS ?out).  — ?ghost
    // is never bound by any earlier positive atom or Bind.
    let program = Program {
        rules: vec![Rule::new(
            atom("bad", vec![var("p")]),
            vec![
                Literal::Positive(atom("person", vec![var("p")])),
                Literal::Bind {
                    var: VariableSymbol::new("out"),
                    expr: BindExpr::Term(var("ghost")),
                },
            ],
        )],
    };
    let err = program.validate().expect_err("program should be invalid");
    assert_eq!(
        err,
        ProgramValidationError::UnsafeBind {
            rule: rule0(),
            literal: lit(1),
            variables: vec![VariableSymbol::new("ghost")],
        }
    );
}

#[test]
fn unsafe_bind_lhs_collision_fails_validation() {
    // bad(?p) :- person(?p, ?h), BIND(?h AS ?h).  — ?h is
    // already bound by `person`, so the Bind would shadow it.
    let program = Program {
        rules: vec![Rule::new(
            atom("bad", vec![var("p")]),
            vec![
                Literal::Positive(atom("person", vec![var("p"), var("h")])),
                Literal::Bind {
                    var: VariableSymbol::new("h"),
                    expr: BindExpr::Term(var("h")),
                },
            ],
        )],
    };
    let err = program.validate().expect_err("program should be invalid");
    assert_eq!(
        err,
        ProgramValidationError::UnsafeBind {
            rule: rule0(),
            literal: lit(1),
            variables: vec![VariableSymbol::new("h")],
        }
    );
}

#[test]
fn bind_output_satisfies_head_safety() {
    // greeted(?msg) :- person(?p), BIND(?p AS ?msg).  — head
    // variable ?msg is produced by the Bind, not by any
    // positive atom.
    let program = Program {
        rules: vec![Rule::new(
            atom("greeted", vec![var("msg")]),
            vec![
                Literal::Positive(atom("person", vec![var("p")])),
                Literal::Bind {
                    var: VariableSymbol::new("msg"),
                    expr: BindExpr::Term(var("p")),
                },
            ],
        )],
    };
    assert!(program.validate().is_ok());
}

#[test]
fn recursion_through_negation_is_rejected() {
    // p(X) :- q(X), not p(X).  — p depends on itself through
    // negation — unstratifiable.
    let program = Program {
        rules: vec![Rule::new(
            atom("p", vec![var("x")]),
            vec![
                Literal::Positive(atom("q", vec![var("x")])),
                Literal::Negative(atom("p", vec![var("x")])),
            ],
        )],
    };

    let err = program.validate().expect_err("program should be invalid");
    assert!(matches!(
        err,
        ProgramValidationError::UnstratifiableCycle(_)
    ));
}
