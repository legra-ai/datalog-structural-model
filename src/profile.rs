//! Checked generic language profiles; RDF/OWL semantics belong in rule packages.
use crate::error_code::ErrorCode;
use crate::{
    DatalogCapabilities, DatalogFeature, EqualityMode, HeadForm, Program, ProgramValidationError,
    RuleIndex, TerminationViolation, ValidationLimits,
};
use std::{error::Error, fmt};

/// Explicit language features, equality regime and structural limits
/// selected for a program.
/// This is neither an RDF entailment profile nor an execution strategy.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatalogProfile {
    capabilities: DatalogCapabilities,
    equality_mode: EqualityMode,
    limits: ValidationLimits,
}
impl DatalogProfile {
    /// Construct an explicit language contract with the standard
    /// [`ValidationLimits`]. Program-specific combinations are checked by
    /// [`Program::validate_for`].
    #[must_use]
    pub fn new(capabilities: DatalogCapabilities, equality_mode: EqualityMode) -> Self {
        Self {
            capabilities,
            equality_mode,
            limits: ValidationLimits::standard(),
        }
    }
    /// Replace this profile's structural limits.
    #[must_use]
    pub fn with_limits(mut self, limits: ValidationLimits) -> Self {
        self.limits = limits;
        self
    }
    /// Positive flat Datalog without built-in expression extensions.
    #[must_use]
    pub fn positive() -> Self {
        Self::new(DatalogCapabilities::positive(), EqualityMode::Default)
    }
    /// Flat Datalog with stratified negation, without built-in extensions.
    #[must_use]
    pub fn stratified() -> Self {
        Self::new(
            DatalogCapabilities::positive().with(DatalogFeature::StratifiedNegation),
            EqualityMode::Default,
        )
    }
    /// The explicitly selected language features.
    #[must_use]
    pub fn capabilities(&self) -> &DatalogCapabilities {
        &self.capabilities
    }
    /// The selected equality validation regime.
    #[must_use]
    pub fn equality_mode(&self) -> EqualityMode {
        self.equality_mode
    }
    /// The structural limits [`Program::validate_for`] enforces.
    #[must_use]
    pub fn limits(&self) -> &ValidationLimits {
        &self.limits
    }
}

/// Features required by the program and absent from the selected profile.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnsupportedCapabilities {
    missing: DatalogCapabilities,
}
impl UnsupportedCapabilities {
    /// Inspect every missing feature, including nested operators.
    #[must_use]
    pub fn missing(&self) -> &DatalogCapabilities {
        &self.missing
    }
}
impl fmt::Display for UnsupportedCapabilities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unsupported Datalog features: {:?}", self.missing)
    }
}
impl Error for UnsupportedCapabilities {}

/// An invalid program or an explicitly unsupported language feature.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ProfileCheckError {
    /// Structural safety, stratification or equality compatibility failed.
    InvalidProgram(ProgramValidationError),
    /// A rule's head form needs a capability the profile lacks. Reported
    /// before, and never folded into, [`Self::Unsupported`].
    UnsupportedHeadForm {
        /// The first rule whose head form is not accepted.
        rule: RuleIndex,
        /// Its head form.
        form: HeadForm,
    },
    /// One or more required features are absent from the profile.
    Unsupported(UnsupportedCapabilities),
    /// The program is not weakly acyclic, so evaluation may not terminate.
    TerminationUncertified(TerminationViolation),
}
impl fmt::Display for ProfileCheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProgram(error) => error.fmt(f),
            Self::UnsupportedHeadForm { rule, form } => write!(
                f,
                "[{}] {rule} has a {form} head, which the profile does not accept",
                self.code()
            ),
            Self::Unsupported(error) => error.fmt(f),
            Self::TerminationUncertified(error) => error.fmt(f),
        }
    }
}
impl ProfileCheckError {
    /// Code of [`Self::InvalidProgram`]; the wrapped error carries its own code.
    pub const CODE_INVALID_PROGRAM: ErrorCode = ErrorCode::new("invalid-program");
    /// Code of [`Self::UnsupportedHeadForm`].
    pub const CODE_UNSUPPORTED_HEAD_FORM: ErrorCode = ErrorCode::new("unsupported-head-form");
    /// Code of [`Self::Unsupported`].
    pub const CODE_UNSUPPORTED_CAPABILITIES: ErrorCode = ErrorCode::new("unsupported-capabilities");
    /// Code of [`Self::TerminationUncertified`]; the wrapped violation
    /// carries its own code.
    pub const CODE_TERMINATION_UNCERTIFIED: ErrorCode = ErrorCode::new("termination-uncertified");
    /// Stable kebab-case code of this variant.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::InvalidProgram(_) => Self::CODE_INVALID_PROGRAM,
            Self::UnsupportedHeadForm { .. } => Self::CODE_UNSUPPORTED_HEAD_FORM,
            Self::Unsupported(_) => Self::CODE_UNSUPPORTED_CAPABILITIES,
            Self::TerminationUncertified(_) => Self::CODE_TERMINATION_UNCERTIFIED,
        }
    }
}
impl Error for ProfileCheckError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidProgram(error) => Some(error),
            Self::UnsupportedHeadForm { .. } => None,
            Self::Unsupported(error) => Some(error),
            Self::TerminationUncertified(error) => Some(error),
        }
    }
}
impl Program {
    /// Validate safety within the profile's limits, reject every feature
    /// missing from the selected contract, and require
    /// [`TerminationCriterion::WeaklyAcyclic`](crate::TerminationCriterion::WeaklyAcyclic)
    /// with invented values no deeper than the profile's Skolem depth limit.
    /// # Errors
    /// Returns typed structural failures (limit errors report the limit and
    /// the value reached), the first rule whose head form the profile does
    /// not accept, the exact unsupported feature set, or the cyclic
    /// value-inventing edge.
    pub fn validate_for(&self, profile: &DatalogProfile) -> Result<(), ProfileCheckError> {
        self.validate_within(profile.equality_mode, &profile.limits)
            .map_err(ProfileCheckError::InvalidProgram)?;
        self.check_head_forms(&profile.capabilities)?;
        let missing = self
            .required_capabilities()
            .missing_from(&profile.capabilities);
        if !missing.is_empty() {
            return Err(ProfileCheckError::Unsupported(UnsupportedCapabilities {
                missing,
            }));
        }
        let certificate = self
            .check_weak_acyclicity()
            .map_err(ProfileCheckError::TerminationUncertified)?;
        let limit = profile.limits.skolem_depth();
        match certificate.deepest() {
            Some(position) if certificate.skolem_depth() > limit => Err(
                ProfileCheckError::InvalidProgram(ProgramValidationError::SkolemDepthExceeded {
                    position: position.clone(),
                    depth: certificate.skolem_depth(),
                    limit,
                }),
            ),
            _ => Ok(()),
        }
    }

    /// Reject the first rule whose head form needs an undeclared capability.
    fn check_head_forms(&self, supported: &DatalogCapabilities) -> Result<(), ProfileCheckError> {
        self.rules
            .iter()
            .enumerate()
            .find_map(|(index, rule)| {
                let form = rule.head_form();
                let feature = form.required_feature()?;
                (!supported.contains(&feature)).then_some(ProfileCheckError::UnsupportedHeadForm {
                    rule: RuleIndex::new(index),
                    form,
                })
            })
            .map_or(Ok(()), Err)
    }
}
