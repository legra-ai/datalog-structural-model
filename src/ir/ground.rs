//! Ground (variable-free) term and atom value forms.

use std::fmt;

use super::symbols::{ConstantSymbol, PredicateSymbol};

/// Ground term — the value form of a [`Term`](super::term::Term)
/// with all variables substituted. Used as the cell type inside
/// tuples so an evaluator can store both flat
/// constant-only facts and quoted-triple facts (TT1).
///
/// `Constant` corresponds to the
/// [`Term::Constant`](super::term::Term::Constant) form;
/// `TripleTerm` is the value form of
/// [`Term::TripleTerm`](super::term::Term::TripleTerm) —
/// a [`GroundAtom`] whose argument list contains only further
/// `GroundTerm` values (no variables, recursively).
///
/// Total order: `Constant(_) < TripleTerm(_)`. Within each
/// variant, the inner symbol or atom's lexicographic order
/// determines the result. This is the determinism contract
/// an evaluator can rely on when serialising grounded facts.
#[derive(
    Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
#[serde(try_from = "super::serde_input::GroundTermInput")]
pub enum GroundTerm {
    /// Ground constant — the value form of
    /// [`Term::Constant`](super::term::Term::Constant).
    Constant(ConstantSymbol),
    /// Ground quoted-triple term — the value form of
    /// [`Term::TripleTerm`](super::term::Term::TripleTerm).
    /// Contains a [`GroundAtom`] whose argument list contains
    /// only further `GroundTerm` values; recursive nesting is
    /// bounded by
    /// [`MAX_TRIPLE_TERM_NESTING`](super::term::MAX_TRIPLE_TERM_NESTING)
    /// (enforced at IR
    /// [`Program::validate`](super::program::Program::validate)
    /// time, before grounding).
    TripleTerm(Box<GroundAtom>),
}

impl GroundTerm {
    /// Construct a flat-constant ground term. Convenience for
    /// the common case where the underlying value is a plain
    /// [`ConstantSymbol`].
    #[must_use]
    pub fn constant(c: ConstantSymbol) -> Self {
        Self::Constant(c)
    }

    /// Pull out the wrapped [`ConstantSymbol`] when this is the
    /// constant form. Returns `None` for [`GroundTerm::TripleTerm`].
    #[must_use]
    pub fn as_constant(&self) -> Option<&ConstantSymbol> {
        match self {
            Self::Constant(c) => Some(c),
            Self::TripleTerm(_) => None,
        }
    }
}

impl From<ConstantSymbol> for GroundTerm {
    fn from(c: ConstantSymbol) -> Self {
        Self::Constant(c)
    }
}

/// Ground form of an [`Atom`](super::atom::Atom) — the value
/// form with no variables anywhere in the argument tree. Used
/// inside [`GroundTerm::TripleTerm`] so quoted-triple facts can
/// be stored alongside flat facts in a fact set (TT1).
#[derive(
    Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct GroundAtom {
    /// Predicate of the ground atom.
    pub predicate: PredicateSymbol,
    /// Ground arguments in positional order.
    pub args: Vec<GroundTerm>,
}

impl GroundAtom {
    /// Arity of the ground atom.
    #[must_use]
    pub fn arity(&self) -> usize {
        self.args.len()
    }
}

/// Display a [`GroundTerm`]: constants as their raw symbol,
/// quoted-triple terms as `<< s p o >>` per N-Triples-star
/// syntax (mirroring [`Term`](super::term::Term)'s display
/// contract).
impl fmt::Display for GroundTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Constant(c) => write!(f, "{}", c.as_str()),
            Self::TripleTerm(atom) => {
                if atom.args.len() == 2 {
                    write!(
                        f,
                        "<< {} {} {} >>",
                        atom.args[0],
                        atom.predicate.as_str(),
                        atom.args[1],
                    )
                } else {
                    write!(f, "<< {atom} >>")
                }
            }
        }
    }
}

/// Display a [`GroundAtom`] as `predicate(arg_1, arg_2, …)`.
/// Used by [`GroundTerm`]'s [`std::fmt::Display`] impl for non-2-arity
/// triple-term values.
impl fmt::Display for GroundAtom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}(", self.predicate.as_str())?;
        for (idx, arg) in self.args.iter().enumerate() {
            if idx > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{arg}")?;
        }
        write!(f, ")")
    }
}
