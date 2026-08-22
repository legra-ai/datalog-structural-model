//! Unit tests for the Datalog IR.

use std::collections::BTreeSet;

use super::aggregate::{AggregateBody, Aggregator};
use super::atom::Atom;
use super::bind::BindExpr;
use super::equality_mode::EqualityMode;
use super::error::ProgramValidationError;
use super::filter::BuiltinFilter;
use super::literal::Literal;
use super::program::Program;
use super::rule::Rule;
use super::symbols::{ConstantSymbol, PredicateSymbol, VariableSymbol};
use super::term::{MAX_TRIPLE_TERM_NESTING, Term};

fn var(name: &str) -> Term {
    Term::Variable(VariableSymbol::new(name))
}

fn constant(name: &str) -> Term {
    Term::Constant(ConstantSymbol::new(name))
}

fn atom(predicate: &str, terms: Vec<Term>) -> Atom {
    Atom {
        predicate: PredicateSymbol::new(predicate),
        terms,
    }
}

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
            rule_index: 0,
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
            rule_index: 0,
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
            rule_index: 0,
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
    assert!(strat.stratum_of(&PredicateSymbol::new("p")).is_some());
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
            rule_index: 0,
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
            rule_index: 0,
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
        ProgramValidationError::UnstratifiableNegation { .. }
    ));
}

// ── TT0: Term::TripleTerm IR extension ─────────────────────

/// Build a triple-term wrapping a 2-ary atom `pred(s, o)`.
/// Mirrors the RDF-star encoding of `<< s pred o >>`.
fn triple(subject: Term, predicate: &str, object: Term) -> Term {
    Term::TripleTerm(Box::new(atom(predicate, vec![subject, object])))
}

/// Safe rule: a triple-term in the body whose nested variable
/// also appears in the head is range-restricted (the body's
/// positive atom binds it).
#[test]
fn triple_term_in_body_recognises_nested_variables() {
    // annotate(?stmt, ?author) :- says(<< ?x knows ?y >>, ?author).
    // ?x and ?y are bound by the body atom; the rule references
    // ?author in the head, which the body literal binds.
    let program = Program {
        rules: vec![Rule::positive(
            atom("annotate", vec![var("x"), var("author")]),
            vec![atom(
                "says",
                vec![triple(var("x"), "knows", var("y")), var("author")],
            )],
        )],
    };
    assert!(
        program.validate().is_ok(),
        "expected nested ?x to count as bound: {:?}",
        program.validate(),
    );
}

/// Safe rule: head wraps a triple-term whose nested variables
/// are all bound by the body.
#[test]
fn triple_term_in_head_with_bound_nested_variables_is_safe() {
    // believes(?a, << ?x knows ?y >>) :- says(?a, ?x, ?y).
    let program = Program {
        rules: vec![Rule::positive(
            atom(
                "believes",
                vec![var("a"), triple(var("x"), "knows", var("y"))],
            ),
            vec![atom("says", vec![var("a"), var("x"), var("y")])],
        )],
    };
    assert!(program.validate().is_ok());
}

/// Unsafe rule: a variable nested inside a head triple-term
/// is not bound by any positive body atom — must be rejected
/// with the same `UnsafeRule` error as for top-level
/// unsafety.
#[test]
fn triple_term_in_head_with_unbound_nested_variable_is_unsafe() {
    // believes(?a, << ?x knows ?y >>) :- says(?a).
    // ?x and ?y are not bound by the body — unsafe head.
    let program = Program {
        rules: vec![Rule::positive(
            atom(
                "believes",
                vec![var("a"), triple(var("x"), "knows", var("y"))],
            ),
            vec![atom("says", vec![var("a")])],
        )],
    };
    let err = program.validate().expect_err("program should be unsafe");
    let ProgramValidationError::UnsafeRule {
        rule_index,
        variables,
    } = err
    else {
        panic!("expected UnsafeRule, got {err:?}");
    };
    assert_eq!(rule_index, 0);
    let names: BTreeSet<String> = variables.iter().map(|v| v.as_str().to_owned()).collect();
    let expected: BTreeSet<String> = ["x", "y"].iter().map(|s| (*s).to_owned()).collect();
    assert_eq!(names, expected);
}

