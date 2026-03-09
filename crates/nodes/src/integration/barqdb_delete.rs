use crate::integration::common::{
    build_standard_output, build_url, ensure_required_string, execute_prepared_request,
    get_optional_param, get_optional_string_param, get_string_param, get_u64_param, parse_body,
    parse_kv_pairs, run_count, PreparedRequest,
};
use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::{IExecuteFunctions, INodeType};
use barqflow_core::types::IDataObject;
use reqwest::Client;
use serde_json::json;

pub struct BarqDbDeleteNode {
    client: Client,
}

impl BarqDbDeleteNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    async fn resolve_credentials(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<(String, String), BarqError> {
        let creds = context.get_credentials("barqDbApi").await?;
        let base_url = creds
            .get("baseUrl")
            .or_else(|| creds.get("host"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let api_key = creds
            .get("apiKey")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();

        let resolved_base_url = ensure_required_string(
            "BarqDB Delete",
            "Base URL",
            Some(base_url),
            "Add BarqDB credential with Base URL in /credentials and bind it to this node.",
        )?;
        let resolved_api_key = ensure_required_string(
            "BarqDB Delete",
            "API Key",
            Some(api_key),
            "Add BarqDB credential with API key in /credentials and bind it to this node.",
        )?;

        Ok((resolved_base_url, resolved_api_key))
    }
}

impl Default for BarqDbDeleteNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for BarqDbDeleteNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({
            "name": "BarqDB Delete",
            "description": "Delete vectors/documents from BarqDB by ID or filter"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let run_count = run_count(context).await;
        let mut output_items = Vec::new();
        let (credential_base_url, api_key) = self.resolve_credentials(context).await?;

        for item_index in 0..run_count {
            let operation = get_string_param(context, "operation", item_index, "deleteById").await;
            let base_url =
                get_string_param(context, "baseUrl", item_index, &credential_base_url).await;
            let timeout_ms = get_u64_param(context, "timeout", item_index, 60_000).await;

            let mut headers = get_optional_param(context, "headers", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();
            headers.push(("x-api-key".to_string(), api_key.clone()));
            headers.push(("Content-Type".to_string(), "application/json".to_string()));

            let query = get_optional_param(context, "queryParameters", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();

            let (method, url, body) = match operation.as_str() {
                "deleteById" => {
                    let collection = ensure_required_string(
                        "BarqDB Delete",
                        "Collection",
                        get_optional_string_param(context, "collection", item_index).await,
                        "Provide the BarqDB collection name.",
                    )?;
                    let id = ensure_required_string(
                        "BarqDB Delete",
                        "ID",
                        get_optional_string_param(context, "id", item_index).await,
                        "Provide the document/vector ID to delete.",
                    )?;
                    (
                        "DELETE".to_string(),
                        build_url(
                            &base_url,
                            &format!("/v1/collections/{collection}/items/{id}"),
                        ),
                        None,
                    )
                }
                "deleteByFilter" => {
                    let collection = ensure_required_string(
                        "BarqDB Delete",
                        "Collection",
                        get_optional_string_param(context, "collection", item_index).await,
                        "Provide the BarqDB collection name.",
                    )?;
                    let filter =
                        parse_body(get_optional_param(context, "filter", item_index).await)
                            .ok_or_else(|| BarqError::NodeOperationError {
                                node_name: "BarqDB Delete".to_string(),
                                message:
                                    "Missing Filter. Provide a JSON object with delete criteria."
                                        .to_string(),
                            })?;

                    (
                        "POST".to_string(),
                        build_url(
                            &base_url,
                            &format!("/v1/collections/{collection}/items/delete"),
                        ),
                        Some(json!({
                            "filter": filter,
                        })),
                    )
                }
                "apiCall" => {
                    let method = get_string_param(context, "method", item_index, "POST").await;
                    let resource_path = ensure_required_string(
                        "BarqDB Delete",
                        "Resource Path",
                        get_optional_string_param(context, "resourcePath", item_index).await,
                        "Provide a BarqDB API path.",
                    )?;
                    let body = parse_body(get_optional_param(context, "body", item_index).await);
                    (method, build_url(&base_url, &resource_path), body)
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "BarqDB Delete".to_string(),
                        message: format!("Operation '{}' is not supported", operation),
                    });
                }
            };

            let response = execute_prepared_request(
                &self.client,
                "BarqDB Delete",
                PreparedRequest {
                    method,
                    url,
                    headers,
                    query,
                    body,
                    auth_token: None,
                    timeout_ms,
                },
            )
            .await?;

            output_items.push(build_standard_output(&operation, response));
        }

        Ok(vec![output_items])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use barqflow_core::schema::{INode, INodeParameters};
    use barqflow_core::types::{GenericValue, NodeId};
    use mockito::Server;
    use std::collections::HashMap;

    struct MockContext {
        params: HashMap<String, GenericValue>,
        creds: HashMap<String, GenericValue>,
        node: INode,
    }

    impl MockContext {
        fn new() -> Self {
            Self {
                params: HashMap::new(),
                creds: HashMap::new(),
                node: INode {
                    id: NodeId("barqdb-delete".into()),
                    name: "BarqDB Delete".into(),
                    r#type: "barqflow-nodes.barqDbDelete".into(),
                    type_version: 1.0,
                    position: [0.0, 0.0],
                    parameters: INodeParameters(HashMap::new()),
                    credentials: vec![],
                    disabled: false,
                },
            }
        }

        fn add_param(&mut self, key: &str, value: serde_json::Value) {
            self.params.insert(key.to_string(), value);
        }

        fn add_credential(&mut self, key: &str, value: serde_json::Value) {
            self.creds.insert(key.to_string(), value);
        }
    }

    #[async_trait]
    impl IExecuteFunctions for MockContext {
        async fn get_node_parameter(
            &self,
            parameter_name: &str,
            fallback_value: Option<GenericValue>,
        ) -> Result<GenericValue, BarqError> {
            if let Some(value) = self.params.get(parameter_name) {
                Ok(value.clone())
            } else if let Some(fallback) = fallback_value {
                Ok(fallback)
            } else {
                Err(BarqError::NodeOperationError {
                    node_name: self.node.name.clone(),
                    message: format!("Parameter '{}' not found", parameter_name),
                })
            }
        }

        async fn get_node_parameter_at_item(
            &self,
            parameter_name: &str,
            _item_index: usize,
            fallback_value: Option<GenericValue>,
        ) -> Result<GenericValue, BarqError> {
            self.get_node_parameter(parameter_name, fallback_value)
                .await
        }

        fn get_node(&self) -> &INode {
            &self.node
        }

        async fn get_input_data(
            &self,
            _input_index: usize,
        ) -> Result<Vec<INodeExecutionData>, BarqError> {
            Ok(vec![])
        }

        async fn get_credentials(
            &self,
            _name: &str,
        ) -> Result<HashMap<String, GenericValue>, BarqError> {
            Ok(self.creds.clone())
        }

        fn log(&self, _message: &str) {}
    }

    #[tokio::test]
    async fn delete_node_calls_delete_endpoint() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("DELETE", "/v1/collections/rag_docs/items/doc-1")
            .match_header("x-api-key", "barq-key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"deleted":1}"#)
            .create_async()
            .await;

        let mut context = MockContext::new();
        context.add_param("operation", json!("deleteById"));
        context.add_param("collection", json!("rag_docs"));
        context.add_param("id", json!("doc-1"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_credential("baseUrl", json!(server.url()));
        context.add_credential("apiKey", json!("barq-key"));

        let result = BarqDbDeleteNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }
}
