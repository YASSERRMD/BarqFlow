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
        let data2 = INodeExecutionData::new(IDataObject::from(json!({"id": 2})))
            .with_binary(
                "image".into(),
                IBinaryData {
                    content: BinaryDataContent::Memory { data: "base64content".into() },
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
