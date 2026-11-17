//! [`TermPath`] — a term inside an atom or expression, through nested
//! triple terms and Skolem argument lists.

use std::fmt;

use super::index::{AtomSlot, ExpressionTermIndex};

/// Where the outermost term of a [`TermPath`] sits.
#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub enum TermRoot {
    /// A slot of the head or body atom.
    Slot(AtomSlot),
    /// A top-level term of a filter or BIND expression.
    ExpressionTerm(ExpressionTermIndex),
}

/// Path to a term: its root, then one argument slot per nested triple term
/// or Skolem call descended into.
#[derive(
    Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct TermPath {
    root: TermRoot,
    // bounded: depth of authored term nesting, which validation caps.
    nested: Vec<AtomSlot>,
}

impl TermPath {
    /// Path to a root term.
    #[must_use]
    pub fn new(root: TermRoot) -> Self {
        Self {
            root,
            nested: Vec::new(),
        }
    }

    /// Path to argument `slot` of the term this path points to.
    #[must_use]
    pub fn child(&self, slot: AtomSlot) -> Self {
        let mut nested = self.nested.clone();
        nested.push(slot);
        Self {
            root: self.root,
            nested,
        }
    }

    /// The outermost term's position.
    #[must_use]
    pub fn root(&self) -> TermRoot {
        self.root
    }

    /// Argument slots descended into below the root.
    #[must_use]
    pub fn nested(&self) -> &[AtomSlot] {
        &self.nested
    }
}

impl fmt::Display for TermPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.root {
            TermRoot::Slot(slot) => write!(f, "slot {}", slot.get())?,
            TermRoot::ExpressionTerm(index) => write!(f, "expression term {}", index.get())?,
        }
        for slot in &self.nested {
            write!(f, " > argument {}", slot.get())?;
        }
        Ok(())
    }
}
