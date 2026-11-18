//! [`WeakAcyclicity`] — the certificate returned for a weakly acyclic program.

use super::criterion::TerminationCriterion;
use super::edge::ArgumentPosition;

/// Evidence that a program satisfies [`TerminationCriterion::WeaklyAcyclic`].
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeakAcyclicity {
    pub(super) positions: usize,
    pub(super) special_edges: usize,
    pub(super) skolem_depth: usize,
    pub(super) deepest: Option<ArgumentPosition>,
}

impl WeakAcyclicity {
    /// The criterion this evidence proves.
    #[must_use]
    pub fn criterion(&self) -> TerminationCriterion {
        TerminationCriterion::WeaklyAcyclic
    }

    /// Number of distinct predicate positions in the graph.
    #[must_use]
    pub fn positions(&self) -> usize {
        self.positions
    }

    /// Number of value-inventing edges, all acyclic.
    #[must_use]
    pub fn special_edges(&self) -> usize {
        self.special_edges
    }

    /// The largest number of value-inventing edges on any path: an upper
    /// bound on how deeply invented values can nest.
    #[must_use]
    pub fn skolem_depth(&self) -> usize {
        self.skolem_depth
    }

    /// The smallest position reaching [`Self::skolem_depth`], when it is
    /// greater than zero.
    #[must_use]
    pub fn deepest(&self) -> Option<&ArgumentPosition> {
        self.deepest.as_ref()
    }
}