/// Unsafe rule: a negated body literal contains a triple-term
/// whose nested variable is not bound by any positive body
/// atom. Negation-as-failure requires the negated atom to be
/// ground at evaluation time, so the nested variable counts
/// just like a top-level negated-body variable.
#[test]
fn negated_triple_term_with_unbound_nested_variable_is_unsafe() {
    // ok(?a) :- says(?a), not annotated(<< ?a knows ?y >>).
    // ?y appears only inside the negated triple-term — unsafe.
    let program = Program {
        rules: vec![Rule::new(
            atom("ok", vec![var("a")]),
            vec![
                Literal::Positive(atom("says", vec![var("a")])),
                Literal::Negative(atom("annotated", vec![triple(var("a"), "knows", var("y"))])),
            ],
        )],
    };
    let err = program.validate().expect_err("program should be unsafe");
    let ProgramValidationError::UnsafeNegation {
        rule_index,
        variables,
    } = err
    else {
        panic!("expected UnsafeNegation, got {err:?}");
    };
    assert_eq!(rule_index, 0);
    let names: BTreeSet<String> = variables.iter().map(|v| v.as_str().to_owned()).collect();
    assert_eq!(
        names,
        ["y".to_owned()].iter().cloned().collect::<BTreeSet<_>>(),
    );
}

/// Triple-term nesting deeper than [`MAX_TRIPLE_TERM_NESTING`]
/// must be rejected. Builds a 5-deep nesting (one more than
/// the default limit of 4).
#[test]
fn excessive_triple_term_nesting_is_rejected() {
    // Build `<< <s> p << <s> p << <s> p << <s> p << <s> p <s> >> >> >> >> >>`
    // — five levels of nesting.
    let leaf = || constant("s");
    let mut nested = triple(leaf(), "p", leaf());
    for _ in 0..4 {
        nested = triple(leaf(), "p", nested);
    }
    let program = Program {
        rules: vec![Rule::positive(
            atom("out", vec![nested]),
            vec![atom("base", vec![])],
        )],
    };
    let err = program.validate().expect_err("nesting must be rejected");
    let ProgramValidationError::TripleTermNestingExceeded {
        rule_index,
        depth,
        limit,
    } = err
    else {
        panic!("expected TripleTermNestingExceeded, got {err:?}");
    };
    assert_eq!(rule_index, 0);
    assert_eq!(limit, MAX_TRIPLE_TERM_NESTING);
    assert!(depth > limit, "depth {depth} must exceed limit {limit}");
}

/// `Display` for a triple term yields the RDF-star
/// `<< s p o >>` form for 2-arity atoms (the standard RDF
/// case).
#[test]
fn triple_term_displays_with_rdf_star_syntax() {
    let t = triple(constant("alice"), "knows", constant("bob"));
    assert_eq!(format!("{t}"), "<< alice knows bob >>");
}

// ── extension aggregation IR ────────────────────────────────────────

fn agg_body(
    result: &str,
    aggregator: Aggregator,
    input: &str,
    group_by: Vec<&str>,
    inner_atoms: Vec<Atom>,
) -> AggregateBody {
    agg_body_with_inner(
        result,
        aggregator,
        input,
        group_by,
        inner_atoms.into_iter().map(Literal::Positive).collect(),
    )
}

fn agg_body_with_inner(
    result: &str,
    aggregator: Aggregator,
    input: &str,
    group_by: Vec<&str>,
    inner_body: Vec<Literal>,
) -> AggregateBody {
    AggregateBody {
        result_var: VariableSymbol::new(result),
        aggregator,
        input_var: VariableSymbol::new(input),
        group_by: group_by.into_iter().map(VariableSymbol::new).collect(),
        inner_body,
    }
}

