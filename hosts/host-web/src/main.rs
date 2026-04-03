use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use axum::extract::{Form, Path, Query, State};
use axum::response::Html;
use axum::routing::{get, post};
use host_core::Playground;
use plugin_manifest::{PluginAction, PluginManifest};
use plugin_protocol::{HostKind, OutputBlock, PluginResponse};

#[derive(Clone)]
struct AppState {
    playground: Arc<Playground>,
}

struct InvocationView {
    payload_text: String,
    response: Result<PluginResponse, String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let state = AppState {
        playground: Arc::new(Playground::load_default()?),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/invoke", post(invoke_form))
        .route("/invoke/{plugin_id}/{action_id}", get(invoke_legacy))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:4000").await?;
    println!("web host running at http://127.0.0.1:4000");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index(
    State(state): State<AppState>,
    Query(query): Query<HashMap<String, String>>,
) -> Html<String> {
    Html(render_page(&state, query, None))
}

async fn invoke_form(
    State(state): State<AppState>,
    Form(form): Form<HashMap<String, String>>,
) -> Html<String> {
    let plugin_id = form.get("plugin_id").cloned().unwrap_or_default();
    let action_id = form.get("action_id").cloned().unwrap_or_default();
    let payload_text = form
        .get("payload")
        .cloned()
        .unwrap_or_else(|| "{}".to_owned());

    let mut selection = HashMap::new();
    selection.insert("plugin".to_owned(), plugin_id.clone());
    selection.insert("action".to_owned(), action_id.clone());
    selection.insert("payload".to_owned(), payload_text.clone());

    let response = state
        .playground
        .invoke_text(&plugin_id, &action_id, &payload_text, HostKind::Web)
        .map_err(|error| error.to_string());

    Html(render_page(
        &state,
        selection,
        Some(InvocationView {
            payload_text,
            response,
        }),
    ))
}

async fn invoke_legacy(
    State(state): State<AppState>,
    Path((plugin_id, action_id)): Path<(String, String)>,
    Query(query): Query<HashMap<String, String>>,
) -> Html<String> {
    let payload_text = query
        .get("payload")
        .cloned()
        .unwrap_or_else(|| "{}".to_owned());

    let mut selection = HashMap::new();
    selection.insert("plugin".to_owned(), plugin_id.clone());
    selection.insert("action".to_owned(), action_id.clone());
    selection.insert("payload".to_owned(), payload_text.clone());

    let response = state
        .playground
        .invoke_text(&plugin_id, &action_id, &payload_text, HostKind::Web)
        .map_err(|error| error.to_string());

    Html(render_page(
        &state,
        selection,
        Some(InvocationView {
            payload_text,
            response,
        }),
    ))
}

fn render_page(
    state: &AppState,
    selection: HashMap<String, String>,
    invocation: Option<InvocationView>,
) -> String {
    let manifests = state.playground.manifests();
    let plugin_count = manifests.len();
    let action_count = manifests
        .iter()
        .map(|manifest| manifest.actions.len())
        .sum::<usize>();
    let warning_count = state.playground.warnings().len();

    let selected_plugin = pick_plugin(&manifests, selection.get("plugin").map(String::as_str));
    let selected_action = selected_plugin
        .and_then(|manifest| pick_action(manifest, selection.get("action").map(String::as_str)));

    let payload_text = invocation
        .as_ref()
        .map(|view| view.payload_text.clone())
        .or_else(|| selection.get("payload").cloned())
        .or_else(|| {
            selected_action
                .and_then(|action| action.payload_hint.clone())
                .or_else(|| Some("{}".to_owned()))
        })
        .unwrap_or_else(|| "{}".to_owned());

    let payload_js = serde_json::to_string(
        &selected_action
            .and_then(|action| action.payload_hint.clone())
            .unwrap_or_else(|| "{}".to_owned()),
    )
    .unwrap_or_else(|_| "\"{}\"".to_owned());

    let selected_plugin_id = selected_plugin
        .map(|manifest| manifest.id.as_str())
        .unwrap_or_default();
    let selected_action_id = selected_action
        .map(|action| action.id.as_str())
        .unwrap_or_default();

    let mut html = String::new();
    html.push_str("<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\" />");
    html.push_str(
        "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" /><title>Rust Plugin Playground · Web Host</title>",
    );
    html.push_str(
        "<style>
        :root { color-scheme: dark; --bg: #0b1020; --panel: #121933; --panel-2: #182042; --border: #2a376d; --text: #edf2ff; --muted: #9fb1e2; --accent: #70a5ff; --accent-2: #87f0d4; --success: #2ecc71; --error: #ff7f7f; }
        * { box-sizing: border-box; }
        body { margin: 0; background: linear-gradient(180deg, #091120 0%, #101933 100%); color: var(--text); font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, sans-serif; }
        a { color: var(--accent); text-decoration: none; }
        a:hover { text-decoration: underline; }
        .page { max-width: 1440px; margin: 0 auto; padding: 24px; }
        .hero { display: grid; gap: 18px; grid-template-columns: 2fr 1fr; align-items: start; margin-bottom: 24px; }
        .hero-card, .panel { background: rgba(18, 25, 51, 0.92); border: 1px solid var(--border); border-radius: 18px; padding: 18px; box-shadow: 0 18px 40px rgba(0,0,0,0.18); }
        .hero h1 { margin: 0 0 8px; font-size: 2rem; }
        .hero p { margin: 0; color: var(--muted); line-height: 1.5; }
        .pill-row, .stats, .badge-row { display: flex; flex-wrap: wrap; gap: 8px; }
        .pill, .badge { display: inline-flex; align-items: center; gap: 6px; border-radius: 999px; border: 1px solid var(--border); padding: 6px 10px; color: var(--muted); background: rgba(255,255,255,0.02); font-size: 0.9rem; }
        .stat { flex: 1 1 0; min-width: 120px; border-radius: 14px; padding: 12px 14px; background: var(--panel-2); border: 1px solid var(--border); }
        .stat strong { display: block; font-size: 1.3rem; color: var(--text); }
        .layout { display: grid; gap: 20px; grid-template-columns: 320px minmax(0, 1fr); }
        .catalog { display: grid; gap: 12px; max-height: calc(100vh - 180px); overflow: auto; }
        .plugin-link { display: block; border-radius: 16px; border: 1px solid var(--border); background: rgba(255,255,255,0.02); padding: 14px; }
        .plugin-link.active { background: rgba(112, 165, 255, 0.15); border-color: var(--accent); }
        .plugin-link h3, .panel h2, .panel h3, .panel h4 { margin-top: 0; }
        .panel-grid { display: grid; gap: 20px; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); }
        .action-card, .result-card, .output-card { border-radius: 16px; border: 1px solid var(--border); background: rgba(255,255,255,0.03); padding: 16px; }
        .action-card.active { border-color: var(--accent-2); box-shadow: inset 0 0 0 1px rgba(135, 240, 212, 0.35); }
        .muted { color: var(--muted); }
        .eyebrow { text-transform: uppercase; letter-spacing: 0.08em; font-size: 0.78rem; color: var(--accent-2); margin-bottom: 8px; }
        textarea, input[type=\"text\"] { width: 100%; border-radius: 14px; border: 1px solid var(--border); background: #091122; color: var(--text); padding: 12px 14px; font: inherit; }
        textarea { min-height: 220px; resize: vertical; font-family: ui-monospace, SFMono-Regular, SFMono-Regular, Menlo, Consolas, monospace; }
        button { border: 0; border-radius: 12px; padding: 11px 14px; font: inherit; cursor: pointer; color: #061126; background: linear-gradient(135deg, var(--accent-2), var(--accent)); font-weight: 700; }
        button.secondary { background: rgba(255,255,255,0.06); color: var(--text); border: 1px solid var(--border); }
        .button-row { display: flex; flex-wrap: wrap; gap: 10px; margin-top: 12px; }
        pre { margin: 0; white-space: pre-wrap; word-break: break-word; font-family: ui-monospace, SFMono-Regular, SFMono-Regular, Menlo, Consolas, monospace; }
        code { font-family: ui-monospace, SFMono-Regular, SFMono-Regular, Menlo, Consolas, monospace; }
        .status { padding: 14px 16px; border-radius: 16px; border: 1px solid var(--border); margin-bottom: 16px; }
        .status.success { border-color: rgba(46, 204, 113, 0.4); background: rgba(46, 204, 113, 0.12); }
        .status.error { border-color: rgba(255, 127, 127, 0.4); background: rgba(255, 127, 127, 0.12); }
        .list { margin: 0; padding-left: 18px; color: var(--muted); }
        details { border-top: 1px solid rgba(255,255,255,0.08); margin-top: 12px; padding-top: 12px; }
        summary { cursor: pointer; color: var(--accent); }
        @media (max-width: 1100px) { .hero, .layout { grid-template-columns: 1fr; } .catalog { max-height: none; } }
        </style>",
    );
    html.push_str("</head><body><div class=\"page\">");

    html.push_str("<section class=\"hero\">");
    html.push_str("<div class=\"hero-card\">");
    html.push_str("<div class=\"eyebrow\">Browser-facing comparison surface</div>");
    html.push_str("<h1>Web Host</h1>");
    html.push_str("<p>Explore plugin manifests, load payload hints into a JSON editor, and review rich response cards. This surface emphasizes human-guided discovery and browser-friendly inspection rather than raw automation.</p>");
    html.push_str("<div class=\"pill-row\" style=\"margin-top: 14px;\">");
    html.push_str("<span class=\"pill\">Manifest browsing</span>");
    html.push_str("<span class=\"pill\">Payload editing</span>");
    html.push_str("<span class=\"pill\">Result storytelling</span>");
    html.push_str("<span class=\"pill\">No network side effects</span>");
    html.push_str("</div></div>");

    html.push_str("<div class=\"hero-card\"><div class=\"eyebrow\">Loaded playground</div>");
    html.push_str("<div class=\"stats\">");
    html.push_str(&format!(
        "<div class=\"stat\"><span class=\"muted\">Plugins</span><strong>{plugin_count}</strong></div>"
    ));
    html.push_str(&format!(
        "<div class=\"stat\"><span class=\"muted\">Actions</span><strong>{action_count}</strong></div>"
    ));
    html.push_str(&format!(
        "<div class=\"stat\"><span class=\"muted\">Warnings</span><strong>{warning_count}</strong></div>"
    ));
    html.push_str("</div>");
    html.push_str(&format!(
        "<p style=\"margin-top: 12px;\"><span class=\"muted\">Plugin directory</span><br /><code>{}</code></p>",
        html_escape(&state.playground.plugin_dir().display().to_string())
    ));
    if !state.playground.warnings().is_empty() {
        html.push_str("<details><summary>Loader warnings</summary><ul class=\"list\">");
        for warning in state.playground.warnings() {
            html.push_str(&format!("<li>{}</li>", html_escape(warning)));
        }
        html.push_str("</ul></details>");
    }
    html.push_str("</div></section>");

    html.push_str("<section class=\"layout\">");
    html.push_str("<aside class=\"panel\"><h2>Plugin catalog</h2><p class=\"muted\">Pick a plugin to compare manifest metadata and action payloads.</p><div class=\"catalog\">");
    if manifests.is_empty() {
        html.push_str("<div class=\"result-card\"><strong>No plugins loaded.</strong><p class=\"muted\">Build one or more plugin crates into target/debug to populate this host.</p></div>");
    } else {
        for manifest in &manifests {
            let is_active = selected_plugin
                .map(|selected| selected.id == manifest.id)
                .unwrap_or(false);
            let action_target = manifest
                .actions
                .first()
                .map(|action| action.id.as_str())
                .unwrap_or_default();
            html.push_str(&format!(
                "<a class=\"plugin-link{}\" href=\"/?plugin={}&action={}\"><h3>{}</h3><p class=\"muted\">{}</p><div class=\"badge-row\"><span class=\"badge\">{} action{}</span><span class=\"badge\">{}</span><span class=\"badge\">{}</span></div></a>",
                if is_active { " active" } else { "" },
                manifest.id,
                action_target,
                html_escape(&manifest.name),
                html_escape(&manifest.description),
                manifest.actions.len(),
                if manifest.actions.len() == 1 { "" } else { "s" },
                html_escape(&format!("{:?}", manifest.architecture)),
                html_escape(&format!("{:?}", manifest.skill_level)),
            ));
        }
    }
    html.push_str("</div></aside>");

    html.push_str("<main class=\"panel\" style=\"display: grid; gap: 20px;\">");
    if let Some(manifest) = selected_plugin {
        html.push_str("<section>");
        html.push_str("<div class=\"eyebrow\">Selected plugin</div>");
        html.push_str(&format!(
            "<h2>{}</h2><p class=\"muted\">{}</p>",
            html_escape(&manifest.name),
            html_escape(&manifest.description)
        ));
        html.push_str("<div class=\"badge-row\" style=\"margin-top: 12px;\">");
        html.push_str(&format!(
            "<span class=\"badge\">ID: {}</span>",
            html_escape(&manifest.id)
        ));
        html.push_str(&format!(
            "<span class=\"badge\">Version {}</span>",
            html_escape(&manifest.version)
        ));
        html.push_str(&format!(
            "<span class=\"badge\">Architecture: {}</span>",
            html_escape(&format!("{:?}", manifest.architecture))
        ));
        html.push_str(&format!(
            "<span class=\"badge\">Skill: {}</span>",
            html_escape(&format!("{:?}", manifest.skill_level))
        ));
        for host in &manifest.supported_hosts {
            html.push_str(&format!(
                "<span class=\"badge\">{}</span>",
                html_escape(host.label())
            ));
        }
        html.push_str("</div>");
        html.push_str("<div class=\"panel-grid\" style=\"margin-top: 18px;\">");
        html.push_str("<div class=\"result-card\"><h3>Capabilities</h3><ul class=\"list\">");
        if manifest.capabilities.is_empty() {
            html.push_str("<li>No explicit capabilities declared.</li>");
        } else {
            for capability in &manifest.capabilities {
                html.push_str(&format!(
                    "<li><strong>{}</strong> — {}</li>",
                    html_escape(&capability.key),
                    html_escape(&capability.description)
                ));
            }
        }
        html.push_str("</ul></div>");
        html.push_str("<div class=\"result-card\"><h3>Notes & tags</h3>");
        if !manifest.tags.is_empty() {
            html.push_str("<div class=\"badge-row\" style=\"margin-bottom: 12px;\">");
            for tag in &manifest.tags {
                html.push_str(&format!(
                    "<span class=\"badge\">#{}</span>",
                    html_escape(tag)
                ));
            }
            html.push_str("</div>");
        }
        if manifest.notes.is_empty() {
            html.push_str("<p class=\"muted\">No additional notes.</p>");
        } else {
            html.push_str("<ul class=\"list\">");
            for note in &manifest.notes {
                html.push_str(&format!("<li>{}</li>", html_escape(note)));
            }
            html.push_str("</ul>");
        }
        html.push_str("</div></div></section>");

        html.push_str("<section><div class=\"eyebrow\">Action browser</div><h3>Available actions</h3><div class=\"panel-grid\">");
        for action in &manifest.actions {
            let is_active = selected_action
                .map(|selected| selected.id == action.id)
                .unwrap_or(false);
            html.push_str(&format!(
                "<div class=\"action-card{}\"><h4>{}</h4><p class=\"muted\">{}</p><div class=\"badge-row\" style=\"margin: 12px 0;\">\
                 <span class=\"badge\">action_id: {}</span>\
                 <span class=\"badge\">payload hint: {}</span>\
                 </div><p><a href=\"/?plugin={}&action={}\">Use in composer</a></p>",
                if is_active { " active" } else { "" },
                html_escape(&action.label),
                html_escape(&action.description),
                html_escape(&action.id),
                if action.payload_hint.is_some() { "available" } else { "optional" },
                manifest.id,
                action.id
            ));
            if let Some(payload_hint) = &action.payload_hint {
                html.push_str(&format!(
                    "<details><summary>Example payload</summary><pre>{}</pre></details>",
                    html_escape(payload_hint)
                ));
            }
            html.push_str("</div>");
        }
        html.push_str("</div></section>");

        html.push_str("<section class=\"panel-grid\">");
        html.push_str(
            "<div class=\"result-card\"><div class=\"eyebrow\">Invocation composer</div>",
        );
        if let Some(action) = selected_action {
            html.push_str(&format!(
                "<h3>{} → {}</h3><p class=\"muted\">Edit JSON locally, then invoke the selected plugin action as the <strong>web</strong> host.</p>",
                html_escape(&manifest.name),
                html_escape(&action.label)
            ));
            html.push_str("<form method=\"post\" action=\"/invoke\">");
            html.push_str(&format!(
                "<input type=\"hidden\" name=\"plugin_id\" value=\"{}\" />",
                html_escape(selected_plugin_id)
            ));
            html.push_str(&format!(
                "<input type=\"hidden\" name=\"action_id\" value=\"{}\" />",
                html_escape(selected_action_id)
            ));
            html.push_str("<label for=\"payload-editor\" class=\"muted\">Payload JSON</label>");
            html.push_str(&format!(
                "<textarea id=\"payload-editor\" name=\"payload\">{}</textarea>",
                html_escape(&payload_text)
            ));
            html.push_str("<div class=\"button-row\">");
            html.push_str("<button type=\"submit\">Invoke in browser workflow</button>");
            html.push_str("<button type=\"button\" class=\"secondary\" onclick=\"applyExample()\">Load payload hint</button>");
            html.push_str("<button type=\"button\" class=\"secondary\" onclick=\"formatPayload()\">Format JSON</button>");
            html.push_str(&format!(
                "<a class=\"pill\" href=\"/invoke/{}/{}?payload=%7B%7D\">Legacy GET endpoint</a>",
                html_escape(selected_plugin_id),
                html_escape(selected_action_id)
            ));
            html.push_str("</div></form>");
        } else {
            html.push_str("<h3>No action selected</h3><p class=\"muted\">Choose an action from the browser to begin editing a payload.</p>");
        }
        html.push_str("</div>");

        html.push_str("<div class=\"result-card\"><div class=\"eyebrow\">Surface comparison</div><h3>Why this host exists</h3><ul class=\"list\">");
        html.push_str("<li>Human-friendly browsing of plugin manifests and action hints.</li>");
        html.push_str("<li>Editable JSON payloads for quick what-if experiments.</li>");
        html.push_str("<li>Response cards that make mixed text, markdown, JSON, and code outputs easy to compare.</li>");
        html.push_str("</ul></div>");
        html.push_str("</section>");

        html.push_str(
            "<section class=\"result-card\"><div class=\"eyebrow\">Invocation result</div>",
        );
        if let Some(view) = invocation {
            match view.response {
                Ok(response) => {
                    html.push_str(&format!(
                        "<div class=\"status {}\"><strong>{}</strong><div class=\"muted\" style=\"margin-top: 6px;\">{}</div></div>",
                        if response.success { "success" } else { "error" },
                        html_escape(&response.title),
                        html_escape(&response.summary)
                    ));
                    html.push_str("<div class=\"panel-grid\">");
                    for output in &response.outputs {
                        html.push_str(&render_output_card(output));
                    }
                    if response.outputs.is_empty() {
                        html.push_str("<div class=\"output-card\"><h4>No output blocks</h4><p class=\"muted\">This action completed without additional blocks.</p></div>");
                    }
                    html.push_str("</div>");
                    if !response.suggested_next_steps.is_empty() {
                        html.push_str("<h4 style=\"margin-top: 18px;\">Suggested next steps</h4><ul class=\"list\">");
                        for next_step in &response.suggested_next_steps {
                            html.push_str(&format!("<li>{}</li>", html_escape(next_step)));
                        }
                        html.push_str("</ul>");
                    }
                    let raw_response =
                        serde_json::to_string_pretty(&response).unwrap_or_else(|error| {
                            format!("{{\"serialization_error\":\"{}\"}}", error)
                        });
                    html.push_str(&format!(
                        "<details><summary>Show automation-friendly JSON</summary><pre>{}</pre></details>",
                        html_escape(&raw_response)
                    ));
                }
                Err(error) => {
                    html.push_str(&format!(
                        "<div class=\"status error\"><strong>Invocation failed</strong><div class=\"muted\" style=\"margin-top: 6px;\">{}</div></div>",
                        html_escape(&error)
                    ));
                }
            }
        } else {
            html.push_str("<p class=\"muted\">Select a plugin action, adjust the payload, and invoke it to compare browser-oriented output presentation with the API-first service host.</p>");
        }
        html.push_str("</section>");
    } else {
        html.push_str("<section class=\"result-card\"><h2>No manifest selected</h2><p class=\"muted\">The web host is ready, but no plugin manifests are currently loaded.</p></section>");
    }
    html.push_str("</main></section>");

    html.push_str(&format!(
        "<script>
        const examplePayload = {payload_js};
        function applyExample() {{
            const editor = document.getElementById('payload-editor');
            if (editor) {{ editor.value = examplePayload; }}
        }}
        function formatPayload() {{
            const editor = document.getElementById('payload-editor');
            if (!editor) return;
            try {{
                editor.value = JSON.stringify(JSON.parse(editor.value), null, 2);
            }} catch (_error) {{
                window.alert('Payload is not valid JSON yet.');
            }}
        }}
        </script>"
    ));

    html.push_str("</div></body></html>");
    html
}

fn pick_plugin<'a>(
    manifests: &'a [PluginManifest],
    plugin_id: Option<&str>,
) -> Option<&'a PluginManifest> {
    plugin_id
        .and_then(|plugin_id| manifests.iter().find(|manifest| manifest.id == plugin_id))
        .or_else(|| manifests.first())
}

fn pick_action<'a>(
    manifest: &'a PluginManifest,
    action_id: Option<&str>,
) -> Option<&'a PluginAction> {
    action_id
        .and_then(|action_id| {
            manifest
                .actions
                .iter()
                .find(|action| action.id == action_id)
        })
        .or_else(|| manifest.actions.first())
}

fn render_output_card(output: &OutputBlock) -> String {
    let mut html = String::from("<div class=\"output-card\">");
    html.push_str(&format!(
        "<div class=\"badge-row\" style=\"margin-bottom: 10px;\"><span class=\"badge\">{:?}</span>{}</div>",
        output.kind,
        output
            .title
            .as_ref()
            .map(|title| format!("<span class=\"badge\">{}</span>", html_escape(title)))
            .unwrap_or_default()
    ));

    if let Some(title) = &output.title {
        html.push_str(&format!("<h4>{}</h4>", html_escape(title)));
    }
    html.push_str(&format!("<pre>{}</pre>", html_escape(&output.body)));
    html.push_str("</div>");
    html
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
