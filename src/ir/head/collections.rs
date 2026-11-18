//! Non-empty collections inside a [`RuleHead`](super::RuleHead): they cannot
//! be built empty, neither by constructor nor by deserialization.

use std::collections::BTreeSet;
use std::{error::Error, fmt};

use crate::error_code::ErrorCode;
use crate::ir::atom::Atom;
use crate::ir::symbols::VariableSymbol;

/// Which head collection was given no elements.
#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum HeadCollectionKind {
    /// A [`HeadConjunction`].
    Conjunction,
    /// A [`HeadDisjunction`].
    Disjunction,
    /// An [`ExistentialVariables`] set.
    ExistentialVariables,
}

impl fmt::Display for HeadCollectionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Conjunction => "head conjunction",
            Self::Disjunction => "head disjunction",
            Self::ExistentialVariables => "existential variable set",
        })
    }
}

/// A head collection was constructed without elements.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmptyHeadCollection {
    kind: HeadCollectionKind,
}

impl EmptyHeadCollection {
    /// Code of every empty head collection.
    pub const CODE_EMPTY_HEAD_COLLECTION: ErrorCode = ErrorCode::new("empty-head-collection");

    const fn new(kind: HeadCollectionKind) -> Self {
        Self { kind }
    }

    /// The collection that was empty.
    #[must_use]
    pub const fn kind(&self) -> HeadCollectionKind {
        self.kind
    }

    /// Stable kebab-case code of this error.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        Self::CODE_EMPTY_HEAD_COLLECTION
    }
}

impl fmt::Display for EmptyHeadCollection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] a {} needs at least one element",
            self.code(),
            self.kind
        )
    }
}

impl Error for EmptyHeadCollection {}

/// A non-empty conjunction of head atoms, in authored order.
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "Vec<Atom>", into = "Vec<Atom>")]
pub struct HeadConjunction(
    // bounded: atoms of one authored rule head.
    Vec<Atom>,
);

impl HeadConjunction {
    /// A conjunction of `atoms`.
    /// # Errors
    /// Rejects an empty atom list.
    pub fn try_new(atoms: Vec<Atom>) -> Result<Self, EmptyHeadCollection> {
        if atoms.is_empty() {
            return Err(EmptyHeadCollection::new(HeadCollectionKind::Conjunction));
        }
        Ok(Self(atoms))
    }

    /// The atoms, never empty.
    #[must_use]
    pub fn atoms(&self) -> &[Atom] {
        &self.0
    }

    /// Mutable access to the atoms; the count cannot change.
    pub fn atoms_mut(&mut self) -> &mut [Atom] {
        &mut self.0
    }
}

impl TryFrom<Vec<Atom>> for HeadConjunction {
    type Error = EmptyHeadCollection;
    fn try_from(atoms: Vec<Atom>) -> Result<Self, Self::Error> {
        Self::try_new(atoms)
    }
}

impl From<HeadConjunction> for Vec<Atom> {
    fn from(conjunction: HeadConjunction) -> Self {
        conjunction.0
    }
}

impl From<Atom> for HeadConjunction {
    fn from(atom: Atom) -> Self {
        Self(vec![atom])
    }
}

/// A non-empty disjunction of head conjunctions, in authored order.
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "Vec<HeadConjunction>", into = "Vec<HeadConjunction>")]
pub struct HeadDisjunction(
    // bounded: disjuncts of one authored rule head.
    Vec<HeadConjunction>,
);

impl HeadDisjunction {
    /// A disjunction of `disjuncts`.
    /// # Errors
    /// Rejects an empty disjunct list.
    pub fn try_new(disjuncts: Vec<HeadConjunction>) -> Result<Self, EmptyHeadCollection> {
        if disjuncts.is_empty() {
            return Err(EmptyHeadCollection::new(HeadCollectionKind::Disjunction));
        }
        Ok(Self(disjuncts))
    }

    /// The disjuncts, never empty.
    #[must_use]
    pub fn disjuncts(&self) -> &[HeadConjunction] {
        &self.0
    }

    /// Mutable access to the disjuncts; the count cannot change.
    pub fn disjuncts_mut(&mut self) -> &mut [HeadConjunction] {
        &mut self.0
    }
}

impl TryFrom<Vec<HeadConjunction>> for HeadDisjunction {
    type Error = EmptyHeadCollection;
    fn try_from(disjuncts: Vec<HeadConjunction>) -> Result<Self, Self::Error> {
        Self::try_new(disjuncts)
    }
}

impl From<HeadDisjunction> for Vec<HeadConjunction> {
    fn from(disjunction: HeadDisjunction) -> Self {
        disjunction.0
    }
}

/// A non-empty set of existentially quantified head variables, kept in
/// symbol order without duplicates so equal sets have one encoding.
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "Vec<VariableSymbol>", into = "Vec<VariableSymbol>")]
pub struct ExistentialVariables(
    // bounded: variables of one authored rule head.
    Vec<VariableSymbol>,
);

impl ExistentialVariables {
    /// The set of `variables`; duplicates collapse.
    /// # Errors
    /// Rejects an empty variable list.
    pub fn try_new(
        variables: impl IntoIterator<Item = VariableSymbol>,
    ) -> Result<Self, EmptyHeadCollection> {
        let set: BTreeSet<VariableSymbol> = variables.into_iter().collect();
        if set.is_empty() {
            return Err(EmptyHeadCollection::new(
                HeadCollectionKind::ExistentialVariables,
            ));
        }
        Ok(Self(set.into_iter().collect()))
    }

    /// The variables in symbol order, never empty.
    #[must_use]
    pub fn variables(&self) -> &[VariableSymbol] {
        &self.0
    }

    /// Whether `variable` is existentially quantified.
    #[must_use]
    pub fn contains(&self, variable: &VariableSymbol) -> bool {
        self.0.binary_search(variable).is_ok()
    }
}

impl TryFrom<Vec<VariableSymbol>> for ExistentialVariables {
    type Error = EmptyHeadCollection;
    fn try_from(variables: Vec<VariableSymbol>) -> Result<Self, Self::Error> {
        Self::try_new(variables)
    }
}

impl From<ExistentialVariables> for Vec<VariableSymbol> {
    fn from(variables: ExistentialVariables) -> Self {
        variables.0
    }
}
