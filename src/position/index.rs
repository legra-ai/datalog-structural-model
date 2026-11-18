//! Index newtypes: a rule in a program, a literal in a body, a slot in an
//! atom or Skolem argument list, and a term inside a filter or BIND
//! expression.

use std::fmt;

/// Position of a rule in [`Program::rules`](crate::Program::rules).
#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct RuleIndex(usize);

impl RuleIndex {
    /// Wrap a zero-based rule position.
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    /// The zero-based rule position.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

impl fmt::Display for RuleIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rule {}", self.0)
    }
}

/// Position of a literal in a rule body or an aggregate's inner body.
#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct LiteralIndex(usize);

impl LiteralIndex {
    /// Wrap a zero-based literal position.
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    /// The zero-based literal position.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

/// Position of a term in an atom, triple term, or Skolem argument list.
#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct AtomSlot(usize);

impl AtomSlot {
    /// Wrap a zero-based slot position.
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    /// The zero-based slot position.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

/// Position of a top-level term inside a filter or BIND expression, in
/// left-to-right source order.
#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct ExpressionTermIndex(usize);

impl ExpressionTermIndex {
    /// Wrap a zero-based expression-term position.
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    /// The zero-based expression-term position.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

/// Position of a disjunct in a rule head. Every non-disjunctive head has
/// exactly one disjunct, at position zero.
#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct HeadDisjunctIndex(usize);

impl HeadDisjunctIndex {
    /// Wrap a zero-based disjunct position.
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    /// The zero-based disjunct position.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

/// Position of an atom inside one head disjunct (conjunction).
#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct HeadAtomIndex(usize);

impl HeadAtomIndex {
    /// Wrap a zero-based head-atom position.
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    /// The zero-based head-atom position.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}
