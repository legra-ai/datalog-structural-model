//! [`RuleSite`] — head or body location inside one rule.

use std::fmt;

use super::head_path::HeadAtomPath;
use super::literal_path::LiteralPath;

/// The part of a rule a location falls in.
#[derive(
    Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub enum RuleSite {
    /// One atom of the rule head.
    Head(HeadAtomPath),
    /// A literal in the rule body.
    Body(LiteralPath),
}

impl fmt::Display for RuleSite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Head(path) => path.fmt(f),
            Self::Body(path) => path.fmt(f),
        }
    }
}
