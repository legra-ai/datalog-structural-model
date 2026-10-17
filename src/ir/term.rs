//! Datalog [`Term`] enum and the triple-term nesting bound.

use std::fmt;

use super::atom::Atom;
use super::symbols::{ConstantSymbol, VariableSymbol};

/// Datalog term (`variable`, `constant`, `quoted-triple term`,
/// or `SKOLEM(name, args...)` function call).
///
/// The `TripleTerm` variant lets rules name quoted-triple
/// arguments per RDF 1.2 / RDF-star, encoded as a 2-arity
/// [`Atom`] whose predicate is the triple's predicate IRI and
/// whose `terms` slot holds `[subject, object]`. Nesting is
/// bounded by [`MAX_TRIPLE_TERM_NESTING`] to keep unification
/// cost finite — programs that exceed the limit are rejected at
/// [`Program::validate`](super::program::Program::validate) time.
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
#[serde(deny_unknown_fields)]
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
    pub(super) fn collect_variables(&self, out: &mut std::collections::BTreeSet<VariableSymbol>) {
        match self {
            Self::Variable(var) => {
                out.insert(var.clone());
            }
            Self::Constant(_) => {}
            Self::TripleTerm(atom) => {
                for term in &atom.terms {
                    term.collect_variables(out);
                }
            }
            Self::Skolem { args, .. } => {
                for term in args {
                    term.collect_variables(out);
                }
            }
        }
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

/// Maximum allowed nesting depth for [`Term::TripleTerm`]
/// variants. Programs whose triple-term nesting exceeds this
/// limit are rejected at
/// [`Program::validate`](super::program::Program::validate) time.
///
/// The bound exists because unification cost grows with nesting
/// and because the W3C RDF-star CG has not specified a
/// normative bound — see
/// [rdf12-triple-term-reasoning.mdx](the RDF-star specification)
/// for the rationale.
pub const MAX_TRIPLE_TERM_NESTING: usize = 4;

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
