use crate::credentials_provider::RepositoryCredentialProvider;
use crate::repositories::credential::CredentialRepository;
use crate::repositories::execution::ExecutionRepository;
use crate::repositories::workflow::WorkflowRepository;
use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::{
    INode, INodeConnections, INodeExecutionData, IWorkflowSettings, WorkflowDef,
};
use barqflow_core::types::{RunId, WorkflowId};
use barqflow_exec::runner::{
    ExecutionConfig, NodeExecutionResult, WorkflowRunContext, WorkflowRunner,
};
use barqflow_exec::subworkflow::{
    SubWorkflowExecutionResult, SubWorkflowExecutor, SubWorkflowParentContext,
};
use barqflow_registry::registry::NodeRegistry;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct RepositorySubWorkflowExecutor {
    workflow_repo: Arc<WorkflowRepository>,
    credential_repo: Arc<CredentialRepository>,
    node_registry: Arc<NodeRegistry>,
    execution_repo: Option<Arc<ExecutionRepository>>,
}

impl RepositorySubWorkflowExecutor {
    pub fn new(
        workflow_repo: Arc<WorkflowRepository>,
        credential_repo: Arc<CredentialRepository>,
        node_registry: Arc<NodeRegistry>,
    ) -> Self {
        Self {
            workflow_repo,
            credential_repo,
            node_registry,
            execution_repo: None,
        }
    }

    pub fn with_execution_repo(mut self, execution_repo: Arc<ExecutionRepository>) -> Self {
        self.execution_repo = Some(execution_repo);
        self
    }

    fn decode_child_workflow(
        &self,
        entity: &barqflow_db::models::WorkflowEntity,
    ) -> Result<(WorkflowDef, Vec<INode>), BarqError> {
        let nodes: Vec<INode> = serde_json::from_value(entity.nodes.clone()).map_err(|e| {
            BarqError::WorkflowConfigurationError {
                message: format!("Failed to parse child workflow nodes: {}", e),
            }
        })?;

        let connections: HashMap<String, INodeConnections> =
            serde_json::from_value(entity.connections.clone()).map_err(|e| {
                BarqError::WorkflowConfigurationError {
                    message: format!("Failed to parse child workflow connections: {}", e),
                }
            })?;

        let settings: IWorkflowSettings =
            serde_json::from_value(entity.settings.clone()).unwrap_or_default();

        let workflow = WorkflowDef {
            id: WorkflowId(entity.id),
            name: entity.name.clone(),
            nodes: nodes.clone(),
            connections: connections.into_iter().collect(),
            active: entity.active,
            settings,
        };

        Ok((workflow, nodes))
    }

    fn has_outgoing_connections(connections: &INodeConnections) -> bool {
        connections
            .0
            .values()
            .any(|outputs| outputs.iter().any(|branch| !branch.is_empty()))
    }

    fn terminal_outputs(
        &self,
        workflow: &WorkflowDef,
        results: &HashMap<String, NodeExecutionResult>,
    ) -> Vec<Vec<INodeExecutionData>> {
        let terminal_names: Vec<String> = workflow
            .nodes
            .iter()
            .filter(|node| {
                workflow
                    .connections
                    .get(&node.name)
                    .map(Self::has_outgoing_connections)
                    .unwrap_or(false)
                    == false
            })
            .map(|node| node.name.clone())
            .collect();

        let candidates = if terminal_names.is_empty() {
            workflow
                .nodes
                .last()
                .map(|node| vec![node.name.clone()])
                .unwrap_or_default()
        } else {
            terminal_names
        };

        let mut merged = Vec::new();
        for node_name in candidates {
            if let Some(node_result) = results.get(&node_name) {
                if let Some(stream) = node_result.outputs.first() {
                    merged.extend(stream.clone());
                }
            }
        }

        vec![merged]
    }

    fn summarize_results(
        &self,
        results: &HashMap<String, NodeExecutionResult>,
    ) -> (String, serde_json::Value) {
        let mut all_success = true;
        let mut summary = serde_json::Map::new();

        for (node_name, result) in results {
            if !result.success {
                all_success = false;
            }

            summary.insert(
                node_name.clone(),
                serde_json::json!({
                    "success": result.success,
                    "error": result.error,
                    "outputs": result.outputs,
                }),
            );
        }

        let status = if all_success { "success" } else { "failed" };
        (status.to_string(), serde_json::Value::Object(summary))
    }
}

