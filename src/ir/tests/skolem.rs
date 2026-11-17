//! SKOLEM head terms and body rejection.

use super::support::*;

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
        ProgramValidationError::SkolemInBodyAtom {
            rule: rule0(),
            literal: lit(0),
            term: slot(1),
        }
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
        ProgramValidationError::SkolemInBodyAtom {
            rule: rule0(),
            literal: lit(1),
            term: slot(1),
        }
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
        ProgramValidationError::SkolemInBodyAtom {
            rule: rule0(),
            literal: lit(0).inner(LiteralIndex::new(0)),
            term: slot(2),
        }
    );
}
