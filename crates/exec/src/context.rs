//! Node Execution Context
//!
//! Implements IExecuteFunctions trait providing execution context to nodes.
//! This is the actual implementation that bridges the engine internals with node logic.

use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::{INode, INodeExecutionData, ITaskDataConnections};
use barqflow_core::traits::IExecuteFunctions;
use barqflow_core::types::{GenericValue, IDataObject};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, span, Level};

/// Shared cache of prior node outputs, keyed by node name.
type WorkflowCache = Arc<RwLock<HashMap<String, Vec<serde_json::Value>>>>;

#[async_trait]
pub trait CredentialProvider: Send + Sync {
    async fn get_credentials(
        &self,
        node_id: &str,
        credential_type: &str,
    ) -> Result<HashMap<String, GenericValue>, BarqError>;
}

/// Execution context passed to each node during execution.
///
/// Provides nodes access to their parameters, input data, and logging facilities.
pub struct NodeExecutionContext {
    /// The node being executed
    node: INode,
    /// Input data connections (mapped by input index)
    input_data: Arc<RwLock<ITaskDataConnections>>,
    /// Static data from workflow for expression evaluation
    _static_data: Option<IDataObject>,
    /// Run ID for tracing
    run_id: uuid::Uuid,
    /// Execution output cache from previous nodes
    workflow_cache: WorkflowCache,
    /// Credentials resolver for runtime secret lookup
    credential_provider: Option<Arc<dyn CredentialProvider>>,
    /// Rhai expression engine — initialised once per context and reused across parameter evaluations.
    expression_engine: barqflow_flow::expression::ExpressionEngine,
}

impl std::fmt::Debug for NodeExecutionContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeExecutionContext")
            .field("node", &self.node.name)
            .field("run_id", &self.run_id)
            .finish_non_exhaustive()
    }
}

impl NodeExecutionContext {
    /// Create a new execution context for a node.
    ///
    /// # Arguments
    /// * `node` - The node being executed
    /// * `input_data` - Input data from previous nodes
    /// * `static_data` - Static workflow data for expression evaluation
    /// * `run_id` - Unique identifier for this execution run
    pub fn new(
        node: INode,
        input_data: ITaskDataConnections,
        static_data: Option<IDataObject>,
        run_id: uuid::Uuid,
        workflow_cache: WorkflowCache,
    ) -> Self {
        Self::new_with_credentials(node, input_data, static_data, run_id, workflow_cache, None)
    }

    pub fn new_with_credentials(
        node: INode,
        input_data: ITaskDataConnections,
        static_data: Option<IDataObject>,
        run_id: uuid::Uuid,
        workflow_cache: WorkflowCache,
        credential_provider: Option<Arc<dyn CredentialProvider>>,
    ) -> Self {
        Self {
            node,
            input_data: Arc::new(RwLock::new(input_data)),
            _static_data: static_data,
            run_id,
            workflow_cache,
            credential_provider,
            expression_engine: barqflow_flow::expression::ExpressionEngine::new()
                .with_custom_functions(),
        }
    }

