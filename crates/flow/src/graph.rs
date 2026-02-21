use barqflow_core::schema::{INode, INodeConnections, INodeParameters, IWorkflowSettings};
use barqflow_core::types::NodeId;
use petgraph::algo::{is_cyclic_directed, toposort};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowNode {
    pub id: NodeId,
    pub name: String,
    pub parameters: INodeParameters,
    pub type_: String,
    pub position: Option<(f64, f64)>,
    pub webhook_test: Option<String>,
    pub webhook_prod: Option<String>,
}

impl From<INode> for WorkflowNode {
    fn from(node: INode) -> Self {
        Self {
            id: node.id,
            name: node.name,
            parameters: node.parameters,
            type_: node.r#type,
            position: Some((node.position[0] as f64, node.position[1] as f64)),
            webhook_test: None,
            webhook_prod: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowDef {
    pub id: String,
    pub name: String,
    pub nodes: Vec<WorkflowNode>,
    pub connections: HashMap<String, INodeConnections>,
    pub settings: Option<IWorkflowSettings>,
    pub static_data: Option<serde_json::Value>,
    pub pin_data: Option<bool>,
    pub version_id: Option<String>,
}

impl Default for WorkflowDef {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            nodes: Vec::new(),
            connections: HashMap::new(),
            settings: Some(IWorkflowSettings::default()),
            static_data: None,
            pin_data: None,
            version_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub source: NodeIndex,
    pub target: NodeIndex,
    pub source_output_index: usize,
    pub target_input_index: usize,
}

#[derive(Debug)]
pub struct ParsedGraph {
    pub graph: DiGraph<WorkflowNode, GraphEdge>,
    pub node_indices: HashMap<NodeId, NodeIndex>,
    pub reverse_indices: HashMap<NodeIndex, NodeId>,
    pub name_to_index: HashMap<String, NodeIndex>,
}

pub struct WorkflowToGraphParser;

impl WorkflowToGraphParser {
    pub fn parse(workflow: &WorkflowDef) -> Result<ParsedGraph, String> {
        let mut graph = DiGraph::new();
        let mut node_indices = HashMap::new();
        let mut reverse_indices = HashMap::new();
        let mut name_to_index = HashMap::new();

        for node in &workflow.nodes {
            let index = graph.add_node(node.clone());
            node_indices.insert(node.id.clone(), index);
            reverse_indices.insert(index, node.id.clone());
            name_to_index.insert(node.name.clone(), index);
        }

        for (source_node_name, node_connections) in &workflow.connections {
            if let Some(&source_index) = name_to_index.get(source_node_name) {
                // node_connections is INodeConnections(HashMap<NodeConnectionType, Vec<Vec<IConnection>>>)
                for (_, output_arrays) in &node_connections.0 {
                    for (output_index, connections) in output_arrays.iter().enumerate() {
                        for connection in connections {
                            if let Some(&target_index) = name_to_index.get(&connection.node) {
                                let edge = GraphEdge {
                                    source: source_index,
                                    target: target_index,
                                    source_output_index: output_index,
                                    target_input_index: connection.index,
                                };
                                graph.add_edge(source_index, target_index, edge);
                            }
                        }
                    }
                }
            }
        }

        Ok(ParsedGraph {
            graph,
            node_indices,
            reverse_indices,
            name_to_index,
        })
    }

    pub fn get_node_index(&self, parsed: &ParsedGraph, node_id: &NodeId) -> Option<NodeIndex> {
        parsed.node_indices.get(node_id).copied()
    }
}

pub struct GraphTraversal;

impl GraphTraversal {
    pub fn is_executable_dag(graph: &DiGraph<WorkflowNode, GraphEdge>) -> bool {
        !is_cyclic_directed(graph)
    }

    pub fn topological_sort(
        graph: &DiGraph<WorkflowNode, GraphEdge>,
    ) -> Result<Vec<NodeIndex>, String> {
        toposort(graph, None).map_err(|e| format!("Cycle detected at node {:?}", e.node_id()))
    }

    pub fn get_trigger_nodes(graph: &DiGraph<WorkflowNode, GraphEdge>) -> Vec<NodeIndex> {
        graph
            .node_indices()
            .filter(|&idx| {
                let node = &graph[idx];
                node.type_.to_lowercase().contains("trigger")
                    || node.type_.to_lowercase().contains("manual")
            })
            .collect()
    }

    pub fn get_parents(
        graph: &DiGraph<WorkflowNode, GraphEdge>,
        node: NodeIndex,
    ) -> Vec<NodeIndex> {
        graph
            .edges(node)
            .filter(|e| e.target() == node)
            .map(|e| e.source())
            .collect()
    }

    pub fn get_children(
        graph: &DiGraph<WorkflowNode, GraphEdge>,
        node: NodeIndex,
    ) -> Vec<NodeIndex> {
        graph
            .edges(node)
            .filter(|e| e.source() == node)
            .map(|e| e.target())
            .collect()
    }

    pub fn get_ancestors(
        graph: &DiGraph<WorkflowNode, GraphEdge>,
        start: NodeIndex,
    ) -> HashSet<NodeIndex> {
        let mut ancestors = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(start);

        while let Some(current) = queue.pop_front() {
            for parent in Self::get_parents(graph, current) {
                if ancestors.insert(parent) {
                    queue.push_back(parent);
                }
            }
        }

        ancestors
    }

    pub fn get_descendants(
        graph: &DiGraph<WorkflowNode, GraphEdge>,
        start: NodeIndex,
    ) -> HashSet<NodeIndex> {
        let mut descendants = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(start);

        while let Some(current) = queue.pop_front() {
            for child in Self::get_children(graph, current) {
                if descendants.insert(child) {
                    queue.push_back(child);
                }
            }
        }

        descendants
    }

    pub fn find_all_paths(
        graph: &DiGraph<WorkflowNode, GraphEdge>,
        from: NodeIndex,
        to: NodeIndex,
    ) -> Vec<Vec<NodeIndex>> {
        let mut all_paths = Vec::new();
        let mut current_path = vec![from];
        let mut visited = HashSet::new();
        visited.insert(from);

        Self::dfs_find_paths(
            graph,
            from,
            to,
            &mut current_path,
            &mut visited,
            &mut all_paths,
        );

        all_paths
    }

    fn dfs_find_paths(
        graph: &DiGraph<WorkflowNode, GraphEdge>,
        current: NodeIndex,
        target: NodeIndex,
        path: &mut Vec<NodeIndex>,
        visited: &mut HashSet<NodeIndex>,
        all_paths: &mut Vec<Vec<NodeIndex>>,
    ) {
        if current == target {
            all_paths.push(path.clone());
            return;
        }

        for child in Self::get_children(graph, current) {
            if !visited.contains(&child) {
                visited.insert(child);
                path.push(child);
                Self::dfs_find_paths(graph, child, target, path, visited, all_paths);
                path.pop();
                visited.remove(&child);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_node(id: &str, name: &str, node_type: &str) -> WorkflowNode {
        WorkflowNode {
            id: NodeId::new(id),
            name: name.to_string(),
            parameters: INodeParameters::default(),
            type_: node_type.to_string(),
            position: Some((0.0, 0.0)),
            webhook_test: None,
            webhook_prod: None,
        }
    }

    #[test]
    fn test_simple_linear_workflow() {
        let workflow = WorkflowDef {
            id: "test-workflow-1".to_string(),
            name: "Linear Workflow".to_string(),
            nodes: vec![
                create_test_node("Start", "Start", "manualTrigger"),
                create_test_node("Process", "Process", "set"),
                create_test_node("End", "End", "noop"),
            ],
            connections: INodeConnections(HashMap::new()),
            settings: None,
            static_data: None,
            pin_data: None,
            version_id: None,
        };

        let parsed = WorkflowToGraphParser::parse(&workflow).unwrap();
        assert_eq!(parsed.graph.node_count(), 3);

        assert!(parsed.node_indices.contains_key(&NodeId::new("Start")));
        assert!(parsed.node_indices.contains_key(&NodeId::new("Process")));
        assert!(parsed.node_indices.contains_key(&NodeId::new("End")));
    }

    #[test]
    fn test_workflow_node_from_inode() {
        let inode = INode {
            id: NodeId::new("TestNode"),
            name: "Test Node".to_string(),
            r#type: "test".to_string(),
            type_version: 1.0,
            position: [100.0, 200.0],
            parameters: INodeParameters::default(),
            disabled: false,
        };

        let workflow_node: WorkflowNode = inode.into();
        assert_eq!(workflow_node.id, NodeId::new("TestNode"));
        assert_eq!(workflow_node.name, "Test Node");
        assert_eq!(workflow_node.type_, "test");
    }

    #[test]
    fn test_is_executable_dag_valid() {
        let workflow = WorkflowDef {
            id: "test-dag".to_string(),
            name: "Valid DAG".to_string(),
            nodes: vec![
                create_test_node("A", "A", "manualTrigger"),
                create_test_node("B", "B", "set"),
                create_test_node("C", "C", "set"),
            ],
            connections: INodeConnections(HashMap::new()),
            settings: None,
            static_data: None,
            pin_data: None,
            version_id: None,
        };

        let parsed = WorkflowToGraphParser::parse(&workflow).unwrap();
        assert!(GraphTraversal::is_executable_dag(&parsed.graph));
    }

    #[test]
    fn test_topological_sort() {
        let workflow = WorkflowDef {
            id: "test-topo".to_string(),
            name: "Topological Sort".to_string(),
            nodes: vec![
                create_test_node("Start", "Start", "manualTrigger"),
                create_test_node("Process", "Process", "set"),
                create_test_node("End", "End", "noop"),
            ],
            connections: INodeConnections(HashMap::new()),
            settings: None,
            static_data: None,
            pin_data: None,
            version_id: None,
        };

        let parsed = WorkflowToGraphParser::parse(&workflow).unwrap();
        let sorted = GraphTraversal::topological_sort(&parsed.graph).unwrap();
        assert_eq!(sorted.len(), 3);
    }

    #[test]
    fn test_get_trigger_nodes() {
        let workflow = WorkflowDef {
            id: "test-triggers".to_string(),
            name: "Triggers Test".to_string(),
            nodes: vec![
                create_test_node("Manual", "Manual", "manualTrigger"),
                create_test_node("Webhook", "Webhook", "webhookTrigger"),
                create_test_node("Process", "Process", "set"),
            ],
            connections: INodeConnections(HashMap::new()),
            settings: None,
            static_data: None,
            pin_data: None,
            version_id: None,
        };

        let parsed = WorkflowToGraphParser::parse(&workflow).unwrap();
        let triggers = GraphTraversal::get_trigger_nodes(&parsed.graph);
        assert_eq!(triggers.len(), 2);
    }

    #[test]
    fn test_get_parents_and_children() {
        let workflow = WorkflowDef {
            id: "test-parents".to_string(),
            name: "Parents Test".to_string(),
            nodes: vec![
                create_test_node("A", "A", "manualTrigger"),
                create_test_node("B", "B", "set"),
                create_test_node("C", "C", "set"),
            ],
            connections: INodeConnections(HashMap::new()),
            settings: None,
            static_data: None,
            pin_data: None,
            version_id: None,
        };

        let parsed = WorkflowToGraphParser::parse(&workflow).unwrap();

        if let Some(b_idx) = parsed.node_indices.get(&NodeId::new("B")) {
            let parents = GraphTraversal::get_parents(&parsed.graph, *b_idx);
            let children = GraphTraversal::get_children(&parsed.graph, *b_idx);
            println!("Parents of B: {:?}, Children of B: {:?}", parents, children);
        }
    }
}
