use std::sync::Arc;

use anyhow::Result;
use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use host_core::Playground;
use plugin_protocol::HostKind;
use serde_json::{Value, json};

#[derive(Clone)]
struct AppState {
    playground: Arc<Playground>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let state = AppState {
        playground: Arc::new(Playground::load_default()?),
    };

    let app = Router::new()
        .route("/plugins", get(list_plugins))
        .route("/invoke/{plugin_id}/{action_id}", post(invoke))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:5000").await?;
    println!("service host running at http://127.0.0.1:5000");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn list_plugins(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "plugin_dir": state.playground.plugin_dir(),
        "warnings": state.playground.warnings(),
        "plugins": state.playground.manifests(),
    }))
}

async fn invoke(
    State(state): State<AppState>,
    Path((plugin_id, action_id)): Path<(String, String)>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    match state
        .playground
        .invoke(&plugin_id, &action_id, payload, HostKind::Service)
    {
        Ok(response) => Json(json!({ "ok": true, "response": response })),
        Err(error) => Json(json!({ "ok": false, "error": error.to_string() })),
    }
}
