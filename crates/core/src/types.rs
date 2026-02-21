use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Unique identifier for an execution run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RunId(pub Uuid);

impl RunId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for RunId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for RunId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a workflow definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WorkflowId(pub Uuid);

impl WorkflowId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for WorkflowId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for WorkflowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a specific item of data processing in a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ItemId(pub Uuid);

impl ItemId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ItemId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a Node within a workflow.
/// NodeIds are typically strings defined by the user (or auto-generated strings)
/// to ensure they are readable when referencing nodes in expressions.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeId(pub String);

impl NodeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_id_serialization() {
        let id = RunId::new();
        let serialized = serde_json::to_string(&id).unwrap();
        let deserialized: RunId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_node_id_serialization() {
        let id = NodeId::new("StartNode");
        let serialized = serde_json::to_string(&id).unwrap();
        assert_eq!(serialized, "\"StartNode\"");
        let deserialized: NodeId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_data_object_serialization() {
        use serde_json::json;
        let val: serde_json::Value = json!({
            "name": "BarqFlow",
            "active": true,
            "nested": {
                "key": "value"
            }
        });

        let data_object = IDataObject::from(val.clone());
        let serialized = serde_json::to_string(&data_object).unwrap();
        let deserialized: IDataObject = serde_json::from_str(&serialized).unwrap();

        assert_eq!(data_object.0, deserialized.0);
        assert_eq!(data_object.0, val);
    }

    #[test]
    fn test_deeply_nested_json_serialization() {
        use serde_json::json;
        // Create a deeply nested JSON object structure
        let deeply_nested = json!({
            "workflow": {
                "name": "Complex Data Pipeline",
                "nodes": [
                    {
                        "id": "node1",
                        "type": "trigger",
                        "parameters": {
                            "expression": "{{ $json.data }}",
                            "options": {
                                "recursive": true,
                                "depth": 5
                            }
                        },
                        "metadata": {
                            "created": "2024-01-15T10:30:00Z",
                            "tags": ["production", "critical"]
                        }
                    },
                    {
                        "id": "node2",
                        "type": "processor",
                        "connections": {
                            "main": [
                                [{"node": "node3", "type": "main", "index": 0}]
                            ]
                        }
                    }
                ],
                "settings": {
                    "execution": {
                        "timeout": 30000,
                        "retry": {
                            "max": 3,
                            "backoff": "exponential"
                        }
                    },
                    "error_workflow": {
                        "enabled": true,
                        "path": "/workflows/error-handler"
                    }
                }
            },
            "environment": {
                "variables": {
                    "API_ENDPOINT": "https://api.example.com",
                    "TIMEOUT": "30"
                },
                "features": {
                    "advanced_mode": true,
                    "beta_features": ["experimental", "preview"]
                }
            }
        });

        let data_object = IDataObject::from(deeply_nested.clone());
        let serialized = serde_json::to_string(&data_object).unwrap();
        let deserialized: IDataObject = serde_json::from_str(&serialized).unwrap();

        // Verify the data integrity after serialization/deserialization
        assert_eq!(data_object.0, deserialized.0);
        assert_eq!(data_object.0, deeply_nested);

        // Verify specific nested values can be accessed
        assert_eq!(
            data_object.0["workflow"]["name"],
            "Complex Data Pipeline"
        );
        assert_eq!(
            data_object.0["workflow"]["nodes"][0]["parameters"]["options"]["depth"],
            5
        );
        assert_eq!(
            data_object.0["environment"]["variables"]["API_ENDPOINT"],
            "https://api.example.com"
        );
    }
}

/// GenericValue wrapping a serde_json::Value
/// Represents any generic data value that can be processed by a node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GenericValue(pub serde_json::Value);

impl From<serde_json::Value> for GenericValue {
    fn from(value: serde_json::Value) -> Self {
        Self(value)
    }
}

impl Default for GenericValue {
    fn default() -> Self {
        Self(serde_json::Value::Null)
    }
}

/// IDataObject wrapping a serde_json::Map or Object
/// Represents a structured collection of key-value pairs typical in workflow data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IDataObject(pub serde_json::Value);

impl IDataObject {
    pub fn new() -> Self {
        Self(serde_json::Value::Object(serde_json::Map::new()))
    }
}

impl Default for IDataObject {
    fn default() -> Self {
        Self::new()
    }
}

impl From<serde_json::Value> for IDataObject {
    fn from(value: serde_json::Value) -> Self {
        if value.is_object() {
            Self(value)
        } else {
            let mut map = serde_json::Map::new();
            map.insert("data".to_string(), value);
            Self(serde_json::Value::Object(map))
        }
    }
}



/// Category of binary file type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BinaryFileType {
    Text,
    Json,
    Image,
    Audio,
    Video,
    Pdf,
    Html,
}

/// Represents the actual content of a binary file.
/// It can either reside in memory as a base64 string or on the filesystem with an ID pointer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BinaryDataContent {
    /// In memory representation (typically base64)
    Memory { data: String },
    /// Filesystem ID reference for large files.
    FileSystem { id: String },
}

/// Standardized representation of binary data passing between workflow nodes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IBinaryData {
    #[serde(flatten)]
    pub content: BinaryDataContent,
    pub mime_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_type: Option<BinaryFileType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_extension: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size: Option<String>,
}

#[cfg(test)]
mod binary_tests {
    use super::*;

    #[test]
    fn test_binary_data_memory_serialization() {
        let bin = IBinaryData {
            content: BinaryDataContent::Memory {
                data: "base64content".into(),
            },
            mime_type: "image/png".into(),
            file_type: Some(BinaryFileType::Image),
            file_name: Some("test.png".into()),
            directory: None,
            file_extension: Some("png".into()),
            file_size: None,
        };

        let serialized = serde_json::to_string(&bin).unwrap();
        assert!(serialized.contains(r#""data":"base64content""#));
        assert!(serialized.contains(r#""mimeType":"image/png""#));
        assert!(serialized.contains(r#""fileType":"image""#));

        let deserialized: IBinaryData = serde_json::from_str(&serialized).unwrap();
        assert_eq!(bin, deserialized);
    }

    #[test]
    fn test_binary_data_fs_serialization() {
        let bin = IBinaryData {
            content: BinaryDataContent::FileSystem {
                id: "1234-abcd".into(),
            },
            mime_type: "application/pdf".into(),
            file_type: Some(BinaryFileType::Pdf),
            file_name: None,
            directory: None,
            file_extension: None,
            file_size: None,
        };

        let serialized = serde_json::to_string(&bin).unwrap();
        assert!(serialized.contains(r#""id":"1234-abcd""#));
        assert!(serialized.contains(r#""fileType":"pdf""#));

        let deserialized: IBinaryData = serde_json::from_str(&serialized).unwrap();
        assert_eq!(bin, deserialized);
    }
}
