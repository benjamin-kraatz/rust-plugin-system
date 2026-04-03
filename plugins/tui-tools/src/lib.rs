use plugin_sdk::plugin_manifest::{
    Capability, HostKind, PluginAction, PluginArchitecture, PluginManifest, SkillLevel,
};
use plugin_sdk::plugin_protocol::{OutputKind, PluginRequest, PluginResponse};
use plugin_sdk::{JsonPlugin, export_plugin};
use serde_json::{Value, json};

const DEFAULT_WIDTH: u16 = 120;
const DEFAULT_HEIGHT: u16 = 36;
const MIN_WIDTH: u16 = 40;
const MAX_WIDTH: u16 = 240;
const MIN_HEIGHT: u16 = 12;
const MAX_HEIGHT: u16 = 100;
const STATUS_HEIGHT: u16 = 1;
const HELP_HEIGHT: u16 = 3;
const MIN_SIDEBAR_WIDTH: u16 = 16;
const MIN_FOOTER_HEIGHT: u16 = 4;
const MIN_MAIN_HEIGHT: u16 = 6;

pub struct TuiToolsPlugin;

impl JsonPlugin for TuiToolsPlugin {
    fn manifest() -> PluginManifest {
        PluginManifest::new(
            "tui-tools",
            "TUI Workflow Tools",
            "0.1.0",
            "Deterministic terminal layout and status helpers for CLI/TUI workflows.",
            PluginArchitecture::NativeJson,
            SkillLevel::Intermediate,
        )
        .with_supported_hosts(vec![HostKind::Cli, HostKind::Tui])
        .with_capabilities(vec![
            Capability::new(
                "layout-planning",
                "Calculates stable pane geometry for terminal dashboards and workbenches.",
            ),
            Capability::new(
                "status-composition",
                "Builds compact status lines from structured terminal session state.",
            ),
        ])
        .with_tags([
            "tui",
            "terminal",
            "layout",
            "status-line",
            "deterministic",
            "native-plugin",
        ])
        .with_actions(vec![
            PluginAction::new(
                "plan-layout",
                "Plan layout",
                "Compute a predictable terminal pane layout for a TUI workflow.",
            )
            .with_payload_hint(
                r#"{"width":120,"height":36,"panes":["nav","editor","logs"],"focus":"editor","include_help":true}"#,
            ),
            PluginAction::new(
                "draft-status-line",
                "Draft status line",
                "Compose a compact terminal status line from structured session state.",
            )
            .with_payload_hint(
                r#"{"mode":"NORMAL","workspace":"rust-plugin-system","branch":"main","dirty":false,"pending_tasks":2,"focus":"editor"}"#,
            ),
        ])
        .with_notes([
            "All actions are pure, deterministic, and derived only from the JSON payload.",
            "The CLI host can exercise these actions even though they are tailored for TUI surfaces.",
        ])
    }

    fn invoke(request: PluginRequest) -> Result<PluginResponse, String> {
        match request.action_id.as_str() {
            "plan-layout" => plan_layout(request),
            "draft-status-line" => draft_status_line(request),
            other => Err(format!("unknown action '{other}'")),
        }
    }
}

