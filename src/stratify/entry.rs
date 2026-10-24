//! Rule stratification entry point.

use super::graph::DependencyGraph;
use super::types::{Stratification, StratificationError, Stratum};
use crate::ir::Program;

pub(crate) fn stratify(program: &Program) -> Result<Stratification, StratificationError> {
    let graph = DependencyGraph::build(program);
    let sccs = graph.strongly_connected_components();
    // bounded: one component assignment per rule in the finite program.
    let mut scc_of = vec![0; program.rules.len()];
    for (index, component) in sccs.iter().enumerate() {
        for &rule in component {
            scc_of[rule] = index;
        }
    }
    for (&source, edges) in &graph.edges {
        for edge in edges {
            if edge.negated && scc_of[source] == scc_of[edge.target] {
                let mut predicates: Vec<_> = sccs[scc_of[source]]
                    .iter()
                    .map(|&index| program.rules[index].head.predicate.clone())
                    .collect();
                predicates.sort();
                predicates.dedup();
                return Err(StratificationError::UnstratifiableCycle { predicates });
            }
        }
    }
    // Tarjan emits sink components first. Propagate each outgoing constraint
    // once, in source-first order, after all predecessors have settled.
    let mut component_stratum = vec![0; sccs.len()];
    for index in (0..sccs.len()).rev() {
        for source in &sccs[index] {
            for edge in graph.edges.get(source).into_iter().flatten() {
                let target = scc_of[edge.target];
                if target != index {
                    component_stratum[target] = component_stratum[target]
                        .max(component_stratum[index] + usize::from(edge.negated));
                }
            }
        }
    }
    let mut rule_stratum = vec![0; program.rules.len()];
    let mut strata = if sccs.is_empty() {
        Vec::new()
    } else {
        vec![Stratum::default(); component_stratum.iter().copied().max().unwrap_or(0) + 1]
    };
    for (component, rules) in sccs.iter().enumerate() {
        let stratum = component_stratum[component];
        for &rule in rules {
            rule_stratum[rule] = stratum;
            strata[stratum].rules.insert(rule);
        }
    }
    Ok(Stratification {
        strata,
        rule_stratum,
    })
}
