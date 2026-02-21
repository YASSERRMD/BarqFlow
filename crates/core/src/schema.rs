use crate::types::{IBinaryData, IDataObject};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the data returned by a node or passed to the next node.
/// Contains JSON data and an optional dictionary of binary data references.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct INodeExecutionData {
    pub json: IDataObject,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary: Option<HashMap<String, IBinaryData>>,
}

impl INodeExecutionData {
    pub fn new(json: IDataObject) -> Self {
        Self { json, binary: None }
    }

    pub fn with_binary(mut self, key: String, data: IBinaryData) -> Self {
        let binary_map = self.binary.get_or_insert_with(HashMap::new);
        binary_map.insert(key, data);
        self
    }
}

/// Represents a hint message returned during execution, typically displayed in the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeExecutionHint {
    pub message: String,
}

/// Represents the inputs passed into a node during execution, mapped by input index.
/// A node can have multiple inputs (e.g. Merge node has 2 inputs).
/// Each input contains an array of `INodeExecutionData` representing the items.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ITaskDataConnections(pub HashMap<usize, Vec<INodeExecutionData>>);

impl ITaskDataConnections {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn push(&mut self, input_index: usize, data: Vec<INodeExecutionData>) {
        self.0.insert(input_index, data);
    }
}

impl Default for ITaskDataConnections {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{BinaryDataContent, BinaryFileType};
    use serde_json::json;

    #[test]
    fn test_execution_data_aggregation() {
        let data1 = INodeExecutionData::new(IDataObject::from(json!({"id": 1})));
        let data2 = INodeExecutionData::new(IDataObject::from(json!({"id": 2}))).with_binary(
            "image".into(),
            IBinaryData {
                content: BinaryDataContent::Memory {
                    data: "base64content".into(),
                },
                mime_type: "image/png".into(),
                file_type: Some(BinaryFileType::Image),
                file_name: None,
                directory: None,
                file_extension: None,
                file_size: None,
            },
        );

        let mut connections = ITaskDataConnections::new();
        connections.push(0, vec![data1.clone()]);
        connections.push(1, vec![data2.clone()]);

        assert_eq!(connections.0.get(&0).unwrap().len(), 1);
        assert_eq!(connections.0.get(&1).unwrap().len(), 1);
        assert!(connections.0.get(&1).unwrap()[0].binary.is_some());
    }
}

/// The type of connection between nodes.
/// Determines if it's the main data flow or a specialized trigger/catch flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NodeConnectionType {
    Main,
}

impl Default for NodeConnectionType {
    fn default() -> Self {
        Self::Main
    }
}

/// Represents an edge in the DAG connecting one node's output to another node's input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IConnection {
    /// Name of the target node
    pub node: String,
    /// Type of connection (usually Main)
    pub r#type: NodeConnectionType,
    /// The input index on the target node this connects to
    pub index: usize,
}

/// Map representing a node's outgoing connections.
/// Key is the connection type (e.g. "main"), value is a map of output indices to connections.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct INodeConnections(pub HashMap<NodeConnectionType, Vec<Vec<IConnection>>>);

/// User-defined parameters for a node configuration, mapping string keys to generic values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct INodeParameters(pub HashMap<String, crate::types::GenericValue>);

/// Represents a single configured Node entity inside a Workflow.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct INode {
    pub id: crate::types::NodeId,
    pub name: String,
    pub r#type: String,
    pub type_version: f32,
    pub position: [f32; 2],
    pub parameters: INodeParameters,
    #[serde(default)]
    pub disabled: bool,
}

/// Settings applied to a workflow globally.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IWorkflowSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save_data_error_execution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save_execution_progress: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save_manual_executions: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller_policy: Option<String>,
}

impl Default for IWorkflowSettings {
    fn default() -> Self {
        Self {
            timezone: None,
            save_data_error_execution: None,
            save_execution_progress: None,
            save_manual_executions: None,
            caller_policy: None,
        }
    }
}

/// Represents an entire completely defined graph (Workflow).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowDef {
    pub id: crate::types::WorkflowId,
    pub name: String,
    pub nodes: Vec<INode>,
    /// Map of source node name to its outgoing connections
    pub connections: HashMap<String, INodeConnections>,
    pub active: bool,
    #[serde(default)]
    pub settings: IWorkflowSettings,
}
