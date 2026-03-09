use crate::integration::common::{
    ensure_required_string, get_optional_param, get_optional_string_param, parse_body,
};
use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::{IExecuteFunctions, INodeType};
use barqflow_core::types::IDataObject;
use serde_json::{json, Value};
use sqlx::{postgres::PgPoolOptions, Column, Row, TypeInfo};

pub struct PostgresNode;

impl PostgresNode {
    pub fn new() -> Self {
        Self
    }

    fn is_safe_identifier(identifier: &str) -> bool {
        !identifier.is_empty()
            && identifier
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
    }

    fn map_rows(rows: Vec<sqlx::postgres::PgRow>) -> Vec<Value> {
        let mut result_array = Vec::new();
        for row in rows {
            let mut row_obj = serde_json::Map::new();
            for col in row.columns() {
                let col_name = col.name();
                let type_name = col.type_info().name();

                let val = match type_name {
                    "INT4" | "INT8" | "INT2" => {
                        if let Ok(v) = row.try_get::<i64, _>(col_name) {
                            json!(v)
                        } else {
                            Value::Null
                        }
                    }
                    "FLOAT4" | "FLOAT8" | "NUMERIC" => {
                        if let Ok(v) = row.try_get::<f64, _>(col_name) {
                            json!(v)
                        } else {
                            Value::Null
                        }
                    }
                    "BOOL" => {
                        if let Ok(v) = row.try_get::<bool, _>(col_name) {
                            json!(v)
                        } else {
                            Value::Null
                        }
                    }
                    "JSON" | "JSONB" => {
                        if let Ok(v) = row.try_get::<Value, _>(col_name) {
                            v
                        } else {
                            Value::Null
                        }
                    }
                    _ => {
                        if let Ok(v) = row.try_get::<String, _>(col_name) {
                            json!(v)
                        } else {
                            Value::Null
                        }
                    }
                };
                row_obj.insert(col_name.to_string(), val);
            }
            result_array.push(Value::Object(row_obj));
        }
        result_array
    }

    fn value_to_sql_literal(value: &Value) -> String {
        match value {
            Value::Null => "NULL".to_string(),
            Value::Bool(v) => {
                if *v {
                    "TRUE".to_string()
                } else {
                    "FALSE".to_string()
                }
            }
            Value::Number(v) => v.to_string(),
            Value::String(v) => format!("'{}'", v.replace('\\', "\\\\").replace('"', "\\\"").replace('\'', "''")),
            other => {
                let serialized = other.to_string().replace('\\', "\\\\").replace('\'', "''");
                format!("'{}'::jsonb", serialized)
            }
        }
    }

    fn build_select_query(
        table: &str,
        columns_raw: Option<&str>,
        where_clause: Option<&str>,
        limit: Option<u64>,
    ) -> Result<String, BarqError> {
        if !Self::is_safe_identifier(table) {
            return Err(BarqError::NodeOperationError {
                node_name: "Postgres".to_string(),
                message: "Invalid table name. Only letters, numbers, and underscore are allowed."
                    .to_string(),
            });
        }

        let columns = columns_raw
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .unwrap_or("*");

        let rendered_columns = if columns == "*" {
            "*".to_string()
        } else {
            let parsed: Result<Vec<String>, BarqError> = columns
                .split(',')
                .map(|part| {
                    let identifier = part.trim();
                    if Self::is_safe_identifier(identifier) {
                        Ok(identifier.to_string())
                    } else {
                        Err(BarqError::NodeOperationError {
                            node_name: "Postgres".to_string(),
                            message: format!(
                                "Invalid column name '{}'. Only letters, numbers, and underscore are allowed.",
                                identifier
                            ),
                        })
                    }
                })
                .collect();
            parsed?.join(",")
        };

        let mut query = format!("SELECT {} FROM {}", rendered_columns, table);

        if let Some(where_clause) = where_clause.map(|v| v.trim()).filter(|v| !v.is_empty()) {
            query.push_str(" WHERE ");
            query.push_str(where_clause);
        }

        if let Some(limit) = limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        Ok(query)
    }

    fn build_insert_query(table: &str, data: &serde_json::Map<String, Value>) -> Result<String, BarqError> {
        if !Self::is_safe_identifier(table) {
            return Err(BarqError::NodeOperationError {
                node_name: "Postgres".to_string(),
                message: "Invalid table name. Only letters, numbers, and underscore are allowed."
                    .to_string(),
            });
        }

        if data.is_empty() {
            return Err(BarqError::NodeOperationError {
                node_name: "Postgres".to_string(),
                message: "Insert data cannot be empty. Provide at least one field.".to_string(),
            });
        }

        let mut columns = Vec::new();
        let mut values = Vec::new();
        for (key, value) in data {
            if !Self::is_safe_identifier(key) {
                return Err(BarqError::NodeOperationError {
                    node_name: "Postgres".to_string(),
                    message: format!(
                        "Invalid column name '{}'. Only letters, numbers, and underscore are allowed.",
                        key
                    ),
                });
            }
            columns.push(key.clone());
            values.push(Self::value_to_sql_literal(value));
        }

        Ok(format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
            table,
            columns.join(","),
            values.join(",")
        ))
    }
}

