//! Predicate dependency graph used by stratification (context-aware:
//! keyed on `(predicate, head-graph class)` so producers that pin
//! different graph slots stay structurally independent).

use std::collections::{BTreeMap, BTreeSet};

use super::tarjan::TarjanState;
use super::types::HeadGraphClass;
use crate::ir::{Atom, ConstantSymbol, Literal, PredicateSymbol, Program, Term};

/// Composite key for stratification dependency-graph nodes:
/// `(head predicate, head-graph class)`. context-aware widens the
/// pre-existing predicate-only key so a producer with a fixed
/// graph head (`GraphExpr::Literal { graph }`) is a separate node
/// from a producer with a body-inherited graph head, while still
/// collapsing to one node per predicate in canonical RDFS / OWL-
/// RL programs (every rule head ends in a graph variable so every
/// node is `(pred, Inherit)`).
pub(super) type NodeKey = (PredicateSymbol, HeadGraphClass);

#[derive(Debug)]
pub(super) struct DependencyGraph {
    /// `node → outgoing edges`. The edge target is the head node
    /// of some rule that mentions this node's predicate in its
    /// body, where the body's trailing graph term is compatible
    /// with this node's head-graph class (see
    /// [`BodyGraphFilter::matches`]).
    pub(super) edges: BTreeMap<NodeKey, Vec<DependencyEdge>>,
    /// All nodes that appear in the program — every rule's
    /// `(head.predicate, HeadGraphClass)` plus every node a body
    /// atom can match against. Pure-EDB predicates (mentioned in
    /// a body but never as a head) appear as
    /// `(predicate, HeadGraphClass::Inherit)` because the EDB is
    /// graph-agnostic.
    pub(super) nodes: BTreeSet<NodeKey>,
}

#[derive(Clone, Debug)]
pub(super) struct DependencyEdge {
    pub(super) target: NodeKey,
    pub(super) negated: bool,
}

impl DependencyGraph {
    pub(super) fn build(program: &Program) -> Self {
        // Pass 1: collect every head node so body-edge fan-out
        // knows which producer classes exist for each predicate.
        let mut head_nodes: BTreeMap<PredicateSymbol, BTreeSet<HeadGraphClass>> = BTreeMap::new();
        for rule in &program.rules {
            head_nodes
                .entry(rule.head.predicate.clone())
                .or_default()
                .insert(HeadGraphClass::classify(&rule.head));
        }

        // Pass 2: walk every rule body. Each body literal
        // contributes one edge per *compatible* producer node:
        // if the body atom's trailing graph slot is a variable,
        // every head class of the body's predicate matches; if
        // the body slot is a constant, only the `Literal(C)`
        // producers (and `Inherit` producers, which may bind any
        // graph at firing time) match.
        let mut edges: BTreeMap<NodeKey, Vec<DependencyEdge>> = BTreeMap::new();
        let mut nodes: BTreeSet<NodeKey> = BTreeSet::new();
        for (pred, classes) in &head_nodes {
            for class in classes {
                nodes.insert((pred.clone(), class.clone()));
            }
        }

        for rule in &program.rules {
            let head_class = HeadGraphClass::classify(&rule.head);
            let head_key = (rule.head.predicate.clone(), head_class);
            nodes.insert(head_key.clone());

            for lit in &rule.body {
                match lit {
                    Literal::Positive(atom) => add_body_atom_edges(
                        atom,
                        &head_key,
                        false,
                        &head_nodes,
                        &mut nodes,
                        &mut edges,
                    ),
                    Literal::Negative(atom) => add_body_atom_edges(
                        atom,
                        &head_key,
                        true,
                        &head_nodes,
                        &mut nodes,
                        &mut edges,
                    ),
                    Literal::Aggregate(agg) => super::entry::add_aggregate_edges(
                        agg,
                        &head_key,
                        &head_nodes,
                        &mut nodes,
                        &mut edges,
                    ),
                    // Filter / Bind literals do not match facts
                    // and so do not contribute
                    // predicate-dependency edges. They are
                    // intra-stratum guards / bindings on the
                    // current candidate.
                    Literal::Filter(_) | Literal::Bind { .. } => continue,
                }
            }
        }

        Self { edges, nodes }
    }

