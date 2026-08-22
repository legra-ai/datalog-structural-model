//! Datalog [`Program`] — an ordered set of [`Rule`]s plus
//! validation and stratification entry points.

use super::atom::Atom;
use super::equality_mode::EqualityMode;
use super::error::ProgramValidationError;
use super::literal::Literal;
use super::rule::Rule;
use super::term::MAX_TRIPLE_TERM_NESTING;
use crate::stratify::{Stratification, stratify};

/// Datalog program as an ordered set of rules.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Program {
    /// Program rules.
    pub rules: Vec<Rule>,
}

impl Program {
    /// Validate all rules in the program for safety and
    /// stratifiability.
    ///
    /// Runs two checks:
    ///
    /// 1. **Safety** — every rule must be range-restricted; head variables and
    ///    variables inside negated body literals must also appear in some
    ///    positive body literal.
    /// 2. **Stratifiability** — the predicate dependency graph must have no
    ///    cycle that includes a negation edge (see [`Program::stratify`]).
    ///
    /// # Errors
    ///
    /// Returns [`ProgramValidationError::UnsafeRule`] on a safety
    /// failure, [`ProgramValidationError::UnsafeNegation`] when a
    /// negated body literal references a variable that no positive
    /// literal binds, or
    /// [`ProgramValidationError::UnstratifiableNegation`] when the
    /// program contains a recursive cycle through negation.
    pub fn validate(&self) -> Result<(), ProgramValidationError> {
        self.validate_with_equality_mode(EqualityMode::Default)
    }

    /// Validate the program under a specified equality-reasoning
    /// mode (extension).
    ///
    /// Runs the same safety / stratification / triple-term-nesting
    /// checks as [`Self::validate`] and additionally rejects
    /// negation and aggregation when
    /// `equality_mode == EqualityMode::SameAs`. The two features
    /// interact non-monotonically with `owl:sameAs` rewriting; see
    /// [`EqualityMode`] for the semantic rationale.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`Self::validate`] plus
    /// [`ProgramValidationError::NegationUnderEqualityMode`] or
    /// [`ProgramValidationError::AggregationUnderEqualityMode`]
    /// when the corresponding feature is used under
    /// [`EqualityMode::SameAs`].
    pub fn validate_with_equality_mode(
        &self,
        equality_mode: EqualityMode,
    ) -> Result<(), ProgramValidationError> {
        if equality_mode == EqualityMode::SameAs {
            check_equality_mode_compatibility(&self.rules)?;
        }
        for (index, rule) in self.rules.iter().enumerate() {
            // Triple-term nesting bound — checked before the
            // safety analyses because deeper nesting indicates a
            // structural problem unrelated to variable binding.
            let head_depth = rule.head.nesting_depth();
            let body_depth = rule
                .body
                .iter()
                .filter_map(Literal::atom)
                .map(Atom::nesting_depth)
                .max()
                .unwrap_or(0);
            let depth = head_depth.max(body_depth);
            if depth > MAX_TRIPLE_TERM_NESTING {
                return Err(ProgramValidationError::TripleTermNestingExceeded {
                    rule_index: index,
                    depth,
                    limit: MAX_TRIPLE_TERM_NESTING,
                });
            }
            let unsafe_head = rule.unsafe_head_variables();
            if !unsafe_head.is_empty() {
                return Err(ProgramValidationError::UnsafeRule {
                    rule_index: index,
                    variables: unsafe_head,
                });
            }
            let unsafe_neg = rule.unsafe_negative_variables();
            if !unsafe_neg.is_empty() {
                return Err(ProgramValidationError::UnsafeNegation {
                    rule_index: index,
                    variables: unsafe_neg,
                });
            }
            let unsafe_filter = rule.unsafe_filter_variables();
            if !unsafe_filter.is_empty() {
                return Err(ProgramValidationError::UnsafeFilter {
                    rule_index: index,
                    variables: unsafe_filter,
                });
            }
            let unsafe_bind = rule.unsafe_bind_variables();
            if !unsafe_bind.is_empty() {
                return Err(ProgramValidationError::UnsafeBind {
                    rule_index: index,
                    variables: unsafe_bind,
                });
            }
            if rule.has_skolem_in_body() {
                return Err(ProgramValidationError::SkolemInBodyAtom { rule_index: index });
            }
            if rule.has_nested_aggregate_in_inner_body() {
                return Err(ProgramValidationError::NestedAggregateInInnerBody {
                    rule_index: index,
                });
            }
            let unsafe_aggregate = rule.unsafe_aggregate_variables();
            if !unsafe_aggregate.is_empty() {
                return Err(ProgramValidationError::UnsafeAggregate {
                    rule_index: index,
                    variables: unsafe_aggregate,
                });
            }
        }
        // Stratification check — reject programs with recursion
        // through negation.
        self.stratify()?;
        Ok(())
    }

    /// Compute the program's stratification.
    ///
    /// Builds the predicate dependency graph (edges from each body
    /// predicate to the head predicate, tagged with polarity),
    /// collapses strongly connected components, checks that no SCC
    /// contains an internal negation edge, and assigns each
    /// predicate to a stratum such that every negation edge goes
    /// from a lower stratum to a strictly higher one.
    ///
    /// # Errors
    ///
    /// Returns [`ProgramValidationError::UnstratifiableNegation`]
    /// when a cycle in the dependency graph contains a negation
    /// edge.
    pub fn stratify(&self) -> Result<Stratification, ProgramValidationError> {
        stratify(self).map_err(ProgramValidationError::from)
    }
}

/// extension — reject negation / aggregation under
/// [`EqualityMode::SameAs`]. Walks each rule's body and surfaces
/// the first offending literal as
/// [`ProgramValidationError::NegationUnderEqualityMode`] or
/// [`ProgramValidationError::AggregationUnderEqualityMode`].
fn check_equality_mode_compatibility(rules: &[Rule]) -> Result<(), ProgramValidationError> {
    for (index, rule) in rules.iter().enumerate() {
        for lit in &rule.body {
            match lit {
                Literal::Negative(_) => {
                    return Err(ProgramValidationError::NegationUnderEqualityMode {
                        rule_index: index,
                    });
                }
                Literal::Aggregate(_) => {
                    return Err(ProgramValidationError::AggregationUnderEqualityMode {
                        rule_index: index,
                    });
                }
                Literal::Positive(_) | Literal::Filter(_) | Literal::Bind { .. } => {}
            }
        }
    }
    Ok(())
}
