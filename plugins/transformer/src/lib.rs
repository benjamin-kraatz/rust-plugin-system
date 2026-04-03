use plugin_sdk::plugin_manifest::{
    Capability, HostKind, PluginAction, PluginArchitecture, PluginManifest, SkillLevel,
};
use plugin_sdk::plugin_protocol::{OutputKind, PluginRequest, PluginResponse};
use plugin_sdk::{JsonPlugin, export_plugin};

pub struct TransformerPlugin;

impl JsonPlugin for TransformerPlugin {
    fn manifest() -> PluginManifest {
        PluginManifest::new(
            "transformer",
            "Slug Transformer",
            "0.1.0",
            "Transforms arbitrary text into a filesystem and URL friendly slug.",
            PluginArchitecture::NativeJson,
            SkillLevel::Intermediate,
        )
        .with_supported_hosts(vec![
            HostKind::Cli,
            HostKind::Tui,
            HostKind::Egui,
            HostKind::Iced,
            HostKind::Dioxus,
            HostKind::Web,
            HostKind::Service,
        ])
        .with_capabilities(vec![Capability::new(
            "text-transform",
            "Converts free text into a predictable slug.",
        )])
        .with_tags(["transform", "slug", "utility"])
        .with_actions(vec![
            PluginAction::new("slugify", "Slugify", "Convert a title into a slug.")
                .with_payload_hint(r#"{"text":"Rust Plugin Systems Course Module"}"#),
        ])
    }

    fn invoke(request: PluginRequest) -> Result<PluginResponse, String> {
        let input = request
            .payload
            .get("text")
            .and_then(|value| value.as_str())
            .unwrap_or("Rust Plugin Systems Course Module");

        let slug = input
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() {
                    character.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .split('-')
            .filter(|segment| !segment.is_empty())
            .collect::<Vec<_>>()
            .join("-");

        Ok(PluginResponse::ok(
            "transformer",
            "slugify",
            "Slug created",
            format!("Converted '{input}' into a stable slug."),
        )
        .with_output(OutputKind::Text, "Slug", slug)
        .with_next_step("Try the same action from the desktop hosts to compare rendering."))
    }
}

export_plugin!(TransformerPlugin);
