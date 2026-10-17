//! [`ProgramValidationError`] and its `Display` / `Error` /
//! `From` integration with [`StratificationError`].

use std::error::Error;
use std::fmt;

use super::symbols::{PredicateSymbol, VariableSymbol};
use crate::stratify::StratificationError;

/// Program validation failures.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ProgramValidationError {
    /// Rule is unsafe because one or more head variables are
    /// missing from the positive body literals.
    UnsafeRule {
        /// Rule index in [`Program::rules`](super::program::Program::rules).
        rule_index: usize,
        /// Missing variables.
        variables: Vec<VariableSymbol>,
    },
    /// Rule has a negated body literal whose variables are not
    /// bound by any positive body literal (range-restriction
    /// violation — negation-as-failure requires the negated atom
    /// to be ground at evaluation time).
    UnsafeNegation {
        /// Rule index in [`Program::rules`](super::program::Program::rules).
        rule_index: usize,
        /// Unbound variables in negated literals.
        variables: Vec<VariableSymbol>,
    },
    /// Rule has a [`Literal::Filter`](super::literal::Literal::Filter)
    /// referencing variables that no positive body literal binds.
    /// Filters guard candidate bindings; they cannot introduce
    /// new ones.
    UnsafeFilter {
        /// Rule index in [`Program::rules`](super::program::Program::rules).
        rule_index: usize,
        /// Unbound variables in filter expressions.
        variables: Vec<VariableSymbol>,
    },
    /// Rule has a [`Literal::Bind`](super::literal::Literal::Bind)
    /// whose `expr` references a variable not bound by an earlier
    /// positive atom or `Bind`, or whose output `var` shadows a
    /// variable that is already bound by an earlier positive atom
    /// or `Bind`. Both cases produce undefined evaluation order
    /// under sequential rule-body semantics.
    UnsafeBind {
        /// Rule index in [`Program::rules`](super::program::Program::rules).
        rule_index: usize,
        /// Offending variables — either an unbound expression
        /// reference or a colliding output name.
        variables: Vec<VariableSymbol>,
    },
    /// Rule has a
    /// [`Literal::Aggregate`](super::literal::Literal::Aggregate)
    /// that violates aggregation safety:
    ///
    /// - `input_var` or a `group_by` key is not bound by the inner body.
    /// - `result_var` is not fresh (collides with an outer-rule binding).
    /// - An inner negative literal or filter references a variable that no
    ///   inner positive atom or `BIND` produces.
    /// - An inner `BIND` references a variable not bound by an earlier inner
    ///   positive atom or `BIND`, or its output variable collides with an
    ///   existing inner-scope binding.
    ///
    /// Nested aggregates inside an aggregate's inner body surface
    /// separately as [`Self::NestedAggregateInInnerBody`].
    UnsafeAggregate {
        /// Rule index in [`Program::rules`](super::program::Program::rules).
        rule_index: usize,
        /// Offending variables.
        variables: Vec<VariableSymbol>,
    },
    /// Rule has a [`Literal::Aggregate`](super::literal::Literal::Aggregate)
    /// whose `inner_body` itself contains a nested
    /// `Literal::Aggregate`. Nested aggregation is out of scope —
    /// the evaluator only folds a flat inner sub-query — and the
    /// inner aggregate's stratification dependencies would
    /// require an extra rewriting pass. Authors that need
    /// aggregation over an aggregate's output should lift the
    /// inner aggregate into its own intermediate rule whose
    /// head feeds the outer aggregate's `inner_body`.
    NestedAggregateInInnerBody {
        /// Rule index in [`Program::rules`](super::program::Program::rules).
        rule_index: usize,
    },
    /// Program contains a recursive cycle through a negation
    /// edge. Such programs have no unique least fixed point under
    /// stratified negation-as-failure semantics.
    UnstratifiableNegation {
        /// Predicate symbols whose stratification could not be
        /// resolved. Includes every predicate in the offending
        /// strongly connected component.
        predicates: Vec<PredicateSymbol>,
    },
    /// Rule contains a [`Literal::Negative`](super::literal::Literal::Negative)
    /// while equality mode is
    /// [`EqualityMode::SameAs`](super::EqualityMode::SameAs).
    ///
    /// Under equality-driven rewriting, terms get rewritten across
    /// equality classes, which interacts
    /// non-deterministically with negation-as-failure (the set
    /// the negated atom is evaluated against changes shape as
    /// equality classes merge). Programs that combine negation with
    /// equality mode are rejected for this reason. This
    /// crate enforces the same restriction so semantics stay stable
    /// under the equality rewrite.
    NegationUnderEqualityMode {
        /// Rule index in [`Program::rules`](super::program::Program::rules).
        rule_index: usize,
    },
    /// Rule contains a
    /// [`Literal::Aggregate`](super::literal::Literal::Aggregate)
    /// while equality mode is
    /// [`EqualityMode::SameAs`](super::EqualityMode::SameAs).
    ///
    /// Aggregation depends on the *complete* extension of its
    /// inner predicates. Under equality mode that extension is
    /// non-monotonic as equality classes merge term identities,
    /// which would silently change aggregate results across
    /// reasoning rounds. This crate rejects the combination.
    AggregationUnderEqualityMode {
        /// Rule index in [`Program::rules`](super::program::Program::rules).
        rule_index: usize,
    },
    /// Rule has a body atom (positive, negative, filter, bind, or
    /// aggregate inner atom) that carries a
    /// [`Term::Skolem`](super::term::Term::Skolem) term. SKOLEM is
    /// a deterministic IRI-minting function and is only
    /// well-defined in head positions — using it as a body pattern
    /// would be a no-op match (the freshly-minted IRI never unifies
    /// with anything in the EDB). See extension.
    SkolemInBodyAtom {
        /// Rule index in [`Program::rules`](super::program::Program::rules).
        rule_index: usize,
    },
    /// Rule contains a
    /// [`Term::TripleTerm`](super::term::Term::TripleTerm) nested
    /// deeper than
    /// [`MAX_TRIPLE_TERM_NESTING`](super::term::MAX_TRIPLE_TERM_NESTING)
    /// in either the head or a body atom. Unification cost grows
    /// with nesting; the bound exists to keep evaluation finite.
    TripleTermNestingExceeded {
        /// Rule index in [`Program::rules`](super::program::Program::rules).
        rule_index: usize,
        /// Observed nesting depth (always >
        /// `MAX_TRIPLE_TERM_NESTING`).
        depth: usize,
        /// Configured limit at the time of validation.
        limit: usize,
    },
    /// A Skolem argument is neither a variable nor a constant.
    InvalidSkolemArguments {
        /// Rule containing the invalid term.
        rule_index: usize,
    },
}

