use std::collections::{HashMap, HashSet};

use petgraph::algo::{is_cyclic_directed, toposort};
use petgraph::graph::{DiGraph, NodeIndex};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::recipe::schema::RecipeStep;

/// Directed Acyclic Graph representation of recipe steps.
pub struct TaskGraph {
    graph: DiGraph<String, ()>,
    node_map: HashMap<String, NodeIndex>,
}

/// A node in the task graph, used for frontend visualisation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskGraphNode {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    /// Zero-based layer index (for layout hints).
    pub layer: usize,
    pub depends_on: Vec<String>,
}

/// Graph data sent to the frontend for React Flow rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskGraphData {
    pub nodes: Vec<TaskGraphNode>,
    pub edges: Vec<TaskGraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskGraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
}

impl TaskGraph {
    /// Build a DAG from a list of recipe steps. Returns an error if:
    /// - a `depends_on` reference points to an unknown step ID, or
    /// - the resulting graph contains a cycle.
    pub fn build(steps: &[RecipeStep]) -> Result<Self, AppError> {
        let mut graph: DiGraph<String, ()> = DiGraph::new();
        let mut node_map: HashMap<String, NodeIndex> = HashMap::new();

        // Add all nodes first so edges can reference them.
        for step in steps {
            let idx = graph.add_node(step.id.clone());
            node_map.insert(step.id.clone(), idx);
        }

        // Add directed edges: dep → step (dep must complete before step runs).
        for step in steps {
            let to = node_map[&step.id];
            for dep_id in &step.depends_on {
                let from = node_map
                    .get(dep_id)
                    .ok_or_else(|| AppError::RecipeParseError {
                        path: String::new(),
                        message: format!("step '{}' depends on unknown step '{}'", step.id, dep_id),
                    })?;
                graph.add_edge(*from, to, ());
            }
        }

        if is_cyclic_directed(&graph) {
            return Err(AppError::RecipeParseError {
                path: String::new(),
                message: "circular dependency detected in step graph".into(),
            });
        }

        Ok(Self { graph, node_map })
    }

    /// Return step IDs in a valid topological order.
    pub fn topological_order(&self) -> Vec<String> {
        toposort(&self.graph, None)
            .expect("task graph should remain acyclic after successful build")
            .into_iter()
            .map(|idx| self.graph[idx].clone())
            .collect()
    }

    /// Return the set of step IDs that are ready to execute:
    /// all their dependencies are in `completed` and they are not
    /// already in `running` or `completed`.
    pub fn get_ready_steps(
        &self,
        completed: &HashSet<String>,
        running: &HashSet<String>,
    ) -> Vec<String> {
        let mut ready = Vec::new();

        for (id, idx) in &self.node_map {
            if completed.contains(id) || running.contains(id) {
                continue;
            }
            // Check that all predecessors (dependencies) are completed.
            let all_deps_done = self
                .graph
                .neighbors_directed(*idx, petgraph::Direction::Incoming)
                .all(|dep_idx| completed.contains(&self.graph[dep_idx]));

            if all_deps_done {
                ready.push(id.clone());
            }
        }

        // Deterministic ordering for reproducibility.
        ready.sort();
        ready
    }

    /// Build the graph data payload for frontend visualisation.
    /// Assigns each node a `layer` based on its position in the longest
    /// dependency chain (critical-path layer).
    pub fn to_graph_data(&self, steps: &[RecipeStep]) -> TaskGraphData {
        let topo = self.topological_order();
        let mut layer_map: HashMap<String, usize> = HashMap::new();

        // Assign layers: a node's layer is 1 + max(layer of predecessors), or 0
        // if it has no predecessors.
        for id in &topo {
            let idx = self.node_map[id];
            let max_pred_layer = self
                .graph
                .neighbors_directed(idx, petgraph::Direction::Incoming)
                .filter_map(|pred| layer_map.get(&self.graph[pred]).copied())
                .max();
            layer_map.insert(id.clone(), max_pred_layer.map(|l| l + 1).unwrap_or(0));
        }

        let step_map: HashMap<&str, &RecipeStep> =
            steps.iter().map(|s| (s.id.as_str(), s)).collect();

        let nodes: Vec<TaskGraphNode> = topo
            .iter()
            .filter_map(|id| {
                let rs = step_map.get(id.as_str())?;
                Some(TaskGraphNode {
                    id: id.clone(),
                    name: rs.name.clone(),
                    description: rs.description.clone(),
                    layer: layer_map[id],
                    depends_on: rs.depends_on.clone(),
                })
            })
            .collect();

        let mut edges: Vec<TaskGraphEdge> = Vec::new();
        for step in steps {
            for dep in &step.depends_on {
                edges.push(TaskGraphEdge {
                    id: format!("{dep}->{}", step.id),
                    source: dep.clone(),
                    target: step.id.clone(),
                });
            }
        }

        TaskGraphData { nodes, edges }
    }
}
