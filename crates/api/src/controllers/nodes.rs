use axum::{
    extract::State,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use barqflow_nodes::is_node_ui_exposed;
use barqflow_registry::registry::NodeRegistry;
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub node_registry: Arc<NodeRegistry>,
}

#[derive(Serialize)]
pub struct NodeSchema {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub is_trigger: bool,
    pub properties: Vec<barqflow_core::properties::INodeProperty>,
    pub defaults: Option<Value>,
}

pub fn node_routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(list_node_schemas))
        .with_state(state)
}

async fn list_node_schemas(State(state): State<AppState>) -> impl IntoResponse {
    let mut schemas = Vec::new();

    let names = state.node_registry.get_all_node_names();
    for name in names {
        if !is_node_ui_exposed(&name) {
            continue;
        }

        if let Some(info) = state.node_registry.get_latest_node(&name) {
            schemas.push(NodeSchema {
                name: info.name,
                display_name: info.display_name,
                description: info.description,
                is_trigger: info.is_trigger,
                properties: info.properties.properties.clone(),
                defaults: None, // We can populate this if we parse the defaults from the schema
            });
        }
    }

    Json(schemas)
}
