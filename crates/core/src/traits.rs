use async_trait::async_trait;
use crate::errors::BarqError;
use crate::schema::{INode, INodeExecutionData};
use std::collections::HashMap;

/// Provides the specific execution context available to a node when it runs.
/// This trait abstracts the engine internals from the node logic itself.
#[async_trait]
pub trait IExecuteFunctions: Send + Sync {
    /// Retrieve a parameter value configured physically on the node instance, evaluated if it maps to an expression.
    async fn get_node_parameter(&self, parameter_name: &str, fallback_value: Option<crate::types::GenericValue>) -> Result<crate::types::GenericValue, BarqError>;
    
    /// Get references to the Node itself
    fn get_node(&self) -> &INode;

    /// Read data from incoming branches
    fn get_input_data(&self, input_index: usize) -> Result<&Vec<INodeExecutionData>, BarqError>;
    
    /// Logs a debug message scoped strictly to this node execution span
    fn log(&self, message: &str);
}

/// Provides context to a trigger node during poll execution (checked periodically, e.g. IMAP reading)
#[async_trait]
pub trait IPollFunctions: Send + Sync {
    /// Retrieve static data persisted from the previous poll interval to deduplicate
    async fn get_poll_data(&self) -> Result<crate::types::IDataObject, BarqError>;
    
    /// Write static data to persist to the next poll interval
    async fn set_poll_data(&self, data: crate::types::IDataObject) -> Result<(), BarqError>;
}

/// The core trait defining a Node's logic, mirroring INodeType in n8n.
#[async_trait]
pub trait INodeType: Send + Sync {
    /// Retrieve the standard metadata for the node (used visually in the UI)
    fn get_description(&self) -> crate::types::IDataObject; // Typically returns INodeTypeDescription mapping to JSON

    /// Execute standard logic (forward pass) mapping an array of incoming items to an array of outgoing arrays.
    async fn execute(&self, context: &dyn IExecuteFunctions) -> Result<Vec<Vec<INodeExecutionData>>, BarqError>;
    
    /// Optional: specifically used for polling trigger logic to avoid full execution contexts
    async fn poll(&self, _context: &dyn IPollFunctions) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        Err(BarqError::NodeOperationError { node_name: self.get_description().0.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(), message: "Poll method not implemented".into() })
    }
}

/// Defines an Authentication Credential blueprint, matching ICredentialType
#[async_trait]
pub trait ICredentialType: Send + Sync {
    /// Returns the UI metadata form for registering these credentials
    fn get_description(&self) -> crate::types::IDataObject; // Maps to ICredentialDescription

    /// Optional hook to validate the credential externally when user clicks "test"
    async fn test_credential(&self, _credential_data: &HashMap<String, crate::types::GenericValue>) -> Result<bool, BarqError> {
        Ok(true) // Implicitly valid if hook not provided
    }
}
