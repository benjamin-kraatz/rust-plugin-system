# Building Your Own Host and Plugin

Use this guide when you want to create a host app or a plugin crate in a separate repository and still reuse the shared contract crates from this workspace.

## What to depend on

### Plugin projects

Start with:

- `plugin-sdk` for the export macro and re-exported protocol modules
- `plugin-manifest` for manifest and action types
- `plugin-protocol` for requests, responses, and output blocks
- `plugin-capabilities` if you want capability metadata directly

### Host projects

Start with:

- `host-core` for shared discovery and invocation helpers
- `plugin-loader` for native JSON plugin loading
- `plugin-runtime` for rendering shared responses
- `plugin-manifest` and `plugin-protocol` for manifest and request/response types

## Install the toolchain

```bash
rustup toolchain install stable
rustup default stable
cargo --version
```

The workspace uses Rust 2024 edition and requires a recent stable toolchain.

## Minimal plugin crate

`Cargo.toml`:

```toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
plugin-sdk = "0.1"
serde_json = "1"
```

`src/lib.rs`:

```rust
use plugin_sdk::{
    export_plugin, JsonPlugin,
    plugin_manifest::{HostKind, PluginAction, PluginArchitecture, PluginManifest, SkillLevel},
    plugin_protocol::{OutputKind, PluginRequest, PluginResponse},
};

pub struct GreeterPlugin;

impl JsonPlugin for GreeterPlugin {
    fn manifest() -> PluginManifest {
        PluginManifest::new(
            "greeter",
            "Greeter",
            env!("CARGO_PKG_VERSION"),
            "A small greeting plugin.",
            PluginArchitecture::NativeJson,
            SkillLevel::Basic,
        )
        .with_supported_hosts(vec![HostKind::Cli, HostKind::Service])
        .with_actions(vec![PluginAction::new(
            "greet",
            "Greet",
            "Return a friendly greeting.",
        )])
    }

    fn invoke(request: PluginRequest) -> Result<PluginResponse, String> {
        let name = request
            .payload
            .get("name")
            .and_then(|value| value.as_str())
            .unwrap_or("friend");

        Ok(PluginResponse::ok(
            "greeter",
            "greet",
            "Greeting ready",
            format!("Hello, {name}!"),
        )
        .with_output(OutputKind::Text, "Greeting", format!("Hello, {name}!")))
    }
}

export_plugin!(GreeterPlugin);
```

## Minimal host crate

`Cargo.toml`:

```toml
[package]
name = "my-host"
version = "0.1.0"
edition = "2024"

[dependencies]
host-core = "0.1"
plugin-protocol = "0.1"
serde_json = "1"
```

`src/main.rs`:

```rust
use host_core::{default_plugin_dir, render_response, Playground};
use plugin_protocol::HostKind;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = Playground::load(default_plugin_dir())?;

    for manifest in host.manifests() {
        println!("{} ({})", manifest.name, manifest.id);
    }

    let response = host.invoke_text(
        "greeter",
        "greet",
        r#"{"name":"Ada"}"#,
        HostKind::Cli,
    )?;

    println!("{}", render_response(&response));
    Ok(())
}
```

## Project layout

Suggested structure:

```text
my-host/
my-plugin/
```

Keep the host and plugin in separate repositories when you want them to evolve independently.

## Runtime setup

1. Build the plugin as a dynamic library.
2. Copy the compiled library next to its manifest.
3. Point the host at that directory.

The default host helper looks in `target/debug`, or you can override it with `RUST_PLUGIN_SYSTEM_PLUGIN_DIR`.

## Defining your own contract

Use these pieces consistently:

- `PluginManifest` for the plugin’s identity and supported hosts
- `PluginAction` for each callable action
- `PluginRequest` for host-to-plugin calls
- `PluginResponse` for plugin output
- `OutputKind` for framing text, JSON, code, or markdown

Keep the manifest stable across releases and bump the package version when you change the protocol or action shape.

## Related docs

- [`docs/reference/version-compatibility.md`](../reference/version-compatibility.md)
- [`docs/reference/testing-packaging.md`](../reference/testing-packaging.md)
- [`docs/reference/production-contracts.md`](../reference/production-contracts.md)
