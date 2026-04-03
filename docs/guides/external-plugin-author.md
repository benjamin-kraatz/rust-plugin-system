# Writing a Plugin Outside the Workspace

This guide shows you how to build a **native JSON plugin** for the Rust Plugin
System from scratch in your own Rust project, completely independent of this
repository.

For the full SDK API reference see [docs.rs/plugin-sdk](https://docs.rs/plugin-sdk).

---

## Prerequisites

* Rust toolchain (stable) — install via [rustup.rs](https://rustup.rs)
* The crates published from this repository (see
  [publishing/crates-io.md](../publishing/crates-io.md))

---

## 1. Create a new library crate

```bash
cargo new --lib my-plugin
cd my-plugin
```

---

## 2. Configure `Cargo.toml`

A plugin is compiled as a **C dynamic library** (`cdylib`).

```toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2021"

[lib]
# cdylib produces the .so / .dylib / .dll that the host loads at runtime.
crate-type = ["cdylib"]

[dependencies]
plugin-sdk = "0.1"
serde_json  = "1"

[dev-dependencies]
plugin-test-kit = "0.1"
```

> **Note:** If you also want to run unit tests that call your plugin logic
> directly (not through FFI), add `"rlib"` to `crate-type`:
>
> ```toml
> crate-type = ["cdylib", "rlib"]
> ```

---

## 3. Implement the `JsonPlugin` trait

Open `src/lib.rs` and implement your plugin:

```rust
use plugin_sdk::{JsonPlugin, export_plugin};
use plugin_sdk::plugin_manifest::{PluginManifest, PluginAction};
use plugin_sdk::plugin_protocol::{PluginRequest, PluginResponse};
use plugin_capabilities::{HostKind, PluginArchitecture, SkillLevel};

pub struct MyPlugin;

impl JsonPlugin for MyPlugin {
    fn manifest() -> PluginManifest {
        PluginManifest {
            id:           "my-plugin".into(),
            name:         "My Plugin".into(),
            version:      "0.1.0".into(),
            description:  "A sample plugin built outside the workspace.".into(),
            architecture: PluginArchitecture::NativeJson,
            skill_level:  SkillLevel::Basic,
            supported_hosts: vec![HostKind::Any],
            actions: vec![
                PluginAction {
                    id:          "greet".into(),
                    label:       "Greet".into(),
                    description: "Returns a personalised greeting.".into(),
                    payload_hint: Some(r#"{"name": "Alice"}"#.into()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }

    fn invoke(request: PluginRequest) -> Result<PluginResponse, String> {
        match request.action_id.as_str() {
            "greet" => {
                let name = request
                    .payload
                    .as_ref()
                    .and_then(|p| p["name"].as_str())
                    .unwrap_or("World");
                Ok(PluginResponse::success(
                    &request.plugin_id,
                    &request.action_id,
                    format!("Hello, {name}!"),
                    "Greeting delivered.",
                ))
            }
            other => Err(format!("unknown action: {other}")),
        }
    }
}

// Generate the three C entry points the host loads at runtime.
export_plugin!(MyPlugin);
```

### What `export_plugin!` generates

The macro expands to three `#[no_mangle]` `extern "C"` functions:

| Symbol | Purpose |
|---|---|
| `plugin_manifest_json` | Returns the manifest as a heap-allocated C string |
| `plugin_invoke_json` | Receives a request JSON C string and returns a response C string |
| `plugin_free_c_string` | Frees a C string returned by either of the above |

---

## 4. Build the plugin

```bash
cargo build --release
```

On Linux the output is `target/release/libmy_plugin.so`; on macOS
`target/release/libmy_plugin.dylib`; on Windows `target/release/my_plugin.dll`.

---

## 5. Test the plugin

### Unit tests (without FFI)

Add `"rlib"` to `crate-type` (see step 2) and write standard Rust tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use plugin_sdk::plugin_protocol::InvocationContext;
    use plugin_capabilities::HostKind;
    use serde_json::json;

    #[test]
    fn greet_returns_hello() {
        let request = plugin_sdk::plugin_protocol::PluginRequest {
            plugin_id: "my-plugin".into(),
            action_id: "greet".into(),
            payload: Some(json!({"name": "Alice"})),
            context: InvocationContext::for_host(HostKind::Cli),
        };
        let response = MyPlugin::invoke(request).unwrap();
        assert!(response.success);
        assert!(response.title.contains("Alice"));
    }

    #[test]
    fn manifest_has_correct_id() {
        let m = MyPlugin::manifest();
        assert_eq!(m.id, "my-plugin");
        assert_eq!(m.actions.len(), 1);
    }
}
```

Run with:

```bash
cargo test
```

### Using `plugin-test-kit` builders

`plugin-test-kit` provides fluent builders for constructing test manifests
and actions:

```rust
use plugin_test_kit::{ManifestBuilder, ActionBuilder};

let manifest = ManifestBuilder::new("my-plugin", "My Plugin", "0.1.0")
    .description("A test plugin")
    .action(ActionBuilder::new("greet").label("Greet").build())
    .build();

assert_eq!(manifest.id, "my-plugin");
```

---

## 6. Load the plugin in a host

Copy (or symlink) the compiled library to the directory your host scans, then
point the host at it.

### Using `host-core` (recommended)

```rust
use host_core::Playground;
use plugin_protocol::HostKind;
use serde_json::json;
use std::env;

fn main() -> anyhow::Result<()> {
    // Set the plugin directory before calling load_default(),
    // or set RUST_PLUGIN_SYSTEM_PLUGIN_DIR in the environment.
    let plugin_dir = env::var("PLUGIN_DIR").unwrap_or_else(|_| "target/release".into());
    let playground = Playground::load(plugin_dir)?;

    let response = playground.invoke(
        "my-plugin",
        "greet",
        Some(json!({"name": "World"})),
        HostKind::Cli,
    )?;
    println!("{}", response.summary);
    Ok(())
}
```

### Using `plugin-loader` directly

```rust
use plugin_loader::load_plugins_from_directory;
use plugin_protocol::{PluginRequest, InvocationContext};
use plugin_capabilities::HostKind;
use serde_json::json;

let catalog = load_plugins_from_directory("target/release")?;
let plugin = catalog.plugins
    .iter()
    .find(|p| p.manifest().id == "my-plugin")
    .expect("plugin not found");

let request = PluginRequest {
    plugin_id: "my-plugin".into(),
    action_id: "greet".into(),
    payload: Some(json!({"name": "World"})),
    context: InvocationContext::for_host(HostKind::Cli),
};
let response = plugin.invoke(&request)?;
println!("{}", response.summary);
```

---

## 7. Adding production metadata (optional but recommended)

For plugins you intend to distribute, fill in the richer manifest fields:

```rust
use plugin_manifest::{
    MaintenanceContract, CompatibilityContract, TrustMetadata, LifecycleContract,
    ExecutionContract, CapabilityContract, VersionRange,
};
use plugin_capabilities::{
    MaintenanceStatus, VersionStrategy, TrustLevel, SandboxLevel, NetworkAccess,
    LifecycleHook, LifecycleState, ExecutionMode,
};

PluginManifest {
    // … basic fields …
    maintenance: Some(MaintenanceContract {
        status: MaintenanceStatus::Active,
        owner: Some("your-name".into()),
        ..Default::default()
    }),
    compatibility: Some(CompatibilityContract {
        strategy: VersionStrategy::Semver,
        protocol_version: "0.1.0".into(),
        host_version_range: Some(VersionRange {
            min: "0.1.0".into(),
            max: Some("0.3.0".into()),
        }),
        ..Default::default()
    }),
    trust: Some(TrustMetadata {
        level: TrustLevel::Reviewed,
        sandbox: SandboxLevel::Process,
        network: NetworkAccess::None,
        ..Default::default()
    }),
    lifecycle: Some(LifecycleContract {
        state: LifecycleState::Ready,
        hooks: vec![LifecycleHook::Load, LifecycleHook::Invoke, LifecycleHook::Shutdown],
        ..Default::default()
    }),
    execution: Some(ExecutionContract {
        mode: ExecutionMode::Sync,
        max_concurrency: Some(4),
        ..Default::default()
    }),
    ..Default::default()
}
```

---

## Next steps

* [Writing a Host](./external-host-author.md) — load your plugin in a custom host.
* [Publishing to crates.io](../publishing/crates-io.md) — distribute your plugin via crates.io.
* [Publishing via GitHub](../publishing/github-packages.md) — distribute via git or a custom registry.
* [Architecture Comparison](../advanced/architecture-comparison.md) — choose between native JSON, ABI-stable, and WASM plugins.
