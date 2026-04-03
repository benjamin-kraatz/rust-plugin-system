use plugin_sdk::plugin_manifest::{
    Capability, HostKind, PluginAction, PluginArchitecture, PluginManifest, SkillLevel,
};
use plugin_sdk::plugin_protocol::{OutputKind, PluginRequest, PluginResponse};
use plugin_sdk::{JsonPlugin, export_plugin};

pub struct CommandPackPlugin;

impl JsonPlugin for CommandPackPlugin {
    fn manifest() -> PluginManifest {
        PluginManifest::new(
            "command-pack",
            "Command Pack",
            "0.1.0",
            "Suggests useful cargo and workspace commands for learning and automation.",
            PluginArchitecture::NativeJson,
            SkillLevel::Intermediate,
        )
        .with_supported_hosts(vec![HostKind::Cli, HostKind::Tui, HostKind::Service])
        .with_capabilities(vec![Capability::new(
            "command-suggestions",
            "Returns curated command snippets for common workflows.",
        )])
        .with_tags(["commands", "teaching", "tooling"])
        .with_actions(vec![PluginAction::new(
            "suggest-commands",
            "Suggest commands",
            "Recommend useful workspace commands.",
        )])
    }

    fn invoke(_request: PluginRequest) -> Result<PluginResponse, String> {
        Ok(PluginResponse::ok(
            "command-pack",
            "suggest-commands",
            "Suggested commands",
            "Generated a compact command pack for the plugin playground.",
        )
        .with_output(
            OutputKind::Code,
            "Commands",
            "cargo build --workspace\ncargo run -p host-cli -- list\ncargo run -p host-cli -- inspect hello-world\ncargo run -p host-cli -- run formatter pretty-json '{\"hello\":\"world\"}'",
        ))
    }
}

export_plugin!(CommandPackPlugin);