impl Default for PostgresNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for PostgresNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({
            "name": "Postgres",
            "description": "Execute SQL queries on PostgreSQL"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let input_data = context.get_input_data(0).await.unwrap_or_default();
        let run_count = if input_data.is_empty() {
            1
        } else {
            input_data.len()
        };
        let mut output_items = Vec::new();

        let creds = context.get_credentials("postgresApi").await?;
        let host = creds.get("host").and_then(|v| v.as_str()).unwrap_or("");
        let port = creds.get("port").and_then(|v| v.as_i64()).unwrap_or(5432);
        let database = creds.get("database").and_then(|v| v.as_str()).unwrap_or("");
        let username = creds.get("user").and_then(|v| v.as_str()).unwrap_or("");
        let password = creds.get("password").and_then(|v| v.as_str()).unwrap_or("");

        let mut missing = Vec::new();
        if host.is_empty() {
            missing.push("host");
        }
        if database.is_empty() {
            missing.push("database");
        }
        if username.is_empty() {
            missing.push("user");
        }
        if password.is_empty() {
            missing.push("password");
        }

        if !missing.is_empty() {
            return Err(BarqError::NodeOperationError {
                node_name: "Postgres".to_string(),
                message: format!(
                    "Missing Postgres credential fields: {}. Go to /credentials and update 'postgresApi'.",
                    missing.join(", ")
                ),
            });
        }

        let conn_str = format!(
            "postgres://{}:{}@{}:{}/{}",
            username, password, host, port, database
        );

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&conn_str)
            .await
            .map_err(|e| BarqError::NodeOperationError {
                node_name: "Postgres".to_string(),
                message: format!("Failed to connect: {}", e),
            })?;

        for item_index in 0..run_count {
            let operation = context
                .get_node_parameter_at_item("operation", item_index, None)
                .await
                .map(|v| v.as_str().unwrap_or("executeQuery").to_string())
                .unwrap_or_else(|_| "executeQuery".to_string());

            let query = match operation.as_str() {
                "executeQuery" => {
                    let query = context
                        .get_node_parameter_at_item("query", item_index, None)
                        .await
                        .map(|v| v.as_str().unwrap_or("").to_string())
                        .unwrap_or_default();

                    if query.trim().is_empty() {
                        return Err(BarqError::NodeOperationError {
                            node_name: "Postgres".to_string(),
                            message: "Query cannot be empty".to_string(),
                        });
                    }
                    query
                }
                "selectRows" => {
                    let table = ensure_required_string(
                        "Postgres",
                        "Table",
                        get_optional_string_param(context, "table", item_index).await,
                        "Set the table name for select operation.",
                    )?;
                    let columns = get_optional_string_param(context, "columns", item_index).await;
                    let where_clause =
                        get_optional_string_param(context, "whereClause", item_index).await;
                    let limit = get_optional_param(context, "limit", item_index)
                        .await
                        .and_then(|v| v.as_u64());
                    Self::build_select_query(
                        &table,
                        columns.as_deref(),
                        where_clause.as_deref(),
                        limit,
                    )?
                }
                "insertRow" => {
                    let table = ensure_required_string(
                        "Postgres",
                        "Table",
                        get_optional_string_param(context, "table", item_index).await,
                        "Set the table name for insert operation.",
                    )?;
                    let data_value = parse_body(get_optional_param(context, "data", item_index).await)
                        .ok_or_else(|| BarqError::NodeOperationError {
                            node_name: "Postgres".to_string(),
                            message: "Missing Data. Provide a JSON object for insert operation."
                                .to_string(),
                        })?;
                    let data_obj = data_value.as_object().ok_or_else(|| BarqError::NodeOperationError {
                        node_name: "Postgres".to_string(),
                        message: "Invalid Data format. Expected a JSON object for insert operation."
                            .to_string(),
                    })?;
                    Self::build_insert_query(&table, data_obj)?
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "Postgres".to_string(),
                        message: format!("Operation '{}' not supported", operation),
                    });
                }
            };

            let rows = sqlx::query(&query)
                .fetch_all(&pool)
                .await
                .map_err(|e| BarqError::NodeOperationError {
                    node_name: "Postgres".to_string(),
                    message: format!("Query execution failed: {}", e),
                })?;

            output_items.push(INodeExecutionData::new(IDataObject::from(json!({
                "success": true,
                "operation": operation,
                "query": query,
                "rows": Self::map_rows(rows),
            }))));
        }

        Ok(vec![output_items])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_select_query_builds_expected_sql() {
        let query = PostgresNode::build_select_query(
            "users",
            Some("id,name"),
            Some("active = true"),
            Some(10),
        )
        .unwrap();

        assert_eq!(query, "SELECT id,name FROM users WHERE active = true LIMIT 10");
    }

    #[test]
    fn build_insert_query_builds_expected_sql() {
        let data = serde_json::json!({"email":"a@b.com","active":true});
        let data_obj = data.as_object().unwrap();

        let query = PostgresNode::build_insert_query("users", data_obj).unwrap();
        assert!(query.starts_with("INSERT INTO users ("));
        assert!(query.contains("email"));
        assert!(query.contains("active"));
        assert!(query.contains("RETURNING *"));
    }

    #[test]
    fn build_query_rejects_invalid_identifiers() {
        let err = PostgresNode::build_select_query("users;drop", None, None, None).unwrap_err();
        assert!(err.to_string().contains("Invalid table name"));
    }
}
