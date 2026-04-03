use std::sync::Arc;

use anyhow::Result;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use host_core::Playground;
use plugin_manifest::{PluginAction, PluginManifest};
use plugin_protocol::HostKind;
use serde_json::{Value, json};

#[derive(Clone)]
struct AppState {
    playground: Arc<Playground>,
}

type ApiResponse = Result<Json<Value>, (StatusCode, Json<Value>)>;

#[tokio::main]
async fn main() -> Result<()> {
    let state = AppState {
        playground: Arc::new(Playground::load_default()?),
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/catalog", get(catalog))
        .route("/examples", get(examples))
        .route("/plugins", get(list_plugins))
        .route("/plugins/{plugin_id}", get(get_plugin))
        .route("/plugins/{plugin_id}/actions/{action_id}", get(get_action))
        .route(
            "/plugins/{plugin_id}/actions/{action_id}/invoke",
            post(invoke),
        )
        .route("/invoke/{plugin_id}/{action_id}", post(invoke))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:5000").await?;
    println!("service host running at http://127.0.0.1:5000");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn root(State(state): State<AppState>) -> Json<Value> {
    let manifests = state.playground.manifests();
    Json(json!({
        "service": "host-service",
        "surface": "backend-api",
        "description": "Automation-oriented plugin discovery and invocation surface.",
        "comparison_focus": [
            "structured catalog and detail endpoints",
            "stable invocation envelopes for scripts and service orchestration",
            "example payload discovery without any external network calls"
        ],
        "stats": service_stats(&state, &manifests),
        "endpoints": [
            endpoint("GET", "/", "Service overview and comparison framing"),
            endpoint("GET", "/health", "Health and loader status"),
            endpoint("GET", "/catalog", "High-level plugin catalog with links"),
            endpoint("GET", "/examples", "Curated invocation payload examples"),
            endpoint("GET", "/plugins", "Plugin summaries for automation"),
            endpoint("GET", "/plugins/{plugin_id}", "Plugin manifest details and actions"),
            endpoint("GET", "/plugins/{plugin_id}/actions/{action_id}", "Action detail plus example payload"),
            endpoint("POST", "/plugins/{plugin_id}/actions/{action_id}/invoke", "Canonical JSON invocation endpoint"),
            endpoint("POST", "/invoke/{plugin_id}/{action_id}", "Backward-compatible invocation endpoint")
        ]
    }))
}

async fn health(State(state): State<AppState>) -> Json<Value> {
    let manifests = state.playground.manifests();
    Json(json!({
        "status": "ok",
        "host": "service",
        "loaded_plugins": manifests.len(),
        "warning_count": state.playground.warnings().len(),
        "plugin_dir": state.playground.plugin_dir(),
        "warnings": state.playground.warnings(),
    }))
}

async fn catalog(State(state): State<AppState>) -> Json<Value> {
    let manifests = state.playground.manifests();
    Json(json!({
        "surface": "service",
        "purpose": "Compare a realistic backend discovery surface with the richer browser UI.",
        "stats": service_stats(&state, &manifests),
        "plugins": manifests
            .iter()
            .map(plugin_catalog_entry)
            .collect::<Vec<_>>(),
        "warnings": state.playground.warnings(),
    }))
}

async fn examples(State(state): State<AppState>) -> Json<Value> {
    let manifests = state.playground.manifests();
    Json(json!({
        "surface": "service",
        "examples": manifests
            .iter()
            .flat_map(|manifest| {
                manifest.actions.iter().map(move |action| {
                    json!({
                        "plugin_id": manifest.id,
                        "plugin_name": manifest.name,
                        "action_id": action.id,
                        "action_label": action.label,
                        "payload_hint_text": action.payload_hint.clone().unwrap_or_else(|| "{}".to_owned()),
                        "payload_hint_json": parse_payload_hint(action.payload_hint.as_deref()),
                        "invoke": format!("/plugins/{}/actions/{}/invoke", manifest.id, action.id),
                    })
                })
            })
            .collect::<Vec<_>>()
    }))
}

async fn list_plugins(State(state): State<AppState>) -> Json<Value> {
    let manifests = state.playground.manifests();
    Json(json!({
        "surface": "service",
        "plugin_dir": state.playground.plugin_dir(),
        "warnings": state.playground.warnings(),
        "plugins": manifests.iter().map(plugin_summary_entry).collect::<Vec<_>>(),
    }))
}

async fn get_plugin(State(state): State<AppState>, Path(plugin_id): Path<String>) -> ApiResponse {
    let manifests = state.playground.manifests();
    let manifest = find_manifest(&manifests, &plugin_id).ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "plugin_not_found",
            format!("No loaded plugin named '{plugin_id}'."),
        )
    })?;

    Ok(Json(json!({
        "surface": "service",
        "plugin": plugin_detail_entry(manifest),
        "warnings": state.playground.warnings(),
    })))
}

async fn get_action(
    State(state): State<AppState>,
    Path((plugin_id, action_id)): Path<(String, String)>,
) -> ApiResponse {
    let manifests = state.playground.manifests();
    let manifest = find_manifest(&manifests, &plugin_id).ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "plugin_not_found",
            format!("No loaded plugin named '{plugin_id}'."),
        )
    })?;
    let action = find_action(manifest, &action_id).ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "action_not_found",
            format!("Plugin '{plugin_id}' does not expose action '{action_id}'."),
        )
    })?;

    Ok(Json(json!({
        "surface": "service",
        "plugin": {
            "id": manifest.id,
            "name": manifest.name,
        },
        "action": action_detail_entry(&manifest.id, action),
        "comparison_notes": [
            "Use this endpoint when scripts need per-action payload examples.",
            "Use the web host when a human wants to inspect the same action visually."
        ]
    })))
}