#[async_trait]
impl SubWorkflowExecutor for RepositorySubWorkflowExecutor {
    async fn execute_subworkflow(
        &self,
        parent: SubWorkflowParentContext,
        child_workflow_id: Uuid,
        input: Vec<INodeExecutionData>,
    ) -> Result<SubWorkflowExecutionResult, BarqError> {
        let child_entity = self
            .workflow_repo
            .find_by_id(child_workflow_id)
            .await
            .map_err(|e| BarqError::SubworkflowError {
                child_execution_id: "unknown".to_string(),
                message: format!(
                    "Failed to load child workflow '{}': {}",
                    child_workflow_id, e
                ),
            })?
            .ok_or_else(|| BarqError::SubworkflowError {
                child_execution_id: "unknown".to_string(),
                message: format!("Child workflow '{}' not found", child_workflow_id),
            })?;

        let (child_workflow, child_nodes) = self.decode_child_workflow(&child_entity)?;

        let child_execution_entity = if let Some(repo) = self.execution_repo.as_ref() {
            Some(
                repo.create(
                    child_workflow_id,
                    "running",
                    serde_json::json!({
                        "parentExecutionId": parent.execution_id.map(|id| id.to_string()),
                        "parentRunId": parent.run_id.to_string(),
                        "inputCount": input.len(),
                    }),
                )
                .await
                .map_err(|e| BarqError::SubworkflowError {
                    child_execution_id: "unknown".to_string(),
                    message: format!("Failed to create child execution row: {}", e),
                })?,
            )
        } else {
            None
        };

        let child_execution_id = child_execution_entity
            .as_ref()
            .map(|entity| entity.id.to_string())
            .unwrap_or_else(|| RunId::new().to_string());

        let credential_provider = Arc::new(RepositoryCredentialProvider::new(
            Arc::clone(&self.credential_repo),
            &child_nodes,
        ));

        let child_runner =
            WorkflowRunner::new(Arc::clone(&self.node_registry), ExecutionConfig::default())
                .with_credential_provider(credential_provider)
                .with_subworkflow_executor(Arc::new(self.clone()));

        let mut static_map = serde_json::Map::new();
        static_map.insert(
            "subworkflowInput".to_string(),
            serde_json::to_value(&input).unwrap_or(serde_json::Value::Null),
        );

        let child_context = WorkflowRunContext {
            run_id: RunId::new(),
            workflow: child_workflow.clone(),
            static_data: Some(barqflow_core::types::IDataObject::from(
                serde_json::Value::Object(static_map),
            )),
            manual: parent.manual,
            execution_id: child_execution_entity.as_ref().map(|entity| entity.id),
            parent_execution_id: parent.execution_id.or(parent.parent_execution_id),
            cancellation_token: None,
            stop_after_node_id: None,
            event_sequence_start: 0,
        };

        let run_result = child_runner.run_workflow(child_context).await;

        match run_result {
            Ok(results) => {
                let outputs = self.terminal_outputs(&child_workflow, &results);

                if let (Some(repo), Some(child_execution)) = (
                    self.execution_repo.as_ref(),
                    child_execution_entity.as_ref(),
                ) {
                    let (status, data) = self.summarize_results(&results);
                    let _ = repo
                        .update_status_and_data(child_execution.id, status.as_str(), data)
                        .await;
                }

                Ok(SubWorkflowExecutionResult {
                    child_execution_id,
                    outputs,
                })
            }
            Err(err) => {
                if let (Some(repo), Some(child_execution)) = (
                    self.execution_repo.as_ref(),
                    child_execution_entity.as_ref(),
                ) {
                    let _ = repo
                        .update_status_and_data(
                            child_execution.id,
                            "failed",
                            serde_json::json!({
                                "error": err.to_string(),
                            }),
                        )
                        .await;
                }

                Err(BarqError::SubworkflowError {
                    child_execution_id,
                    message: err.to_string(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::workflow::WorkflowRepository;
    use barqflow_core::schema::{
        IConnection, INode, INodeConnections, INodeParameters, IWorkflowSettings,
        NodeConnectionType,
    };
    use barqflow_core::types::NodeId;
    use barqflow_exec::runner::WorkflowRunContext;
    use serde_json::json;
    use sqlx::PgPool;

    fn manual_trigger_node(id: &str, name: &str) -> INode {
        INode {
            id: NodeId::new(id),
            name: name.to_string(),
            r#type: "n8n-nodes-base.manualTrigger".to_string(),
            type_version: 1.0,
            position: [0.0, 0.0],
            parameters: INodeParameters::default(),
            credentials: vec![],
            disabled: false,
        }
    }

    fn set_node(id: &str, name: &str, assignments: serde_json::Value) -> INode {
        INode {
            id: NodeId::new(id),
            name: name.to_string(),
            r#type: "n8n-nodes-base.set".to_string(),
            type_version: 1.0,
            position: [260.0, 0.0],
            parameters: INodeParameters(HashMap::from([("assignments".to_string(), assignments)])),
            credentials: vec![],
            disabled: false,
        }
    }

    fn execute_workflow_node(id: &str, name: &str, workflow_id: Uuid) -> INode {
        INode {
            id: NodeId::new(id),
            name: name.to_string(),
            r#type: "barqflow-nodes.executeWorkflow".to_string(),
            type_version: 1.0,
            position: [260.0, 0.0],
            parameters: INodeParameters(HashMap::from([
                ("workflowId".to_string(), json!(workflow_id.to_string())),
                ("mode".to_string(), json!("wait")),
            ])),
            credentials: vec![],
            disabled: false,
        }
    }

    fn single_main_connection(from: &str, to: &str) -> HashMap<String, INodeConnections> {
        HashMap::from([(
            from.to_string(),
            INodeConnections(HashMap::from([(
                NodeConnectionType::Main,
                vec![vec![IConnection {
                    node: to.to_string(),
                    r#type: NodeConnectionType::Main,
                    index: 0,
                }]],
            )])),
        )])
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn executes_child_workflow_and_returns_terminal_output(pool: PgPool) {
        let workflow_repo = Arc::new(WorkflowRepository::new(pool.clone()));
        let credential_repo = Arc::new(CredentialRepository::new(pool));
        let node_registry = Arc::new(NodeRegistry::new());
        barqflow_nodes::register_all_nodes(&node_registry);

        let child_nodes = vec![
            manual_trigger_node("child_trigger", "Child Trigger"),
            set_node(
                "child_set",
                "Child Set",
                json!([
                    {"name":"fromChild","value":"yes","type":"string"}
                ]),
            ),
        ];
        let child_connections = single_main_connection("Child Trigger", "Child Set");

        let child_entity = workflow_repo
            .create(
                "Child Flow",
                serde_json::to_value(&child_nodes).unwrap(),
                serde_json::to_value(&child_connections).unwrap(),
                json!({}),
            )
            .await
            .unwrap();

        let parent_nodes = vec![
            manual_trigger_node("parent_trigger", "Parent Trigger"),
            execute_workflow_node("exec_child", "Execute Child", child_entity.id),
        ];
        let parent_connections = single_main_connection("Parent Trigger", "Execute Child");

        let parent_workflow = WorkflowDef {
            id: WorkflowId::new(),
            name: "Parent".to_string(),
            nodes: parent_nodes.clone(),
            connections: parent_connections.into_iter().collect(),
            active: true,
            settings: IWorkflowSettings::default(),
        };

        let subworkflow_executor = Arc::new(RepositorySubWorkflowExecutor::new(
            Arc::clone(&workflow_repo),
            Arc::clone(&credential_repo),
            Arc::clone(&node_registry),
        ));

        let runner = WorkflowRunner::new(Arc::clone(&node_registry), ExecutionConfig::default())
            .with_subworkflow_executor(subworkflow_executor);

        let result = runner
            .run_workflow(WorkflowRunContext {
                run_id: RunId::new(),
                workflow: parent_workflow,
                static_data: None,
                manual: true,
                execution_id: None,
                parent_execution_id: None,
                cancellation_token: None,
                stop_after_node_id: None,
                event_sequence_start: 0,
            })
            .await
            .unwrap();

        let execute_result = result.get("Execute Child").expect("Execute Child result");
        let output_item = execute_result
            .outputs
            .first()
            .and_then(|stream| stream.first())
            .expect("child output item");
        assert_eq!(
            output_item.json.0.get("fromChild").and_then(|v| v.as_str()),
            Some("yes")
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn missing_child_workflow_returns_subworkflow_error(pool: PgPool) {
        let workflow_repo = Arc::new(WorkflowRepository::new(pool.clone()));
        let credential_repo = Arc::new(CredentialRepository::new(pool));
        let node_registry = Arc::new(NodeRegistry::new());
        barqflow_nodes::register_all_nodes(&node_registry);

        let parent_nodes = vec![
            manual_trigger_node("parent_trigger", "Parent Trigger"),
            execute_workflow_node("exec_child", "Execute Child", Uuid::new_v4()),
        ];
        let parent_connections = single_main_connection("Parent Trigger", "Execute Child");

        let parent_workflow = WorkflowDef {
            id: WorkflowId::new(),
            name: "Parent".to_string(),
            nodes: parent_nodes,
            connections: parent_connections.into_iter().collect(),
            active: true,
            settings: IWorkflowSettings::default(),
        };

        let subworkflow_executor = Arc::new(RepositorySubWorkflowExecutor::new(
            Arc::clone(&workflow_repo),
            Arc::clone(&credential_repo),
            Arc::clone(&node_registry),
        ));

        let runner = WorkflowRunner::new(Arc::clone(&node_registry), ExecutionConfig::default())
            .with_subworkflow_executor(subworkflow_executor);

        let err = runner
            .run_workflow(WorkflowRunContext {
                run_id: RunId::new(),
                workflow: parent_workflow,
                static_data: None,
                manual: true,
                execution_id: None,
                parent_execution_id: None,
                cancellation_token: None,
                stop_after_node_id: None,
                event_sequence_start: 0,
            })
            .await
            .unwrap_err();

        match err {
            BarqError::SubworkflowError { message, .. } => {
                assert!(message.contains("not found"));
            }
            other => panic!("expected SubworkflowError, got {}", other),
        }
    }
}
