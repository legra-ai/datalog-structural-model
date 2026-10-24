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

struct TraversalFrame {
    node: NodeKey,
    next_edge: usize,
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
        self.enter(v);
        // bounded: at most one frame per node in the finite rule program.
        let mut frames = vec![TraversalFrame {
            node: v.clone(),
            next_edge: 0,
        }];
        while let Some(frame) = frames.last_mut() {
            let edge = graph
                .edges
                .get(&frame.node)
                .and_then(|edges| edges.get(frame.next_edge));
            if let Some(edge) = edge {
                frame.next_edge += 1;
                let target = &edge.target;
                if !self.indices.contains_key(target) {
                    self.enter(target);
                    frames.push(TraversalFrame {
                        node: target.clone(),
                        next_edge: 0,
                    });
                } else if *self.on_stack.get(target).unwrap_or(&false) {
                    let low = self.lowlink[&frame.node].min(self.indices[target]);
                    self.lowlink.insert(frame.node.clone(), low);
                }
                continue;
            }
            let completed = frames.pop().expect("active traversal frame").node;
            self.finish(&completed);
            if let Some(parent) = frames.last() {
                let low = self.lowlink[&parent.node].min(self.lowlink[&completed]);
                self.lowlink.insert(parent.node.clone(), low);
            }
        }
    }

    fn enter(&mut self, v: &NodeKey) {
        self.indices.insert(v.clone(), self.index);
        self.lowlink.insert(v.clone(), self.index);
        self.index += 1;
        self.stack.push(v.clone());
        self.on_stack.insert(v.clone(), true);
    }

    fn finish(&mut self, v: &NodeKey) {
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
