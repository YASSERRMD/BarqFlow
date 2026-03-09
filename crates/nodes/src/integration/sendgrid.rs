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

pub struct SendGridNode {
    client: Client,
}

impl SendGridNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for SendGridNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for SendGridNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({
            "name": "SendGrid",
            "description": "Send emails through SendGrid"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let mut output_items = Vec::new();
        let run_count = run_count(context).await;

        for item_index in 0..run_count {
            let operation = get_string_param(context, "operation", item_index, "sendEmail").await;
            let base_url =
                get_string_param(context, "baseUrl", item_index, "https://api.sendgrid.com").await;
            let timeout_ms = get_u64_param(context, "timeout", item_index, 60_000).await;
            let auth_token = resolve_auth_token(
                context,
                "SendGrid",
                item_index,
                "sendGridApi",
                &["apiKey", "accessToken"],
            )
            .await?;

            let mut headers = get_optional_param(context, "headers", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();
            headers.push(("Content-Type".to_string(), "application/json".to_string()));

            let query = get_optional_param(context, "queryParameters", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();

            let (method, url, body) = match operation.as_str() {
                "sendEmail" => {
                    let to_email = ensure_required_string(
                        "SendGrid",
                        "Recipient Email",
                        get_optional_string_param(context, "toEmail", item_index).await,
                        "Set the recipient email address.",
                    )?;
                    let from_email = ensure_required_string(
                        "SendGrid",
                        "From Email",
                        get_optional_string_param(context, "fromEmail", item_index).await,
                        "Set the sender email address.",
                    )?;
                    let subject = ensure_required_string(
                        "SendGrid",
                        "Subject",
                        get_optional_string_param(context, "subject", item_index).await,
                        "Set the email subject.",
                    )?;
                    let content = ensure_required_string(
                        "SendGrid",
                        "Content",
                        get_optional_string_param(context, "content", item_index).await,
                        "Set the email body content.",
                    )?;

                    (
                        "POST".to_string(),
                        build_url(&base_url, "/v3/mail/send"),
                        Some(json!({
                            "personalizations": [{ "to": [{ "email": to_email }] }],
                            "from": { "email": from_email },
                            "subject": subject,
                            "content": [{
                                "type": "text/plain",
                                "value": content,
                            }]
                        })),
                    )
                }
                "apiCall" => {
                    let method = get_string_param(context, "method", item_index, "GET").await;
                    let resource_path = ensure_required_string(
                        "SendGrid",
                        "Resource Path",
                        get_optional_string_param(context, "resourcePath", item_index).await,
                        "Provide an API path like /v3/suppression/bounces.",
                    )?;
                    let body = parse_body(get_optional_param(context, "body", item_index).await);
                    (method, build_url(&base_url, &resource_path), body)
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "SendGrid".to_string(),
                        message: format!("Operation '{}' is not supported", operation),
                    });
                }
            };

            let response = execute_prepared_request(
                &self.client,
                "SendGrid",
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
    async fn sendgrid_send_email_executes_request() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/v3/mail/send")
            .match_header("authorization", "Bearer sg_test")
            .with_status(202)
            .create_async()
            .await;

        let mut context = MockContext::new("SendGrid", "barqflow-nodes.sendGrid");
        context.add_param("operation", json!("sendEmail"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("authToken", json!("sg_test"));
        context.add_param("fromEmail", json!("from@example.com"));
        context.add_param("toEmail", json!("to@example.com"));
        context.add_param("subject", json!("Hello"));
        context.add_param("content", json!("Body"));

        let result = SendGridNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(202)
        );
    }

    #[tokio::test]
    async fn sendgrid_uses_bound_credential_when_auth_token_is_empty() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/v3/mail/send")
            .match_header("authorization", "Bearer sg_test")
            .with_status(202)
            .create_async()
            .await;

        let mut context = MockContext::new("SendGrid", "barqflow-nodes.sendGrid");
        context.add_param("operation", json!("sendEmail"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("fromEmail", json!("from@example.com"));
        context.add_param("toEmail", json!("to@example.com"));
        context.add_param("subject", json!("Hello"));
        context.add_param("content", json!("Body"));
        context.add_credential("sendGridApi", "apiKey", json!("sg_test"));

        let result = SendGridNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(202)
        );
    }

    #[tokio::test]
    async fn sendgrid_requires_auth_token() {
        let mut context = MockContext::new("SendGrid", "barqflow-nodes.sendGrid");
        context.add_param("operation", json!("sendEmail"));
        context.add_param("fromEmail", json!("from@example.com"));
        context.add_param("toEmail", json!("to@example.com"));
        context.add_param("subject", json!("Hello"));
        context.add_param("content", json!("Body"));

        let err = SendGridNode::new().execute(&context).await.unwrap_err();
        assert!(err.to_string().contains("Auth Token"));
    }
}
