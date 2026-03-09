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

pub struct SheetsNode {
    client: Client,
}

impl SheetsNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for SheetsNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for SheetsNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({
            "name": "Google Sheets",
            "description": "Read and write spreadsheet ranges"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let run_count = run_count(context).await;
        let mut output_items = Vec::new();

        for item_index in 0..run_count {
            let operation = get_string_param(context, "operation", item_index, "readRange").await;
            let base_url =
                get_string_param(context, "baseUrl", item_index, "https://sheets.googleapis.com").await;
            let timeout_ms = get_u64_param(context, "timeout", item_index, 60_000).await;
            let auth_token = require_auth_token(
                "Google Sheets",
                get_optional_string_param(context, "authToken", item_index).await,
            )?;

            let headers = get_optional_param(context, "headers", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();
            let mut query = get_optional_param(context, "queryParameters", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();

            let spreadsheet_id = get_optional_string_param(context, "spreadsheetId", item_index).await;
            let range = get_optional_string_param(context, "range", item_index).await;

            let (method, url, body) = match operation.as_str() {
                "readRange" => {
                    let spreadsheet_id = ensure_required_string(
                        "Google Sheets",
                        "Spreadsheet ID",
                        spreadsheet_id.clone(),
                        "Set Spreadsheet ID.",
                    )?;
                    let range = ensure_required_string(
                        "Google Sheets",
                        "Range",
                        range.clone(),
                        "Set sheet range in A1 notation.",
                    )?;
                    (
                        "GET".to_string(),
                        build_url(
                            &base_url,
                            &format!("/v4/spreadsheets/{spreadsheet_id}/values/{range}"),
                        ),
                        None,
                    )
                }
                "appendValues" => {
                    let spreadsheet_id = ensure_required_string(
                        "Google Sheets",
                        "Spreadsheet ID",
                        spreadsheet_id.clone(),
                        "Set Spreadsheet ID.",
                    )?;
                    let range = ensure_required_string(
                        "Google Sheets",
                        "Range",
                        range.clone(),
                        "Set sheet range in A1 notation.",
                    )?;
                    query.push(("valueInputOption".to_string(), "RAW".to_string()));
                    let values = parse_body(get_optional_param(context, "values", item_index).await)
                        .ok_or_else(|| BarqError::NodeOperationError {
                            node_name: "Google Sheets".to_string(),
                            message: "Missing Values. Provide a JSON array like [[\"a\",\"b\"]]."
                                .to_string(),
                        })?;
                    (
                        "POST".to_string(),
                        build_url(
                            &base_url,
                            &format!("/v4/spreadsheets/{spreadsheet_id}/values/{range}:append"),
                        ),
                        Some(json!({ "values": values })),
                    )
                }
                "apiCall" => {
                    let method = get_string_param(context, "method", item_index, "GET").await;
                    let resource_path = ensure_required_string(
                        "Google Sheets",
                        "Resource Path",
                        get_optional_string_param(context, "resourcePath", item_index).await,
                        "Provide an API path like /v4/spreadsheets.",
                    )?;
                    let body = parse_body(get_optional_param(context, "body", item_index).await);
                    (method, build_url(&base_url, &resource_path), body)
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "Google Sheets".to_string(),
                        message: format!("Operation '{}' is not supported", operation),
                    });
                }
            };

            let response = execute_prepared_request(
                &self.client,
                "Google Sheets",
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
    async fn sheets_read_range_works() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/v4/spreadsheets/sheet1/values/A1:B2")
            .match_header("authorization", "Bearer gs-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"range":"A1:B2"}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("Google Sheets", "barqflow-nodes.googleSheets");
        context.add_param("operation", json!("readRange"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("authToken", json!("gs-token"));
        context.add_param("spreadsheetId", json!("sheet1"));
        context.add_param("range", json!("A1:B2"));

        let result = SheetsNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(result[0][0].json.0.get("status").and_then(|v| v.as_u64()), Some(200));
    }

    #[tokio::test]
    async fn sheets_requires_auth_token() {
        let mut context = MockContext::new("Google Sheets", "barqflow-nodes.googleSheets");
        context.add_param("operation", json!("readRange"));
        context.add_param("spreadsheetId", json!("sheet1"));
        context.add_param("range", json!("A1:B2"));

        let err = SheetsNode::new().execute(&context).await.unwrap_err();
        assert!(err.to_string().contains("Auth Token"));
    }
}
