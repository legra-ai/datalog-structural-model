//! Checked generic language profiles; RDF/OWL semantics belong in rule packages.
use crate::{DatalogCapabilities, DatalogFeature, EqualityMode, Program, ProgramValidationError};
use std::{error::Error, fmt};

/// Explicit language features and equality regime selected for a program.
/// This is neither an RDF entailment profile nor an execution strategy.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatalogProfile {
    capabilities: DatalogCapabilities,
    equality_mode: EqualityMode,
}
impl DatalogProfile {
    /// Construct an explicit language contract. Program-specific combinations
    /// are checked by [`Program::validate_for`].
    #[must_use]
    pub fn new(capabilities: DatalogCapabilities, equality_mode: EqualityMode) -> Self {
        Self {
            capabilities,
            equality_mode,
        }
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
    /// One or more required features are absent from the profile.
    Unsupported(UnsupportedCapabilities),
}
impl fmt::Display for ProfileCheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProgram(error) => error.fmt(f),
            Self::Unsupported(error) => error.fmt(f),
        }
    }
}
impl Error for ProfileCheckError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidProgram(error) => Some(error),
            Self::Unsupported(error) => Some(error),
        }
    }
}
impl Program {
    /// Validate safety and reject every feature missing from the selected contract.
    /// # Errors
    /// Returns typed structural failures or the exact unsupported feature set.
    pub fn validate_for(&self, profile: &DatalogProfile) -> Result<(), ProfileCheckError> {
        self.validate_with_equality_mode(profile.equality_mode)
            .map_err(ProfileCheckError::InvalidProgram)?;
        let missing = self
            .required_capabilities()
            .missing_from(&profile.capabilities);
        if !missing.is_empty() {
            return Err(ProfileCheckError::Unsupported(UnsupportedCapabilities {
                missing,
            }));
        }
        Ok(())
    }
}
