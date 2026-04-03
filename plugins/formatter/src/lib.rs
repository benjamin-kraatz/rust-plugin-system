use plugin_sdk::plugin_manifest::{
    Capability, HostKind, PluginAction, PluginArchitecture, PluginManifest, SkillLevel,
};
use plugin_sdk::plugin_protocol::{OutputKind, PluginRequest, PluginResponse};
use plugin_sdk::{JsonPlugin, export_plugin};

pub struct FormatterPlugin;

impl JsonPlugin for FormatterPlugin {
    fn manifest() -> PluginManifest {
        PluginManifest::new(
            "formatter",
            "JSON Formatter",
            "0.1.0",
            "Pretty-prints JSON payloads to show useful text-transform plugins.",
            PluginArchitecture::NativeJson,
            SkillLevel::Basic,
        )
        .with_supported_hosts(vec![HostKind::Cli, HostKind::Tui, HostKind::Web, HostKind::Service])
        .with_capabilities(vec![Capability::new(
            "formatting",
            "Reformats JSON values into human-readable output.",
        )])
        .with_tags(["formatter", "text", "json"])
        .with_actions(vec![
            PluginAction::new("pretty-json", "Pretty JSON", "Pretty-print a JSON value.")
                .with_payload_hint(r#"{"project":"rust-plugin-system","depth":2}"#),
        ])
    }

    fn invoke(request: PluginRequest) -> Result<PluginResponse, String> {
        let pretty = serde_json::to_string_pretty(&request.payload).map_err(|error| error.to_string())?;

        Ok(PluginResponse::ok(
            "formatter",
            "pretty-json",
            "Formatted JSON",
            "Rendered the incoming payload as pretty JSON.",
        )
        .with_output(OutputKind::Code, "Pretty JSON", pretty)
        .with_next_step("Use the same payload in the transformer plugin to compare behavior."))
    }
}

export_plugin!(FormatterPlugin);