fn plan_layout(request: PluginRequest) -> Result<PluginResponse, String> {
    let width = payload_u16(
        &request.payload,
        "width",
        DEFAULT_WIDTH,
        MIN_WIDTH,
        MAX_WIDTH,
    );
    let height = payload_u16(
        &request.payload,
        "height",
        DEFAULT_HEIGHT,
        MIN_HEIGHT,
        MAX_HEIGHT,
    );
    let include_help = payload_bool(&request.payload, "include_help");
    let reserved_help = if include_help { HELP_HEIGHT } else { 0 };
    let available_height = height.saturating_sub(STATUS_HEIGHT + reserved_help);
    if available_height < MIN_MAIN_HEIGHT {
        return Err(format!(
            "height {height} is too small to reserve status/help rows while keeping a usable content area"
        ));
    }

    let panes = payload_strings(&request.payload, "panes", &["nav", "editor", "logs"]);
    let focus = request
        .payload
        .get("focus")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| default_focus(&panes));
    let sidebar_width = payload_u16(
        &request.payload,
        "sidebar_width",
        default_sidebar_width(width),
        MIN_SIDEBAR_WIDTH,
        width.saturating_sub(MIN_MAIN_HEIGHT),
    );
    let footer_height = payload_u16(
        &request.payload,
        "footer_height",
        default_footer_height(height),
        MIN_FOOTER_HEIGHT,
        available_height.saturating_sub(MIN_MAIN_HEIGHT),
    );

    let mut pane_layouts = Vec::new();
    if panes.len() <= 1 {
        let name = panes
            .first()
            .cloned()
            .unwrap_or_else(|| String::from("editor"));
        pane_layouts.push(json!({
            "name": name,
            "role": "main",
            "x": 0,
            "y": 0,
            "width": width,
            "height": available_height,
            "focused": focus == name,
        }));
    } else if panes.len() == 2 {
        let side_width = sidebar_width.min(width.saturating_sub(MIN_MAIN_HEIGHT));
        let main_width = width.saturating_sub(side_width);
        pane_layouts.push(json!({
            "name": panes[0],
            "role": "sidebar",
            "x": 0,
            "y": 0,
            "width": side_width,
            "height": available_height,
            "focused": focus == panes[0],
        }));
        pane_layouts.push(json!({
            "name": panes[1],
            "role": "main",
            "x": side_width,
            "y": 0,
            "width": main_width,
            "height": available_height,
            "focused": focus == panes[1],
        }));
    } else {
        let side_width = sidebar_width.min(width.saturating_sub(MIN_MAIN_HEIGHT));
        let right_width = width.saturating_sub(side_width);
        let bottom_height = footer_height.min(available_height.saturating_sub(MIN_MAIN_HEIGHT));
        let top_height = available_height.saturating_sub(bottom_height);
        let main_name = panes[1].clone();
        let main_tabs = panes[1..panes.len() - 1].to_vec();
        let active_tab = if main_tabs.iter().any(|pane| pane == &focus) {
            focus.clone()
        } else {
            main_name.clone()
        };
        let overflow_tabs = main_tabs
            .iter()
            .filter(|pane| *pane != &active_tab)
            .cloned()
            .collect::<Vec<_>>();
        let footer_name = panes
            .last()
            .cloned()
            .unwrap_or_else(|| String::from("logs"));

        pane_layouts.push(json!({
            "name": panes[0],
            "role": "sidebar",
            "x": 0,
            "y": 0,
            "width": side_width,
            "height": available_height,
            "focused": focus == panes[0],
        }));
        pane_layouts.push(json!({
            "name": active_tab,
            "role": "main",
            "x": side_width,
            "y": 0,
            "width": right_width,
            "height": top_height,
            "focused": main_tabs.iter().any(|pane| pane == &focus),
            "overflow_tabs": overflow_tabs,
        }));
        pane_layouts.push(json!({
            "name": footer_name,
            "role": "footer",
            "x": side_width,
            "y": top_height,
            "width": right_width,
            "height": bottom_height,
            "focused": focus == footer_name,
        }));
    }

    if include_help {
        pane_layouts.push(json!({
            "name": "help",
            "role": "help",
            "x": 0,
            "y": available_height,
            "width": width,
            "height": HELP_HEIGHT,
            "focused": false,
        }));
    }

    let status_line = format!(
        "{} {}x{} | focus={} | panes={} | mode={}",
        request.context.host.label(),
        width,
        height,
        focus,
        panes.join("/"),
        request.context.mode.as_deref().unwrap_or("interactive"),
    );
    let layout_json = json!({
        "terminal": {
            "width": width,
            "height": height,
            "host": request.context.host.label(),
        },
        "status_line": status_line,
        "panes": pane_layouts,
    });
    let layout_text = render_layout_text(width, height, &layout_json["panes"]);

    Ok(PluginResponse::ok(
        "tui-tools",
        "plan-layout",
        "Terminal layout planned",
        format!(
            "Built a deterministic {} pane plan for a {} terminal workspace.",
            panes.len(),
            request.context.host.label(),
        ),
    )
    .with_output(OutputKind::Code, "Layout plan", layout_text)
    .with_output(
        OutputKind::Json,
        "Layout JSON",
        serde_json::to_string_pretty(&layout_json).map_err(|error| error.to_string())?,
    )
    .with_next_step("Adjust width, height, or pane names to model another terminal surface."))
}

fn draft_status_line(request: PluginRequest) -> Result<PluginResponse, String> {
    let mode = request
        .payload
        .get("mode")
        .and_then(Value::as_str)
        .unwrap_or("NORMAL");
    let workspace = request
        .payload
        .get("workspace")
        .and_then(Value::as_str)
        .unwrap_or("workspace");
    let branch = request
        .payload
        .get("branch")
        .and_then(Value::as_str)
        .unwrap_or("main");
    let focus = request
        .payload
        .get("focus")
        .and_then(Value::as_str)
        .unwrap_or("editor");
    let pending_tasks = request
        .payload
        .get("pending_tasks")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let dirty = payload_bool(&request.payload, "dirty");
    let recording = payload_bool(&request.payload, "recording_macro");

    let mut segments = vec![
        format!("[{mode}]"),
        workspace.to_owned(),
        format!("branch:{branch}"),
        format!("focus:{focus}"),
        format!("pending:{pending_tasks}"),
    ];
    segments.push(if dirty {
        String::from("state:dirty")
    } else {
        String::from("state:clean")
    });
    if recording {
        segments.push(String::from("macro:recording"));
    }

    let line = segments.join(" | ");
    let status_json = json!({
        "host": request.context.host.label(),
        "segments": segments,
        "line": line,
    });

    Ok(PluginResponse::ok(
        "tui-tools",
        "draft-status-line",
        "Status line drafted",
        "Composed a terminal-safe status line from structured workflow state.",
    )
    .with_output(OutputKind::Text, "Status line", line)
    .with_output(
        OutputKind::Json,
        "Status line JSON",
        serde_json::to_string_pretty(&status_json).map_err(|error| error.to_string())?,
    )
    .with_next_step("Feed different mode, branch, or pending task counts to compare layouts."))
}

