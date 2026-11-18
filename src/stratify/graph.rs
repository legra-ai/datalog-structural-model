//! Producer-to-consumer dependencies over complete tuple patterns.

use super::cycle::StrictDependencyKind;
use super::pattern::overlaps;
use crate::ir::{Atom, Literal, PredicateSymbol, Program};
use crate::position::{LiteralIndex, LiteralPath};
use crate::scc::SuccessorGraph;
use std::collections::BTreeMap;

pub(super) type NodeKey = usize;

#[derive(Debug)]
pub(super) struct DependencyGraph {
    pub(super) edges: BTreeMap<NodeKey, Vec<DependencyEdge>>,
    pub(super) nodes: usize,
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
        // bounded: one entry per head atom in the finite rule program. Every
        // atom of a conjunctive, existential or disjunctive head is a producer;
        // a falsum head produces nothing.
        let mut producers: BTreeMap<&PredicateSymbol, Vec<(usize, &Atom)>> = BTreeMap::new();
        for (index, rule) in program.rules.iter().enumerate() {
            for (_, head) in rule.head.atoms() {
                producers
                    .entry(&head.predicate)
                    .or_default()
                    .push((index, head));
            }
        }
        let mut graph = Self {
            edges: BTreeMap::new(),
            nodes: program.rules.len(),
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
}

impl SuccessorGraph for DependencyGraph {
    fn node_count(&self) -> usize {
        self.nodes
    }

    fn successor(&self, node: usize, nth: usize) -> Option<usize> {
        self.edges.get(&node)?.get(nth).map(|edge| edge.target)
    }
}