    /// Evaluate a parameter value, handling expression evaluation if needed.
    ///
    /// This method:
    /// 1. Retrieves the raw parameter value from the node's parameters
    /// 2. If it's an expression (starts with `{{`), evaluates it against the input data
    /// 3. Returns the resolved value or the fallback if parameter doesn't exist
    ///
    /// # Arguments
    /// * `parameter_name` - Name of the parameter to retrieve
    /// * `fallback_value` - Value to return if parameter doesn't exist
    async fn evaluate_parameter(
        &self,
        parameter_name: &str,
        item_index: usize,
        fallback_value: Option<GenericValue>,
    ) -> Result<GenericValue, BarqError> {
        // Get the raw parameter value
        let raw_value = self
            .node
            .parameters
            .0
            .get(parameter_name)
            .or(fallback_value.as_ref())
            .ok_or_else(|| BarqError::NodeOperationError {
                node_name: self.node.name.clone(),
                message: format!("Required parameter '{}' not found", parameter_name),
            })?;

        // Check if this is an expression
        if let Some(expr_str) = raw_value.as_str() {
            if expr_str.starts_with("{{") && expr_str.ends_with("}}") {
                // Remove the {{ }} wrappers for Rhai
                let stripped_expr = expr_str[2..expr_str.len() - 2].trim().to_string();
                let expr_str_owned = expr_str.to_string();

                // Get data we need across await boundary FIRST
                let json_data = {
                    let input = self.input_data.read().await;
                    // Evaluate against specific item index or first item if not found
                    input
                        .0
                        .get(&0)
                        .and_then(|items| items.get(item_index).or_else(|| items.first()))
                        .map(|item| serde_json::Value::Object(item.json.0.clone()))
                        .unwrap_or_else(|| serde_json::json!({}))
                };

                let params_map: HashMap<String, serde_json::Value> = self
                    .node
                    .parameters
                    .0
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                let node_name = self.node.name.clone();

                // Perform the evaluation in a strict scope so non-Send types are dropped
                let eval_result = {
                    let cache_snapshot = {
                        let cache = self.workflow_cache.read().await;
                        (*cache).clone()
                    };

                    let expr_ctx = barqflow_flow::expression::ExpressionContext {
                        json_data,
                        binary_keys: vec![],
                        parameters: params_map,
                        workflow_cache: cache_snapshot,
                    };

                    self.expression_engine
                        .eval_with_context(&stripped_expr, &expr_ctx)
                };

                return match eval_result {
                    Ok(dyn_val) if dyn_val.is_unit() => Ok(serde_json::Value::Null),
                    Ok(dyn_val) if dyn_val.is_string() => {
                        Ok(serde_json::Value::String(dyn_val.into_string().unwrap()))
                    }
                    Ok(dyn_val) if dyn_val.is_int() => {
                        Ok(serde_json::json!(dyn_val.as_int().unwrap()))
                    }
                    Ok(dyn_val) if dyn_val.is_bool() => {
                        Ok(serde_json::Value::Bool(dyn_val.as_bool().unwrap()))
                    }
                    Ok(dyn_val) if dyn_val.is_float() => {
                        Ok(serde_json::json!(dyn_val.as_float().unwrap()))
                    }
                    Ok(_) => Err(BarqError::ExpressionError {
                        node_name,
                        message: format!(
                            "Expression '{}' resulted in unsupported complex Rhai Dynamic type",
                            expr_str_owned
                        ),
                    }),
                    Err(e) => Err(BarqError::ExpressionError {
                        node_name,
                        message: format!(
                            "Failed to evaluate expression '{}': {}",
                            expr_str_owned, e
                        ),
                    }),
                };
            }
        }

        Ok(raw_value.clone())
    }

    /// Update input data for this context.
    ///
    /// Used when data flows through multiple nodes in a workflow.
    pub async fn update_input_data(&self, new_data: ITaskDataConnections) {
        let mut input = self.input_data.write().await;
        *input = new_data;
    }
}

#[async_trait]
impl IExecuteFunctions for NodeExecutionContext {
    /// Retrieve a parameter value, evaluating expressions if needed.
    async fn get_node_parameter(
        &self,
        parameter_name: &str,
        fallback_value: Option<GenericValue>,
    ) -> Result<GenericValue, BarqError> {
        self.evaluate_parameter(parameter_name, 0, fallback_value)
            .await
    }

    /// Retrieve a parameter value evaluated against a specific item index.
    async fn get_node_parameter_at_item(
        &self,
        parameter_name: &str,
        item_index: usize,
        fallback_value: Option<GenericValue>,
    ) -> Result<GenericValue, BarqError> {
        self.evaluate_parameter(parameter_name, item_index, fallback_value)
            .await
    }

    /// Get reference to the node being executed.
    fn get_node(&self) -> &INode {
        &self.node
    }

    async fn get_input_data(
        &self,
        input_index: usize,
    ) -> Result<Vec<INodeExecutionData>, BarqError> {
        let data = self.input_data.read().await;
        data.0
            .get(&input_index)
            .cloned()
            .ok_or_else(|| BarqError::NodeOperationError {
                node_name: self.node.name.clone(),
                message: format!("Input branch {} not found", input_index),
            })
    }

    async fn get_credentials(
        &self,
        name: &str,
    ) -> Result<HashMap<String, GenericValue>, BarqError> {
        let provider =
            self.credential_provider
                .as_ref()
                .ok_or_else(|| BarqError::NodeOperationError {
                    node_name: self.node.name.clone(),
                    message: format!(
                        "Credential provider is not configured; cannot resolve '{}' for node '{}'",
                        name, self.node.id
                    ),
                })?;

        let creds = provider.get_credentials(&self.node.id.0, name).await?;
        self.log(&format!(
            "Resolved credentials for '{}' on node '{}'",
            name, self.node.id
        ));
        Ok(creds)
    }

