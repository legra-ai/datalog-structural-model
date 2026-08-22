//! Tarjan's strongly-connected-components state machine.

use std::collections::{BTreeSet, HashMap};

use super::graph::{DependencyGraph, NodeKey};

pub(super) struct TarjanState {
    pub(super) index: usize,
    pub(super) indices: HashMap<NodeKey, usize>,
    pub(super) lowlink: HashMap<NodeKey, usize>,
    pub(super) on_stack: HashMap<NodeKey, bool>,
    pub(super) stack: Vec<NodeKey>,
    pub(super) result: Vec<BTreeSet<NodeKey>>,
}

impl TarjanState {
    pub(super) fn new() -> Self {
        Self {
            index: 0,
            indices: HashMap::new(),
            lowlink: HashMap::new(),
            on_stack: HashMap::new(),
            stack: Vec::new(),
            result: Vec::new(),
        }
    }

    pub(super) fn strong_connect(&mut self, v: &NodeKey, graph: &DependencyGraph) {
        self.indices.insert(v.clone(), self.index);
        self.lowlink.insert(v.clone(), self.index);
        self.index += 1;
        self.stack.push(v.clone());
        self.on_stack.insert(v.clone(), true);

        if let Some(edges) = graph.edges.get(v) {
            for edge in edges.clone() {
                let w = &edge.target;
                if !self.indices.contains_key(w) {
                    self.strong_connect(w, graph);
                    let w_low = self.lowlink[w];
                    let v_low = self.lowlink[v];
                    self.lowlink.insert(v.clone(), v_low.min(w_low));
                } else if *self.on_stack.get(w).unwrap_or(&false) {
                    let w_idx = self.indices[w];
                    let v_low = self.lowlink[v];
                    self.lowlink.insert(v.clone(), v_low.min(w_idx));
                }
            }
        }

        // Root of an SCC?
        if self.lowlink[v] == self.indices[v] {
            let mut scc = BTreeSet::new();
            loop {
                let w = self.stack.pop().expect("stack non-empty at SCC root");
                self.on_stack.insert(w.clone(), false);
                let is_v = w == *v;
                scc.insert(w);
                if is_v {
                    break;
                }
            }
            self.result.push(scc);
        }
    }
}
