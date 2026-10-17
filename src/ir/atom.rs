//! Datalog [`Atom`] — predicate applied to an ordered term list.

use std::collections::BTreeSet;
use std::fmt;

use super::symbols::{PredicateSymbol, VariableSymbol};
use super::term::Term;

/// Datalog atom: `predicate(term_1, ..., term_n)`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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
            collect_term_variables(term, acc);
        }
    }

    /// Returns `true` when any argument (including those nested
    /// inside a [`Term::TripleTerm`]) is a `SKOLEM(...)` term.
    /// Used by validation to reject `SKOLEM` outside head atoms —
    /// a SKOLEM term has no pattern semantics, so it can't appear
    /// in a body literal.
    #[must_use]
    pub(super) fn contains_skolem(&self) -> bool {
        self.terms.iter().any(term_contains_skolem)
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

fn collect_term_variables(term: &Term, acc: &mut BTreeSet<VariableSymbol>) {
    match term {
        Term::Variable(var) => {
            acc.insert(var.clone());
        }
        Term::Constant(_) => {}
        Term::TripleTerm(inner) => {
            for child in &inner.terms {
                collect_term_variables(child, acc);
            }
        }
        Term::Skolem { args, .. } => {
            for arg in args {
                collect_term_variables(arg, acc);
            }
        }
    }
}

fn term_contains_skolem(term: &Term) -> bool {
    match term {
        Term::Skolem { .. } => true,
        Term::TripleTerm(inner) => inner.terms.iter().any(term_contains_skolem),
        Term::Variable(_) | Term::Constant(_) => false,
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