/// extension — a well-formed aggregate body literal whose
/// `input_var` and `group_by` are bound by the inner body and
/// whose `result_var` is fresh passes validation.
#[test]
fn safe_aggregate_passes_validation() {
    // total(?dept, ?total) :- AGG ?sal SUM(?sal) GROUP BY ?dept
    //                          WHERE { salary(?e, ?dept, ?sal) }
    let program = Program {
        rules: vec![Rule::new(
            atom("total", vec![var("dept"), var("total")]),
            vec![Literal::Aggregate(agg_body(
                "total",
                Aggregator::Sum,
                "sal",
                vec!["dept"],
                vec![atom("salary", vec![var("e"), var("dept"), var("sal")])],
            ))],
        )],
    };
    assert!(program.validate().is_ok());
}

/// extension — `input_var` not produced by the inner body is an
/// `UnsafeAggregate` failure.
#[test]
fn unsafe_aggregate_unbound_input_fails_validation() {
    let program = Program {
        rules: vec![Rule::new(
            atom("total", vec![var("dept"), var("total")]),
            vec![Literal::Aggregate(agg_body(
                "total",
                Aggregator::Sum,
                "missing_sal", // <-- not in inner body
                vec!["dept"],
                vec![atom("salary", vec![var("e"), var("dept"), var("sal")])],
            ))],
        )],
    };
    let err = program.validate().unwrap_err();
    let ProgramValidationError::UnsafeAggregate {
        rule_index,
        variables,
    } = err
    else {
        panic!("expected UnsafeAggregate, got {err:?}");
    };
    assert_eq!(rule_index, 0);
    assert!(
        variables.contains(&VariableSymbol::new("missing_sal")),
        "expected `missing_sal` in offending list, got {variables:?}"
    );
}

/// extension — a `group_by` key not bound by the inner body is an
/// `UnsafeAggregate` failure.
#[test]
fn unsafe_aggregate_unbound_group_key_fails_validation() {
    let program = Program {
        rules: vec![Rule::new(
            atom("total", vec![var("region"), var("total")]),
            vec![Literal::Aggregate(agg_body(
                "total",
                Aggregator::Sum,
                "sal",
                vec!["region"], // <-- not in inner body
                vec![atom("salary", vec![var("e"), var("dept"), var("sal")])],
            ))],
        )],
    };
    let err = program.validate().unwrap_err();
    let ProgramValidationError::UnsafeAggregate { variables, .. } = err else {
        panic!("expected UnsafeAggregate, got {err:?}");
    };
    assert!(
        variables.contains(&VariableSymbol::new("region")),
        "expected `region` in offending list, got {variables:?}"
    );
}

/// extension — `result_var` colliding with a variable bound by a
/// positive body atom is an `UnsafeAggregate` failure.
#[test]
fn unsafe_aggregate_result_var_collides_fails_validation() {
    let program = Program {
        rules: vec![Rule::new(
            atom("h", vec![var("dept"), var("total")]),
            vec![
                Literal::Positive(atom("anchor", vec![var("total")])),
                Literal::Aggregate(agg_body(
                    "total", // <-- collides with anchor's binding
                    Aggregator::Sum,
                    "sal",
                    vec!["dept"],
                    vec![atom("salary", vec![var("e"), var("dept"), var("sal")])],
                )),
            ],
        )],
    };
    let err = program.validate().unwrap_err();
    let ProgramValidationError::UnsafeAggregate { variables, .. } = err else {
        panic!("expected UnsafeAggregate, got {err:?}");
    };
    assert!(
        variables.contains(&VariableSymbol::new("total")),
        "expected `total` in offending list, got {variables:?}"
    );
}

