use barqflow_core::types::{INodeParameters, NodeId};
use petgraph::graph::{DiGraph, NodeIndex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GraphNode {
    pub index: NodeIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: NodeIndex,
    pub target: NodeIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NodeConnectionType {
    Main,
    Trigger,
    Catch,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionOutputIndex {
    pub node: NodeId,
    pub r#type: NodeConnectionType,
    pub output_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IConnection {
    pub node: NodeId,
    pub r#type: NodeConnectionType,
    pub input_index: usize,
    pub output: Vec<ConnectionOutputIndex>,
}

pub type INodeConnections = HashMap<String, Vec<IConnection>>;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowSettings {
    pub timezone: Option<String>,
    pub error_workflow_id: Option<String>,
    pub deduplication_scope: Option<String>,
}

impl Default for WorkflowSettings {
    fn default() -> Self {
        Self {
            timezone: Some("UTC".to_string()),
            error_workflow_id: None,
            deduplication_scope: None,
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
    pub settings: Option<WorkflowSettings>,
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
            settings: Some(WorkflowSettings::default()),
            static_data: None,
            pin_data: None,
            version_id: None,
        }
    }
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

        for (source_node_id, connections) in &workflow.connections {
            let source_index = node_indices
                .get(source_node_id)
                .ok_or_else(|| format!("Source node {} not found", source_node_id))?;

            for conn in connections {
                for output in &conn.output {
                    if let Some(target_index) = node_indices.get(&output.node) {
                        let edge = GraphEdge {
                            source: *source_index,
                            target: *target_index,
                        };
                        graph.add_edge(*source_index, *target_index, edge);
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

    #[test]
    fn test_simple_linear_workflow() {
        let workflow = WorkflowDef {
            id: "test-workflow-1".to_string(),
            name: "Linear Workflow".to_string(),
            nodes: vec![
                WorkflowNode {
                    id: NodeId::new("Start"),
                    name: "Start".to_string(),
                    parameters: INodeParameters::new(),
                    type_: "manualTrigger".to_string(),
                    position: Some((0.0, 0.0)),
                    webhook_test: None,
                    webhook_prod: None,
                },
                WorkflowNode {
                    id: NodeId::new("Process"),
                    name: "Process".to_string(),
                    parameters: INodeParameters::new(),
                    type_: "set".to_string(),
                    position: Some((100.0, 0.0)),
                    webhook_test: None,
                    webhook_prod: None,
                },
                WorkflowNode {
                    id: NodeId::new("End"),
                    name: "End".to_string(),
                    parameters: INodeParameters::new(),
                    type_: "noop".to_string(),
                    position: Some((200.0, 0.0)),
                    webhook_test: None,
                    webhook_prod: None,
                },
            ],
            connections: {
                let mut m = HashMap::new();
                m.insert(
                    "Start".to_string(),
                    vec![IConnection {
                        node: NodeId::new("Start"),
                        r#type: NodeConnectionType::Main,
                        input_index: 0,
                        output: vec![ConnectionOutputIndex {
                            node: NodeId::new("Process"),
                            r#type: NodeConnectionType::Main,
                            output_index: 0,
                        }],
                    }],
                );
                m.insert(
                    "Process".to_string(),
                    vec![IConnection {
                        node: NodeId::new("Process"),
                        r#type: NodeConnectionType::Main,
                        input_index: 0,
                        output: vec![ConnectionOutputIndex {
                            node: NodeId::new("End"),
                            r#type: NodeConnectionType::Main,
                            output_index: 0,
                        }],
                    }],
                );
                m
            },
            settings: None,
            static_data: None,
            pin_data: None,
            version_id: None,
        };

        let parsed = WorkflowToGraphParser::parse(&workflow).unwrap();
        assert_eq!(parsed.graph.node_count(), 3);
        assert_eq!(parsed.graph.edge_count(), 2);

        assert!(parsed.node_indices.contains_key(&NodeId::new("Start")));
        assert!(parsed.node_indices.contains_key(&NodeId::new("Process")));
        assert!(parsed.node_indices.contains_key(&NodeId::new("End")));
    }

    #[test]
    fn test_parallel_branch_workflow() {
        let workflow = WorkflowDef {
            id: "test-workflow-2".to_string(),
            name: "Parallel Branch".to_string(),
            nodes: vec![
                WorkflowNode {
                    id: NodeId::new("Start"),
                    name: "Start".to_string(),
                    parameters: INodeParameters::new(),
                    type_: "manualTrigger".to_string(),
                    position: Some((0.0, 0.0)),
                    webhook_test: None,
                    webhook_prod: None,
                },
                WorkflowNode {
                    id: NodeId::new("Branch1"),
                    name: "Branch1".to_string(),
                    parameters: INodeParameters::new(),
                    type_: "set".to_string(),
                    position: Some((100.0, -50.0)),
                    webhook_test: None,
                    webhook_prod: None,
                },
                WorkflowNode {
                    id: NodeId::new("Branch2"),
                    name: "Branch2".to_string(),
                    parameters: INodeParameters::new(),
                    type_: "set".to_string(),
                    position: Some((100.0, 50.0)),
                    webhook_test: None,
                    webhook_prod: None,
                },
                WorkflowNode {
                    id: NodeId::new("Merge"),
                    name: "Merge".to_string(),
                    parameters: INodeParameters::new(),
                    type_: "merge".to_string(),
                    position: Some((200.0, 0.0)),
                    webhook_test: None,
                    webhook_prod: None,
                },
            ],
            connections: {
                let mut m = HashMap::new();
                m.insert(
                    "Start".to_string(),
                    vec![IConnection {
                        node: NodeId::new("Start"),
                        r#type: NodeConnectionType::Main,
                        input_index: 0,
                        output: vec![
                            ConnectionOutputIndex {
                                node: NodeId::new("Branch1"),
                                r#type: NodeConnectionType::Main,
                                output_index: 0,
                            },
                            ConnectionOutputIndex {
                                node: NodeId::new("Branch2"),
                                r#type: NodeConnectionType::Main,
                                output_index: 0,
                            },
                        ],
                    }],
                );
                m.insert(
                    "Branch1".to_string(),
                    vec![IConnection {
                        node: NodeId::new("Branch1"),
                        r#type: NodeConnectionType::Main,
                        input_index: 0,
                        output: vec![ConnectionOutputIndex {
                            node: NodeId::new("Merge"),
                            r#type: NodeConnectionType::Main,
                            output_index: 0,
                        }],
                    }],
                );
                m.insert(
                    "Branch2".to_string(),
                    vec![IConnection {
                        node: NodeId::new("Branch2"),
                        r#type: NodeConnectionType::Main,
                        input_index: 0,
                        output: vec![ConnectionOutputIndex {
                            node: NodeId::new("Merge"),
                            r#type: NodeConnectionType::Main,
                            output_index: 1,
                        }],
                    }],
                );
                m
            },
            settings: None,
            static_data: None,
            pin_data: None,
            version_id: None,
        };

        let parsed = WorkflowToGraphParser::parse(&workflow).unwrap();
        assert_eq!(parsed.graph.node_count(), 4);
        assert_eq!(parsed.graph.edge_count(), 4);
    }
}
