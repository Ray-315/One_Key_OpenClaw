#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::recipe::schema::{OnErrorStrategy, RecipeStep, StepAction};
    use crate::task::graph::TaskGraph;

    fn shell_step(id: &str, deps: Vec<&str>) -> RecipeStep {
        RecipeStep {
            id: id.to_string(),
            name: id.to_string(),
            description: None,
            action: StepAction::Shell {
                command: "echo".to_string(),
                args: vec![id.to_string()],
                env: Default::default(),
            },
            depends_on: deps.into_iter().map(String::from).collect(),
            condition: None,
            retry: None,
            timeout_secs: None,
            on_error: OnErrorStrategy::Fail,
        }
    }

    #[test]
    fn build_linear_graph() {
        let steps = vec![
            shell_step("a", vec![]),
            shell_step("b", vec!["a"]),
            shell_step("c", vec!["b"]),
        ];
        let graph = TaskGraph::build(&steps).expect("should build");
        let order = graph.topological_order();
        assert_eq!(order, vec!["a", "b", "c"]);
    }

    #[test]
    fn build_parallel_graph() {
        let steps = vec![
            shell_step("a", vec![]),
            shell_step("b", vec![]),
            shell_step("c", vec!["a", "b"]),
        ];
        let graph = TaskGraph::build(&steps).expect("should build");
        let order = graph.topological_order();
        // c must come last
        assert_eq!(order.last().unwrap(), "c");
        // a and b come before c
        let pos_a = order.iter().position(|x| x == "a").unwrap();
        let pos_b = order.iter().position(|x| x == "b").unwrap();
        let pos_c = order.iter().position(|x| x == "c").unwrap();
        assert!(pos_a < pos_c);
        assert!(pos_b < pos_c);
    }

    #[test]
    fn build_rejects_cycle() {
        let steps = vec![
            shell_step("a", vec!["c"]),
            shell_step("b", vec!["a"]),
            shell_step("c", vec!["b"]),
        ];
        let result = TaskGraph::build(&steps);
        assert!(result.is_err());
        match result {
            Err(e) => {
                let msg = e.to_string();
                assert!(msg.contains("circular") || msg.contains("cycle"));
            }
            Ok(_) => panic!("expected error"),
        }
    }

    #[test]
    fn build_rejects_unknown_dep() {
        let steps = vec![shell_step("a", vec!["nonexistent"])];
        let result = TaskGraph::build(&steps);
        assert!(result.is_err());
        match result {
            Err(e) => {
                let msg = e.to_string();
                assert!(msg.contains("unknown step"));
            }
            Ok(_) => panic!("expected error"),
        }
    }

    #[test]
    fn get_ready_steps_returns_roots() {
        let steps = vec![
            shell_step("a", vec![]),
            shell_step("b", vec![]),
            shell_step("c", vec!["a", "b"]),
        ];
        let graph = TaskGraph::build(&steps).unwrap();
        let completed: HashSet<String> = HashSet::new();
        let running: HashSet<String> = HashSet::new();
        let ready = graph.get_ready_steps(&completed, &running);
        assert!(ready.contains(&"a".to_string()));
        assert!(ready.contains(&"b".to_string()));
        assert!(!ready.contains(&"c".to_string()));
    }

    #[test]
    fn get_ready_steps_after_partial_completion() {
        let steps = vec![
            shell_step("a", vec![]),
            shell_step("b", vec![]),
            shell_step("c", vec!["a", "b"]),
        ];
        let graph = TaskGraph::build(&steps).unwrap();
        let completed: HashSet<String> = ["a".to_string()].into();
        let running: HashSet<String> = HashSet::new();
        let ready = graph.get_ready_steps(&completed, &running);
        // b is ready, c still blocked on b
        assert!(ready.contains(&"b".to_string()));
        assert!(!ready.contains(&"c".to_string()));
    }

    #[test]
    fn get_ready_steps_c_unlocked_when_all_deps_done() {
        let steps = vec![
            shell_step("a", vec![]),
            shell_step("b", vec![]),
            shell_step("c", vec!["a", "b"]),
        ];
        let graph = TaskGraph::build(&steps).unwrap();
        let completed: HashSet<String> = ["a".to_string(), "b".to_string()].into();
        let running: HashSet<String> = HashSet::new();
        let ready = graph.get_ready_steps(&completed, &running);
        assert!(ready.contains(&"c".to_string()));
    }

    #[test]
    fn to_graph_data_includes_all_nodes_and_edges() {
        let steps = vec![
            shell_step("a", vec![]),
            shell_step("b", vec!["a"]),
        ];
        let graph = TaskGraph::build(&steps).unwrap();
        let data = graph.to_graph_data(&steps);
        assert_eq!(data.nodes.len(), 2);
        assert_eq!(data.edges.len(), 1);
        assert_eq!(data.edges[0].source, "a");
        assert_eq!(data.edges[0].target, "b");
    }

    #[test]
    fn to_graph_data_assigns_layers() {
        let steps = vec![
            shell_step("a", vec![]),
            shell_step("b", vec!["a"]),
            shell_step("c", vec!["b"]),
        ];
        let graph = TaskGraph::build(&steps).unwrap();
        let data = graph.to_graph_data(&steps);
        let node_a = data.nodes.iter().find(|n| n.id == "a").unwrap();
        let node_b = data.nodes.iter().find(|n| n.id == "b").unwrap();
        let node_c = data.nodes.iter().find(|n| n.id == "c").unwrap();
        assert_eq!(node_a.layer, 0);
        assert_eq!(node_b.layer, 1);
        assert_eq!(node_c.layer, 2);
    }
}
