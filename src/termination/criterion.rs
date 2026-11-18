//! [`TerminationCriterion`] — the versioned sufficient condition proved.

use std::fmt;

/// Version of the evidence law a [`TerminationCriterion`] proves. A change to
/// what a criterion accepts or how it reports bumps the version.
#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct LawVersion(u32);

impl LawVersion {
    /// Wrap a law version number.
    #[must_use]
    pub const fn new(version: u32) -> Self {
        Self(version)
    }

    /// The version number.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Display for LawVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}", self.0)
    }
}

/// A sufficient criterion for the termination of bottom-up evaluation.
///
/// Failing a criterion is not a proof of non-termination.
#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub enum TerminationCriterion {
    /// No value-inventing edge of the position graph lies on a cycle.
    WeaklyAcyclic,
}

impl TerminationCriterion {
    /// Stable kebab-case name, independent of presentation.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::WeaklyAcyclic => "weakly-acyclic",
        }
    }

    /// Version of the law this criterion currently proves.
    #[must_use]
    pub const fn law_version(self) -> LawVersion {
        match self {
            Self::WeaklyAcyclic => LawVersion::new(1),
        }
    }
}

impl fmt::Display for TerminationCriterion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.label(), self.law_version())
    }
}
