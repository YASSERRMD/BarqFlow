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
use base64::{engine::general_purpose::STANDARD, Engine as _};
use reqwest::Client;
use serde_json::json;

pub struct TwilioNode {
    client: Client,
}

impl TwilioNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for TwilioNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for TwilioNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({"name":"Twilio","description":"Send SMS and call Twilio API"}))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let run_count = run_count(context).await;
        let mut output_items = Vec::new();

        for item_index in 0..run_count {
            let operation = get_string_param(context, "operation", item_index, "sendSms").await;
            let base_url =
                get_string_param(context, "baseUrl", item_index, "https://api.twilio.com").await;
            let timeout_ms = get_u64_param(context, "timeout", item_index, 60_000).await;

            let account_sid = ensure_required_string(
                "Twilio",
                "Account SID",
                get_optional_string_param(context, "accountSid", item_index).await,
                "Set Twilio account SID.",
            )?;
            let auth_token = ensure_required_string(
                "Twilio",
                "Auth Token",
                get_optional_string_param(context, "authToken", item_index).await,
                "Set Twilio auth token.",
            )?;
            let auth_header = format!(
                "Basic {}",
                STANDARD.encode(format!("{}:{}", account_sid, auth_token))
            );

            let mut headers = get_optional_param(context, "headers", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();
            headers.push(("Authorization".to_string(), auth_header));

            let query = get_optional_param(context, "queryParameters", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();

            let (method, url, body) = match operation.as_str() {
                "sendSms" => {
                    let to = ensure_required_string(
                        "Twilio",
                        "To",
                        get_optional_string_param(context, "to", item_index).await,
                        "Set recipient phone number.",
                    )?;
                    let from = ensure_required_string(
                        "Twilio",
                        "From",
                        get_optional_string_param(context, "from", item_index).await,
                        "Set Twilio sender number.",
                    )?;
                    let message = ensure_required_string(
                        "Twilio",
                        "Message",
                        get_optional_string_param(context, "message", item_index).await,
                        "Set SMS body text.",
                    )?;
                    (
                        "POST".to_string(),
                        build_url(
                            &base_url,
                            &format!("/2010-04-01/Accounts/{}/Messages.json", account_sid),
                        ),
                        Some(json!({"To": to, "From": from, "Body": message})),
                    )
                }
                "apiCall" => {
                    let method = get_string_param(context, "method", item_index, "GET").await;
                    let resource_path = ensure_required_string(
                        "Twilio",
                        "Resource Path",
                        get_optional_string_param(context, "resourcePath", item_index).await,
                        "Provide Twilio resource path.",
                    )?;
                    let body = parse_body(get_optional_param(context, "body", item_index).await);
                    (method, build_url(&base_url, &resource_path), body)
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "Twilio".to_string(),
                        message: format!("Operation '{}' is not supported", operation),
                    })
                }
            };

            let response = execute_prepared_request(
                &self.client,
                "Twilio",
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
    use mockito::{Matcher, Server};

    #[tokio::test]
    async fn twilio_send_sms_works() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/2010-04-01/Accounts/AC123/Messages.json")
            .match_header("authorization", Matcher::Regex("Basic .+".to_string()))
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(r#"{"sid":"SM1"}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("Twilio", "barqflow-nodes.twilio");
        context.add_param("operation", json!("sendSms"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("accountSid", json!("AC123"));
        context.add_param("authToken", json!("secret"));
        context.add_param("to", json!("+1555111"));
        context.add_param("from", json!("+1555222"));
        context.add_param("message", json!("hello"));

        let result = TwilioNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(201)
        );
    }
}
