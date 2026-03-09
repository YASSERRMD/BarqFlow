use crate::integration::common::{
    build_standard_output, build_url, ensure_required_string, execute_prepared_request,
    get_optional_param, get_optional_string_param, get_string_param, get_u64_param, parse_body,
    parse_kv_pairs, require_auth_token, run_count, PreparedRequest,
};
use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::{IExecuteFunctions, INodeType};
use barqflow_core::types::IDataObject;
use reqwest::Client;
use serde_json::json;

pub struct AirtableNode {
    client: Client,
}

impl AirtableNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for AirtableNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for AirtableNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({
            "name": "Airtable",
            "description": "Interact with Airtable records"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let mut output_items = Vec::new();
        let run_count = run_count(context).await;

        for item_index in 0..run_count {
            let operation = get_string_param(context, "operation", item_index, "listRecords").await;
            let base_url =
                get_string_param(context, "baseUrl", item_index, "https://api.airtable.com").await;
            let timeout_ms = get_u64_param(context, "timeout", item_index, 60_000).await;
            let auth_token = require_auth_token(
                "Airtable",
                get_optional_string_param(context, "authToken", item_index).await,
            )?;

            let mut headers = get_optional_param(context, "headers", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();
            headers.push(("Content-Type".to_string(), "application/json".to_string()));

            let mut query = get_optional_param(context, "queryParameters", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();

            let base_id = get_optional_string_param(context, "baseId", item_index).await;
            let table = get_optional_string_param(context, "table", item_index).await;

            let (method, url, body) = match operation.as_str() {
                "listRecords" => {
                    let base_id = ensure_required_string(
                        "Airtable",
                        "Base ID",
                        base_id.clone(),
                        "Provide the Airtable base ID.",
                    )?;
                    let table = ensure_required_string(
                        "Airtable",
                        "Table",
                        table.clone(),
                        "Provide the Airtable table name.",
                    )?;

                    (
                        "GET".to_string(),
                        build_url(&base_url, &format!("/v0/{base_id}/{table}")),
                        None,
                    )
                }
                "createRecord" => {
                    let base_id = ensure_required_string(
                        "Airtable",
                        "Base ID",
                        base_id.clone(),
                        "Provide the Airtable base ID.",
                    )?;
                    let table = ensure_required_string(
                        "Airtable",
                        "Table",
                        table.clone(),
                        "Provide the Airtable table name.",
                    )?;
                    let fields =
                        parse_body(get_optional_param(context, "fields", item_index).await)
                            .ok_or_else(|| BarqError::NodeOperationError {
                                node_name: "Airtable".to_string(),
                                message: "Missing Fields. Provide a JSON object for record fields."
                                    .to_string(),
                            })?;
                    (
                        "POST".to_string(),
                        build_url(&base_url, &format!("/v0/{base_id}/{table}")),
                        Some(json!({ "fields": fields })),
                    )
                }
                "updateRecord" => {
                    let base_id = ensure_required_string(
                        "Airtable",
                        "Base ID",
                        base_id.clone(),
                        "Provide the Airtable base ID.",
                    )?;
                    let table = ensure_required_string(
                        "Airtable",
                        "Table",
                        table.clone(),
                        "Provide the Airtable table name.",
                    )?;
                    let record_id = ensure_required_string(
                        "Airtable",
                        "Record ID",
                        get_optional_string_param(context, "recordId", item_index).await,
                        "Provide the Airtable record ID.",
                    )?;
                    let fields =
                        parse_body(get_optional_param(context, "fields", item_index).await)
                            .ok_or_else(|| BarqError::NodeOperationError {
                                node_name: "Airtable".to_string(),
                                message: "Missing Fields. Provide a JSON object for record fields."
                                    .to_string(),
                            })?;
                    (
                        "PATCH".to_string(),
                        build_url(&base_url, &format!("/v0/{base_id}/{table}/{record_id}")),
                        Some(json!({ "fields": fields })),
                    )
                }
                "apiCall" => {
                    let method = get_string_param(context, "method", item_index, "GET").await;
                    let resource_path = ensure_required_string(
                        "Airtable",
                        "Resource Path",
                        get_optional_string_param(context, "resourcePath", item_index).await,
                        "Provide an API path like /v0/{baseId}/{table}.",
                    )?;
                    query = get_optional_param(context, "queryParameters", item_index)
                        .await
                        .map(|v| parse_kv_pairs(&v))
                        .unwrap_or_default();
                    let body = parse_body(get_optional_param(context, "body", item_index).await);
                    (method, build_url(&base_url, &resource_path), body)
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "Airtable".to_string(),
                        message: format!("Operation '{}' is not supported", operation),
                    });
                }
            };

            let response = execute_prepared_request(
                &self.client,
                "Airtable",
                PreparedRequest {
                    method,
                    url,
                    headers,
                    query,
                    body,
                    auth_token: Some(auth_token),
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
    use crate::integration::common::test_utils::MockContext;
    use mockito::Server;

    #[tokio::test]
    async fn airtable_list_records_works() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/v0/app123/Table1")
            .match_header("authorization", "Bearer airtable-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"records":[]}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("Airtable", "barqflow-nodes.airtable");
        context.add_param("operation", json!("listRecords"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("authToken", json!("airtable-token"));
        context.add_param("baseId", json!("app123"));
        context.add_param("table", json!("Table1"));

        let result = AirtableNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }

    #[tokio::test]
    async fn airtable_requires_auth_token() {
        let mut context = MockContext::new("Airtable", "barqflow-nodes.airtable");
        context.add_param("operation", json!("listRecords"));
        context.add_param("baseId", json!("app123"));
        context.add_param("table", json!("Table1"));

        let err = AirtableNode::new().execute(&context).await.unwrap_err();
        assert!(err.to_string().contains("Auth Token"));
    }
}
