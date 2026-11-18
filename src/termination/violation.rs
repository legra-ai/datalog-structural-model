//! [`TerminationViolation`] — the cyclic special edge that defeats a criterion.

use std::{error::Error, fmt};

use super::criterion::TerminationCriterion;
use super::edge::{ArgumentPosition, SpecialEdge};
use crate::error_code::ErrorCode;

/// A value-inventing edge whose endpoints share a strongly connected
/// component of the position graph, so a recursive program can keep
/// inventing values. This defeats the criterion; it does not prove that
/// every evaluation diverges.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminationViolation {
    criterion: TerminationCriterion,
    edge: Box<SpecialEdge>,
    // bounded: positions of one component of the finite program.
    component: Vec<ArgumentPosition>,
}

impl TerminationViolation {
    pub(super) fn new(
        criterion: TerminationCriterion,
        edge: SpecialEdge,
        mut component: Vec<ArgumentPosition>,
    ) -> Self {
        component.sort();
        Self {
            criterion,
            edge: Box::new(edge),
            component,
        }
    }

    /// The criterion that was not met.
    #[must_use]
    pub fn criterion(&self) -> TerminationCriterion {
        self.criterion
    }

    /// The first cyclic special edge in canonical order.
    #[must_use]
    pub fn edge(&self) -> &SpecialEdge {
        &self.edge
    }

    /// Positions of the edge's strongly connected component, sorted.
    #[must_use]
    pub fn component(&self) -> &[ArgumentPosition] {
        &self.component
    }

    /// Code of every violation of [`TerminationCriterion::WeaklyAcyclic`].
    pub const CODE_NOT_WEAKLY_ACYCLIC: ErrorCode = ErrorCode::new("not-weakly-acyclic");

    /// Stable kebab-case code of this violation.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self.criterion {
            TerminationCriterion::WeaklyAcyclic => Self::CODE_NOT_WEAKLY_ACYCLIC,
        }
    }
}

impl fmt::Display for TerminationViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} is not met: {} lies on a cycle through positions ",
            self.code(),
            self.criterion,
            self.edge
        )?;
        for (index, position) in self.component.iter().enumerate() {
            if index > 0 {
                f.write_str("; ")?;
            }
            position.fmt(f)?;
        }
        Ok(())
    }
}

impl Error for TerminationViolation {}
