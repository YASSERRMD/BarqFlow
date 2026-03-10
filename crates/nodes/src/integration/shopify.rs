use crate::integration::common::{
    build_standard_output, build_url, ensure_required_string, execute_prepared_request,
    get_optional_param, get_optional_string_param, get_string_param, get_u64_param, parse_body,
    parse_kv_pairs, resolve_parameter_from_node_or_credentials, run_count, PreparedRequest,
};
use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::{IExecuteFunctions, INodeType};
use barqflow_core::types::IDataObject;
use reqwest::Client;
use serde_json::json;

pub struct ShopifyNode {
    client: Client,
}

impl ShopifyNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for ShopifyNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for ShopifyNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({
            "name": "Shopify",
            "description": "List and create Shopify products"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let run_count = run_count(context).await;
        let mut output_items = Vec::new();

        for item_index in 0..run_count {
            let operation =
                get_string_param(context, "operation", item_index, "listProducts").await;
            let base_url = get_string_param(
                context,
                "baseUrl",
                item_index,
                "https://your-store.myshopify.com",
            )
            .await;
            let timeout_ms = get_u64_param(context, "timeout", item_index, 60_000).await;
            let access_token = resolve_parameter_from_node_or_credentials(
                context,
                "Shopify",
                "accessToken",
                "Access Token",
                item_index,
                "shopifyApi",
                &["accessToken"],
            )
            .await?;

            let mut headers = get_optional_param(context, "headers", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();
            headers.push(("X-Shopify-Access-Token".to_string(), access_token));
            headers.push(("Content-Type".to_string(), "application/json".to_string()));

            let query = get_optional_param(context, "queryParameters", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();

            let (method, url, body) = match operation.as_str() {
                "listProducts" => (
                    "GET".to_string(),
                    build_url(&base_url, "/admin/api/2024-01/products.json"),
                    None,
                ),
                "createProduct" => {
                    let title = ensure_required_string(
                        "Shopify",
                        "Title",
                        get_optional_string_param(context, "title", item_index).await,
                        "Provide the product title.",
                    )?;
                    (
                        "POST".to_string(),
                        build_url(&base_url, "/admin/api/2024-01/products.json"),
                        Some(json!({
                            "product": {
                                "title": title,
                            }
                        })),
                    )
                }
                "apiCall" => {
                    let method = get_string_param(context, "method", item_index, "GET").await;
                    let resource_path = ensure_required_string(
                        "Shopify",
                        "Resource Path",
                        get_optional_string_param(context, "resourcePath", item_index).await,
                        "Provide a Shopify Admin API path.",
                    )?;
                    let body = parse_body(get_optional_param(context, "body", item_index).await);
                    (method, build_url(&base_url, &resource_path), body)
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "Shopify".to_string(),
                        message: format!("Operation '{}' is not supported", operation),
                    });
                }
            };

            let response = execute_prepared_request(
                &self.client,
                "Shopify",
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
    use crate::integration::common::test_utils::MockContext;
    use mockito::Server;

    #[tokio::test]
    async fn shopify_list_products_works() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/admin/api/2024-01/products.json")
            .match_header("x-shopify-access-token", "shopify-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"products":[]}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("Shopify", "barqflow-nodes.shopify");
        context.add_param("operation", json!("listProducts"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("accessToken", json!("shopify-token"));

        let result = ShopifyNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }

    #[tokio::test]
    async fn shopify_uses_bound_credential_when_access_token_is_missing() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/admin/api/2024-01/products.json")
            .match_header("x-shopify-access-token", "shopify-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"products":[]}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("Shopify", "barqflow-nodes.shopify");
        context.add_param("operation", json!("listProducts"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_credential("shopifyApi", "accessToken", json!("shopify-token"));

        let result = ShopifyNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }
}
