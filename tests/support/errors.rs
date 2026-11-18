//! One sample of every error variant, with typed positions filled in.
use datalog_structural_model::{
    ArgumentPosition, Atom, AtomSlot, ConstantSymbol, ExpressionTermIndex, LiteralIndex,
    LiteralPath, PredicateSymbol, Program, ProgramValidationError, Rule, RuleIndex, RuleSite,
    StrictDependency, StrictDependencyKind, Term, TermPath, TermRoot, TerminationViolation,
    UnstratifiableCycle, VariableSymbol,
};

pub fn literal() -> LiteralPath {
    LiteralPath::top(LiteralIndex::new(2)).inner(LiteralIndex::new(1))
}

pub fn term() -> TermPath {
    TermPath::new(TermRoot::Slot(AtomSlot::new(1))).child(AtomSlot::new(0))
}

pub fn position(predicate: &str, slot: usize) -> ArgumentPosition {
    ArgumentPosition::new(
        PredicateSymbol::new(predicate),
        AtomSlot::new(slot),
        RuleIndex::new(0),
        RuleSite::Head,
    )
}

/// `node(SKOLEM(n, x)) :- node(x)`: the canonical cyclic Skolem edge.
pub fn termination_violation() -> TerminationViolation {
    let x = Term::Variable(VariableSymbol::new("x"));
    let skolem = Term::Skolem {
        name: ConstantSymbol::new("n"),
        args: vec![x.clone()],
    };
    let atom = |term| Atom {
        predicate: PredicateSymbol::new("node"),
        terms: vec![term],
    };
    Program {
        rules: vec![Rule::positive(atom(skolem), vec![atom(x)])],
    }
    .check_weak_acyclicity()
    .unwrap_err()
}

pub fn cycle() -> UnstratifiableCycle {
    UnstratifiableCycle {
        rules: vec![RuleIndex::new(0), RuleIndex::new(3)],
        predicates: vec![PredicateSymbol::new("cycle")],
        edge: StrictDependency {
            producer: RuleIndex::new(0),
            consumer: RuleIndex::new(3),
            literal: LiteralPath::top(LiteralIndex::new(1)),
            kind: StrictDependencyKind::Negation,
        },
    }
}

/// Every [`ProgramValidationError`] variant, once.
pub fn every_program_error() -> Vec<ProgramValidationError> {
    let rule = RuleIndex::new(3);
    let variables = vec![VariableSymbol::new("missing")];
    vec![
        ProgramValidationError::UnsafeRule {
            rule,
            variables: variables.clone(),
        },
        ProgramValidationError::UnsafeNegation {
            rule,
            literal: literal(),
            variables: variables.clone(),
        },
        ProgramValidationError::UnsafeFilter {
            rule,
            literal: literal(),
            variables: variables.clone(),
        },
        ProgramValidationError::UnsafeBind {
            rule,
            literal: literal(),
            variables: variables.clone(),
        },
        ProgramValidationError::UnsafeAggregate {
            rule,
            literal: literal(),
            variables,
        },
        ProgramValidationError::NestedAggregateInInnerBody {
            rule,
            literal: literal(),
        },
        ProgramValidationError::UnstratifiableCycle(cycle()),
        ProgramValidationError::NegationUnderEqualityMode {
            rule,
            literal: literal(),
        },
        ProgramValidationError::AggregationUnderEqualityMode {
            rule,
            literal: literal(),
        },
        ProgramValidationError::SkolemInBodyAtom {
            rule,
            literal: literal(),
            term: term(),
        },
        ProgramValidationError::TripleTermNestingExceeded {
            rule,
            site: RuleSite::Head,
            term: term(),
            depth: 5,
            limit: 4,
        },
        ProgramValidationError::InvalidSkolemArguments {
            rule,
            site: RuleSite::Body(literal()),
            term: TermPath::new(TermRoot::ExpressionTerm(ExpressionTermIndex::new(0))),
        },
        ProgramValidationError::RuleCountExceeded { count: 9, limit: 8 },
        ProgramValidationError::BodyLengthExceeded {
            rule,
            length: 3,
            limit: 2,
        },
        ProgramValidationError::SkolemDepthExceeded {
            position: position("node", 1),
            depth: 2,
            limit: 1,
        },
    ]
}
