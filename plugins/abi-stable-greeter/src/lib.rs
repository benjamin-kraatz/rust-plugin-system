use abi_stable::{export_root_module, prefix_type::PrefixTypeTrait, std_types::RString};
use plugin_abi::{AbiPluginModule, AbiPluginModuleRef};
use plugin_manifest::{
    Capability, HostKind, PluginAction, PluginArchitecture, PluginManifest, SkillLevel,
};
use plugin_protocol::{OutputKind, PluginRequest, PluginResponse};

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
    .with_capabilities(vec![Capability::new(
        "abi-stable-greeting",
        "Produces a greeting through an ABI-stable root module.",
    )])
    .with_tags(["abi-stable", "advanced", "native"])
    .with_actions(vec![
        PluginAction::new(
            "greet",
            "Greet",
            "Return a greeting from an ABI-stable runtime-loaded plugin.",
        )
        .with_payload_hint(r#"{"name":"Plugin Explorer"}"#),
    ])
}

extern "C" fn manifest_json() -> RString {
    RString::from(
        serde_json::to_string(&manifest()).expect("manifest serialization should succeed"),
    )
}

extern "C" fn invoke_json(request_json: RString) -> RString {
    let request = serde_json::from_str::<PluginRequest>(request_json.as_str());
    let response = match request {
        Ok(request) => {
            let name = request
                .payload
                .get("name")
                .and_then(|value| value.as_str())
                .unwrap_or("Plugin Explorer");

            PluginResponse::ok(
                "abi-stable-greeter",
                "greet",
                "Hello from abi_stable",
                format!("Sent an ABI-stable greeting to '{name}'."),
            )
            .with_output(
                OutputKind::Text,
                "Greeting",
                format!("Hello, {name}! This came through an abi_stable root module."),
            )
            .with_next_step("Compare this result with the native JSON dylib greeting plugin.")
        }
        Err(error) => PluginResponse::error(
            "abi-stable-greeter",
            "greet",
            "Failed to decode request",
            error.to_string(),
        ),
    };

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
