//! Triple-term variables, safety and nesting.

use super::support::*;

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
    let ProgramValidationError::UnsafeRule { rule, variables } = err else {
        panic!("expected UnsafeRule, got {err:?}");
    };
    assert_eq!(rule, rule0());
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
        rule,
        literal,
        variables,
    } = err
    else {
        panic!("expected UnsafeNegation, got {err:?}");
    };
    assert_eq!(rule, rule0());
    assert_eq!(literal, lit(1));
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
        rule,
        site,
        term,
        depth,
        limit,
    } = err
    else {
        panic!("expected TripleTermNestingExceeded, got {err:?}");
    };
    assert_eq!(rule, rule0());
    assert_eq!(site, RuleSite::Head);
    assert_eq!(term, slot(0));
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