impl From<StratificationError> for ProgramValidationError {
    fn from(err: StratificationError) -> Self {
        let StratificationError::UnstratifiableCycle { predicates } = err;
        Self::UnstratifiableNegation { predicates }
    }
}

impl fmt::Display for ProgramValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsafeRule {
                rule_index,
                variables,
            } => {
                write!(
                    f,
                    "rule at index {rule_index} is unsafe; missing head variables in body: "
                )?;
                for (idx, var) in variables.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", var.as_str())?;
                }
                Ok(())
            }
            Self::UnsafeNegation {
                rule_index,
                variables,
            } => {
                write!(
                    f,
                    "rule at index {rule_index} has unsafe negation; variables in negated \
                     literals are not bound by positive body: "
                )?;
                for (idx, var) in variables.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", var.as_str())?;
                }
                Ok(())
            }
            Self::UnsafeFilter {
                rule_index,
                variables,
            } => {
                write!(
                    f,
                    "rule at index {rule_index} has an unsafe filter; variables in filter \
                     expressions are not bound by positive body: "
                )?;
                for (idx, var) in variables.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", var.as_str())?;
                }
                Ok(())
            }
            Self::UnsafeBind {
                rule_index,
                variables,
            } => {
                write!(
                    f,
                    "rule at index {rule_index} has an unsafe BIND; offending variables \
                     (unbound expr reference or colliding output name): "
                )?;
                for (idx, var) in variables.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", var.as_str())?;
                }
                Ok(())
            }
            Self::UnsafeAggregate {
                rule_index,
                variables,
            } => {
                write!(
                    f,
                    "rule at index {rule_index} has an unsafe aggregate; offending variables \
                     (unbound input / group-by, colliding result_var, or unsupported inner \
                     literal): "
                )?;
                for (idx, var) in variables.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", var.as_str())?;
                }
                Ok(())
            }
            Self::NestedAggregateInInnerBody { rule_index } => {
                write!(
                    f,
                    "rule at index {rule_index} has an aggregate whose inner body contains a \
                     nested aggregate; nested aggregation is not supported — lift the inner \
                     aggregate into its own rule"
                )
            }
            Self::NegationUnderEqualityMode { rule_index } => {
                write!(
                    f,
                    "rule at index {rule_index} contains a negative literal while equality mode \
                     is `SameAs`; negation and equality rewriting are mutually \
                     exclusive (extension)"
                )
            }
            Self::AggregationUnderEqualityMode { rule_index } => {
                write!(
                    f,
                    "rule at index {rule_index} contains an aggregate body literal while equality \
                     mode is `SameAs`; aggregation and equality rewriting are \
                     mutually exclusive (extension)"
                )
            }
            Self::UnstratifiableNegation { predicates } => {
                write!(f, "program contains a recursive cycle through negation: ")?;
                for (idx, p) in predicates.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p.as_str())?;
                }
                Ok(())
            }
            Self::SkolemInBodyAtom { rule_index } => {
                write!(
                    f,
                    "rule at index {rule_index} has a body atom carrying a SKOLEM term; \
                     SKOLEM is only allowed in head atoms (extension)"
                )
            }
            Self::InvalidSkolemArguments { rule_index } => write!(
                f,
                "rule at index {rule_index} has non-flat Skolem arguments"
            ),
            Self::TripleTermNestingExceeded {
                rule_index,
                depth,
                limit,
            } => {
                write!(
                    f,
                    "rule at index {rule_index} contains a triple-term nested {depth} deep; \
                     limit is {limit}"
                )
            }
        }
    }
}

impl Error for ProgramValidationError {}