/// extension — an aggregate's exported variables (`result_var` +
/// `group_by` keys) satisfy outer-rule head safety. The head
/// references variables only the aggregate exports; the rule
/// passes validation.
#[test]
fn aggregate_exports_satisfy_head_safety() {
    let program = Program {
        rules: vec![Rule::new(
            atom("total", vec![var("dept"), var("total")]),
            vec![Literal::Aggregate(agg_body(
                "total",
                Aggregator::Count,
                "sal",
                vec!["dept"],
                vec![atom("salary", vec![var("e"), var("dept"), var("sal")])],
            ))],
        )],
    };
    assert!(program.validate().is_ok());
}

/// extension — a rule whose head references the inner-only variable
/// of an aggregate is rejected as unsafe-head. Inner variables
/// do not leak past the aggregate boundary.
#[test]
fn aggregate_inner_only_variable_does_not_leak_to_head() {
    let program = Program {
        rules: vec![Rule::new(
            atom("h", vec![var("e"), var("total")]),
            vec![Literal::Aggregate(agg_body(
                "total",
                Aggregator::Count,
                "sal",
                vec!["dept"],
                vec![atom("salary", vec![var("e"), var("dept"), var("sal")])],
            ))],
        )],
    };
    let err = program.validate().unwrap_err();
    let ProgramValidationError::UnsafeRule { variables, .. } = err else {
        panic!("expected UnsafeRule, got {err:?}");
    };
    assert!(
        variables.contains(&VariableSymbol::new("e")),
        "inner-only `e` must not satisfy head safety; got {variables:?}"
    );
}

/// extension — recursion through an aggregate body literal is
/// rejected the same way negation cycles are: the aggregate's
/// inner predicate must sit in a strictly lower stratum than
/// the rule's head.
#[test]
fn recursion_through_aggregate_is_rejected() {
    // r(?d, ?t) :- AGG ?sal COUNT(?sal) GROUP BY ?d WHERE { r(?d, ?sal) }
    let program = Program {
        rules: vec![Rule::new(
            atom("r", vec![var("d"), var("t")]),
            vec![Literal::Aggregate(agg_body(
                "t",
                Aggregator::Count,
                "sal",
                vec!["d"],
                vec![atom("r", vec![var("d"), var("sal")])],
            ))],
        )],
    };
    let err = program.validate().unwrap_err();
    let ProgramValidationError::UnstratifiableNegation { predicates } = err else {
        panic!("expected UnstratifiableNegation, got {err:?}");
    };
    assert!(
        predicates.iter().any(|p| p.as_str() == "r"),
        "expected `r` in unstratifiable cycle, got {predicates:?}"
    );
}

/// extension — an aggregate whose inner body carries an inner
/// negation alongside an inner positive atom validates, as long
/// as every variable named by the negated atom is bound by some
/// inner positive atom or `BIND` (the inner-scope range
/// restriction).
#[test]
fn safe_aggregate_inner_negation_passes_validation() {
    // total_unflagged(?dept, ?total) :- AGG ?sal SUM(?sal)
    //   GROUP BY ?dept
    //   WHERE { salary(?e, ?dept, ?sal), NOT flagged(?e) }
    let program = Program {
        rules: vec![Rule::new(
            atom("total_unflagged", vec![var("dept"), var("total")]),
            vec![Literal::Aggregate(agg_body_with_inner(
                "total",
                Aggregator::Sum,
                "sal",
                vec!["dept"],
                vec![
                    Literal::Positive(atom("salary", vec![var("e"), var("dept"), var("sal")])),
                    Literal::Negative(atom("flagged", vec![var("e")])),
                ],
            ))],
        )],
    };
    assert!(program.validate().is_ok());
}