    fn log(&self, message: &str) {
        let _span = span!(
            Level::DEBUG,
            "node_execution",
            run_id = %self.run_id,
            node_id = %self.node.id,
            node_name = %self.node.name
        );
        debug!("{}", message);
    }
}

/// Execution context passed to trigger nodes during polling intervals.
/// Provides access to read and update static memory data across ticks.
pub struct PollExecutionContext {
    /// The trigger node being polled
    node: INode,
    workflow_id: uuid::Uuid,
    /// Shared static data map for the workflow
    static_repo: Arc<dyn barqflow_core::traits::IStaticDataStorage>,
}

impl PollExecutionContext {
    pub fn new(
        node: INode,
        workflow_id: uuid::Uuid,
        static_repo: Arc<dyn barqflow_core::traits::IStaticDataStorage>,
    ) -> Self {
        Self {
            node,
            workflow_id,
            static_repo,
        }
    }
}

#[async_trait]
impl barqflow_core::traits::IPollFunctions for PollExecutionContext {
    async fn get_poll_data(&self) -> Result<IDataObject, BarqError> {
        let opt = self
            .static_repo
            .get(self.node.id.to_string(), self.workflow_id)
            .await?;
        Ok(opt.unwrap_or_default())
    }

    async fn set_poll_data(&self, data: IDataObject) -> Result<(), BarqError> {
        self.static_repo
            .upsert(self.node.id.to_string(), self.workflow_id, data)
            .await
    }

    async fn get_node_parameter(
        &self,
        parameter_name: &str,
        fallback_value: Option<GenericValue>,
    ) -> Result<GenericValue, BarqError> {
        let raw_value = self
            .node
            .parameters
            .0
            .get(parameter_name)
            .or(fallback_value.as_ref())
            .ok_or_else(|| BarqError::NodeOperationError {
                node_name: self.node.name.clone(),
                message: format!("Required parameter '{}' not found", parameter_name),
            })?;
        Ok(raw_value.clone())
    }

    fn get_node(&self) -> &INode {
        &self.node
    }

    async fn get_credentials(
        &self,
        _name: &str,
    ) -> Result<HashMap<String, GenericValue>, BarqError> {
        Ok(HashMap::new())
    }

    fn log(&self, message: &str) {
        let _span_guard = span!(
            Level::DEBUG,
            "poll_execution",
            node_id = %self.node.id,
            node_name = %self.node.name
        );
        debug!("{}", message);
    }
}

/// Builder for creating NodeExecutionContext instances.
pub struct NodeExecutionContextBuilder {
    node: Option<INode>,
    input_data: Option<ITaskDataConnections>,
    static_data: Option<IDataObject>,
    run_id: Option<uuid::Uuid>,
    workflow_cache: Option<WorkflowCache>,
    credential_provider: Option<Arc<dyn CredentialProvider>>,
}

impl Default for NodeExecutionContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeExecutionContextBuilder {
    pub fn new() -> Self {
        Self {
            node: None,
            input_data: None,
            static_data: None,
            run_id: None,
            workflow_cache: None,
            credential_provider: None,
        }
    }

    pub fn with_node(mut self, node: INode) -> Self {
        self.node = Some(node);
        self
    }

    pub fn with_input_data(mut self, input_data: ITaskDataConnections) -> Self {
        self.input_data = Some(input_data);
        self
    }

    pub fn with_static_data(mut self, static_data: IDataObject) -> Self {
        self.static_data = Some(static_data);
        self
    }

    pub fn with_run_id(mut self, run_id: uuid::Uuid) -> Self {
        self.run_id = Some(run_id);
        self
    }

    pub fn with_workflow_cache(mut self, workflow_cache: WorkflowCache) -> Self {
        self.workflow_cache = Some(workflow_cache);
        self
    }

    pub fn with_credential_provider(
        mut self,
        credential_provider: Arc<dyn CredentialProvider>,
    ) -> Self {
        self.credential_provider = Some(credential_provider);
        self
    }

