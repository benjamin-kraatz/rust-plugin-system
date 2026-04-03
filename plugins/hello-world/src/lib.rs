use plugin_sdk::plugin_manifest::{
    Capability, HostKind, PluginAction, PluginArchitecture, PluginManifest, SkillLevel,
};
use plugin_sdk::plugin_protocol::{OutputKind, PluginRequest, PluginResponse};
use plugin_sdk::{JsonPlugin, export_plugin};

pub struct HelloWorldPlugin;

impl JsonPlugin for HelloWorldPlugin {
    fn manifest() -> PluginManifest {
        PluginManifest::new(
            "hello-world",
            "Hello World",
            "0.1.0",
            "The simplest runtime-loaded plugin in the playground.",
            PluginArchitecture::NativeJson,
            SkillLevel::Basic,
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
            "greeting",
            "Returns a friendly greeting for any host surface.",
        )])
        .with_tags(["starter", "native-plugin", "teaching"])
        .with_actions(vec![
            PluginAction::new("greet", "Greet", "Say hello to a supplied name.")
                .with_payload_hint(r#"{"name":"Rustacean"}"#),
        ])
        .with_notes([
            "This plugin is meant to be the first one you inspect in the repo.",
            "It demonstrates the JSON-over-FFI boundary with minimal complexity.",
        ])
    }

    fn invoke(request: PluginRequest) -> Result<PluginResponse, String> {
        match request.action_id.as_str() {
            "greet" => {
                let name = request
                    .payload
                    .get("name")
                    .and_then(|value| value.as_str())
                    .unwrap_or("Rustacean");

                Ok(PluginResponse::ok(
                    "hello-world",
                    "greet",
                    "Hello from a runtime-loaded plugin",
                    format!("The plugin greeted '{name}' for the {:?} host.", request.context.host),
                )
                .with_output(OutputKind::Text, "Greeting", format!("Hello, {name}!"))
                .with_next_step("Try running the same plugin from another host to compare the UX."))
            }
            other => Err(format!("unknown action '{other}'")),
        }
    }
}

export_plugin!(HelloWorldPlugin);
