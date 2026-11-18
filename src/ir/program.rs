//! Datalog [`Program`] — an ordered set of [`Rule`]s plus
//! validation and stratification entry points.

use super::equality_mode::EqualityMode;
use super::error::ProgramValidationError;
use super::head::{ExistentialFault, HeadFault};
use super::literal::Literal;
use super::rule::{LiteralFault, Rule};
use crate::limits::ValidationLimits;
use crate::position::{LiteralIndex, LiteralPath, RuleIndex};
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
    /// [`ProgramValidationError::UnstratifiableCycle`] when the
    /// program contains a recursive cycle through negation.
    pub fn validate(&self) -> Result<(), ProgramValidationError> {
        self.validate_within(EqualityMode::Default, &ValidationLimits::standard())
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
        self.validate_within(equality_mode, &ValidationLimits::standard())
    }

    /// Validate under an equality mode and explicit structural `limits`.
    ///
    /// Checks the rule count, each rule's body length and triple-term
    /// nesting against `limits` before the safety and stratification checks
    /// of [`Self::validate_with_equality_mode`]. The Skolem depth limit
    /// needs the termination analysis and is enforced by
    /// [`Self::validate_for`].
    ///
    /// # Errors
    ///
    /// Returns [`ProgramValidationError::RuleCountExceeded`],
    /// [`ProgramValidationError::BodyLengthExceeded`] or
    /// [`ProgramValidationError::TripleTermNestingExceeded`] with the limit
    /// and the value reached, or any error of
    /// [`Self::validate_with_equality_mode`].
    pub fn validate_within(
        &self,
        equality_mode: EqualityMode,
        limits: &ValidationLimits,
    ) -> Result<(), ProgramValidationError> {
        if self.rules.len() > limits.rules() {
            return Err(ProgramValidationError::RuleCountExceeded {
                count: self.rules.len(),
                limit: limits.rules(),
            });
        }
        if equality_mode == EqualityMode::SameAs {
            check_equality_mode_compatibility(&self.rules)?;
        }
        for (index, rule) in self.rules.iter().enumerate() {
            rule.validate(RuleIndex::new(index), limits)?;
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
    /// Returns [`ProgramValidationError::UnstratifiableCycle`]
    /// when a cycle in the dependency graph contains a negation
    /// edge.
    pub fn stratify(&self) -> Result<Stratification, ProgramValidationError> {
        stratify(self).map_err(ProgramValidationError::from)
    }
}

/// Reject negation and aggregation under [`EqualityMode::SameAs`], naming
/// the first offending top-level body literal.
fn check_equality_mode_compatibility(rules: &[Rule]) -> Result<(), ProgramValidationError> {
    for (index, rule) in rules.iter().enumerate() {
        let rule_index = RuleIndex::new(index);
        for (position, lit) in rule.body.iter().enumerate() {
            let literal = LiteralPath::top(LiteralIndex::new(position));
            match lit {
                Literal::Negative(_) => {
                    return Err(ProgramValidationError::NegationUnderEqualityMode {
                        rule: rule_index,
                        literal,
                    });
                }
                Literal::Aggregate(_) => {
                    return Err(ProgramValidationError::AggregationUnderEqualityMode {
                        rule: rule_index,
                        literal,
                    });
                }
                Literal::Positive(_) | Literal::Filter(_) | Literal::Bind { .. } => {}
            }
        }
    }
    Ok(())
}

impl Rule {
    /// Per-rule shape and safety checks, in reporting order.
    fn validate(
        &self,
        rule: RuleIndex,
        limits: &ValidationLimits,
    ) -> Result<(), ProgramValidationError> {
        let length = self.literal_count();
        if length > limits.body_literals() {
            return Err(ProgramValidationError::BodyLengthExceeded {
                rule,
                length,
                limit: limits.body_literals(),
            });
        }
        self.validate_term_shapes(rule, limits)?;
        match self.existential_fault() {
            Some(ExistentialFault::Unused(variable)) => {
                return Err(ProgramValidationError::UnusedExistentialVariable { rule, variable });
            }
            Some(ExistentialFault::BoundInBody(variable)) => {
                return Err(ProgramValidationError::ExistentialVariableBoundInBody {
                    rule,
                    variable,
                });
            }
            None => {}
        }
        if let Some(HeadFault { atom, variables }) = self.unsafe_head_variables() {
            return Err(ProgramValidationError::UnsafeRule {
                rule,
                atom,
                variables,
            });
        }
        if let Some(LiteralFault { literal, variables }) = self.unsafe_negative_variables() {
            return Err(ProgramValidationError::UnsafeNegation {
                rule,
                literal,
                variables,
            });
        }
        if let Some(LiteralFault { literal, variables }) = self.unsafe_filter_variables() {
            return Err(ProgramValidationError::UnsafeFilter {
                rule,
                literal,
                variables,
            });
        }
        if let Some(LiteralFault { literal, variables }) = self.unsafe_bind_variables() {
            return Err(ProgramValidationError::UnsafeBind {
                rule,
                literal,
                variables,
            });
        }
        if let Some(literal) = self.nested_aggregate_in_inner_body() {
            return Err(ProgramValidationError::NestedAggregateInInnerBody { rule, literal });
        }
        if let Some(LiteralFault { literal, variables }) = self.unsafe_aggregate_variables() {
            return Err(ProgramValidationError::UnsafeAggregate {
                rule,
                literal,
                variables,
            });
        }
        Ok(())
    }
}