    /// Tarjan's strongly connected components, returning the
    /// components in reverse topological order (sinks first).
    pub(super) fn strongly_connected_components(&self) -> Vec<BTreeSet<NodeKey>> {
        let mut state = TarjanState::new();
        for node in &self.nodes {
            if !state.indices.contains_key(node) {
                state.strong_connect(node, self);
            }
        }
        state.result
    }
}

/// Filter expressing which producer head-graph classes a body
/// atom's trailing graph slot matches.
pub(super) enum BodyGraphFilter {
    /// Body's trailing slot is a variable (or a `TripleTerm` /
    /// `Skolem` term, both of which produce a graph value bound
    /// at firing time): matches any producer class.
    Any,
    /// Body's trailing slot is a constant `C`: matches the
    /// `Literal(C)` producer and any `Inherit` producer (which
    /// could bind its head graph to `C` depending on the firing
    /// binding).
    Constant(ConstantSymbol),
    /// Body atom has no trailing slot: matches only the
    /// pre-context-aware narrow producer node.
    Narrow,
}

impl BodyGraphFilter {
    fn classify(atom: &Atom) -> Self {
        match atom.terms.last() {
            None => Self::Narrow,
            Some(Term::Constant(c)) => Self::Constant(c.clone()),
            Some(Term::Variable(_) | Term::TripleTerm(_) | Term::Skolem { .. }) => Self::Any,
        }
    }

    fn matches(&self, class: &HeadGraphClass) -> bool {
        match (self, class) {
            (Self::Any, _) => true,
            (Self::Constant(_), HeadGraphClass::Inherit) => true,
            (Self::Constant(c), HeadGraphClass::Literal(d)) => c == d,
            (Self::Narrow, HeadGraphClass::Narrow) => true,
            _ => false,
        }
    }
}

/// Add the dependency edges contributed by a single body atom.
/// The fan-out covers every producer node of the body's
/// predicate whose head-graph class is compatible with the
/// body's trailing graph filter (see [`BodyGraphFilter`]). Pure
/// EDB predicates (no head node in the program) still get a
/// single `(pred, Inherit)` node and a single edge so the
/// pre-existing predicate-level behaviour is preserved.
pub(super) fn add_body_atom_edges(
    atom: &Atom,
    head_key: &NodeKey,
    negated: bool,
    head_nodes: &BTreeMap<PredicateSymbol, BTreeSet<HeadGraphClass>>,
    nodes: &mut BTreeSet<NodeKey>,
    edges: &mut BTreeMap<NodeKey, Vec<DependencyEdge>>,
) {
    let filter = BodyGraphFilter::classify(atom);
    let producer_classes = head_nodes.get(&atom.predicate);

    // Collect compatible producer classes; fall back to a single
    // `Inherit` node when the body's predicate is a pure EDB
    // predicate or when the body's filter matches no recorded
    // producer (the consumer still needs *some* dependency node
    // to participate in the SCC).
    let mut targets: Vec<HeadGraphClass> = producer_classes
        .into_iter()
        .flat_map(|classes| classes.iter().cloned())
        .filter(|class| filter.matches(class))
        .collect();
    if targets.is_empty() {
        targets.push(default_body_class(&filter));
    }

    for class in targets {
        let source = (atom.predicate.clone(), class);
        nodes.insert(source.clone());
        edges.entry(source).or_default().push(DependencyEdge {
            target: head_key.clone(),
            negated,
        });
    }
}

fn default_body_class(filter: &BodyGraphFilter) -> HeadGraphClass {
    match filter {
        BodyGraphFilter::Any => HeadGraphClass::Inherit,
        BodyGraphFilter::Constant(c) => HeadGraphClass::Literal(c.clone()),
        BodyGraphFilter::Narrow => HeadGraphClass::Narrow,
    }
}
