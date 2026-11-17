//! The [`ProgramValidationError`] variants.

use crate::ir::symbols::VariableSymbol;
use crate::position::{LiteralPath, RuleIndex, RuleSite, TermPath};
use crate::stratify::UnstratifiableCycle;

/// Program validation failures. Every failure tied to a location names the
/// rule and, where narrower, the literal and term inside it.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ProgramValidationError {
    /// Head variables are missing from the positive body, `BIND` outputs, and
    /// aggregate exports.
    UnsafeRule {
        /// Rule whose head is unsafe.
        rule: RuleIndex,
        /// Missing variables.
        variables: Vec<VariableSymbol>,
    },
    /// A negated body literal references variables no positive body literal
    /// binds (negation-as-failure requires a ground negated atom).
    UnsafeNegation {
        /// Rule containing the literal.
        rule: RuleIndex,
        /// The first offending negated literal.
        literal: LiteralPath,
        /// Its unbound variables.
        variables: Vec<VariableSymbol>,
    },
    /// A [`Literal::Filter`](crate::Literal::Filter) references unbound
    /// variables. Filters guard bindings; they cannot introduce them.
    UnsafeFilter {
        /// Rule containing the literal.
        rule: RuleIndex,
        /// The first offending filter.
        literal: LiteralPath,
        /// Its unbound variables.
        variables: Vec<VariableSymbol>,
    },
    /// A [`Literal::Bind`](crate::Literal::Bind) references a variable not
    /// bound earlier, or its output shadows an already-bound variable.
    UnsafeBind {
        /// Rule containing the literal.
        rule: RuleIndex,
        /// The first offending `BIND`.
        literal: LiteralPath,
        /// Unbound expression references or the colliding output name.
        variables: Vec<VariableSymbol>,
    },
    /// A [`Literal::Aggregate`](crate::Literal::Aggregate) violates
    /// aggregation safety: unbound `input_var` or group key, non-fresh
    /// `result_var`, or an unsafe inner negation, filter, or `BIND`.
    UnsafeAggregate {
        /// Rule containing the literal.
        rule: RuleIndex,
        /// The first offending aggregate.
        literal: LiteralPath,
        /// Offending variables.
        variables: Vec<VariableSymbol>,
    },
    /// An aggregate's inner body contains another aggregate. Lift the inner
    /// aggregate into its own rule.
    NestedAggregateInInnerBody {
        /// Rule containing the literal.
        rule: RuleIndex,
        /// Path to the nested aggregate.
        literal: LiteralPath,
    },
    /// The program contains a recursive cycle through negation or
    /// aggregation.
    UnstratifiableCycle(UnstratifiableCycle),
    /// A negative literal under
    /// [`EqualityMode::SameAs`](crate::EqualityMode::SameAs); equality
    /// rewriting and negation-as-failure are mutually exclusive.
    NegationUnderEqualityMode {
        /// Rule containing the literal.
        rule: RuleIndex,
        /// The negative literal.
        literal: LiteralPath,
    },
    /// An aggregate literal under
    /// [`EqualityMode::SameAs`](crate::EqualityMode::SameAs); aggregation
    /// needs a complete extension that equality rewriting keeps changing.
    AggregationUnderEqualityMode {
        /// Rule containing the literal.
        rule: RuleIndex,
        /// The aggregate literal.
        literal: LiteralPath,
    },
    /// A body literal carries a [`Term::Skolem`](crate::Term::Skolem).
    /// SKOLEM mints IRIs and is only well-defined in the head.
    SkolemInBodyAtom {
        /// Rule containing the literal.
        rule: RuleIndex,
        /// The body literal.
        literal: LiteralPath,
        /// The Skolem term inside it.
        term: TermPath,
    },
    /// A [`Term::TripleTerm`](crate::Term::TripleTerm) nests deeper than
    /// [`MAX_TRIPLE_TERM_NESTING`](crate::MAX_TRIPLE_TERM_NESTING).
    TripleTermNestingExceeded {
        /// Rule containing the term.
        rule: RuleIndex,
        /// Head or body literal containing the term.
        site: RuleSite,
        /// The outermost term whose nesting exceeds the limit.
        term: TermPath,
        /// Observed nesting depth (always greater than `limit`).
        depth: usize,
        /// Configured limit at the time of validation.
        limit: usize,
    },
    /// A Skolem argument is neither a variable nor a constant.
    InvalidSkolemArguments {
        /// Rule containing the term.
        rule: RuleIndex,
        /// Head or body literal containing the term.
        site: RuleSite,
        /// The Skolem term.
        term: TermPath,
    },
}
