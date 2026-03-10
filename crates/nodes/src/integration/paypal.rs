use crate::integration::common::{
    build_standard_output, build_url, ensure_required_string, execute_prepared_request,
    get_optional_param, get_optional_string_param, get_string_param, get_u64_param, parse_body,
    parse_kv_pairs, resolve_auth_token, run_count, PreparedRequest,
};
use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::{IExecuteFunctions, INodeType};
use barqflow_core::types::IDataObject;
use reqwest::Client;
use serde_json::json;

pub struct PaypalNode {
    client: Client,
}

impl PaypalNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for PaypalNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for PaypalNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({
            "name": "PayPal",
            "description": "Create and capture PayPal orders"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let run_count = run_count(context).await;
        let mut output_items = Vec::new();

        for item_index in 0..run_count {
            let operation = get_string_param(context, "operation", item_index, "createOrder").await;
            let base_url = get_string_param(
                context,
                "baseUrl",
                item_index,
                "https://api-m.sandbox.paypal.com",
            )
            .await;
            let timeout_ms = get_u64_param(context, "timeout", item_index, 60_000).await;
            let auth_token =
                resolve_auth_token(context, "PayPal", item_index, "paypalApi", &["accessToken"])
                    .await?;

            let headers = get_optional_param(context, "headers", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();
            let query = get_optional_param(context, "queryParameters", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();

            let (method, url, body) = match operation.as_str() {
                "createOrder" => (
                    "POST".to_string(),
                    build_url(&base_url, "/v2/checkout/orders"),
                    Some(json!({
                        "intent": "CAPTURE",
                        "purchase_units": [{
                            "amount": {
                                "currency_code": "USD",
                                "value": "10.00"
                            }
                        }]
                    })),
                ),
                "captureOrder" => {
                    let order_id = ensure_required_string(
                        "PayPal",
                        "Order ID",
                        get_optional_string_param(context, "orderId", item_index).await,
                        "Provide the order ID to capture.",
                    )?;
                    (
                        "POST".to_string(),
                        build_url(
                            &base_url,
                            &format!("/v2/checkout/orders/{order_id}/capture"),
                        ),
                        None,
                    )
                }
                "apiCall" => {
                    let method = get_string_param(context, "method", item_index, "GET").await;
                    let resource_path = ensure_required_string(
                        "PayPal",
                        "Resource Path",
                        get_optional_string_param(context, "resourcePath", item_index).await,
                        "Provide a PayPal REST API path.",
                    )?;
                    let body = parse_body(get_optional_param(context, "body", item_index).await);
                    (method, build_url(&base_url, &resource_path), body)
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "PayPal".to_string(),
                        message: format!("Operation '{}' is not supported", operation),
                    });
                }
            };

            let response = execute_prepared_request(
                &self.client,
                "PayPal",
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
    async fn paypal_create_order_works() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/v2/checkout/orders")
            .match_header("authorization", "Bearer pp-token")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"ORDER-1"}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("PayPal", "barqflow-nodes.paypal");
        context.add_param("operation", json!("createOrder"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("authToken", json!("pp-token"));

        let result = PaypalNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(201)
        );
    }

    #[tokio::test]
    async fn paypal_uses_bound_credential_when_auth_token_is_empty() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/v2/checkout/orders")
            .match_header("authorization", "Bearer pp-token")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"ORDER-1"}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("PayPal", "barqflow-nodes.paypal");
        context.add_param("operation", json!("createOrder"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_credential("paypalApi", "accessToken", json!("pp-token"));

        let result = PaypalNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(201)
        );
    }
}
