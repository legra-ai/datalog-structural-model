//! The `stratify` entry point and the aggregate-edge helper it shares with the
//! dependency-graph builder.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use super::graph::{DependencyEdge, DependencyGraph, NodeKey, add_body_atom_edges};
use super::types::{HeadGraphClass, Stratification, StratificationError, Stratum};
use crate::ir::{AggregateBody, Literal, PredicateSymbol, Program};

/// Compute the stratification of a program.
///
/// # Errors
///
/// Returns [`StratificationError::UnstratifiableCycle`] when the
/// predicate dependency graph contains a cycle through negation.
pub(crate) fn stratify(program: &Program) -> Result<Stratification, StratificationError> {
    let graph = DependencyGraph::build(program);
    let sccs = graph.strongly_connected_components();

    // Map each node to its SCC index.
    let mut scc_of: HashMap<NodeKey, usize> = HashMap::new();
    for (idx, scc) in sccs.iter().enumerate() {
        for node in scc {
            scc_of.insert(node.clone(), idx);
        }
    }

    // Reject any SCC that contains an internal negation edge.
    for (idx, scc) in sccs.iter().enumerate() {
        for src in scc {
            let Some(edges) = graph.edges.get(src) else {
                continue;
            };
            for edge in edges {
                if edge.negated && scc_of.get(&edge.target) == Some(&idx) && scc.len() > 1 {
                    let mut predicates: Vec<_> = scc.iter().map(|(p, _)| p.clone()).collect();
                    predicates.sort();
                    predicates.dedup();
                    return Err(StratificationError::UnstratifiableCycle { predicates });
                }
                // Self-loop through negation — e.g. p :- q, not p.
                if edge.negated && edge.target == *src {
                    return Err(StratificationError::UnstratifiableCycle {
                        predicates: vec![src.0.clone()],
                    });
                }
            }
        }
    }

    // Assign stratum indices.  For each SCC (processed in
    // topological order — SCCs with no incoming edges first), the
    // stratum is 1 + max(predecessor stratum + is_negation).
    //
    // Tarjan's SCC algorithm returns components in reverse
    // topological order (sinks before sources), so we iterate in
    // reverse to process sources first.
    let mut scc_stratum: Vec<usize> = vec![0; sccs.len()];
    for idx in (0..sccs.len()).rev() {
        // Collect predecessor constraints by scanning every edge
        // that enters this SCC from a different SCC.
        let mut stratum = 0usize;
        for (from_node, edges) in &graph.edges {
            let Some(&from_scc) = scc_of.get(from_node) else {
                continue;
            };
            if from_scc == idx {
                continue;
            }
            for edge in edges {
                let Some(&target_scc) = scc_of.get(&edge.target) else {
                    continue;
                };
                if target_scc != idx {
                    continue;
                }
                // Edge from `from_node` (in SCC `from_scc`) into
                // this SCC (`idx`), tagged `edge.negated`.
                let pred_stratum = scc_stratum[from_scc];
                let candidate = if edge.negated {
                    pred_stratum + 1
                } else {
                    pred_stratum
                };
                if candidate > stratum {
                    stratum = candidate;
                }
            }
        }
        scc_stratum[idx] = stratum;
    }

    // Build the output: one Stratum per distinct index.
    // context-aware rolls up node strata into per-predicate strata by
    // taking the MAX stratum across all head-graph classes of a
    // predicate — a producer that emits facts under more than
    // one class must be scheduled late enough for every
    // downstream consumer.
    let max_stratum = scc_stratum.iter().copied().max().unwrap_or(0);
    let mut strata: Vec<Stratum> = (0..=max_stratum).map(|_| Stratum::default()).collect();
    let mut predicate_stratum: BTreeMap<PredicateSymbol, usize> = BTreeMap::new();
    for (scc_idx, scc) in sccs.iter().enumerate() {
        let stratum = scc_stratum[scc_idx];
        for (predicate, _class) in scc {
            let entry = predicate_stratum
                .entry(predicate.clone())
                .or_insert(stratum);
            if *entry < stratum {
                *entry = stratum;
            }
        }
    }
    for (predicate, stratum) in &predicate_stratum {
        strata[*stratum].predicates.insert(predicate.clone());
    }

    // Drop trailing empty strata (shouldn't happen, but be
    // defensive).
    while strata
        .last()
        .map(|s| s.predicates.is_empty())
        .unwrap_or(false)
    {
        strata.pop();
    }

    Ok(Stratification {
        strata,
        predicate_stratum,
    })
}

/// Add the negation-like dependency edges contributed by a single
/// [`Literal::Aggregate`] body literal. An aggregate folds the
/// *complete* extension of its inner predicates into a per-group
/// result, so each inner predicate — referenced by either a
/// positive or a negative inner-body literal — gets a
/// `negated: true` edge to the rule's head. That matches the
/// stratification treatment classical negation gets. Programs
/// with a cycle that runs through an aggregate are rejected the
/// same way cycles through negation are.
///
/// Inner [`Literal::Filter`] and [`Literal::Bind`] do not
/// introduce predicate dependencies — they reference only
/// already-bound variables and never name a predicate. Nested
/// [`Literal::Aggregate`] is rejected upstream by
/// [`Rule::has_nested_aggregate_in_inner_body`](crate::ir::Rule::has_nested_aggregate_in_inner_body)
/// so we never see one here.
pub(super) fn add_aggregate_edges(
    agg: &AggregateBody,
    head_key: &NodeKey,
    head_nodes: &BTreeMap<PredicateSymbol, BTreeSet<HeadGraphClass>>,
    nodes: &mut BTreeSet<NodeKey>,
    edges: &mut BTreeMap<NodeKey, Vec<DependencyEdge>>,
) {
    for lit in &agg.inner_body {
        let inner_atom = match lit {
            Literal::Positive(atom) | Literal::Negative(atom) => atom,
            Literal::Filter(_) | Literal::Bind { .. } | Literal::Aggregate(_) => continue,
        };
        add_body_atom_edges(inner_atom, head_key, true, head_nodes, nodes, edges);
    }
}