/// extension — an inner negation that names a variable no inner
/// positive atom or `BIND` binds is rejected as `UnsafeAggregate`.
/// Mirrors the outer-rule `UnsafeNegation` check applied to the
/// inner scope.
#[test]
fn unsafe_aggregate_inner_negation_unbound_var_fails_validation() {
    let program = Program {
        rules: vec![Rule::new(
            atom("h", vec![var("dept"), var("total")]),
            vec![Literal::Aggregate(agg_body_with_inner(
                "total",
                Aggregator::Sum,
                "sal",
                vec!["dept"],
                vec![
                    Literal::Positive(atom("salary", vec![var("e"), var("dept"), var("sal")])),
                    // `m` is not bound by any inner positive atom or BIND.
                    Literal::Negative(atom("flagged", vec![var("m")])),
                ],
            ))],
        )],
    };
    let err = program.validate().unwrap_err();
    let ProgramValidationError::UnsafeAggregate { variables, .. } = err else {
        panic!("expected UnsafeAggregate, got {err:?}");
    };
    assert!(
        variables.contains(&VariableSymbol::new("m")),
        "expected unbound `m` in offending list, got {variables:?}"
    );
}

/// extension — an inner BIND whose output variable feeds the
/// aggregate's `input_var` or a group key is safe and validates.
/// Exercises the inner-scope `BIND` safety path.
#[test]
fn safe_aggregate_inner_bind_feeds_input_var() {
    // total_bonus(?dept, ?total) :- AGG ?bonus SUM(?bonus)
    //   GROUP BY ?dept
    //   WHERE { salary(?e, ?dept, ?sal), BIND(?sal + ?sal AS ?bonus) }
    let program = Program {
        rules: vec![Rule::new(
            atom("total_bonus", vec![var("dept"), var("total")]),
            vec![Literal::Aggregate(agg_body_with_inner(
                "total",
                Aggregator::Sum,
                "bonus",
                vec!["dept"],
                vec![
                    Literal::Positive(atom("salary", vec![var("e"), var("dept"), var("sal")])),
                    Literal::Bind {
                        var: VariableSymbol::new("bonus"),
                        expr: BindExpr::Add(
                            Box::new(BindExpr::Term(var("sal"))),
                            Box::new(BindExpr::Term(var("sal"))),
                        ),
                    },
                ],
            ))],
        )],
    };
    assert!(program.validate().is_ok());
}

/// extension — an inner filter referencing a variable bound by an
/// inner positive atom validates. Mirrors the outer-rule
/// `UnsafeFilter` rules applied to the inner scope.
#[test]
fn safe_aggregate_inner_filter_passes_validation() {
    let program = Program {
        rules: vec![Rule::new(
            atom("high_total", vec![var("dept"), var("total")]),
            vec![Literal::Aggregate(agg_body_with_inner(
                "total",
                Aggregator::Sum,
                "sal",
                vec!["dept"],
                vec![
                    Literal::Positive(atom("salary", vec![var("e"), var("dept"), var("sal")])),
                    Literal::Filter(BuiltinFilter::Gt(var("sal"), constant("0"))),
                ],
            ))],
        )],
    };
    assert!(program.validate().is_ok());
}

/// extension — a nested `Literal::Aggregate` inside another
/// aggregate's `inner_body` surfaces as
/// [`ProgramValidationError::NestedAggregateInInnerBody`].
#[test]
fn nested_aggregate_in_inner_body_is_rejected() {
    let inner_agg = AggregateBody {
        result_var: VariableSymbol::new("inner_total"),
        aggregator: Aggregator::Sum,
        input_var: VariableSymbol::new("sal"),
        group_by: vec![VariableSymbol::new("dept")],
        inner_body: vec![Literal::Positive(atom(
            "salary",
            vec![var("e"), var("dept"), var("sal")],
        ))],
    };
    let outer_agg = AggregateBody {
        result_var: VariableSymbol::new("outer_total"),
        aggregator: Aggregator::Sum,
        input_var: VariableSymbol::new("inner_total"),
        group_by: vec![],
        inner_body: vec![Literal::Aggregate(inner_agg)],
    };
    let program = Program {
        rules: vec![Rule::new(
            atom("grand_total", vec![var("outer_total")]),
            vec![Literal::Aggregate(outer_agg)],
        )],
    };
    let err = program.validate().unwrap_err();
    assert_eq!(
        err,
        ProgramValidationError::NestedAggregateInInnerBody { rule_index: 0 }
    );
}