    pub fn build(self) -> Result<NodeExecutionContext, String> {
        Ok(NodeExecutionContext::new_with_credentials(
            self.node.ok_or("node is required")?,
            self.input_data.unwrap_or_default(),
            self.static_data,
            self.run_id.unwrap_or_else(uuid::Uuid::new_v4),
            self.workflow_cache
                .unwrap_or_else(|| Arc::new(RwLock::new(HashMap::new()))),
            self.credential_provider,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use barqflow_core::traits::IExecuteFunctions;
    use barqflow_core::types::NodeId;
    use serde_json::json;
    use tokio::runtime::Builder;

    fn create_test_node(name: &str) -> INode {
        let mut params = HashMap::new();
        params.insert("testParam".to_string(), json!("testValue"));

        INode {
            id: NodeId::new(name),
            name: name.to_string(),
            r#type: "testNode".to_string(),
            type_version: 1.0,
            position: [0.0, 0.0],
            parameters: barqflow_core::schema::INodeParameters(params),
            credentials: vec![],
            disabled: false,
        }
    }

    struct MockCredentialProvider;

    #[async_trait]
    impl CredentialProvider for MockCredentialProvider {
        async fn get_credentials(
            &self,
            _node_id: &str,
            name: &str,
        ) -> Result<HashMap<String, GenericValue>, BarqError> {
            if name != "openAiApi" {
                return Err(BarqError::NodeOperationError {
                    node_name: "MockProvider".to_string(),
                    message: format!("unknown credential ref '{}'", name),
                });
            }

            Ok(HashMap::from([(
                "apiKey".to_string(),
                json!("test-key-123"),
            )]))
        }
    }

    #[tokio::test]
    async fn test_context_creation() {
        let node = create_test_node("TestNode");
        let context = NodeExecutionContext::new(
            node,
            ITaskDataConnections::default(),
            None,
            uuid::Uuid::new_v4(),
            Arc::new(RwLock::new(HashMap::new())),
        );

        assert_eq!(context.node.name, "TestNode");
    }

    #[tokio::test]
    async fn test_get_node_parameter() {
        let node = create_test_node("TestNode");
        let context = NodeExecutionContext::new(
            node,
            ITaskDataConnections::default(),
            None,
            uuid::Uuid::new_v4(),
            Arc::new(RwLock::new(HashMap::new())),
        );

        let result = context.get_node_parameter("testParam", None).await.unwrap();

        assert_eq!(result, json!("testValue"));
    }

    #[tokio::test]
    async fn test_get_node_parameter_with_fallback() {
        let node = create_test_node("TestNode");
        let context = NodeExecutionContext::new(
            node,
            ITaskDataConnections::default(),
            None,
            uuid::Uuid::new_v4(),
            Arc::new(RwLock::new(HashMap::new())),
        );

        let fallback = json!("fallbackValue");
        let result = context
            .get_node_parameter("nonExistentParam", Some(fallback))
            .await
            .unwrap();

        assert_eq!(result, json!("fallbackValue"));
    }

    #[tokio::test]
    async fn test_get_node_parameter_missing() {
        let node = create_test_node("TestNode");
        let context = NodeExecutionContext::new(
            node,
            ITaskDataConnections::default(),
            None,
            uuid::Uuid::new_v4(),
            Arc::new(RwLock::new(HashMap::new())),
        );

        let result = context.get_node_parameter("nonExistentParam", None).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_node() {
        let node = create_test_node("TestNode");
        let context = NodeExecutionContext::new(
            node.clone(),
            ITaskDataConnections::default(),
            None,
            uuid::Uuid::new_v4(),
            Arc::new(RwLock::new(HashMap::new())),
        );

        let retrieved_node = context.get_node();
        assert_eq!(retrieved_node.name, "TestNode");
        assert_eq!(retrieved_node.r#type, "testNode");
    }

    #[tokio::test]
    async fn test_get_input_data_inside_runtime_does_not_panic() {
        let node = create_test_node("InputNode");
        let mut input_data = ITaskDataConnections::new();
        input_data.push(
            0,
            vec![INodeExecutionData::new(IDataObject::from(
                json!({ "foo": "bar" }),
            ))],
        );

        let context = NodeExecutionContext::new(
            node,
            input_data,
            None,
            uuid::Uuid::new_v4(),
            Arc::new(RwLock::new(HashMap::new())),
        );

        let input = context.get_input_data(0).await.unwrap();
        assert_eq!(input.len(), 1);
    }

    #[tokio::test]
    async fn test_builder_pattern() {
        let node = create_test_node("BuilderNode");
        let run_id = uuid::Uuid::new_v4();

        let context = NodeExecutionContextBuilder::new()
            .with_node(node.clone())
            .with_input_data(ITaskDataConnections::default())
            .with_run_id(run_id)
            .build()
            .unwrap();

        assert_eq!(context.node.name, "BuilderNode");
    }

    #[tokio::test]
    async fn test_builder_missing_required_field() {
        let result = NodeExecutionContextBuilder::new()
            .with_input_data(ITaskDataConnections::default())
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("node is required"));
    }

    #[tokio::test]
    async fn test_expression_detection() {
        let mut params = HashMap::new();
        params.insert("exprParam".to_string(), json!("{{ json[\"someValue\"] }}"));

        let node = INode {
            id: NodeId::new("ExprNode"),
            name: "ExprNode".to_string(),
            r#type: "testNode".to_string(),
            type_version: 1.0,
            position: [0.0, 0.0],
            parameters: barqflow_core::schema::INodeParameters(params),
            credentials: vec![],
            disabled: false,
        };

        let context = NodeExecutionContext::new(
            node,
            ITaskDataConnections::default(),
            None,
            uuid::Uuid::new_v4(),
            Arc::new(RwLock::new(HashMap::new())),
        );
        let mut new_data = ITaskDataConnections::new();
        new_data.push(
            0,
            vec![INodeExecutionData::new(IDataObject::from(
                json!({ "someValue": "evalResult" }),
            ))],
        );
        context.update_input_data(new_data).await;

        let result = context.get_node_parameter("exprParam", None).await.unwrap();

        assert_eq!(result, json!("evalResult"));
    }

    #[tokio::test]
    async fn test_update_input_data() {
        let node = create_test_node("UpdateNode");
        let context = NodeExecutionContext::new(
            node,
            ITaskDataConnections::default(),
            None,
            uuid::Uuid::new_v4(),
            Arc::new(RwLock::new(HashMap::new())),
        );

        let mut new_data = ITaskDataConnections::new();
        new_data.push(0, vec![INodeExecutionData::new(IDataObject::new())]);

        context.update_input_data(new_data).await;

        // Verify the data was updated
        let input = context.input_data.read().await;
        assert!(input.0.contains_key(&0));
    }

    #[tokio::test]
    async fn test_get_credentials_uses_runtime_provider() {
        let node = create_test_node("CredNode");
        let context = NodeExecutionContext::new_with_credentials(
            node,
            ITaskDataConnections::default(),
            None,
            uuid::Uuid::new_v4(),
            Arc::new(RwLock::new(HashMap::new())),
            Some(Arc::new(MockCredentialProvider)),
        );

        let creds = context.get_credentials("openAiApi").await.unwrap();
        assert_eq!(
            creds.get("apiKey").and_then(|v| v.as_str()),
            Some("test-key-123")
        );
    }

    #[tokio::test]
    async fn test_get_credentials_without_provider_fails() {
        let node = create_test_node("NoProviderNode");
        let context = NodeExecutionContext::new(
            node,
            ITaskDataConnections::default(),
            None,
            uuid::Uuid::new_v4(),
            Arc::new(RwLock::new(HashMap::new())),
        );

        let err = context.get_credentials("openAiApi").await.unwrap_err();
        assert!(err
            .to_string()
            .contains("Credential provider is not configured"));
    }

    #[test]
    fn context_module_disallows_blocking_runtime_calls() {
        let source = include_str!("context.rs");
        let thread_spawn = format!("{}{}", "std::thread::", "spawn");
        let block_in_place = format!("{}{}", "tokio::task::", "block_in_place");

        assert!(
            !source.contains(&thread_spawn),
            "context module must not spawn OS threads directly"
        );
        assert!(
            !source.contains(&block_in_place),
            "context module must not block the runtime executor"
        );
    }

    #[test]
    fn concurrent_get_input_data_is_panic_free_on_multithread_runtime() {
        let rt = Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
            .expect("runtime should build");

        rt.block_on(async {
            let node = create_test_node("ConcurrentNode");
            let mut input_data = ITaskDataConnections::new();
            input_data.push(
                0,
                vec![INodeExecutionData::new(IDataObject::from(
                    json!({ "id": 1 }),
                ))],
            );

            let context = Arc::new(NodeExecutionContext::new(
                node,
                input_data,
                None,
                uuid::Uuid::new_v4(),
                Arc::new(RwLock::new(HashMap::new())),
            ));

            let mut tasks = Vec::new();
            for _ in 0..10 {
                let context = Arc::clone(&context);
                tasks.push(tokio::spawn(async move {
                    let items = context.get_input_data(0).await?;
                    Ok::<usize, BarqError>(items.len())
                }));
            }

            for task in tasks {
                let item_count = task
                    .await
                    .expect("task should not panic")
                    .expect("context read should succeed");
                assert_eq!(item_count, 1);
            }
        });
    }
}
