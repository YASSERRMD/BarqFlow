//! Node Registry
//!
//! Thread-safe registry for managing node types with versioning support.

use barqflow_core::properties::INodeProperties;
use barqflow_core::traits::ICredentialType;
use barqflow_core::traits::INodeType;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Information about a registered Credential type
#[derive(Clone)]
pub struct CredentialInfo {
    /// The credential type identifier (e.g. "httpBasicAuth")
    pub name: String,
    /// Credential Type object implementation trait
    pub cred_impl: Arc<dyn ICredentialType>,
}

/// Thread-safe Credential registry for authentication flows
pub struct CredentialRegistry {
    credentials: RwLock<HashMap<String, CredentialInfo>>,
}

impl CredentialRegistry {
    pub fn new() -> Self {
        Self {
            credentials: RwLock::new(HashMap::new()),
        }
    }

    pub fn register_credential(&self, info: CredentialInfo) -> Result<(), String> {
        let mut creds = self.credentials.write().map_err(|e| e.to_string())?;
        if creds.contains_key(&info.name) {
            return Err(format!("Credential '{}' already registered", info.name));
        }
        creds.insert(info.name.clone(), info);
        Ok(())
    }

    pub fn get_credential(&self, name: &str) -> Option<CredentialInfo> {
        let creds = self.credentials.read().ok()?;
        creds.get(name).cloned()
    }

    /// Retrieves the test rules for a given credential type if available
    pub fn test_credential_rules(
        &self,
        name: &str,
    ) -> Result<Option<barqflow_core::traits::ICredentialTestRequest>, String> {
        let creds = self.credentials.read().map_err(|e| e.to_string())?;
        if let Some(info) = creds.get(name) {
            Ok(info.cred_impl.test_request())
        } else {
            Err(format!("Credential '{}' not found", name))
        }
    }
}

impl Default for CredentialRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a registered node type.
#[derive(Clone)]
pub struct NodeInfo {
    /// The node type identifier (e.g., "n8n-nodes-base.httpRequest")
    pub name: String,
    /// Display name shown in the UI
    pub display_name: String,
    /// Node version
    pub version: f32,
    /// Description of what this node does
    pub description: String,
    /// UI properties for this node
    pub properties: INodeProperties,
    /// Whether this node can be used as a trigger
    pub is_trigger: bool,
    /// The maximum number of items this node can process
    pub max_inputs: usize,
    /// The node implementation trait object
    pub node_impl: Arc<dyn INodeType + Send + Sync>,
}

/// Version entry in the version index.
struct VersionEntry {
    version: f32,
    key: String,
}

/// Thread-safe node type registry.
///
/// Manages all registered node types with support for versioning and
/// duplicate detection.
pub struct NodeRegistry {
    /// Map of node name (with version suffix) to node info
    nodes: RwLock<HashMap<String, NodeInfo>>,
    /// Map of node name without version to available versions
    /// Key: node name (e.g., "httpRequest")
    /// Value: Vec of (version, full_key) tuples
    version_index: RwLock<HashMap<String, Vec<VersionEntry>>>,
}

impl NodeRegistry {
    /// Create a new empty node registry.
    pub fn new() -> Self {
        Self {
            nodes: RwLock::new(HashMap::new()),
            version_index: RwLock::new(HashMap::new()),
        }
    }

    /// Register a new node type.
    ///
    /// # Arguments
    /// * `info` - The node information to register
    ///
    /// # Returns
    /// * `Ok(())` if registration succeeded
    /// * `Err(String)` if a node with the same name and version already exists
    pub fn register_node(&self, info: NodeInfo) -> Result<(), String> {
        let key = Self::make_key(&info.name, info.version);

        // Check for duplicates
        {
            let nodes = self.nodes.read().map_err(|e| e.to_string())?;
            if nodes.contains_key(&key) {
                return Err(format!(
                    "Node '{}' version {} is already registered",
                    info.name, info.version
                ));
            }
        }

        // Add to version index
        {
            let mut version_index = self.version_index.write().map_err(|e| e.to_string())?;
            let entries = version_index
                .entry(info.name.clone())
                .or_insert_with(Vec::new);

            // Check for duplicate version
            if entries
                .iter()
                .any(|e| (e.version - info.version).abs() < f32::EPSILON)
            {
                return Err(format!(
                    "Node '{}' version {} is already registered",
                    info.name, info.version
                ));
            }

            entries.push(VersionEntry {
                version: info.version,
                key: key.clone(),
            });
        }

        // Add to main registry
        {
            let mut nodes = self.nodes.write().map_err(|e| e.to_string())?;
            nodes.insert(key, info);
        }

        Ok(())
    }