/// extension — `Aggregator::name` returns the canonical uppercase
/// rendering for every operator.
#[test]
fn aggregator_name_renders_canonical_strings() {
    assert_eq!(Aggregator::Count.name(), "COUNT");
    assert_eq!(Aggregator::Sum.name(), "SUM");
    assert_eq!(Aggregator::Min.name(), "MIN");
    assert_eq!(Aggregator::Max.name(), "MAX");
    assert_eq!(Aggregator::Avg.name(), "AVG");
}

/// extension — `AggregateBody::exported_variables` exposes
/// `result_var` plus every `group_by` key; inner-only variables
/// stay scoped.
#[test]
fn aggregate_exported_variables_match_outer_scope() {
    let agg = agg_body(
        "t",
        Aggregator::Sum,
        "sal", // inner-only
        vec!["dept"],
        vec![atom("salary", vec![var("e"), var("dept"), var("sal")])],
    );
    let exported = agg.exported_variables();
    let expected: BTreeSet<VariableSymbol> = ["t", "dept"]
        .iter()
        .copied()
        .map(VariableSymbol::new)
        .collect();
    assert_eq!(exported, expected);
}

// ── extension — SKOLEM rule-head terms ──────────────────────────────

/// extension — a head atom carrying a `SKOLEM(...)` term whose
/// arguments are bound by the positive body validates. Mirrors
/// the existing head-variable safety check applied to SKOLEM args.
#[test]
fn safe_skolem_head_passes_validation() {
    // hasDerivedKey(?x, SKOLEM("example-key", ?a, ?b)) :- pair(?x, ?a, ?b).
    let head = Atom {
        predicate: PredicateSymbol::new("hasDerivedKey"),
        terms: vec![
            var("x"),
            Term::Skolem {
                name: ConstantSymbol::new("example-key"),
                args: vec![var("a"), var("b")],
            },
        ],
    };
    let program = Program {
        rules: vec![Rule::positive(
            head,
            vec![atom("pair", vec![var("x"), var("a"), var("b")])],
        )],
    };
    assert!(program.validate().is_ok());
}

/// extension — a `SKOLEM(...)` term whose argument is not bound by
/// the positive body surfaces as `UnsafeRule` (the inner variables
/// flow into head-variable safety).
#[test]
fn unsafe_skolem_head_unbound_arg_fails_validation() {
    let head = Atom {
        predicate: PredicateSymbol::new("h"),
        terms: vec![
            var("x"),
            Term::Skolem {
                name: ConstantSymbol::new("example-key"),
                // `?orphan` never appears in the body.
                args: vec![var("x"), var("orphan")],
            },
        ],
    };
    let program = Program {
        rules: vec![Rule::positive(head, vec![atom("anchor", vec![var("x")])])],
    };
    let err = program
        .validate()
        .expect_err("orphan SKOLEM arg should reject");
    let ProgramValidationError::UnsafeRule { variables, .. } = err else {
        panic!("expected UnsafeRule, got {err:?}");
    };
    assert!(
        variables.contains(&VariableSymbol::new("orphan")),
        "expected `orphan` in offending list, got {variables:?}"
    );
}

