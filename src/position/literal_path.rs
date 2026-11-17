//! [`LiteralPath`] — a literal in a rule body, possibly inside aggregates.

use std::fmt;

use super::index::LiteralIndex;

/// Path from a rule body to one literal: the outer body position, then one
/// inner-body position per enclosing aggregate.
#[derive(
    Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct LiteralPath(
    // bounded: one step per authored aggregate nesting level.
    Vec<LiteralIndex>,
);

impl LiteralPath {
    /// Path to a literal directly in the rule body.
    #[must_use]
    pub fn top(index: LiteralIndex) -> Self {
        Self(vec![index])
    }

    /// Path to a literal inside the aggregate this path points to.
    #[must_use]
    pub fn inner(&self, index: LiteralIndex) -> Self {
        let mut steps = self.0.clone();
        steps.push(index);
        Self(steps)
    }

    /// Outer body position followed by each inner-body position.
    #[must_use]
    pub fn steps(&self) -> &[LiteralIndex] {
        &self.0
    }

    /// Position in the rule body of the outermost literal.
    #[must_use]
    pub fn outer(&self) -> LiteralIndex {
        self.0[0]
    }
}

impl fmt::Display for LiteralPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "body literal {}", self.0[0].get())?;
        for step in &self.0[1..] {
            write!(f, " > inner literal {}", step.get())?;
        }
        Ok(())
    }
}
