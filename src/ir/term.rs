//! Datalog [`Term`] enum.

use std::fmt;

use super::atom::Atom;
use super::symbols::{ConstantSymbol, VariableSymbol};

/// Datalog term (`variable`, `constant`, `quoted-triple term`,
/// or `SKOLEM(name, args...)` function call).
///
/// The `TripleTerm` variant wraps a structured [`Atom`]. An adapter owns
/// the interpretation of that atom's relation symbol and ordered operands.
/// A two-operand encoding can put a fixed RDF predicate IRI in `predicate`
/// and `[subject, object]` in `terms`. For a variable RDF predicate, an
/// adapter can instead use a fixed triple relation with three operands
/// `[subject, predicate, object]`. The RDF reasoning catalogue uses
/// `urn:rdf-reasoning:triple` for this three-slot convention, recursively.
/// Its relation symbol is distinct from the RDF predicate operand.
///
/// These encodings are structurally distinct; this model neither converts
/// between them nor validates RDF term domains or blank-node scope. Display
/// text is not a canonical identity or serialization. Nesting is bounded by
/// [`ValidationLimits::triple_term_nesting`](crate::ValidationLimits::triple_term_nesting);
/// programs exceeding it are rejected by
/// [`Program::validate`](super::program::Program::validate).
///
/// The `Skolem` variant (extension) is a deterministic fresh-IRI
/// minting operator: given a label and a tuple of argument terms,
/// the evaluator produces a stable
/// `urn:example:skolem:<label>:<hash>` IRI that's content-addressed
/// from the argument bindings. SKOLEM is only meaningful in head
/// positions — body atoms that carry a `Skolem` term are rejected
/// at validation time as
/// [`super::error::ProgramValidationError::SkolemInBodyAtom`]
/// because a SKOLEM term has no pattern semantics (it's a
/// function, not a wildcard).
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "super::serde_input::TermInput")]
pub enum Term {
    /// Variable term.
    Variable(VariableSymbol),
    /// Constant term.
    Constant(ConstantSymbol),
    /// Quoted-triple term: an inner [`Atom`] used as a value in
    /// the enclosing atom. Conceptually represents `<< s p o >>`
    /// per RDF 1.2 N-Triples-star syntax.
    TripleTerm(Box<Atom>),
    /// `SKOLEM(name, arg_1, …, arg_n)` — a deterministic fresh-IRI
    /// minting function (extension). Permitted in rule heads only;
    /// arguments must be [`Self::Variable`] or [`Self::Constant`]
    /// (no nested SKOLEM, no quoted-triple terms).
    Skolem {
        /// Label that namespaces the skolemized IRI — typically a
        /// short ASCII tag like `"example-key"` so collisions across
        /// independent rule sets stay scoped per author.
        name: ConstantSymbol,
        /// Argument terms whose ground values feed the
        /// content-addressed hash. Must be variables or constants.
        args: Vec<Term>,
    },
}

impl Term {
    /// Distinct variables in stable symbol order, recursively including triple
    /// terms and Skolem arguments. Constants and relation names are excluded.
    /// This inspects syntax; it does not resolve bindings or evaluate Skolem calls.
    #[must_use]
    pub fn variables(&self) -> std::collections::BTreeSet<VariableSymbol> {
        // bounded: variable symbols in this finite authored term, never facts.
        let mut variables = std::collections::BTreeSet::new();
        self.collect_variables(&mut variables);
        variables
    }

    /// Visit borrowed variable occurrences in operand order, including nested
    /// triple terms and Skolem arguments. Repeated occurrences are retained.
    /// This traverses authored syntax without cloning symbols or resolving values.
    pub fn visit_variables<'a>(&'a self, visitor: &mut impl FnMut(&'a VariableSymbol)) {
        match self {
            Self::Variable(variable) => visitor(variable),
            Self::Constant(_) => {}
            Self::TripleTerm(atom) => atom.visit_variables(visitor),
            Self::Skolem { args, .. } => {
                for term in args {
                    term.visit_variables(visitor);
                }
            }
        }
    }

    pub(super) fn collect_variables(&self, out: &mut std::collections::BTreeSet<VariableSymbol>) {
        self.visit_variables(&mut |variable| {
            out.insert(variable.clone());
        });
    }

    /// Depth of the term's triple-term nesting. `0` for plain
    /// variables, constants, and SKOLEM terms (a SKOLEM produces a
    /// flat IRI so it doesn't extend the triple-term tree);
    /// `1 + max_child_depth` for a `TripleTerm` whose inner atom
    /// contains other terms.
    #[must_use]
    pub(super) fn nesting_depth(&self) -> usize {
        match self {
            Self::Variable(_) | Self::Constant(_) | Self::Skolem { .. } => 0,
            Self::TripleTerm(atom) => 1 + atom.nesting_depth(),
        }
    }

    /// Returns `true` when the term is a `SKOLEM(...)` function
    /// call. Used by body-atom validation to reject SKOLEM terms
    /// outside head positions.
    #[must_use]
    pub fn is_skolem(&self) -> bool {
        matches!(self, Self::Skolem { .. })
    }
}

/// Display a [`Term`]: variables as `?<name>`, constants as their
/// raw symbol, and triple terms as `<< s p o >>` per
/// N-Triples-star syntax.
impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Variable(var) => write!(f, "?{}", var.as_str()),
            Self::Constant(c) => write!(f, "{}", c.as_str()),
            Self::TripleTerm(atom) => {
                // RDF-star convention: `<< s p o >>` for the
                // 2-arity case (subject + object as the inner
                // atom's `terms`, predicate as the inner atom's
                // `predicate`). For other arities — which are
                // valid as a Datalog encoding even though they
                // are not standard RDF triples — fall back to
                // `<< pred(t1, t2, …) >>` so the print still
                // round-trips through a developer's eye.
                if atom.terms.len() == 2 {
                    write!(
                        f,
                        "<< {} {} {} >>",
                        atom.terms[0],
                        atom.predicate.as_str(),
                        atom.terms[1],
                    )
                } else {
                    write!(f, "<< {atom} >>")
                }
            }
            Self::Skolem { name, args } => {
                write!(f, "SKOLEM({}", name.as_str())?;
                for arg in args {
                    write!(f, ", {arg}")?;
                }
                write!(f, ")")
            }
        }
    }
}
