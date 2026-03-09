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

pub struct StripeNode {
    client: Client,
}

impl StripeNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    fn resource_path(resource: &str) -> &'static str {
        match resource {
            "charge" => "charges",
            _ => "customers",
        }
    }
}

impl Default for StripeNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for StripeNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({
            "name": "Stripe",
            "description": "Interact with Stripe resources"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let mut output_items = Vec::new();
        let run_count = run_count(context).await;

        for item_index in 0..run_count {
            let resource = get_string_param(context, "resource", item_index, "customer").await;
            let operation = get_string_param(context, "operation", item_index, "list").await;
            let base_url =
                get_string_param(context, "baseUrl", item_index, "https://api.stripe.com").await;
            let timeout_ms = get_u64_param(context, "timeout", item_index, 60_000).await;
            let auth_token = require_auth_token(
                "Stripe",
                get_optional_string_param(context, "authToken", item_index).await,
            )?;

            let mut headers = get_optional_param(context, "headers", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();
            headers.push(("Accept".to_string(), "application/json".to_string()));

            let query = get_optional_param(context, "queryParameters", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();

            let resource_segment = Self::resource_path(resource.as_str());

            let (method, url, body) = match operation.as_str() {
                "list" => (
                    "GET".to_string(),
                    build_url(&base_url, &format!("/v1/{resource_segment}")),
                    None,
                ),
                "retrieve" => {
                    let resource_id = ensure_required_string(
                        "Stripe",
                        "Resource ID",
                        get_optional_string_param(context, "resourceId", item_index).await,
                        "Set the target resource ID.",
                    )?;
                    (
                        "GET".to_string(),
                        build_url(&base_url, &format!("/v1/{resource_segment}/{resource_id}")),
                        None,
                    )
                }
                "apiCall" => {
                    let method = get_string_param(context, "method", item_index, "GET").await;
                    let resource_path = ensure_required_string(
                        "Stripe",
                        "Resource Path",
                        get_optional_string_param(context, "resourcePath", item_index).await,
                        "Provide an API path like /v1/customers.",
                    )?;
                    let body = parse_body(get_optional_param(context, "body", item_index).await);
                    (method, build_url(&base_url, &resource_path), body)
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "Stripe".to_string(),
                        message: format!("Operation '{}' is not supported", operation),
                    });
                }
            };

            let response = execute_prepared_request(
                &self.client,
                "Stripe",
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
    async fn stripe_list_customers_works() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/v1/customers")
            .match_header("authorization", "Bearer sk_test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"data":[]}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("Stripe", "barqflow-nodes.stripe");
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("resource", json!("customer"));
        context.add_param("operation", json!("list"));
        context.add_param("authToken", json!("sk_test"));

        let result = StripeNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }

    #[tokio::test]
    async fn stripe_requires_auth_token() {
        let mut context = MockContext::new("Stripe", "barqflow-nodes.stripe");
        context.add_param("resource", json!("customer"));
        context.add_param("operation", json!("list"));

        let err = StripeNode::new().execute(&context).await.unwrap_err();
        assert!(err.to_string().contains("Auth Token"));
    }
}
