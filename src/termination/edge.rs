//! Positions of the position graph and its value-inventing edges.

use std::fmt;

use crate::ir::PredicateSymbol;
use crate::position::{AtomSlot, RuleIndex, RuleSite};

/// One argument slot of one atom occurrence: a node of the position graph.
///
/// Positions are refined by occurrence: a head slot feeds a body slot only
/// when the head and body atoms can denote the same fact, so constants that
/// keep two patterns apart also keep their positions apart.
#[derive(
    Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct ArgumentPosition {
    /// The atom's predicate.
    pub predicate: PredicateSymbol,
    /// The top-level argument slot.
    pub slot: AtomSlot,
    /// The rule containing the atom.
    pub rule: RuleIndex,
    /// The head, or the positive body literal holding the atom.
    pub site: RuleSite,
}

impl ArgumentPosition {
    /// Slot `slot` of the `predicate` atom at `site` of `rule`.
    #[must_use]
    pub fn new(
        predicate: PredicateSymbol,
        slot: AtomSlot,
        rule: RuleIndex,
        site: RuleSite,
    ) -> Self {
        Self {
            predicate,
            slot,
            rule,
            site,
        }
    }
}

impl fmt::Display for ArgumentPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}[{}] ({}, {})",
            self.predicate.as_str(),
            self.slot.get(),
            self.rule,
            self.site
        )
    }
}

/// How a rule head builds a value that need not occur in its body.
#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum InventionKind {
    /// A `SKOLEM(...)` head term mints a value from its arguments.
    Skolem,
    /// A head triple term wraps body values into a larger term.
    TripleTerm,
    /// A `BIND` expression computes a value from body values.
    Bind,
    /// An aggregate folds body values into a result value.
    Aggregate,
    /// An existential head variable stands for a witness that depends on
    /// the rule's frontier variables.
    Existential,
}

impl fmt::Display for InventionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Skolem => "Skolem",
            Self::TripleTerm => "triple-term",
            Self::Bind => "BIND",
            Self::Aggregate => "aggregate",
            Self::Existential => "existential",
        })
    }
}

/// A special (value-inventing) edge of the position graph: a value bound at
/// body position `from` feeds a value invented at head position `to` of the
/// same rule.
///
/// The derived order — `from`, `to`, then `kind` — is the canonical order
/// used to pick the reported violation.
#[derive(
    Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct SpecialEdge {
    /// Body position the input value is bound at.
    pub from: ArgumentPosition,
    /// Head position receiving the invented value.
    pub to: ArgumentPosition,
    /// The inventing construct.
    pub kind: InventionKind,
}

impl fmt::Display for SpecialEdge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} edge from {} to {}", self.kind, self.from, self.to)
    }
}