/// extension — a `SKOLEM(...)` term appearing in a positive body
/// atom surfaces as
/// [`ProgramValidationError::SkolemInBodyAtom`]. SKOLEM is a
/// function, not a pattern, so a body match against a freshly
/// minted IRI would unify against a value the EDB can't produce.
#[test]
fn skolem_in_positive_body_atom_is_rejected() {
    let body_atom = Atom {
        predicate: PredicateSymbol::new("p"),
        terms: vec![
            var("x"),
            Term::Skolem {
                name: ConstantSymbol::new("example-key"),
                args: vec![var("x")],
            },
        ],
    };
    let program = Program {
        rules: vec![Rule::positive(atom("h", vec![var("x")]), vec![body_atom])],
    };
    let err = program
        .validate()
        .expect_err("SKOLEM in body should reject");
    assert_eq!(
        err,
        ProgramValidationError::SkolemInBodyAtom { rule_index: 0 }
    );
}

/// extension — a `SKOLEM(...)` term appearing in a negative body
/// atom is rejected with the same diagnostic as the positive
/// case. Negation flips polarity but not the no-pattern-match
/// argument against SKOLEM body terms.
#[test]
fn skolem_in_negative_body_atom_is_rejected() {
    let negated = Atom {
        predicate: PredicateSymbol::new("p"),
        terms: vec![
            var("x"),
            Term::Skolem {
                name: ConstantSymbol::new("example-key"),
                args: vec![var("x")],
            },
        ],
    };
    let program = Program {
        rules: vec![Rule::new(
            atom("h", vec![var("x")]),
            vec![
                Literal::Positive(atom("anchor", vec![var("x")])),
                Literal::Negative(negated),
            ],
        )],
    };
    let err = program
        .validate()
        .expect_err("SKOLEM in negative body should reject");
    assert_eq!(
        err,
        ProgramValidationError::SkolemInBodyAtom { rule_index: 0 }
    );
}

/// extension — a SKOLEM term nested inside an aggregate's
/// `inner_body` is rejected with the same diagnostic. Aggregate
/// inner atoms are body atoms by another name.
#[test]
fn skolem_in_aggregate_inner_body_is_rejected() {
    let inner = Atom {
        predicate: PredicateSymbol::new("salary"),
        terms: vec![
            var("e"),
            var("dept"),
            Term::Skolem {
                name: ConstantSymbol::new("example-key"),
                args: vec![var("e")],
            },
        ],
    };
    let program = Program {
        rules: vec![Rule::new(
            atom("total", vec![var("dept"), var("total")]),
            vec![Literal::Aggregate(AggregateBody {
                result_var: VariableSymbol::new("total"),
                aggregator: Aggregator::Sum,
                input_var: VariableSymbol::new("dept"),
                group_by: vec![VariableSymbol::new("dept")],
                inner_body: vec![Literal::Positive(inner)],
            })],
        )],
    };
    let err = program
        .validate()
        .expect_err("SKOLEM in aggregate inner should reject");
    assert_eq!(
        err,
        ProgramValidationError::SkolemInBodyAtom { rule_index: 0 }
    );
}

// ── extension — negation / aggregation under equality mode ──────────

/// extension — a program with only positive atoms validates under
/// `EqualityMode::SameAs`. The check only rejects negation and
/// aggregation.
#[test]
fn equality_mode_sameas_accepts_positive_only_program() {
    let program = Program {
        rules: vec![Rule::positive(
            atom("parent", vec![var("x"), var("y")]),
            vec![atom("has_parent", vec![var("x"), var("y")])],
        )],
    };
    assert!(
        program
            .validate_with_equality_mode(EqualityMode::SameAs)
            .is_ok()
    );
}

/// extension — a program containing a negative literal is rejected
/// under `EqualityMode::SameAs` with
/// `ProgramValidationError::NegationUnderEqualityMode`.
#[test]
fn equality_mode_sameas_rejects_negation() {
    let program = Program {
        rules: vec![Rule::new(
            atom("bachelor", vec![var("x")]),
            vec![
                Literal::Positive(atom("person", vec![var("x")])),
                Literal::Negative(atom("married", vec![var("x")])),
            ],
        )],
    };
    // Under the default mode the same program validates.
    assert!(program.validate().is_ok());
    let err = program
        .validate_with_equality_mode(EqualityMode::SameAs)
        .expect_err("expected extension rejection");
    assert_eq!(
        err,
        ProgramValidationError::NegationUnderEqualityMode { rule_index: 0 }
    );
}

