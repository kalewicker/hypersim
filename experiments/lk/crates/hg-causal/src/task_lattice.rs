//! Task lattice: DAG of (input attribute -> output attribute) transitions.

use crate::{Attribute, CertaintyClass, Task, TaskStatus};
use petgraph::graph::NodeIndex;
use petgraph::Graph;

#[derive(Default)]
pub struct TaskLattice {
    graph: Graph<Task, ()>,
}

impl TaskLattice {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
        }
    }

    /// Find the task (input -> output) or create it; bumps witness count
    /// and updates status/certainty per docs/metric_formulas.md.
    pub fn witness(&mut self, input: &Attribute, output: &Attribute, step: u64) -> NodeIndex {
        for ix in self.graph.node_indices() {
            let t = &mut self.graph[ix];
            if t.input.wl_hash == input.wl_hash && t.output.wl_hash == output.wl_hash {
                t.witness_count += 1;
                t.last_seen = step;
                if t.witness_count >= 3 {
                    t.status = TaskStatus::Confirmed;
                    t.certainty = CertaintyClass::Empirical;
                }
                return ix;
            }
        }
        let id = self.graph.node_count() as u64;
        let ix = self.graph.add_node(Task {
            id,
            input: input.clone(),
            output: output.clone(),
            status: TaskStatus::Candidate,
            certainty: CertaintyClass::Intuition,
            witness_count: 1,
            first_seen: step,
            last_seen: step,
        });
        // Dependency edges: task B depends on task A if A's output class
        // is B's input class.
        let new_in = self.graph[ix].input.wl_hash;
        let new_out = self.graph[ix].output.wl_hash;
        let others: Vec<NodeIndex> = self.graph.node_indices().filter(|i| *i != ix).collect();
        for other in others {
            if self.graph[other].output.wl_hash == new_in {
                self.graph.add_edge(other, ix, ());
            }
            if self.graph[other].input.wl_hash == new_out {
                self.graph.add_edge(ix, other, ());
            }
        }
        ix
    }

    pub fn task_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn tasks(&self) -> impl Iterator<Item = &Task> {
        self.graph.node_weights()
    }

    /// Longest path length in tasks (docs/metric_formulas.md); 0 if empty
    /// or cyclic.
    pub fn depth(&self) -> usize {
        let Ok(order) = petgraph::algo::toposort(&self.graph, None) else {
            return if self.graph.node_count() > 0 { 1 } else { 0 };
        };
        if order.is_empty() {
            return 0;
        }
        let mut dist: Vec<usize> = vec![1; self.graph.node_count()];
        for &u in &order {
            let du = dist[u.index()];
            for v in self.graph.neighbors(u) {
                if du + 1 > dist[v.index()] {
                    dist[v.index()] = du + 1;
                }
            }
        }
        dist.into_iter().max().unwrap_or(0)
    }
}
