use plugin_sdk::plugin_manifest::{
    Capability, HostKind, PluginAction, PluginArchitecture, PluginManifest, SkillLevel,
};
use plugin_sdk::plugin_protocol::{OutputKind, PluginRequest, PluginResponse};
use plugin_sdk::{JsonPlugin, export_plugin};

pub struct LoggerPlugin;

impl JsonPlugin for LoggerPlugin {
    fn manifest() -> PluginManifest {
        PluginManifest::new(
            "logger",
            "Structured Logger Demo",
            "0.1.0",
            "Demonstrates plugin-produced logs and operational metadata.",
            PluginArchitecture::NativeJson,
            SkillLevel::Basic,
        )
        .with_supported_hosts(vec![
            HostKind::Cli,
            HostKind::Tui,
            HostKind::Web,
            HostKind::Service,
        ])
        .with_capabilities(vec![Capability::new(
            "logging",
            "Returns log lines that hosts can render or forward.",
        )])
        .with_tags(["operations", "logging", "observability"])
        .with_actions(vec![
            PluginAction::new(
                "emit-demo-log",
                "Emit demo log",
                "Generate a small structured log batch.",
            )
            .with_payload_hint(r#"{"service":"host-service","level":"info"}"#),
        ])
    }

    fn invoke(request: PluginRequest) -> Result<PluginResponse, String> {
        let service = request
            .payload
            .get("service")
            .and_then(|value| value.as_str())
            .unwrap_or("demo-service");
        let level = request
            .payload
            .get("level")
            .and_then(|value| value.as_str())
            .unwrap_or("info");

        Ok(PluginResponse::ok(
            "logger",
            "emit-demo-log",
            "Structured log batch",
            format!("Generated example '{level}' logs for '{service}'."),
        )
        .with_output(
            OutputKind::Code,
            "Logs",
            format!(
                "{{\"ts\":\"2026-04-03T08:00:00Z\",\"level\":\"{level}\",\"service\":\"{service}\",\"message\":\"plugin host started\"}}\n{{\"ts\":\"2026-04-03T08:00:01Z\",\"level\":\"{level}\",\"service\":\"{service}\",\"message\":\"plugin invocation complete\"}}"
            ),
        )
        .with_next_step("Use this plugin as a baseline for metrics and tracing examples."))
    }
}

export_plugin!(LoggerPlugin);