/// extension — a program containing an aggregate body literal is
/// rejected under `EqualityMode::SameAs` with
/// `ProgramValidationError::AggregationUnderEqualityMode`.
#[test]
fn equality_mode_sameas_rejects_aggregation() {
    let program = Program {
        rules: vec![Rule::new(
            atom("total", vec![var("dept"), var("total")]),
            vec![Literal::Aggregate(AggregateBody {
                result_var: VariableSymbol::new("total"),
                aggregator: Aggregator::Sum,
                input_var: VariableSymbol::new("sal"),
                group_by: vec![VariableSymbol::new("dept")],
                inner_body: vec![Literal::Positive(atom(
                    "salary",
                    vec![var("e"), var("dept"), var("sal")],
                ))],
            })],
        )],
    };
    // Default mode accepts.
    assert!(program.validate().is_ok());
    let err = program
        .validate_with_equality_mode(EqualityMode::SameAs)
        .expect_err("expected extension rejection");
    assert_eq!(
        err,
        ProgramValidationError::AggregationUnderEqualityMode { rule_index: 0 }
    );
}

/// extension — `EqualityMode::Default` (the bare `validate` path)
/// permits everything the program would otherwise pass — neither
/// negation nor aggregation gets blocked when equality mode is
/// off. Regression guard against an inverted mode comparison.
#[test]
fn equality_mode_default_permits_negation_and_aggregation() {
    let program = Program {
        rules: vec![
            Rule::new(
                atom("a", vec![var("x")]),
                vec![
                    Literal::Positive(atom("p", vec![var("x")])),
                    Literal::Negative(atom("q", vec![var("x")])),
                ],
            ),
            Rule::new(
                atom("b", vec![var("dept"), var("c")]),
                vec![Literal::Aggregate(AggregateBody {
                    result_var: VariableSymbol::new("c"),
                    aggregator: Aggregator::Count,
                    input_var: VariableSymbol::new("e"),
                    group_by: vec![VariableSymbol::new("dept")],
                    inner_body: vec![Literal::Positive(atom(
                        "works_in",
                        vec![var("e"), var("dept")],
                    ))],
                })],
            ),
        ],
    };
    assert!(program.validate().is_ok());
    assert!(
        program
            .validate_with_equality_mode(EqualityMode::Default)
            .is_ok()
    );
}

/// extension — the rejection reports the first offending rule
/// index when several rules conflict, walking the rule list in
/// declaration order. Pins the diagnostic shape so external
/// tools can surface a single targeted error rather than a
/// flood.
#[test]
fn equality_mode_sameas_reports_first_offending_rule() {
    let program = Program {
        rules: vec![
            Rule::positive(atom("ok", vec![var("x")]), vec![atom("p", vec![var("x")])]),
            Rule::new(
                atom("bad", vec![var("x")]),
                vec![
                    Literal::Positive(atom("p", vec![var("x")])),
                    Literal::Negative(atom("q", vec![var("x")])),
                ],
            ),
            Rule::new(
                atom("worse", vec![var("dept"), var("c")]),
                vec![Literal::Aggregate(AggregateBody {
                    result_var: VariableSymbol::new("c"),
                    aggregator: Aggregator::Count,
                    input_var: VariableSymbol::new("e"),
                    group_by: vec![VariableSymbol::new("dept")],
                    inner_body: vec![Literal::Positive(atom(
                        "works_in",
                        vec![var("e"), var("dept")],
                    ))],
                })],
            ),
        ],
    };
    let err = program
        .validate_with_equality_mode(EqualityMode::SameAs)
        .expect_err("expected extension rejection");
    assert_eq!(
        err,
        ProgramValidationError::NegationUnderEqualityMode { rule_index: 1 }
    );
}
