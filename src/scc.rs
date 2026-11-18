//! Iterative Tarjan strongly-connected components over a dense node range.
//!
//! Shared by rule stratification and the weak-acyclicity position graph. The
//! traversal uses a heap-backed frame stack, so graph depth never consumes
//! the native call stack.

use std::collections::BTreeSet;

/// A finite directed graph whose nodes are `0..node_count()`.
pub(crate) trait SuccessorGraph {
    /// Number of nodes.
    fn node_count(&self) -> usize;
    /// The `nth` successor of `node`, or `None` past the last one.
    fn successor(&self, node: usize, nth: usize) -> Option<usize>;

    /// Components with sinks before sources.
    fn strongly_connected_components(&self) -> Vec<BTreeSet<usize>>
    where
        Self: Sized,
    {
        let mut state = TarjanState::new(self.node_count());
        for node in 0..self.node_count() {
            if state.indices[node].is_none() {
                state.strong_connect(node, self);
            }
        }
        state.result
    }
}

struct TarjanState {
    index: usize,
    // bounded: one slot per node of the finite authored graph.
    indices: Vec<Option<usize>>,
    lowlink: Vec<usize>,
    on_stack: Vec<bool>,
    stack: Vec<usize>,
    result: Vec<BTreeSet<usize>>,
}

struct TraversalFrame {
    node: usize,
    next_edge: usize,
}

impl TarjanState {
    fn new(nodes: usize) -> Self {
        Self {
            index: 0,
            indices: vec![None; nodes],
            lowlink: vec![0; nodes],
            on_stack: vec![false; nodes],
            stack: Vec::new(),
            result: Vec::new(),
        }
    }

    fn strong_connect(&mut self, v: usize, graph: &impl SuccessorGraph) {
        self.enter(v);
        // bounded: at most one frame per node in the finite graph.
        let mut frames = vec![TraversalFrame {
            node: v,
            next_edge: 0,
        }];
        while let Some(frame) = frames.last_mut() {
            if let Some(target) = graph.successor(frame.node, frame.next_edge) {
                frame.next_edge += 1;
                match self.indices[target] {
                    None => {
                        self.enter(target);
                        frames.push(TraversalFrame {
                            node: target,
                            next_edge: 0,
                        });
                    }
                    Some(index) if self.on_stack[target] => {
                        self.lowlink[frame.node] = self.lowlink[frame.node].min(index);
                    }
                    Some(_) => {}
                }
                continue;
            }
            let completed = frames.pop().expect("active traversal frame").node;
            self.finish(completed);
            if let Some(parent) = frames.last() {
                self.lowlink[parent.node] = self.lowlink[parent.node].min(self.lowlink[completed]);
            }
        }
    }

    fn enter(&mut self, v: usize) {
        self.indices[v] = Some(self.index);
        self.lowlink[v] = self.index;
        self.index += 1;
        self.stack.push(v);
        self.on_stack[v] = true;
    }

    fn finish(&mut self, v: usize) {
        if Some(self.lowlink[v]) != self.indices[v] {
            return;
        }
        let mut scc = BTreeSet::new();
        loop {
            let w = self.stack.pop().expect("stack non-empty at SCC root");
            self.on_stack[w] = false;
            scc.insert(w);
            if w == v {
                break;
            }
        }
        self.result.push(scc);
    }
}
