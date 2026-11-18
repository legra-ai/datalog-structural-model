//! The position graph and [`Program::check_weak_acyclicity`].

use std::collections::{BTreeMap, BTreeSet};

use super::criterion::TerminationCriterion;
use super::edge::{ArgumentPosition, InventionKind, SpecialEdge};
use super::sources::VariableSources;
use super::verdict::WeakAcyclicity;
use super::violation::TerminationViolation;
use crate::ir::{Atom, Program, Term};
use crate::position::{AtomSlot, RuleIndex, RuleSite};
use crate::scc::SuccessorGraph;
use crate::stratify::overlaps;

/// Nodes are predicate positions; an edge carries a value from a body
/// position to a head position, specially when the head invents a value.
#[derive(Debug, Default)]
struct PositionGraph {
    // bounded: one node per distinct position of the finite program.
    ids: BTreeMap<ArgumentPosition, usize>,
    positions: Vec<ArgumentPosition>,
    // bounded: edges are pairs of authored occurrences, never facts.
    successors: Vec<Vec<(usize, bool)>>,
    special: Vec<SpecialEdge>,
}

impl PositionGraph {
    fn build(program: &Program) -> Self {
        let mut graph = Self::default();
        // bounded: one scope per rule of the finite program.
        let scopes: Vec<_> = program
            .rules
            .iter()
            .enumerate()
            .map(|(index, rule)| VariableSources::of_body(RuleIndex::new(index), &rule.body))
            .collect();
        for (index, (rule, sources)) in program.rules.iter().zip(&scopes).enumerate() {
            graph.add_head(RuleIndex::new(index), &rule.head, sources);
        }
        for (producer, rule) in program.rules.iter().enumerate() {
            graph.add_flows(RuleIndex::new(producer), &rule.head, &scopes);
        }
        graph
    }

    fn node(&mut self, position: &ArgumentPosition) -> usize {
        if let Some(&id) = self.ids.get(position) {
            return id;
        }
        let id = self.positions.len();
        self.ids.insert(position.clone(), id);
        self.positions.push(position.clone());
        self.successors.push(Vec::new());
        id
    }

    fn head_position(rule: RuleIndex, head: &Atom, slot: usize) -> ArgumentPosition {
        ArgumentPosition::new(
            head.predicate.clone(),
            AtomSlot::new(slot),
            rule,
            RuleSite::Head,
        )
    }

    /// Edges inside one rule, from the body positions binding each head
    /// variable to the head slot holding it.
    fn add_head(&mut self, rule: RuleIndex, head: &Atom, sources: &VariableSources<'_>) {
        for (slot, term) in head.terms.iter().enumerate() {
            let to = Self::head_position(rule, head, slot);
            let target = self.node(&to);
            let wrapper = match term {
                Term::Variable(_) | Term::Constant(_) => None,
                Term::TripleTerm(_) => Some(InventionKind::TripleTerm),
                Term::Skolem { .. } => Some(InventionKind::Skolem),
            };
            term.visit_variables(&mut |variable| {
                for source in sources.of(variable) {
                    let invention = wrapper.or(source.invention);
                    let from = self.node(&source.position);
                    self.successors[from].push((target, invention.is_some()));
                    if let Some(kind) = invention {
                        self.special.push(SpecialEdge {
                            from: source.position.clone(),
                            to: to.clone(),
                            kind,
                        });
                    }
                }
            });
        }
    }

    /// Ordinary edges from a head slot to the same slot of every positive
    /// body atom, in any rule, whose pattern can match the head's facts.
    fn add_flows(&mut self, producer: RuleIndex, head: &Atom, scopes: &[VariableSources<'_>]) {
        for (consumer, scope) in scopes.iter().enumerate() {
            for body in scope.atoms.iter().filter(|body| overlaps(head, body.atom)) {
                for slot in 0..head.arity() {
                    let from = self.node(&Self::head_position(producer, head, slot));
                    let to = self.node(&body.position(RuleIndex::new(consumer), slot));
                    self.successors[from].push((to, false));
                }
            }
        }
    }

    fn component_of(&self, sccs: &[BTreeSet<usize>]) -> Vec<usize> {
        // bounded: one component id per position of the finite program.
        let mut component = vec![0; self.positions.len()];
        for (index, members) in sccs.iter().enumerate() {
            for &node in members {
                component[node] = index;
            }
        }
        component
    }

    fn violation(
        &self,
        sccs: &[BTreeSet<usize>],
        component: &[usize],
    ) -> Option<TerminationViolation> {
        let edge = self
            .special
            .iter()
            .filter(|edge| component[self.ids[&edge.from]] == component[self.ids[&edge.to]])
            .min()?;
        let members = &sccs[component[self.ids[&edge.from]]];
        let positions = members
            .iter()
            .map(|&id| self.positions[id].clone())
            .collect();
        Some(TerminationViolation::new(
            TerminationCriterion::WeaklyAcyclic,
            edge.clone(),
            positions,
        ))
    }

    /// Longest special-edge count over the acyclic condensation. Tarjan
    /// emits sinks first, so walking components in reverse settles every
    /// predecessor before its successors.
    fn certificate(&self, sccs: &[BTreeSet<usize>], component: &[usize]) -> WeakAcyclicity {
        let mut rank = vec![0_usize; sccs.len()];
        for index in (0..sccs.len()).rev() {
            for &node in &sccs[index] {
                for &(target, special) in &self.successors[node] {
                    let target = component[target];
                    if target != index {
                        rank[target] = rank[target].max(rank[index] + usize::from(special));
                    }
                }
            }
        }
        let skolem_depth = rank.iter().copied().max().unwrap_or(0);
        let deepest = (skolem_depth > 0)
            .then(|| {
                self.ids
                    .iter()
                    .find(|&(_, &id)| rank[component[id]] == skolem_depth)
                    .map(|(position, _)| position.clone())
            })
            .flatten();
        WeakAcyclicity {
            positions: self.positions.len(),
            special_edges: self.special.len(),
            skolem_depth,
            deepest,
        }
    }
}

impl SuccessorGraph for PositionGraph {
    fn node_count(&self) -> usize {
        self.positions.len()
    }

    fn successor(&self, node: usize, nth: usize) -> Option<usize> {
        self.successors[node].get(nth).map(|&(target, _)| target)
    }
}

impl Program {
    /// Check [`TerminationCriterion::WeaklyAcyclic`] on the position graph.
    ///
    /// Nodes are argument slots of atom occurrences. Inside a rule, a head
    /// variable copied directly from a body slot adds an ordinary edge; a
    /// head value built by a Skolem term, a triple term, a `BIND` or an
    /// aggregate adds a special edge from each body slot feeding it. Across
    /// rules, a head slot feeds the same slot of every positive body atom
    /// whose complete pattern can match the head, so constants that keep two
    /// patterns apart break the cycle. The program is weakly acyclic when no
    /// special edge lies on a cycle, which bounds the nesting of invented
    /// values and so the number of derivable facts.
    ///
    /// The analysis reads only the program: its time and memory are bounded
    /// by program size, never by data. A program without value invention is
    /// always weakly acyclic.
    ///
    /// # Errors
    ///
    /// Returns the first cyclic special edge in canonical order, with the
    /// positions of its component.
    pub fn check_weak_acyclicity(&self) -> Result<WeakAcyclicity, TerminationViolation> {
        let graph = PositionGraph::build(self);
        let sccs = graph.strongly_connected_components();
        let component = graph.component_of(&sccs);
        match graph.violation(&sccs, &component) {
            Some(violation) => Err(violation),
            None => Ok(graph.certificate(&sccs, &component)),
        }
    }
}
