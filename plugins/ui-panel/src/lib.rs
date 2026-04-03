use plugin_sdk::plugin_manifest::{
    Capability, HostKind, PluginAction, PluginArchitecture, PluginManifest, SkillLevel,
};
use plugin_sdk::plugin_protocol::{OutputKind, PluginRequest, PluginResponse};
use plugin_sdk::{JsonPlugin, export_plugin};

pub struct UiPanelPlugin;

impl JsonPlugin for UiPanelPlugin {
    fn manifest() -> PluginManifest {
        PluginManifest::new(
            "ui-panel",
            "UI Panel Snapshot",
            "0.1.0",
            "Returns dashboard-friendly content for visual hosts.",
            PluginArchitecture::NativeJson,
            SkillLevel::Intermediate,
        )
        .with_supported_hosts(vec![HostKind::Egui, HostKind::Iced, HostKind::Dioxus, HostKind::Web])
        .with_capabilities(vec![Capability::new(
            "panel-content",
            "Produces markdown-friendly content that visual hosts can surface.",
        )])
        .with_tags(["ui", "dashboard", "visual"])
        .with_actions(vec![
            PluginAction::new("dashboard-snapshot", "Dashboard snapshot", "Return a compact visual summary."),
        ])
    }

    fn invoke(_request: PluginRequest) -> Result<PluginResponse, String> {
        Ok(PluginResponse::ok(
            "ui-panel",
            "dashboard-snapshot",
            "Dashboard snapshot",
            "Generated a snapshot suitable for GUI and web hosts.",
        )
        .with_output(
            OutputKind::Markdown,
            "Panel",
            "## Plugin Dashboard\n- Active plugins: 6\n- Native JSON plugins: 6\n- Suggested next lesson: compare this output in egui vs Iced vs Dioxus",
        ))
    }
}

export_plugin!(UiPanelPlugin);
