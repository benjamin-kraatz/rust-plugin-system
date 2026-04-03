# Contributor Guide

How to add plugins and hosts, the development workflow, and project conventions.

## How to Add a New Plugin

### 1. Create a crate under `plugins/`

```bash
cargo new --lib plugins/your-plugin
```

### 2. Set up `Cargo.toml`

```toml
[package]
name = "your-plugin"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
plugin-sdk = { version = "0.1.0", path = "../../crates/plugin-sdk" }
serde_json = "1.0.149"
```

`cdylib` produces the shared library loaded at runtime. `rlib` enables unit tests.

### 3. Implement `JsonPlugin`

```rust
use plugin_sdk::plugin_manifest::{PluginAction, PluginArchitecture, PluginManifest, SkillLevel};
use plugin_sdk::plugin_protocol::{PluginRequest, PluginResponse};
use plugin_sdk::{JsonPlugin, export_plugin};

pub struct YourPlugin;

impl JsonPlugin for YourPlugin {
    fn manifest() -> PluginManifest {
        PluginManifest::new(
            "your-plugin",
            "Your Plugin",
            "0.1.0",
            "A short description of what the plugin does.",
            PluginArchitecture::NativeJson,
            SkillLevel::Basic,
        )
        .with_actions(vec![
            PluginAction::new("do-thing", "Do Thing", "Explain the action.")
                .with_payload_hint(r#"{"key":"value"}"#),
        ])
    }

    fn invoke(request: PluginRequest) -> Result<PluginResponse, String> {
        match request.action_id.as_str() {
            "do-thing" => Ok(PluginResponse::ok(
                "your-plugin", "do-thing", "Title", "Summary",
            )),
            other => Err(format!("unknown action '{other}'")),
        }
    }
}

export_plugin!(YourPlugin);
```

The `export_plugin!` macro generates three FFI exports:

- `plugin_manifest_json` — returns the manifest as a JSON C-string
- `plugin_invoke_json` — accepts a JSON request, returns a JSON response
- `plugin_free_c_string` — frees a C-string previously returned by the plugin

### 4. Add to workspace members

In the root `Cargo.toml`, add `"plugins/your-plugin"` to the `[workspace]
members` list.

### 5. Build and verify

```bash
cargo build -p your-plugin
cargo run -p host-cli -- list          # should show your plugin
cargo run -p host-cli -- inspect your-plugin
cargo run -p host-cli -- run your-plugin do-thing '{"key":"value"}'
```
---

## How to Add a New Host

### 1. Create a crate under `hosts/`

```bash
cargo new hosts/your-host
```

### 2. Add `host-core` as a dependency

```toml
[dependencies]
host-core = { version = "0.1.0", path = "../../crates/host-core" }
plugin-protocol = { version = "0.1.0", path = "../../crates/plugin-protocol" }
anyhow = "1"
```

### 3. Discover plugins and invoke actions

```rust
use anyhow::Result;
use host_core::Playground;
use plugin_protocol::HostKind;

fn main() -> Result<()> {
    let playground = Playground::load_default()?;
    for manifest in playground.manifests() {
        for action in &manifest.actions {
            let response = playground.invoke_text(
                &manifest.id, &action.id,
                action.payload_hint.as_deref().unwrap_or("{}"),
                HostKind::Cli,
            )?;
            println!("{}", host_core::render_response(&response));
        }
    }
    Ok(())
}
```

### 4. Add to workspace members

Add `"hosts/your-host"` to the root `Cargo.toml` `[workspace] members` list.
---

## Development Workflow

```bash
# Build everything
cargo build --workspace

# Run tests
cargo test --workspace

# Lint
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --all
```

The workspace enforces strict Clippy lints (`unsafe_op_in_unsafe_fn`,
`dbg_macro`, `todo`, `unwrap_used` — all denied).

---

## Project Conventions

- **Manifests** — every plugin returns a `PluginManifest` (from
  `plugin-manifest`) declaring id, name, version, architecture, and skill level.
- **Invocations** — hosts build an `InvocationContext` and send a
  `PluginRequest`; plugins return a `PluginResponse` (from `plugin-protocol`).
- **Supported hosts** — use `.with_supported_hosts(vec![...])` with `HostKind`
  variants (Cli, Tui, Egui, Iced, Dioxus, Web, Service, Any).
- **Skill level / Architecture** — `SkillLevel::Basic|Intermediate|Advanced`,
  `PluginArchitecture::NativeJson|AbiStable|Wasm`.
- **Payload** — use `serde_json`; provide a `payload_hint` on each action.
- **Production metadata** — `ExecutionContract`, `CompatibilityContract`,
  `MaintenanceContract`, and `TrustMetadata` are optional but encouraged.
---

## Architecture Quick Reference

| Approach | How it works |
|---|---|
| **Native JSON** | `cdylib` loaded with `libloading`. JSON strings pass over the FFI boundary via `*mut c_char`. |
| **ABI-stable** | Uses the `abi_stable` crate for a version-tolerant binary interface. Loaded via `plugin-abi`. |
| **WASM** | Wasmtime runtime executes `.wasm` modules. Manifests live in `wasm-plugin.json` files. Loaded via `plugin-wasm`. |

Host-cli demonstrates all three: `Playground` for native JSON, `AbiPluginCatalog`
for ABI-stable, and `WasmPluginCatalog` for WASM plugins.
