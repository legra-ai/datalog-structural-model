//! Producer-to-consumer dependencies over complete tuple patterns.

use super::pattern::overlaps;
use super::tarjan::TarjanState;
use crate::ir::{Atom, Literal, PredicateSymbol, Program};
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
    pub(super) negated: bool,
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
            graph.add_literals(&rule.body, consumer, false, &producers);
        }
        graph
    }

    fn add_literals(
        &mut self,
        body: &[Literal],
        consumer: usize,
        strict: bool,
        producers: &BTreeMap<&PredicateSymbol, Vec<(usize, &Atom)>>,
    ) {
        for literal in body {
            let (atom, negated) = match literal {
                Literal::Positive(atom) => (atom, strict),
                Literal::Negative(atom) => (atom, true),
                Literal::Aggregate(aggregate) => {
                    self.add_literals(&aggregate.inner_body, consumer, true, producers);
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
                            negated,
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
