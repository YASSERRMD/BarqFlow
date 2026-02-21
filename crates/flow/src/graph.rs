use barqflow_core::schema::{INode, INodeConnections, INodeParameters, IWorkflowSettings};
use barqflow_core::types::NodeId;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub connections: INodeConnections,
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
            connections: INodeConnections(HashMap::new()),
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
}

#[derive(Debug)]
pub struct ParsedGraph {
    pub graph: DiGraph<WorkflowNode, GraphEdge>,
    pub node_indices: HashMap<NodeId, NodeIndex>,
}

pub struct WorkflowToGraphParser;

impl WorkflowToGraphParser {
    pub fn parse(workflow: &WorkflowDef) -> Result<ParsedGraph, String> {
        let mut graph = DiGraph::new();
        let mut node_indices = HashMap::new();

        for node in &workflow.nodes {
            let index = graph.add_node(node.clone());
            node_indices.insert(node.id.clone(), index);
        }

        for (_, output_indices) in &workflow.connections.0 {
            for outputs in output_indices {
                for output in outputs {
                    let target_node_id = NodeId::new(output.node.clone());
                    if let Some(&target_index) = node_indices.get(&target_node_id) {
                        for (_, source_index) in &node_indices {
                            let has_edge = graph
                                .edges(*source_index)
                                .any(|e| e.target() == target_index);
                            if !has_edge {
                                let edge = GraphEdge {
                                    source: *source_index,
                                    target: target_index,
                                };
                                graph.add_edge(*source_index, target_index, edge);
                            }
                        }
                    }
                }
            }
        }

        Ok(ParsedGraph {
            graph,
            node_indices,
        })
    }

    pub fn get_node_index(&self, parsed: &ParsedGraph, node_id: &NodeId) -> Option<NodeIndex> {
        parsed.node_indices.get(node_id).copied()
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
}
