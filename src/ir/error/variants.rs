//! The [`ProgramValidationError`] variants.

use crate::ir::symbols::VariableSymbol;
use crate::position::{HeadAtomPath, LiteralPath, RuleIndex, RuleSite, TermPath};
use crate::stratify::UnstratifiableCycle;
use crate::termination::ArgumentPosition;

/// Program validation failures. Every failure tied to a location names the
/// rule and, where narrower, the literal and term inside it.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ProgramValidationError {
    /// Head variables are missing from the positive body, `BIND` outputs, and
    /// aggregate exports, and are not existentially quantified.
    UnsafeRule {
        /// Rule whose head is unsafe.
        rule: RuleIndex,
        /// The first head atom with missing variables.
        atom: HeadAtomPath,
        /// Its missing variables.
        variables: Vec<VariableSymbol>,
    },
    /// An existentially quantified head variable occurs in no head atom.
    UnusedExistentialVariable {
        /// Rule whose head quantifies the variable.
        rule: RuleIndex,
        /// The first unused existential variable.
        variable: VariableSymbol,
    },
    /// An existentially quantified head variable is also bound by the body,
    /// so it would denote a body value rather than a witness.
    ExistentialVariableBoundInBody {
        /// Rule whose head quantifies the variable.
        rule: RuleIndex,
        /// The first such existential variable.
        variable: VariableSymbol,
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
    /// [`ValidationLimits::triple_term_nesting`](crate::ValidationLimits::triple_term_nesting).
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
    /// The program has more rules than
    /// [`ValidationLimits::rules`](crate::ValidationLimits::rules).
    RuleCountExceeded {
        /// Number of rules in the program.
        count: usize,
        /// Configured limit.
        limit: usize,
    },
    /// A rule body has more literals than
    /// [`ValidationLimits::body_literals`](crate::ValidationLimits::body_literals).
    BodyLengthExceeded {
        /// The rule.
        rule: RuleIndex,
        /// Its literal count, aggregate inner bodies included.
        length: usize,
        /// Configured limit.
        limit: usize,
    },
    /// Invented values nest deeper than
    /// [`ValidationLimits::skolem_depth`](crate::ValidationLimits::skolem_depth),
    /// as measured by [`WeakAcyclicity::skolem_depth`](crate::WeakAcyclicity::skolem_depth).
    SkolemDepthExceeded {
        /// The smallest position reaching `depth`.
        position: ArgumentPosition,
        /// Depth reached.
        depth: usize,
        /// Configured limit.
        limit: usize,
    },
}