    /// Get a node by name and version.
    ///
    /// # Arguments
    /// * `name` - The node type name
    /// * `version` - The specific version to retrieve
    ///
    /// # Returns
    /// * `Some(NodeInfo)` if found
    /// * `None` if not found
    pub fn get_node_by_name(&self, name: &str, version: f32) -> Option<NodeInfo> {
        let key = Self::make_key(name, version);
        let nodes = self.nodes.read().ok()?;
        nodes.get(&key).cloned()
    }

    /// Get a node by name, using version alias resolution.
    ///
    /// If a specific version is requested, returns that version.
    /// If the version is not found, falls back to version 1.0.
    ///
    /// # Arguments
    /// * `name` - The node type name
    /// * `version` - The version to retrieve (will fall back to 1.0 if not found)
    ///
    /// # Returns
    /// * `Some(NodeInfo)` if found (either requested version or v1.0)
    /// * `None` if neither version is found
    pub fn get_node_by_name_with_fallback(&self, name: &str, version: f32) -> Option<NodeInfo> {
        // Try requested version first
        if let Some(node) = self.get_node_by_name(name, version) {
            return Some(node);
        }

        // Fall back to version 1.0
        if (version - 1.0).abs() > f32::EPSILON {
            if let Some(node) = self.get_node_by_name(name, 1.0) {
                return Some(node);
            }
        }

        None
    }

    /// Get the highest available version of a node.
    ///
    /// # Arguments
    /// * `name` - The node type name
    ///
    /// # Returns
    /// * `Some(NodeInfo)` with the highest version
    /// * `None` if node not found
    pub fn get_latest_node(&self, name: &str) -> Option<NodeInfo> {
        let version_index = self.version_index.read().ok()?;
        let entries = version_index.get(name)?;

        // Find the highest version
        let latest_entry = entries.iter().max_by(|a, b| {
            a.version
                .partial_cmp(&b.version)
                .unwrap_or(std::cmp::Ordering::Equal)
        })?;

        let nodes = self.nodes.read().ok()?;
        nodes.get(&latest_entry.key).cloned()
    }

    /// Get all registered node names.
    pub fn get_all_node_names(&self) -> Vec<String> {
        let version_index = self.version_index.read().ok();
        match version_index {
            Some(index) => index.keys().cloned().collect(),
            None => Vec::new(),
        }
    }

    /// Get all available versions for a node.
    ///
    /// # Returns
    /// A sorted vector of versions (lowest to highest)
    pub fn get_node_versions(&self, name: &str) -> Vec<f32> {
        let version_index = self.version_index.read().ok();
        match version_index {
            Some(index) => {
                let mut versions: Vec<f32> = index
                    .get(name)
                    .map(|v| v.iter().map(|e| e.version).collect())
                    .unwrap_or_default();
                versions.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                versions
            }
            None => Vec::new(),
        }
    }

    /// Create the registry key for a node.
    fn make_key(name: &str, version: f32) -> String {
        format!("{}@{}", name, version)
    }
}

