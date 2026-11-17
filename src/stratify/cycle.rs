//! [`UnstratifiableCycle`] — the component and the strict edge inside it
//! that make a program unstratifiable.

use std::fmt;

use crate::ir::PredicateSymbol;
use crate::position::{LiteralPath, RuleIndex};

/// Why a dependency must cross strata.
#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub enum StrictDependencyKind {
    /// The consumer literal is negated.
    Negation,
    /// The consumer literal sits inside an aggregate's inner body.
    Aggregation,
}

impl fmt::Display for StrictDependencyKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Negation => "negation",
            Self::Aggregation => "aggregation",
        })
    }
}

/// A producer-to-consumer dependency that requires a strictly earlier
/// producer stratum.
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StrictDependency {
    /// Rule whose head can match the consumer literal.
    pub producer: RuleIndex,
    /// Rule whose body contains the consumer literal.
    pub consumer: RuleIndex,
    /// The consumer literal.
    pub literal: LiteralPath,
    /// Negation or aggregation.
    pub kind: StrictDependencyKind,
}

impl fmt::Display for StrictDependency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} edge from {} into {}, {}",
            self.kind, self.producer, self.consumer, self.literal
        )
    }
}

/// A strongly connected rule component containing a strict dependency.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnstratifiableCycle {
    /// Rules of the component, sorted.
    // bounded: rules of one component of the finite program.
    pub rules: Vec<RuleIndex>,
    /// Head predicates of those rules, sorted and unique.
    pub predicates: Vec<PredicateSymbol>,
    /// The offending strict edge inside the component.
    pub edge: StrictDependency,
}

impl fmt::Display for UnstratifiableCycle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("recursive cycle through ")?;
        self.edge.kind.fmt(f)?;
        f.write_str(" among rules ")?;
        for (index, rule) in self.rules.iter().enumerate() {
            if index > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{}", rule.get())?;
        }
        f.write_str(" (predicates ")?;
        for (index, predicate) in self.predicates.iter().enumerate() {
            if index > 0 {
                f.write_str(", ")?;
            }
            f.write_str(predicate.as_str())?;
        }
        write!(f, "); offending {}", self.edge)
    }
}
