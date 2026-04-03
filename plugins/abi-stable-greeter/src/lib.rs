use abi_stable::{export_root_module, prefix_type::PrefixTypeTrait, std_types::RString};
use plugin_abi::{AbiPluginModule, AbiPluginModuleRef};
use plugin_manifest::{
    ActionContract, Capability, CapabilityContract, CapabilityRequirement, CompatibilityContract,
    DegradationRule, DegradationSeverity, ExecutionContract, ExecutionMode, HostKind,
    LifecycleContract, LifecycleHook, LifecycleState, MaintenanceContract, MaintenanceStatus,
    NetworkAccess, PluginAction, PluginArchitecture, PluginManifest, SandboxLevel,
    SchemaDescriptor, SkillLevel, TestedHost, TrustLevel, TrustMetadata, VersionRange,
    VersionStrategy,
};
use plugin_protocol::{OutputKind, PluginRequest, PluginResponse};
use serde_json::json;

fn manifest() -> PluginManifest {
    PluginManifest::new(
        "abi-stable-greeter",
        "ABI-Stable Greeter",
        "0.1.0",
        "Demonstrates a runtime-loaded plugin using abi_stable instead of raw FFI symbols.",
        PluginArchitecture::AbiStable,
        SkillLevel::Advanced,
    )
    .with_supported_hosts(vec![HostKind::Cli, HostKind::Service])
    .with_capabilities(vec![
        Capability::new(
            "abi-stable-greeting",
            "Produces a greeting through an ABI-stable root module.",
        ),
        Capability::new(
            "compatibility-handshake",
            "Summarizes compatibility windows for long-lived ABI-stable plugins.",
        ),
    ])
    .with_tags(["abi-stable", "advanced", "native", "compatibility"])
    .with_actions(vec![
        PluginAction::new(
            "greet",
            "Greet",
            "Return a greeting from an ABI-stable runtime-loaded plugin.",
        )
        .with_payload_hint(r#"{"name":"Plugin Explorer"}"#)
        .with_contract(ActionContract::new(ExecutionMode::Sync).with_timeout_ms(1_000))
        .with_input_schema(
            SchemaDescriptor::new(
                "json-schema",
                "docs/schemas/abi-stable-greeter/greet-input.schema.json",
            )
            .with_version("1.0.0"),
        )
        .with_output_schema(
            SchemaDescriptor::new(
                "json-schema",
                "docs/schemas/abi-stable-greeter/greet-output.schema.json",
            )
            .with_version("1.0.0"),
        ),
        PluginAction::new(
            "plan-upgrade",
            "Plan upgrade",
            "Summarize a host compatibility window and rollout notes for this ABI-stable plugin.",
        )
        .with_payload_hint(r#"{"from_host":"0.1.0","to_host":"0.2.0","consumer":"host-cli"}"#)
        .with_contract(ActionContract::new(ExecutionMode::Sync).with_timeout_ms(1_500))
        .with_input_schema(
            SchemaDescriptor::new(
                "json-schema",
                "docs/schemas/abi-stable-greeter/plan-upgrade-input.schema.json",
            )
            .with_version("1.0.0"),
        )
        .with_output_schema(
            SchemaDescriptor::new(
                "json-schema",
                "docs/schemas/abi-stable-greeter/plan-upgrade-output.schema.json",
            )
            .with_version("1.0.0"),
        ),
    ])
    .with_notes([
        "The root module surface stays intentionally small so additive metadata can evolve independently.",
        "Phase 4 uses this plugin to demonstrate compatibility windows and ABI-oriented rollout guidance.",
    ])
    .with_maintenance(
        MaintenanceContract::new(MaintenanceStatus::Active)
            .with_owner("plugin-platform")
            .with_support_tier("course-demo")
            .with_channel("stable"),
    )
    .with_compatibility(
        CompatibilityContract::new(VersionStrategy::Semver)
            .with_protocol_version("0.1.0")
            .with_host_version(
                VersionRange::new()
                    .with_minimum("0.1.0")
                    .with_maximum("0.3.0"),
            )
            .with_tested_hosts(vec![
                TestedHost::new(HostKind::Cli, "0.1.0")
                    .with_notes("CLI hosts validate the full manifest and action catalog."),
                TestedHost::new(HostKind::Service, "0.1.0")
                    .with_notes("Service hosts can forward structured upgrade reports downstream."),
            ])
            .with_notes([
                "The ABI-stable root module keeps greet and plan-upgrade callable while additive manifest metadata evolves.",
                "Host upgrades inside the declared window should not require recompiling consumers that only use the exported root module.",
            ]),
    )
    .with_trust(
        TrustMetadata::new(TrustLevel::Reviewed, SandboxLevel::Process, NetworkAccess::None)
            .with_data_access(["request-payload-only"])
            .with_provenance("bundled-first-party")
            .with_notes([
                "All responses are deterministic and side-effect free.",
                "This crate is intended to model long-lived ABI contracts, not privileged host access.",
            ]),
    )
    .with_lifecycle(
        LifecycleContract::new(LifecycleState::Ready)
            .with_hooks(vec![LifecycleHook::Load, LifecycleHook::Invoke, LifecycleHook::Shutdown])
            .with_health_probe("Call plan-upgrade with a canned 0.1.x -> 0.2.x transition request.")
            .with_notes([
                "The sample is stateless; shutdown exists to document the lifecycle a production ABI module would expose.",
            ]),
    )
    .with_execution(
        ExecutionContract::new(ExecutionMode::Sync)
            .with_async_support(false)
            .with_cancellable(false)
            .with_idempotent(true)
            .with_timeout_ms(1_500)
            .with_max_concurrency(8)
            .with_notes([
                "Responses are immediate because all compatibility data is embedded in the manifest.",
            ]),
    )
    .with_capability_contract(
        CapabilityContract::new()
            .with_required(vec![CapabilityRequirement::new(
                "stdout-json",
                "Hosts should be able to surface structured compatibility envelopes.",
            )])
            .with_optional(vec![CapabilityRequirement::new(
                "code-output",
                "Command snippets and ABI notes read best in code blocks.",
            )
            .with_fallback("Hosts can fall back to plain text guidance when code blocks are unavailable.")])
            .with_degradation(vec![DegradationRule::new(
                "formatted-rollout-guidance",
                "Without code-output the host should render upgrade guidance as plain text.",
                DegradationSeverity::Low,
            )
            .when_missing(["code-output"])])
            .with_notes([
                "This contract is intentionally lightweight so both CLI and service hosts stay compatible.",
            ]),
    )
}

extern "C" fn manifest_json() -> RString {
    RString::from(
        serde_json::to_string(&manifest()).expect("manifest serialization should succeed"),
    )
}

extern "C" fn invoke_json(request_json: RString) -> RString {
    let request = serde_json::from_str::<PluginRequest>(request_json.as_str());
    let response = match request {
        Ok(request) => invoke(request),
        Err(error) => PluginResponse::error(
            "abi-stable-greeter",
            "decode-request",
            "Failed to decode request",
            error.to_string(),
        ),
    };

    RString::from(serde_json::to_string(&response).expect("response serialization should succeed"))
}

fn invoke(request: PluginRequest) -> PluginResponse {
    match request.action_id.as_str() {
        "greet" => greet(request),
        "plan-upgrade" => plan_upgrade(request),
        other => PluginResponse::error(
            "abi-stable-greeter",
            other,
            "Unknown action",
            format!("unknown action '{other}'"),
        ),
    }
}

fn greet(request: PluginRequest) -> PluginResponse {
    let name = request
        .payload
        .get("name")
        .and_then(|value| value.as_str())
        .unwrap_or("Plugin Explorer");

    PluginResponse::ok(
        "abi-stable-greeter",
        "greet",
        "Hello from abi_stable",
        format!(
            "Sent an ABI-stable greeting to '{name}' for the {:?} host.",
            request.context.host
        ),
    )
    .with_output(
        OutputKind::Text,
        "Greeting",
        format!("Hello, {name}! This came through an abi_stable root module."),
    )
    .with_output(
        OutputKind::Json,
        "Greeting envelope",
        serde_json::to_string_pretty(&json!({
            "name": name,
            "host": format!("{:?}", request.context.host),
            "abi_contract": "root-module-v1",
            "compatibility_window": "0.1.0..0.3.0"
        }))
        .expect("greeting envelope serialization should succeed"),
    )
    .with_next_step(
        "Run plan-upgrade to see how the ABI-stable contract communicates upgrade guidance.",
    )
}

fn plan_upgrade(request: PluginRequest) -> PluginResponse {
    let from_host = request
        .payload
        .get("from_host")
        .and_then(|value| value.as_str())
        .unwrap_or("0.1.0");
    let to_host = request
        .payload
        .get("to_host")
        .and_then(|value| value.as_str())
        .unwrap_or("0.2.0");
    let consumer = request
        .payload
        .get("consumer")
        .and_then(|value| value.as_str())
        .unwrap_or("host-cli");
    let compatible = version_in_declared_window(from_host) && version_in_declared_window(to_host);
    let rollout = json!({
        "consumer": consumer,
        "from_host": from_host,
        "to_host": to_host,
        "protocol_version": "0.1.0",
        "compatibility_window": {
            "minimum": "0.1.0",
            "maximum": "0.3.0"
        },
        "compatible": compatible,
        "notes": [
            "Keep the exported root module symbols stable.",
            "Additive manifest metadata can grow without breaking existing consumers.",
            "Retest on both CLI and service hosts before publishing a new bundle."
        ]
    });
    let markdown = format!(
        "### ABI upgrade plan\n- Consumer: **{consumer}**\n- Host transition: **{from_host} → {to_host}**\n- Protocol: **0.1.0**\n- Window satisfied: **{compatible}**\n- Root module contract: `root-module-v1`"
    );
    let commands = format!(
        "cargo build -p abi-stable-greeter --release\ncargo run -p host-cli -- inspect abi-stable-greeter\ncargo run -p host-cli -- run abi-stable-greeter plan-upgrade '{{\"from_host\":\"{from_host}\",\"to_host\":\"{to_host}\",\"consumer\":\"{consumer}\"}}'"
    );

    PluginResponse::ok(
        "abi-stable-greeter",
        "plan-upgrade",
        "ABI upgrade plan ready",
        format!(
            "{} the ABI-stable greeter remains {} for the declared host window.",
            consumer,
            if compatible { "compatible" } else { "outside compatibility bounds" }
        ),
    )
    .with_output(
        OutputKind::Json,
        "Upgrade report",
        serde_json::to_string_pretty(&rollout)
            .expect("upgrade report serialization should succeed"),
    )
    .with_output(OutputKind::Markdown, "Upgrade summary", markdown)
    .with_output(OutputKind::Code, "Release commands", commands)
    .with_next_step("Compare this ABI rollout with the WASM and native-json packaging examples in examples/packaging.")
}

fn version_in_declared_window(version: &str) -> bool {
    ("0.1.0"..="0.3.0").contains(&version)
}

#[export_root_module]
pub fn instantiate_root_module() -> AbiPluginModuleRef {
    AbiPluginModule {
        manifest_json,
        invoke_json,
    }
    .leak_into_prefix()
}

#[cfg(test)]
mod tests {
    use super::*;
    use plugin_protocol::InvocationContext;

    fn request(action_id: &str, payload: serde_json::Value) -> PluginRequest {
        PluginRequest {
            plugin_id: "abi-stable-greeter".to_owned(),
            action_id: action_id.to_owned(),
            payload,
            context: InvocationContext::for_host(HostKind::Cli),
        }
    }

    #[test]
    fn manifest_includes_upgrade_action() {
        let manifest = manifest();
        assert!(manifest.compatibility.is_some());
        assert!(manifest.capability_contract.is_some());
        assert!(
            manifest
                .actions
                .iter()
                .any(|action| action.id == "plan-upgrade")
        );
    }

    #[test]
    fn greet_returns_json_envelope() {
        let response = greet(request("greet", json!({"name": "Phase 4"})));
        assert!(response.outputs.iter().any(|output| output.title.as_deref()
            == Some("Greeting envelope")
            && output.body.contains("root-module-v1")));
    }

    #[test]
    fn plan_upgrade_reports_window_status() {
        let response = plan_upgrade(request(
            "plan-upgrade",
            json!({"from_host": "0.1.0", "to_host": "0.2.0", "consumer": "host-cli"}),
        ));

        assert!(response.summary.contains("compatible"));
        assert!(
            response
                .outputs
                .iter()
                .any(|output| output.title.as_deref() == Some("Upgrade report")
                    && output.body.contains("\"compatible\": true"))
        );
    }
}
