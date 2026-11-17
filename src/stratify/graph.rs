//! Producer-to-consumer dependencies over complete tuple patterns.

use super::cycle::StrictDependencyKind;
use super::pattern::overlaps;
use super::tarjan::TarjanState;
use crate::ir::{Atom, Literal, PredicateSymbol, Program};
use crate::position::{LiteralIndex, LiteralPath};
use std::collections::{BTreeMap, BTreeSet};

pub(super) type NodeKey = usize;

#[derive(Debug)]
pub(super) struct DependencyGraph {
    pub(super) edges: BTreeMap<NodeKey, Vec<DependencyEdge>>,
    pub(super) nodes: BTreeSet<NodeKey>,
}

#[derive(Clone, Debug)]
pub(super) struct DependencyEdge {
    pub(super) target: NodeKey,
    pub(super) strict: Option<StrictDependencyKind>,
    pub(super) literal: LiteralPath,
}

impl DependencyEdge {
    pub(super) fn is_strict(&self) -> bool {
        self.strict.is_some()
    }
}

impl DependencyGraph {
    pub(super) fn build(program: &Program) -> Self {
        // bounded: one entry per head in the finite rule program.
        let mut producers: BTreeMap<&PredicateSymbol, Vec<(usize, &Atom)>> = BTreeMap::new();
        for (index, rule) in program.rules.iter().enumerate() {
            producers
                .entry(&rule.head.predicate)
                .or_default()
                .push((index, &rule.head));
        }
        let mut graph = Self {
            edges: BTreeMap::new(),
            nodes: (0..program.rules.len()).collect(),
        };
        for (consumer, rule) in program.rules.iter().enumerate() {
            graph.add_literals(&rule.body, None, consumer, None, &producers);
        }
        graph
    }

    fn add_literals(
        &mut self,
        body: &[Literal],
        parent: Option<&LiteralPath>,
        consumer: usize,
        strict: Option<StrictDependencyKind>,
        producers: &BTreeMap<&PredicateSymbol, Vec<(usize, &Atom)>>,
    ) {
        for (index, literal) in body.iter().enumerate() {
            let index = LiteralIndex::new(index);
            let path = parent.map_or_else(|| LiteralPath::top(index), |path| path.inner(index));
            let (atom, kind) = match literal {
                Literal::Positive(atom) => (atom, strict),
                Literal::Negative(atom) => (atom, Some(StrictDependencyKind::Negation)),
                Literal::Aggregate(aggregate) => {
                    self.add_literals(
                        &aggregate.inner_body,
                        Some(&path),
                        consumer,
                        Some(StrictDependencyKind::Aggregation),
                        producers,
                    );
                    continue;
                }
                Literal::Filter(_) | Literal::Bind { .. } => continue,
            };
            for &(producer, head) in producers.get(&atom.predicate).into_iter().flatten() {
                if overlaps(head, atom) {
                    self.edges
                        .entry(producer)
                        .or_default()
                        .push(DependencyEdge {
                            target: consumer,
                            strict: kind,
                            literal: path.clone(),
                        });
                }
            }
        }
    }

    /// Iterative Tarjan traversal, with sinks before sources.
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
