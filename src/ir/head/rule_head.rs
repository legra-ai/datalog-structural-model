//! [`RuleHead`] — what a rule concludes from its body.

use std::collections::BTreeSet;

use super::collections::{ExistentialVariables, HeadConjunction, HeadDisjunction};
use super::form::HeadForm;
use crate::ir::atom::Atom;
use crate::ir::symbols::VariableSymbol;
use crate::position::{HeadAtomIndex, HeadAtomPath, HeadDisjunctIndex};

/// The head of a [`Rule`](crate::Rule).
///
/// Every head reads as a disjunction of conjunctions of atoms: an atom is one
/// disjunct of one atom, and [`Self::Falsum`] has no disjunct at all. Atoms
/// are addressed by [`HeadAtomPath`] in that reading.
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RuleHead {
    /// One atom: plain Datalog.
    Atom(Atom),
    /// Several atoms derived together from one body match.
    Conjunction(HeadConjunction),
    /// `∃ variables . conjunction`: each body match requires witnesses for
    /// the existential variables.
    Existential {
        /// The existentially quantified variables.
        variables: ExistentialVariables,
        /// The atoms over body and existential variables.
        conjunction: HeadConjunction,
    },
    /// At least one of the disjuncts holds for each body match.
    Disjunction(HeadDisjunction),
    /// `false`: the body must have no match. An inconsistency constraint.
    Falsum,
}

impl RuleHead {
    /// The head's logical form; see [`HeadForm`] for the precedence.
    #[must_use]
    pub fn form(&self) -> HeadForm {
        match self {
            Self::Falsum => HeadForm::Inconsistency,
            Self::Existential { .. } => HeadForm::Existential,
            Self::Disjunction(disjunction) if disjunction.disjuncts().len() > 1 => {
                HeadForm::Disjunctive
            }
            Self::Atom(_) | Self::Conjunction(_) | Self::Disjunction(_) => {
                if self
                    .disjunct(HeadDisjunctIndex::new(0))
                    .map_or(0, <[Atom]>::len)
                    > 1
                {
                    HeadForm::Conjunctive
                } else {
                    HeadForm::Atomic
                }
            }
        }
    }

    /// The single atom of a [`Self::Atom`] head; `None` for any other head.
    #[must_use]
    pub fn as_atom(&self) -> Option<&Atom> {
        match self {
            Self::Atom(atom) => Some(atom),
            Self::Conjunction(_)
            | Self::Existential { .. }
            | Self::Disjunction(_)
            | Self::Falsum => None,
        }
    }

    /// Mutable access to the single atom of a [`Self::Atom`] head.
    pub fn as_atom_mut(&mut self) -> Option<&mut Atom> {
        match self {
            Self::Atom(atom) => Some(atom),
            Self::Conjunction(_)
            | Self::Existential { .. }
            | Self::Disjunction(_)
            | Self::Falsum => None,
        }
    }

    /// Number of disjuncts: zero for [`Self::Falsum`], otherwise at least one.
    #[must_use]
    pub fn disjunct_count(&self) -> usize {
        match self {
            Self::Falsum => 0,
            Self::Disjunction(disjunction) => disjunction.disjuncts().len(),
            Self::Atom(_) | Self::Conjunction(_) | Self::Existential { .. } => 1,
        }
    }

    /// The atoms of one disjunct, or `None` past the last disjunct.
    #[must_use]
    pub fn disjunct(&self, index: HeadDisjunctIndex) -> Option<&[Atom]> {
        match (self, index.get()) {
            (Self::Atom(atom), 0) => Some(std::slice::from_ref(atom)),
            (Self::Conjunction(conjunction) | Self::Existential { conjunction, .. }, 0) => {
                Some(conjunction.atoms())
            }
            (Self::Disjunction(disjunction), index) => disjunction
                .disjuncts()
                .get(index)
                .map(HeadConjunction::atoms),
            _ => None,
        }
    }

    /// Every head atom with its path, disjunct by disjunct in authored order.
    pub fn atoms(&self) -> impl Iterator<Item = (HeadAtomPath, &Atom)> {
        (0..self.disjunct_count()).flat_map(move |disjunct| {
            let disjunct = HeadDisjunctIndex::new(disjunct);
            self.disjunct(disjunct)
                .into_iter()
                .flatten()
                .enumerate()
                .map(move |(atom, value)| {
                    (HeadAtomPath::new(disjunct, HeadAtomIndex::new(atom)), value)
                })
        })
    }

    /// Mutable access to every head atom, disjunct by disjunct.
    pub fn atoms_mut(&mut self) -> Box<dyn Iterator<Item = &mut Atom> + '_> {
        match self {
            Self::Atom(atom) => Box::new(std::iter::once(atom)),
            Self::Conjunction(conjunction) | Self::Existential { conjunction, .. } => {
                Box::new(conjunction.atoms_mut().iter_mut())
            }
            Self::Disjunction(disjunction) => Box::new(
                disjunction
                    .disjuncts_mut()
                    .iter_mut()
                    .flat_map(|disjunct| disjunct.atoms_mut().iter_mut()),
            ),
            Self::Falsum => Box::new(std::iter::empty()),
        }
    }

    /// The existentially quantified variables; empty unless existential.
    #[must_use]
    pub fn existential_variables(&self) -> &[VariableSymbol] {
        match self {
            Self::Existential { variables, .. } => variables.variables(),
            Self::Atom(_) | Self::Conjunction(_) | Self::Disjunction(_) | Self::Falsum => &[],
        }
    }

    /// Whether `variable` is existentially quantified by this head.
    #[must_use]
    pub fn is_existential(&self, variable: &VariableSymbol) -> bool {
        match self {
            Self::Existential { variables, .. } => variables.contains(variable),
            Self::Atom(_) | Self::Conjunction(_) | Self::Disjunction(_) | Self::Falsum => false,
        }
    }

    /// Distinct variables of every head atom, existential ones included.
    #[must_use]
    pub fn variables(&self) -> BTreeSet<VariableSymbol> {
        self.atoms()
            .flat_map(|(_, atom)| atom.variables())
            .collect()
    }
}

impl From<Atom> for RuleHead {
    fn from(atom: Atom) -> Self {
        Self::Atom(atom)
    }
}
