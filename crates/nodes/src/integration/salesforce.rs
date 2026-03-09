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

pub struct SalesforceNode {
    client: Client,
}

impl SalesforceNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    fn object_name(resource: &str) -> &'static str {
        match resource {
            "account" => "Account",
            _ => "Contact",
        }
    }
}

impl Default for SalesforceNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for SalesforceNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({
            "name": "Salesforce",
            "description": "Read/create Salesforce sObject records"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let run_count = run_count(context).await;
        let mut output_items = Vec::new();

        for item_index in 0..run_count {
            let resource = get_string_param(context, "resource", item_index, "contact").await;
            let operation = get_string_param(context, "operation", item_index, "get").await;
            let base_url =
                get_string_param(context, "baseUrl", item_index, "https://your-instance.salesforce.com").await;
            let api_version = get_string_param(context, "apiVersion", item_index, "v59.0").await;
            let timeout_ms = get_u64_param(context, "timeout", item_index, 60_000).await;
            let auth_token = require_auth_token(
                "Salesforce",
                get_optional_string_param(context, "authToken", item_index).await,
            )?;

            let headers = get_optional_param(context, "headers", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();
            let query = get_optional_param(context, "queryParameters", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();

            let object = Self::object_name(&resource);

            let (method, url, body) = match operation.as_str() {
                "get" => {
                    let record_id = ensure_required_string(
                        "Salesforce",
                        "Record ID",
                        get_optional_string_param(context, "recordId", item_index).await,
                        "Provide Salesforce record ID.",
                    )?;
                    (
                        "GET".to_string(),
                        build_url(
                            &base_url,
                            &format!("/services/data/{api_version}/sobjects/{object}/{record_id}"),
                        ),
                        None,
                    )
                }
                "create" => {
                    let fields = parse_body(get_optional_param(context, "fields", item_index).await)
                        .ok_or_else(|| BarqError::NodeOperationError {
                            node_name: "Salesforce".to_string(),
                            message: "Missing Fields. Provide a JSON object for record fields."
                                .to_string(),
                        })?;
                    (
                        "POST".to_string(),
                        build_url(
                            &base_url,
                            &format!("/services/data/{api_version}/sobjects/{object}"),
                        ),
                        Some(fields),
                    )
                }
                "apiCall" => {
                    let method = get_string_param(context, "method", item_index, "GET").await;
                    let resource_path = ensure_required_string(
                        "Salesforce",
                        "Resource Path",
                        get_optional_string_param(context, "resourcePath", item_index).await,
                        "Provide Salesforce API path.",
                    )?;
                    let body = parse_body(get_optional_param(context, "body", item_index).await);
                    (method, build_url(&base_url, &resource_path), body)
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "Salesforce".to_string(),
                        message: format!("Operation '{}' is not supported", operation),
                    });
                }
            };

            let response = execute_prepared_request(
                &self.client,
                "Salesforce",
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
    async fn salesforce_get_record_works() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/services/data/v59.0/sobjects/Contact/003xx")
            .match_header("authorization", "Bearer sf-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"Id":"003xx"}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("Salesforce", "barqflow-nodes.salesforce");
        context.add_param("operation", json!("get"));
        context.add_param("resource", json!("contact"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("authToken", json!("sf-token"));
        context.add_param("recordId", json!("003xx"));

        let result = SalesforceNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(result[0][0].json.0.get("status").and_then(|v| v.as_u64()), Some(200));
    }

    #[tokio::test]
    async fn salesforce_requires_auth_token() {
        let mut context = MockContext::new("Salesforce", "barqflow-nodes.salesforce");
        context.add_param("operation", json!("get"));
        context.add_param("recordId", json!("003xx"));

        let err = SalesforceNode::new().execute(&context).await.unwrap_err();
        assert!(err.to_string().contains("Auth Token"));
    }
}