async fn invoke(
    State(state): State<AppState>,
    Path((plugin_id, action_id)): Path<(String, String)>,
    Json(payload): Json<Value>,
) -> ApiResponse {
    let manifests = state.playground.manifests();
    let manifest = find_manifest(&manifests, &plugin_id).ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "plugin_not_found",
            format!("No loaded plugin named '{plugin_id}'."),
        )
    })?;
    let action = find_action(manifest, &action_id).ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "action_not_found",
            format!("Plugin '{plugin_id}' does not expose action '{action_id}'."),
        )
    })?;

    let payload_for_response = payload.clone();
    match state
        .playground
        .invoke(&plugin_id, &action_id, payload, HostKind::Service)
    {
        Ok(response) => Ok(Json(json!({
            "ok": response.success,
            "surface": "service",
            "request": {
                "plugin_id": plugin_id,
                "plugin_name": manifest.name,
                "action_id": action_id,
                "action_label": action.label,
                "payload": payload_for_response,
                "host_kind": "service",
            },
            "response": response,
            "links": {
                "plugin": format!("/plugins/{}", manifest.id),
                "action": format!("/plugins/{}/actions/{}", manifest.id, action.id),
                "catalog": "/catalog",
                "examples": "/examples",
            },
            "comparison_notes": [
                "This envelope is designed for backend callers and automated comparisons.",
                "The web host presents the same invocation data with browser-specific layout and navigation."
            ]
        }))),
        Err(error) => Err(error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invocation_failed",
            format!("Invocation failed: {error}"),
        )),
    }
}

fn service_stats(state: &AppState, manifests: &[PluginManifest]) -> Value {
    json!({
        "plugins": manifests.len(),
        "actions": manifests.iter().map(|manifest| manifest.actions.len()).sum::<usize>(),
        "warnings": state.playground.warnings().len(),
        "plugin_dir": state.playground.plugin_dir(),
    })
}

fn endpoint(method: &str, path: &str, description: &str) -> Value {
    json!({
        "method": method,
        "path": path,
        "description": description,
    })
}

fn plugin_catalog_entry(manifest: &PluginManifest) -> Value {
    json!({
        "id": manifest.id,
        "name": manifest.name,
        "description": manifest.description,
        "version": manifest.version,
        "architecture": manifest.architecture,
        "skill_level": manifest.skill_level,
        "supported_hosts": manifest.supported_hosts,
        "tags": manifest.tags,
        "action_count": manifest.actions.len(),
        "links": {
            "self": format!("/plugins/{}", manifest.id),
            "actions": manifest.actions.iter().map(|action| format!("/plugins/{}/actions/{}", manifest.id, action.id)).collect::<Vec<_>>(),
        }
    })
}

fn plugin_summary_entry(manifest: &PluginManifest) -> Value {
    json!({
        "id": manifest.id,
        "name": manifest.name,
        "description": manifest.description,
        "version": manifest.version,
        "supported_hosts": manifest.supported_hosts,
        "tags": manifest.tags,
        "actions": manifest.actions.iter().map(|action| {
            json!({
                "id": action.id,
                "label": action.label,
                "detail": format!("/plugins/{}/actions/{}", manifest.id, action.id),
                "invoke": format!("/plugins/{}/actions/{}/invoke", manifest.id, action.id),
            })
        }).collect::<Vec<_>>(),
    })
}

fn plugin_detail_entry(manifest: &PluginManifest) -> Value {
    json!({
        "id": manifest.id,
        "name": manifest.name,
        "version": manifest.version,
        "description": manifest.description,
        "architecture": manifest.architecture,
        "skill_level": manifest.skill_level,
        "supported_hosts": manifest.supported_hosts,
        "capabilities": manifest.capabilities,
        "tags": manifest.tags,
        "notes": manifest.notes,
        "actions": manifest.actions.iter().map(|action| action_detail_entry(&manifest.id, action)).collect::<Vec<_>>(),
    })
}

fn action_detail_entry(plugin_id: &str, action: &PluginAction) -> Value {
    json!({
        "id": action.id,
        "label": action.label,
        "description": action.description,
        "payload_hint_text": action.payload_hint.clone().unwrap_or_else(|| "{}".to_owned()),
        "payload_hint_json": parse_payload_hint(action.payload_hint.as_deref()),
        "links": {
            "detail": format!("/plugins/{plugin_id}/actions/{}", action.id),
            "invoke": format!("/plugins/{plugin_id}/actions/{}/invoke", action.id),
        }
    })
}

fn parse_payload_hint(payload_hint: Option<&str>) -> Value {
    payload_hint
        .and_then(|payload_hint| serde_json::from_str(payload_hint).ok())
        .unwrap_or_else(|| json!({}))
}

fn find_manifest<'a>(
    manifests: &'a [PluginManifest],
    plugin_id: &str,
) -> Option<&'a PluginManifest> {
    manifests.iter().find(|manifest| manifest.id == plugin_id)
}

fn find_action<'a>(manifest: &'a PluginManifest, action_id: &str) -> Option<&'a PluginAction> {
    manifest
        .actions
        .iter()
        .find(|action| action.id == action_id)
}

fn error_response(status: StatusCode, code: &str, message: String) -> (StatusCode, Json<Value>) {
    (
        status,
        Json(json!({
            "ok": false,
            "error": {
                "code": code,
                "message": message,
            }
        })),
    )
}
