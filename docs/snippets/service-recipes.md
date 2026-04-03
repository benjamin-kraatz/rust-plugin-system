# Service Recipes

## Shared state with Arc\<Playground\>

```rust
#[derive(Clone)]
struct AppState {
    playground: Arc<Playground>,
}

let state = AppState {
    playground: Arc::new(Playground::load_default()?),
};
```

## Set up Axum routes for plugin invocation

```rust
let app = Router::new()
    .route("/health", get(health))
    .route("/plugins", get(list_plugins))
    .route("/plugins/{plugin_id}", get(get_plugin))
    .route("/plugins/{plugin_id}/actions/{action_id}", get(get_action))
    .route("/plugins/{plugin_id}/actions/{action_id}/invoke", post(invoke))
    .with_state(state);

let listener = tokio::net::TcpListener::bind("127.0.0.1:5000").await?;
axum::serve(listener, app).await?;
```

## List plugins as a JSON API

```rust
async fn list_plugins(State(state): State<AppState>) -> Json<Value> {
    let manifests = state.playground.manifests();
    Json(json!({
        "plugins": manifests.iter().map(|m| json!({
            "id": m.id, "name": m.name, "version": m.version,
            "actions": m.actions.iter().map(|a| json!({
                "id": a.id, "label": a.label,
                "invoke": format!("/plugins/{}/actions/{}/invoke", m.id, a.id),
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    }))
}
```

## Inspect a plugin manifest via GET

```rust
async fn get_plugin(State(state): State<AppState>, Path(id): Path<String>) -> ApiResponse {
    let manifests = state.playground.manifests();
    let manifest = find_manifest(&manifests, &id).ok_or_else(|| {
        error_response(StatusCode::NOT_FOUND, "plugin_not_found",
            format!("No loaded plugin named '{id}'."))
    })?;
    Ok(Json(json!({ "plugin": plugin_detail(manifest) })))
}
```

## Invoke a plugin action via POST

```rust
async fn invoke(
    State(state): State<AppState>,
    Path((plugin_id, action_id)): Path<(String, String)>,
    Json(payload): Json<Value>,
) -> ApiResponse {
    match state.playground.invoke(&plugin_id, &action_id, payload, HostKind::Service) {
        Ok(resp) => Ok(Json(json!({ "ok": resp.success, "response": resp }))),
        Err(e) => Err(error_response(
            StatusCode::UNPROCESSABLE_ENTITY, "invocation_failed",
            format!("Invocation failed: {e}"))),
    }
}
```

## Structured error response helper

```rust
type ApiResponse = Result<Json<Value>, (StatusCode, Json<Value>)>;

fn error_response(status: StatusCode, code: &str, message: String) -> (StatusCode, Json<Value>) {
    (status, Json(json!({
        "ok": false,
        "error": { "code": code, "message": message }
    })))
}
```

## Health endpoint with loader diagnostics

```rust
async fn health(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "status": "ok",
        "loaded_plugins": state.playground.manifests().len(),
        "warning_count": state.playground.warnings().len(),
    }))
}
```

## Find-or-404 helpers for plugins and actions

```rust
fn find_manifest<'a>(manifests: &'a [PluginManifest], id: &str) -> Option<&'a PluginManifest> {
    manifests.iter().find(|m| m.id == id)
}

fn find_action<'a>(manifest: &'a PluginManifest, id: &str) -> Option<&'a PluginAction> {
    manifest.actions.iter().find(|a| a.id == id)
}
```

## Example curl calls

```bash
# List all plugins
curl http://127.0.0.1:5000/plugins

# Inspect a specific plugin
curl http://127.0.0.1:5000/plugins/formatter

# Invoke an action with a JSON payload
curl -X POST http://127.0.0.1:5000/plugins/formatter/actions/pretty-json/invoke \
     -H 'Content-Type: application/json' \
     -d '{"hello":"world","n":42}'
```
