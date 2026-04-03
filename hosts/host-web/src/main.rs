use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::response::Html;
use axum::routing::get;
use host_core::{Playground, render_response};
use plugin_protocol::HostKind;

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
        .route("/", get(index))
        .route("/invoke/{plugin_id}/{action_id}", get(invoke))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:4000").await?;
    println!("web host running at http://127.0.0.1:4000");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index(State(state): State<AppState>) -> Html<String> {
    let mut html = String::from(
        "<html><head><title>Rust Plugin Playground - Web Host</title></head><body><h1>Web Host</h1>",
    );

    for manifest in state.playground.manifests() {
        html.push_str(&format!(
            "<section style=\"margin-bottom: 24px;\"><h2>{}</h2><p>{}</p>",
            manifest.name, manifest.description
        ));
        for action in manifest.actions {
            html.push_str(&format!(
                "<form method=\"get\" action=\"/invoke/{}/{}\"><input type=\"text\" name=\"payload\" value=\"{{}}\" size=\"48\" /><button type=\"submit\">Run {}</button></form>",
                manifest.id, action.id, action.label
            ));
        }
        html.push_str("</section>");
    }

    html.push_str("</body></html>");
    Html(html)
}

async fn invoke(
    State(state): State<AppState>,
    Path((plugin_id, action_id)): Path<(String, String)>,
    Query(query): Query<HashMap<String, String>>,
) -> Html<String> {
    let payload = query.get("payload").map(String::as_str).unwrap_or("{}");
    let body = match state
        .playground
        .invoke_text(&plugin_id, &action_id, payload, HostKind::Web)
    {
        Ok(response) => render_response(&response),
        Err(error) => error.to_string(),
    };

    Html(format!(
        "<html><body><p><a href=\"/\">Back</a></p><pre>{}</pre></body></html>",
        html_escape(&body)
    ))
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
