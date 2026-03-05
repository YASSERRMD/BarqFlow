use barqflow_core::traits::INodeType;
use barqflow_core::types::IDataObject;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, thiserror::Error)]
pub enum NodeRegistryError {
    #[error("Node '{0}' with version '{1}' is already registered.")]
    AlreadyExists(String, u32),
    #[error("Node '{0}' not found.")]
    NotFound(String),
}

/// The NodeRegistry acts as a central repository for all available node definitions in the system.
/// It enables dynamic discovery, execution instantiation, and mapping of Graph definition IDs to actual logic.
#[derive(Clone)]
pub struct NodeRegistry {
    // Maps a composite key (name:version) to the node trait implementation
    nodes: Arc<RwLock<HashMap<String, Arc<dyn INodeType>>>>,
    // Maps a base name to its latest version natively, or just tracks defaults
    alias_map: Arc<RwLock<HashMap<String, String>>>,
}

impl Default for NodeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            alias_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generates the composite key used for internal storage given a node's base name and version.
    fn make_key(name: &str, version: u32) -> String {
        format!("{}:{}", name, version)
    }

    /// Registers a given `INodeType` implementation. 
    /// If the node with the same name and version already exists, returns an error.
    pub fn register_node(&self, node: Arc<dyn INodeType>) -> Result<(), NodeRegistryError> {
        let desc = node.get_description();
        
        // Extract required fields from the generic JSON payload
        let name = desc.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("UnknownNode")
            .to_string();
            
        let version = desc.get("version")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as u32;
        
        let target_key = Self::make_key(&name, version);

        let mut nodes_write = self.nodes.write().unwrap();
        if nodes_write.contains_key(&target_key) {
            return Err(NodeRegistryError::AlreadyExists(name.clone(), version));
        }

        nodes_write.insert(target_key.clone(), node);

        // Also update the alias map to point the base name to this specific version wrapper
        let mut alias_write = self.alias_map.write().unwrap();
        alias_write.insert(name.clone(), target_key);

        Ok(())
    }

    /// Retrieves an `INodeType` implementation by its base name and explicit version.
    pub fn get_node(&self, name: &str, version: u32) -> Result<Arc<dyn INodeType>, NodeRegistryError> {
        let target_key = Self::make_key(name, version);
        let nodes_read = self.nodes.read().unwrap();
        
        if let Some(node) = nodes_read.get(&target_key) {
            Ok(node.clone())
        } else {
            Err(NodeRegistryError::NotFound(target_key))
        }
    }

    /// Attempts to retrieve a node by name. If you only provide the name without a version,
    /// it looks up the alias map to find the registered version keys.
    pub fn get_node_by_name(&self, name: &str) -> Result<Arc<dyn INodeType>, NodeRegistryError> {
        let alias_read = self.alias_map.read().unwrap();
        
        if let Some(target_key) = alias_read.get(name) {
            let nodes_read = self.nodes.read().unwrap();
            if let Some(node) = nodes_read.get(target_key) {
                return Ok(node.clone());
            }
        }
        
        Err(NodeRegistryError::NotFound(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use barqflow_core::traits::IExecuteFunctions;
    use barqflow_core::schema::INodeExecutionData;
    use barqflow_core::errors::BarqError;
    use serde_json::json;

    struct DummyNode {
        name: String,
        version: u32,
    }

    impl DummyNode {
        fn new(name: &str, version: u32) -> Self {
            Self {
                name: name.to_string(),
                version,
            }
        }
    }

    #[async_trait]
    impl INodeType for DummyNode {
        fn get_description(&self) -> IDataObject {
            let map = json!({
                "name": self.name,
                "version": self.version,
                "displayName": self.name,
                "description": "Dummy Node for Testing"
            }).as_object().unwrap().clone();
            
            IDataObject::from(serde_json::Value::Object(map))
        }

        async fn execute(
            &self,
            _context: &dyn IExecuteFunctions,
        ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
            Ok(vec![])
        }
    }

    #[test]
    fn test_node_registry_registration_and_lookup() {
        let registry = NodeRegistry::new();

        let node_v1 = Arc::new(DummyNode::new("testNode", 1));
        assert!(registry.register_node(node_v1).is_ok());

        // Test explicit version lookup
        let retrieved_v1 = registry.get_node("testNode", 1);
        assert!(retrieved_v1.is_ok());
        
        let desc = retrieved_v1.unwrap().get_description();
        assert_eq!(desc.get("name").unwrap().as_str().unwrap(), "testNode");

        // Test alias lookup
        let retrieved_by_name = registry.get_node_by_name("testNode");
        assert!(retrieved_by_name.is_ok());
        let desc2 = retrieved_by_name.unwrap().get_description();
        assert_eq!(desc2.get("version").unwrap().as_u64().unwrap(), 1);
    }

    #[test]
    fn test_node_registry_duplicate_registration() {
        let registry = NodeRegistry::new();

        let node_v1 = Arc::new(DummyNode::new("testNode", 1));
        let node_v1_duplicate = Arc::new(DummyNode::new("testNode", 1));

        assert!(registry.register_node(node_v1).is_ok());
        
        // This should fail
        let duplicate_result = registry.register_node(node_v1_duplicate);
        assert!(duplicate_result.is_err());
        
        if let Err(NodeRegistryError::AlreadyExists(name, version)) = duplicate_result {
            assert_eq!(name, "testNode");
            assert_eq!(version, 1);
        } else {
            panic!("Expected AlreadyExists error");
        }
    }

    #[test]
    fn test_node_registry_version_aliasing() {
        let registry = NodeRegistry::new();

        let node_v1 = Arc::new(DummyNode::new("testNode", 1));
        let node_v2 = Arc::new(DummyNode::new("testNode", 2));

        assert!(registry.register_node(node_v1).is_ok());
        assert!(registry.register_node(node_v2).is_ok());

        // Ensure retrieving explicit versions still works
        assert!(registry.get_node("testNode", 1).is_ok());
        assert!(registry.get_node("testNode", 2).is_ok());

        // Alias should point to V2 (latest registered)
        let retrieved_alias = registry.get_node_by_name("testNode");
        assert!(retrieved_alias.is_ok());
        let desc = retrieved_alias.unwrap().get_description();
        assert_eq!(desc.get("version").unwrap().as_u64().unwrap(), 2);
    }
}
