//! Datalog [`Atom`] — predicate applied to an ordered term list.

use std::collections::BTreeSet;
use std::fmt;

use super::symbols::{PredicateSymbol, VariableSymbol};
use super::term::Term;

/// Datalog atom: `predicate(term_1, ..., term_n)`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Atom {
    /// Predicate identifier.
    pub predicate: PredicateSymbol,
    /// Ordered list of terms.
    pub terms: Vec<Term>,
}

impl Atom {
    /// Arity of the atom.
    #[must_use]
    pub fn arity(&self) -> usize {
        self.terms.len()
    }

    /// All variables that appear anywhere in this atom, including
    /// variables nested inside [`Term::TripleTerm`] arguments
    /// (recursively).
    #[must_use]
    pub(super) fn variables(&self) -> BTreeSet<VariableSymbol> {
        let mut acc = BTreeSet::new();
        self.collect_variables(&mut acc);
        acc
    }

    fn collect_variables(&self, acc: &mut BTreeSet<VariableSymbol>) {
        for term in &self.terms {
            term.collect_variables(acc);
        }
    }

    /// Maximum [`Term::TripleTerm`] nesting depth observed
    /// anywhere in this atom's argument tree. `0` for a
    /// plain-term atom; otherwise the depth of the deepest
    /// nested triple-term.
    #[must_use]
    pub(super) fn nesting_depth(&self) -> usize {
        self.terms
            .iter()
            .map(Term::nesting_depth)
            .max()
            .unwrap_or(0)
    }
}

/// Display an [`Atom`] as `predicate(term_1, term_2, …)`. Used
/// directly by [`Term`]'s [`std::fmt::Display`] impl for non-2-arity
/// triple terms and available to error messages / debugging.
impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}(", self.predicate.as_str())?;
        for (idx, term) in self.terms.iter().enumerate() {
            if idx > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{term}")?;
        }
        write!(f, ")")
    }
}
