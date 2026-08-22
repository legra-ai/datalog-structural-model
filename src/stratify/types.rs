//! Public stratification result types and error.

use std::collections::{BTreeMap, BTreeSet};

use crate::ir::{Atom, ConstantSymbol, PredicateSymbol, Term};

/// Stratification of a Datalog¬ program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stratification {
    /// Strata ordered by ascending index. Stratum `k` must be
    /// evaluated to a fixed point before stratum `k+1`.
    pub strata: Vec<Stratum>,
    /// Map from predicate to its stratum index. context-aware rolls up
    /// the underlying `(predicate, head-graph-class)` dependency
    /// nodes by taking the **maximum** stratum across head-graph
    /// classes of the same predicate — so a producer that emits
    /// facts in more than one graph class is scheduled late
    /// enough for every downstream consumer.
    pub predicate_stratum: BTreeMap<PredicateSymbol, usize>,
}

impl Stratification {
    /// Returns the stratum index for a predicate, or `None` if
    /// the predicate does not appear in the program (pure EDB
    /// predicates not derived by any rule).
    #[must_use]
    pub fn stratum_of(&self, predicate: &PredicateSymbol) -> Option<usize> {
        self.predicate_stratum.get(predicate).copied()
    }
}

/// A single stratum — the set of predicates defined together.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Stratum {
    /// Predicates whose rules must be evaluated to a fixed point
    /// within this stratum.
    pub predicates: BTreeSet<PredicateSymbol>,
}

/// Stratification failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StratificationError {
    /// The predicate dependency graph contains a cycle that
    /// passes through a negation edge.
    UnstratifiableCycle {
        /// Every predicate in the offending strongly connected
        /// component.
        predicates: Vec<PredicateSymbol>,
    },
}

/// Classification of a rule head's trailing graph slot (context-aware).
///
/// Stratification keys dependency-graph nodes on
/// `(PredicateSymbol, HeadGraphClass)` so a producer that pins
/// every inferred quad to a fixed graph
/// (`GraphExpr::Literal { graph }` in the reasoner — head ends in
/// a constant graph IRI) creates a node that's distinct from a
/// producer emitting into a body-inherited graph
/// (`GraphExpr::InheritFromBody { .. }` — head ends in a graph
/// variable like `__g0`). Downstream consumers that constrain
/// their body graph slot to a constant IRI then depend only on
/// the producers compatible with that graph.
///
/// For the canonical RDFS / OWL-RL profile every rule head ends
/// in a graph variable (context-aware's `InheritFromBody { 0 }` default),
/// so every node collapses to [`HeadGraphClass::Inherit`] and the
/// per-predicate strata that come out the other end match the
/// pre-context-aware algorithm bit-for-bit.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) enum HeadGraphClass {
    /// Head has no trailing term — pre-context-aware narrow atoms
    /// (predicate-only facts or rules whose head atom is
    /// constructed without a graph slot).
    Narrow,
    /// Head's trailing graph term is a variable — the inferred
    /// quad inherits its graph from whichever body atom binds
    /// that variable, so the rule contributes facts to every
    /// graph class its body atom matches.
    Inherit,
    /// Head's trailing graph term is a constant IRI — every
    /// inferred quad lands in this fixed graph.
    Literal(ConstantSymbol),
}

impl HeadGraphClass {
    /// Classify a head atom's trailing term.
    ///
    /// The classification mirrors how the reasoner's
    /// `GraphExpr` rewrites head atoms at compile time
    /// ([`super::super`] of this module sees only the resulting
    /// term, not the original `GraphExpr`):
    ///
    /// - trailing `Term::Variable` → [`HeadGraphClass::Inherit`]
    /// - trailing `Term::Constant` → [`HeadGraphClass::Literal`]
    /// - no trailing term → [`HeadGraphClass::Narrow`]
    /// - trailing `Term::TripleTerm` / `Term::Skolem` collapses to
    ///   [`HeadGraphClass::Inherit`] because both produce a value computed from
    ///   body bindings rather than a fixed IRI known at compile time.
    #[must_use]
    pub(super) fn classify(head: &Atom) -> Self {
        match head.terms.last() {
            None => Self::Narrow,
            Some(Term::Constant(c)) => Self::Literal(c.clone()),
            Some(Term::Variable(_) | Term::TripleTerm(_) | Term::Skolem { .. }) => Self::Inherit,
        }
    }
}