fn payload_u16(payload: &Value, key: &str, default: u16, min: u16, max: u16) -> u16 {
    let bounded_default = default.clamp(min, max);
    let Some(value) = payload.get(key).and_then(Value::as_u64) else {
        return bounded_default;
    };

    value.clamp(min.into(), max.into()) as u16
}

fn payload_bool(payload: &Value, key: &str) -> bool {
    payload.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn payload_strings(payload: &Value, key: &str, defaults: &[&str]) -> Vec<String> {
    let parsed = payload
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if parsed.is_empty() {
        defaults.iter().map(|item| (*item).to_owned()).collect()
    } else {
        parsed
    }
}

fn default_focus(panes: &[String]) -> String {
    panes
        .iter()
        .find(|pane| pane.as_str() == "editor")
        .cloned()
        .or_else(|| panes.get(1).cloned())
        .or_else(|| panes.first().cloned())
        .unwrap_or_else(|| String::from("editor"))
}

fn default_sidebar_width(width: u16) -> u16 {
    if width >= 120 {
        28
    } else if width >= 90 {
        24
    } else {
        20
    }
}

fn default_footer_height(height: u16) -> u16 {
    if height >= 40 {
        8
    } else if height >= 28 {
        6
    } else {
        4
    }
}

fn render_layout_text(width: u16, height: u16, panes: &Value) -> String {
    let mut lines = vec![format!("terminal {width}x{height}")];
    if let Some(panes) = panes.as_array() {
        for pane in panes {
            let name = pane.get("name").and_then(Value::as_str).unwrap_or("pane");
            let role = pane.get("role").and_then(Value::as_str).unwrap_or("main");
            let x = pane.get("x").and_then(Value::as_u64).unwrap_or(0);
            let y = pane.get("y").and_then(Value::as_u64).unwrap_or(0);
            let pane_width = pane.get("width").and_then(Value::as_u64).unwrap_or(0);
            let pane_height = pane.get("height").and_then(Value::as_u64).unwrap_or(0);
            let focused = if pane
                .get("focused")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                " focus"
            } else {
                ""
            };
            lines.push(format!(
                "- {name:<12} role={role:<7} x={x:<3} y={y:<3} w={pane_width:<3} h={pane_height:<3}{focused}",
            ));
        }
    }
    lines.join("\n")
}

export_plugin!(TuiToolsPlugin);

#[cfg(test)]
mod tests {
    use super::*;
    use plugin_sdk::plugin_protocol::InvocationContext;

    #[test]
    fn manifest_exposes_tui_friendly_metadata() {
        let manifest = TuiToolsPlugin::manifest();
        assert_eq!(manifest.id, "tui-tools");
        assert!(manifest.supported_hosts.contains(&HostKind::Tui));
        assert!(
            manifest
                .actions
                .iter()
                .any(|action| action.id == "plan-layout")
        );
    }

    #[test]
    fn plan_layout_uses_defaults_for_empty_payload() {
        let response = plan_layout(PluginRequest {
            plugin_id: String::from("tui-tools"),
            action_id: String::from("plan-layout"),
            payload: json!({}),
            context: InvocationContext::for_host(HostKind::Tui),
        })
        .expect("layout response");

        assert!(response.success);
        assert!(
            response
                .outputs
                .iter()
                .any(|output| output.body.contains("terminal 120x36"))
        );
    }

    #[test]
    fn draft_status_line_reflects_dirty_recording_state() {
        let response = draft_status_line(PluginRequest {
            plugin_id: String::from("tui-tools"),
            action_id: String::from("draft-status-line"),
            payload: json!({
                "mode": "INSERT",
                "workspace": "rust-plugin-system",
                "branch": "main",
                "dirty": true,
                "pending_tasks": 3,
                "recording_macro": true,
            }),
            context: InvocationContext::for_host(HostKind::Cli),
        })
        .expect("status line response");

        let line = &response.outputs[0].body;
        assert!(line.contains("[INSERT]"));
        assert!(line.contains("state:dirty"));
        assert!(line.contains("macro:recording"));
    }
}
