use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::INodeType;
use barqflow_core::types::IDataObject;
use sqlx::{postgres::PgPoolOptions, Row, Column, TypeInfo};

pub struct PostgresNode;

impl PostgresNode {
    pub fn new() -> Self {
        Self
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
        IDataObject::from(serde_json::json!({
            "name": "Postgres",
            "description": "Execute SQL queries on PostgreSQL"
        }))
    }

    async fn execute(
        &self,
        context: &dyn barqflow_core::traits::IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let input_data = context.get_input_data(0)?;
        let mut output_items = Vec::new();

        // Get credentials
        let creds = context.get_credentials("postgresApi").await?;
        let host = creds.get("host").and_then(|v| v.as_str()).unwrap_or("localhost");
        let port = creds.get("port").and_then(|v| v.as_i64()).unwrap_or(5432);
        let database = creds.get("database").and_then(|v| v.as_str()).unwrap_or("postgres");
        let username = creds.get("user").and_then(|v| v.as_str()).unwrap_or("postgres");
        let password = creds.get("password").and_then(|v| v.as_str()).unwrap_or("");

        let conn_str = format!("postgres://{}:{}@{}:{}/{}", username, password, host, port, database);

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&conn_str)
            .await
            .map_err(|e| BarqError::NodeOperationError {
                node_name: "Postgres".to_string(),
                message: format!("Failed to connect: {}", e),
            })?;

        for (item_index, _item) in input_data.iter().enumerate() {
            let operation = context
                .get_node_parameter_at_item("operation", item_index, None)
                .await
                .map(|v| v.as_str().unwrap_or("executeQuery").to_string())
                .unwrap_or_else(|_| "executeQuery".to_string());

            if operation == "executeQuery" {
                let query = context
                    .get_node_parameter_at_item("query", item_index, None)
                    .await
                    .map(|v| v.as_str().unwrap_or("").to_string())
                    .unwrap_or_default();

                if query.is_empty() {
                    return Err(BarqError::NodeOperationError {
                        node_name: "Postgres".to_string(),
                        message: "Query cannot be empty".to_string(),
                    });
                }

                let rows = sqlx::query(&query)
                    .fetch_all(&pool)
                    .await
                    .map_err(|e| BarqError::NodeOperationError {
                        node_name: "Postgres".to_string(),
                        message: format!("Query execution failed: {}", e),
                    })?;

                let mut result_array = Vec::new();
                for row in rows {
                    let mut row_obj = serde_json::Map::new();
                    for col in row.columns() {
                        let col_name = col.name();
                        let type_name = col.type_info().name();
                        
                        // Simple dynamic type mapping based on postgres internal type names
                        let val = match type_name {
                            "INT4" | "INT8" | "INT2" => {
                                if let Ok(v) = row.try_get::<i64, _>(col_name) {
                                    serde_json::json!(v)
                                } else {
                                    serde_json::Value::Null
                                }
                            },
                            "FLOAT4" | "FLOAT8" | "NUMERIC" => {
                                if let Ok(v) = row.try_get::<f64, _>(col_name) {
                                    serde_json::json!(v)
                                } else {
                                    serde_json::Value::Null
                                }
                            },
                            "BOOL" => {
                                if let Ok(v) = row.try_get::<bool, _>(col_name) {
                                    serde_json::json!(v)
                                } else {
                                    serde_json::Value::Null
                                }
                            },
                            "JSON" | "JSONB" => {
                                if let Ok(v) = row.try_get::<serde_json::Value, _>(col_name) {
                                    v
                                } else {
                                    serde_json::Value::Null
                                }
                            },
                            _ => {
                                // Default to string for UUID, VARCHAR, TEXT, etc.
                                if let Ok(v) = row.try_get::<String, _>(col_name) {
                                    serde_json::json!(v)
                                } else {
                                    serde_json::Value::Null
                                }
                            }
                        };
                        row_obj.insert(col_name.to_string(), val);
                    }
                    result_array.push(serde_json::Value::Object(row_obj));
                }

                let output_obj = serde_json::json!({
                    "success": true,
                    "rows": result_array
                });
                
                output_items.push(INodeExecutionData::new(IDataObject::from(output_obj)));
            } else {
                return Err(BarqError::NodeOperationError {
                    node_name: "Postgres".to_string(),
                    message: format!("Operation '{}' not supported", operation),
                });
            }
        }

        Ok(vec![output_items])
    }
}