impl Default for NodeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use barqflow_core::errors::BarqError;
    use barqflow_core::properties::{INodeProperty, NodePropertyType};
    use barqflow_core::traits::IExecuteFunctions;
    use barqflow_core::types::IDataObject;
    use serde_json::json;

    // Mock node implementation for testing
    struct MockNode;

    #[async_trait]
    impl INodeType for MockNode {
        fn get_description(&self) -> IDataObject {
            IDataObject::from(json!({
                "name": "MockNode",
                "displayName": "Mock Node",
                "description": "A mock node for testing"
            }))
        }

        async fn execute(
            &self,
            _context: &dyn IExecuteFunctions,
        ) -> Result<Vec<Vec<barqflow_core::schema::INodeExecutionData>>, BarqError> {
            Ok(vec![vec![barqflow_core::schema::INodeExecutionData::new(
                IDataObject::new(),
            )]])
        }
    }

    fn create_mock_node_info(name: &str, version: f32) -> NodeInfo {
        NodeInfo {
            name: name.to_string(),
            display_name: format!("{} Node", name),
            version,
            description: format!("A test node for {}", name),
            properties: INodeProperties {
                display_name: Some("Test Properties".to_string()),
                properties: vec![INodeProperty {
                    display_name: "Test".to_string(),
                    name: "test".to_string(),
                    r#type: NodePropertyType::String,
                    default: None,
                    description: None,
                    hint: None,
                    required: false,
                    options: None,
                    display_options: None,
                }],
                required_values: None,
            },
            is_trigger: false,
            max_inputs: 1,
            node_impl: Arc::new(MockNode),
        }
    }

    #[test]
    fn test_register_new_node() {
        let registry = NodeRegistry::new();
        let node_info = create_mock_node_info("httpRequest", 1.0);

        let result = registry.register_node(node_info);
        assert!(result.is_ok());

        let retrieved = registry.get_node_by_name("httpRequest", 1.0);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "httpRequest");
    }

    #[test]
    fn test_register_duplicate_node_fails() {
        let registry = NodeRegistry::new();
        let node_info1 = create_mock_node_info("httpRequest", 1.0);
        let node_info2 = create_mock_node_info("httpRequest", 1.0);

        registry.register_node(node_info1).unwrap();
        let result = registry.register_node(node_info2);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already registered"));
    }

    #[test]
    fn test_register_same_node_different_versions() {
        let registry = NodeRegistry::new();
        let node_info1 = create_mock_node_info("httpRequest", 1.0);
        let node_info2 = create_mock_node_info("httpRequest", 2.0);

        registry.register_node(node_info1).unwrap();
        registry.register_node(node_info2).unwrap();

        let v1 = registry.get_node_by_name("httpRequest", 1.0);
        let v2 = registry.get_node_by_name("httpRequest", 2.0);

        assert!(v1.is_some());
        assert!(v2.is_some());
        assert_eq!(v1.unwrap().version, 1.0);
        assert_eq!(v2.unwrap().version, 2.0);
    }

    #[test]
    fn test_version_fallback() {
        let registry = NodeRegistry::new();
        let node_info = create_mock_node_info("httpRequest", 1.0);

        registry.register_node(node_info).unwrap();

        // Request version 2.0, should fall back to 1.0
        let retrieved = registry.get_node_by_name_with_fallback("httpRequest", 2.0);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().version, 1.0);
    }

    #[test]
    fn test_get_latest_node() {
        let registry = NodeRegistry::new();
        registry
            .register_node(create_mock_node_info("httpRequest", 1.0))
            .unwrap();
        registry
            .register_node(create_mock_node_info("httpRequest", 2.0))
            .unwrap();
        registry
            .register_node(create_mock_node_info("httpRequest", 1.5))
            .unwrap();

        let latest = registry.get_latest_node("httpRequest");
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().version, 2.0);
    }

    #[test]
    fn test_get_node_versions() {
        let registry = NodeRegistry::new();
        registry
            .register_node(create_mock_node_info("httpRequest", 1.0))
            .unwrap();
        registry
            .register_node(create_mock_node_info("httpRequest", 2.0))
            .unwrap();
        registry
            .register_node(create_mock_node_info("httpRequest", 1.5))
            .unwrap();

        let versions = registry.get_node_versions("httpRequest");
        assert_eq!(versions, vec![1.0, 1.5, 2.0]);
    }

    #[test]
    fn test_get_all_node_names() {
        let registry = NodeRegistry::new();
        registry
            .register_node(create_mock_node_info("httpRequest", 1.0))
            .unwrap();
        registry
            .register_node(create_mock_node_info("webhook", 1.0))
            .unwrap();
        registry
            .register_node(create_mock_node_info("code", 1.0))
            .unwrap();

        let names = registry.get_all_node_names();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"httpRequest".to_string()));
        assert!(names.contains(&"webhook".to_string()));
        assert!(names.contains(&"code".to_string()));
    }

    struct MockCredential {
        name: String,
    }

    #[async_trait]
    impl ICredentialType for MockCredential {
        fn get_description(&self) -> barqflow_core::properties::ICredentialProperties {
            barqflow_core::properties::ICredentialProperties {
                name: self.name.clone(),
                display_name: format!("{} Credential", self.name),
                properties: vec![],
                authenticate: None,
                documentation_url: None,
                notice: None,
            }
        }

        fn test_request(&self) -> Option<barqflow_core::traits::ICredentialTestRequest> {
            Some(barqflow_core::traits::ICredentialTestRequest {
                method: "GET".to_string(),
                url: "https://api.example.com/me".to_string(),
                expected_status: vec![200],
            })
        }
    }

    #[test]
    fn test_credential_validation_rules() {
        let registry = CredentialRegistry::new();
        let cred_impl = Arc::new(MockCredential {
            name: "testAuth".to_string(),
        });

        registry
            .register_credential(CredentialInfo {
                name: "testAuth".to_string(),
                cred_impl,
            })
            .unwrap();

        let rules = registry.test_credential_rules("testAuth").unwrap().unwrap();
        assert_eq!(rules.method, "GET");
        assert_eq!(rules.url, "https://api.example.com/me");
        assert_eq!(rules.expected_status, vec![200]);

        let missing = registry.test_credential_rules("doesNotExist");
        assert!(missing.is_err());
    }
}
