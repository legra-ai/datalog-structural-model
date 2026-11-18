//! [`HeadAtomPath`] — one atom of a rule head, through its disjunct.

use std::fmt;

use super::index::{HeadAtomIndex, HeadDisjunctIndex};

/// Path from a rule head to one atom: the disjunct, then the atom inside that
/// disjunct's conjunction. An atomic head is disjunct 0, atom 0.
#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct HeadAtomPath {
    disjunct: HeadDisjunctIndex,
    atom: HeadAtomIndex,
}

impl HeadAtomPath {
    /// Atom `atom` of head disjunct `disjunct`.
    #[must_use]
    pub const fn new(disjunct: HeadDisjunctIndex, atom: HeadAtomIndex) -> Self {
        Self { disjunct, atom }
    }

    /// The only atom of an atomic head.
    #[must_use]
    pub const fn atomic() -> Self {
        Self::new(HeadDisjunctIndex::new(0), HeadAtomIndex::new(0))
    }

    /// The disjunct holding the atom.
    #[must_use]
    pub const fn disjunct(self) -> HeadDisjunctIndex {
        self.disjunct
    }

    /// The atom's position inside its disjunct.
    #[must_use]
    pub const fn atom(self) -> HeadAtomIndex {
        self.atom
    }
}

impl fmt::Display for HeadAtomPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "head disjunct {} atom {}",
            self.disjunct.get(),
            self.atom.get()
        )
    }
}
