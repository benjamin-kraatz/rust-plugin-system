use abi_stable::{export_root_module, prefix_type::PrefixTypeTrait, std_types::RString};
use plugin_abi::{AbiPluginModule, AbiPluginModuleRef};
use plugin_manifest::{
    Capability, CapabilityContract, CapabilityRequirement, CompatibilityContract, DegradationRule,
    DegradationSeverity, ExecutionContract, ExecutionMode, HostKind, LifecycleContract,
    LifecycleHook, LifecycleState, NetworkAccess, PluginAction, PluginArchitecture, PluginManifest,
    SandboxLevel, SkillLevel, TestedHost, TrustLevel, TrustMetadata, VersionRange, VersionStrategy,
};
use plugin_protocol::{OutputKind, PluginResponse};

fn manifest() -> PluginManifest {
    PluginManifest::new(
        "abi-stable-command-pack",
        "ABI-Stable Command Pack",
        "0.1.0",
        "Shows abi_stable plugins returning curated command guidance for plugin development.",
        PluginArchitecture::AbiStable,
        SkillLevel::Advanced,
    )
    .with_supported_hosts(vec![HostKind::Cli, HostKind::Service])
    .with_capabilities(vec![Capability::new(
        "abi-stable-commands",
        "Returns command recommendations from an ABI-stable plugin.",
    )])
    .with_tags(["abi-stable", "commands", "advanced"])
    .with_actions(vec![PluginAction::new(
        "suggest",
        "Suggest commands",
        "Return commands for exploring advanced plugin tracks.",
    )])
    .with_compatibility(
        CompatibilityContract::new(VersionStrategy::Semver)
            .with_protocol_version("0.1.0")
            .with_host_version(
                VersionRange::new()
                    .with_minimum("0.1.0")
                    .with_maximum("0.2.0"),
            )
            .with_tested_hosts(vec![
                TestedHost::new(HostKind::Cli, "0.1.0"),
                TestedHost::new(HostKind::Service, "0.1.0"),
            ]),
    )
    .with_trust(
        TrustMetadata::new(
            TrustLevel::Reviewed,
            SandboxLevel::Process,
            NetworkAccess::None,
        )
        .with_data_access(["request-payload-only"])
        .with_provenance("bundled-first-party"),
    )
    .with_lifecycle(
        LifecycleContract::new(LifecycleState::Ready)
            .with_hooks(vec![LifecycleHook::Load, LifecycleHook::Invoke]),
    )
    .with_execution(
        ExecutionContract::new(ExecutionMode::Sync)
            .with_async_support(false)
            .with_timeout_ms(1_000)
            .with_notes(["Response generation is immediate and side-effect free."]),
    )
    .with_capability_contract(
        CapabilityContract::new()
            .with_required(vec![CapabilityRequirement::new(
                "code-output",
                "The plugin is most useful when the host can present shell commands verbatim.",
            )])
            .with_optional(vec![CapabilityRequirement::new(
                "stdout-text",
                "A text fallback keeps the command pack readable in plain terminals.",
            )])
            .with_degradation(vec![
                DegradationRule::new(
                    "formatted-command-blocks",
                    "Without code-output the host should fall back to raw text lines.",
                    DegradationSeverity::Medium,
                )
                .when_missing(["code-output"]),
            ]),
    )
}

extern "C" fn manifest_json() -> RString {
    RString::from(
        serde_json::to_string(&manifest()).expect("manifest serialization should succeed"),
    )
}

extern "C" fn invoke_json(_request_json: RString) -> RString {
    let response = PluginResponse::ok(
        "abi-stable-command-pack",
        "suggest",
        "ABI-stable command pack",
        "Generated commands for exploring ABI-stable and WASM tracks.",
    )
    .with_output(
        OutputKind::Code,
        "Commands",
        "cargo build -p abi-stable-greeter -p abi-stable-command-pack\ncargo run -p host-cli -- list\ncargo run -p host-cli -- inspect abi-stable-greeter\ncargo run -p host-cli -- run abi-stable-command-pack suggest '{}'",
    );

    RString::from(serde_json::to_string(&response).expect("response serialization should succeed"))
}

#[export_root_module]
pub fn instantiate_root_module() -> AbiPluginModuleRef {
    AbiPluginModule {
        manifest_json,
        invoke_json,
    }
    .leak_into_prefix()
}
