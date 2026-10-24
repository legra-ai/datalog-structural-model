//! Datalog [`Program`] — an ordered set of [`Rule`]s plus
//! validation and stratification entry points.

use super::equality_mode::EqualityMode;
use super::error::ProgramValidationError;
use super::literal::Literal;
use super::rule::Rule;
use crate::stratify::{Stratification, stratify};

/// Datalog program as an ordered set of rules.
#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "super::serde_input::ProgramInput")]
pub struct Program {
    /// Program rules.
    pub rules: Vec<Rule>,
}

impl Program {
    /// Construct a program only when all safety and stratification checks pass.
    /// # Errors
    /// Returns the same structural failures as [`Self::validate`].
    pub fn try_new(rules: Vec<Rule>) -> Result<Self, ProgramValidationError> {
        let program = Self { rules };
        program.validate()?;
        Ok(program)
    }

    /// Validate all rules in the program for safety and
    /// stratifiability.
    ///
    /// Runs two checks:
    ///
    /// 1. **Safety** — every rule must be range-restricted; head variables and
    ///    variables inside negated body literals must also appear in some
    ///    positive body literal.
    /// 2. **Stratifiability** — the compatible-rule dependency graph must have no
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
            rule.validate_term_shapes(index)?;
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
    /// Connects producers to consumers whose complete tuple patterns can
    /// unify, collapses strongly connected components, and assigns each rule
    /// to a stratum. Negative and aggregate dependencies require a strictly
    /// earlier producer stratum. Pure EDB inputs are available before stratum
    /// zero. Rule indices refer to this program's rule order.
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
