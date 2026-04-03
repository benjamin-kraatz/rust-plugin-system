# First Runtime Plugin

This tutorial walks you through building a runtime-loaded plugin from scratch,
covering all three plugin architecture tracks. By the end you will have a
working plugin that every host in the playground can discover and invoke.

## Prerequisites

- Rust toolchain installed (the workspace pins the edition in `rust-toolchain.toml`)
- This repository cloned and a successful `cargo build` from the workspace root
- A terminal open at the repository root

---

## Track A — Native JSON Plugin

The **NativeJson** track is the simplest path. Your plugin compiles to a
`cdylib`, exports three C-ABI symbols, and communicates with the host through
JSON strings. The `plugin-sdk` crate and its `export_plugin!` macro handle the
FFI boilerplate for you.

### 1. Create the crate

```bash
mkdir -p plugins/my-first-plugin/src
```

Create `plugins/my-first-plugin/Cargo.toml`:

```toml
[package]
name = "my-first-plugin"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
plugin-sdk = { version = "0.1.0", path = "../../crates/plugin-sdk" }
serde_json = "1.0.149"
```

Key points:

- `crate-type = ["cdylib", "rlib"]` — `cdylib` produces a shared library the
  host can `dlopen`; `rlib` lets you run unit tests normally.
- `plugin-sdk` re-exports `plugin-manifest` and `plugin-protocol` so you only
  need a single dependency.

### 2. Register the crate in the workspace

Open the root `Cargo.toml` and add your plugin to the `members` list:

```toml
[workspace]
members = [
    # ... existing members ...
    "plugins/my-first-plugin",
]
```

### 3. Implement the plugin

Create `plugins/my-first-plugin/src/lib.rs`:

```rust
use plugin_sdk::plugin_manifest::{
    Capability, HostKind, PluginAction, PluginArchitecture,
    PluginManifest, SkillLevel,
};
use plugin_sdk::plugin_protocol::{OutputKind, PluginRequest, PluginResponse};
use plugin_sdk::{JsonPlugin, export_plugin};

pub struct MyFirstPlugin;

impl JsonPlugin for MyFirstPlugin {
    fn manifest() -> PluginManifest {
        PluginManifest::new(
            "my-first-plugin",           // unique plugin id
            "My First Plugin",           // human-readable name
            "0.1.0",                     // semver version
            "A tutorial plugin that echoes back a greeting.",
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
        .with_capabilities(vec![
            Capability::new(
                "greeting",
                "Returns a friendly greeting.",
            ),
        ])
        .with_tags(["tutorial", "native-plugin"])
        .with_actions(vec![
            PluginAction::new(
                "greet",
                "Greet",
                "Say hello to a supplied name.",
            )
            .with_payload_hint(r#"{"name":"Rustacean"}"#),
        ])
        .with_notes([
            "This plugin was created by following the first-runtime-plugin tutorial.",
        ])
    }

    fn invoke(request: PluginRequest) -> Result<PluginResponse, String> {
        match request.action_id.as_str() {
            "greet" => {
                let name = request
                    .payload
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Rustacean");

                Ok(PluginResponse::ok(
                    "my-first-plugin",
                    "greet",
                    "Hello from my first plugin",
                    format!(
                        "Greeted '{name}' for the {:?} host.",
                        request.context.host
                    ),
                )
                .with_output(
                    OutputKind::Text,
                    "Greeting",
                    format!("Hello, {name}!"),
                )
                .with_next_step("Try running from another host to compare the UX."))
            }
            other => Err(format!("unknown action '{other}'")),
        }
    }
}

export_plugin!(MyFirstPlugin);
```

#### Understanding the manifest

`PluginManifest::new` takes six required fields:

| Field           | Purpose                                             |
|-----------------|-----------------------------------------------------|
| `id`            | Machine-readable identifier, must be unique         |
| `name`          | Display name shown in host UIs                      |
| `version`       | Semver string for compatibility checking            |
| `description`   | Short summary of what the plugin does               |
| `architecture`  | One of `NativeJson`, `AbiStable`, or `Wasm`         |
| `skill_level`   | `Basic`, `Intermediate`, `Advanced`, or `Expert`    |

Builder methods add optional metadata:

- `.with_supported_hosts(...)` — which host kinds can run this plugin
- `.with_capabilities(...)` — advertised capabilities for negotiation
- `.with_actions(...)` — the actions the host can invoke
- `.with_tags(...)` — free-form tags for discovery
- `.with_notes(...)` — human-readable notes for documentation

#### Understanding the invoke function

`invoke` receives a `PluginRequest` containing:

```rust
pub struct PluginRequest {
    pub plugin_id: String,       // which plugin was targeted
    pub action_id: String,       // which action to run
    pub payload: Value,          // arbitrary JSON input
    pub context: InvocationContext, // host info, workspace, timeouts
}
```

`InvocationContext` tells you which host is calling (`context.host`), the
workspace root, request/trace IDs, timeout budget, and runtime capabilities.

You return a `PluginResponse`:

```rust
PluginResponse::ok(plugin_id, action_id, title, summary)
    .with_output(OutputKind::Text, "Title", "body")
    .with_next_step("Suggested follow-up action.")
```

For errors, return `Err(message)` from the trait method — the `export_plugin!`
macro converts it to `PluginResponse::error(...)` automatically.

#### Understanding `export_plugin!`

The macro generates three `extern "C"` functions:

| Symbol                   | Purpose                                    |
|--------------------------|--------------------------------------------|
| `plugin_manifest_json`   | Returns the manifest as a `*mut c_char`    |
| `plugin_invoke_json`     | Accepts a JSON request, returns a response |
| `plugin_free_c_string`   | Frees a string the host received           |

These are the symbols the native plugin loader resolves via `dlopen`/`dlsym`.

### 4. Build and test

```bash
cargo build -p my-first-plugin
```

Verify the host discovers your plugin:

```bash
cargo run -p host-cli -- list
```

You should see a line like:

```
  my-first-plugin  0.1.0  native-json  basic  A tutorial plugin that echoes back a greeting.
```

Inspect the full manifest:

```bash
cargo run -p host-cli -- inspect my-first-plugin
```

Run the greet action:

```bash
cargo run -p host-cli -- run my-first-plugin greet '{"name":"World"}'
```

Expected output:

```
── Hello from my first plugin ──
Greeted 'World' for the Cli host.

  Text ▸ Greeting
  Hello, World!

Next steps:
  • Try running from another host to compare the UX.
```

### 5. Add unit tests (optional)

The `rlib` crate type lets you test the logic directly:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use plugin_sdk::plugin_protocol::InvocationContext;
    use plugin_sdk::plugin_manifest::HostKind;
    use serde_json::json;

    fn test_request(action: &str, payload: serde_json::Value) -> PluginRequest {
        PluginRequest {
            plugin_id: "my-first-plugin".to_owned(),
            action_id: action.to_owned(),
            payload,
            context: InvocationContext::for_host(HostKind::Cli),
        }
    }

    #[test]
    fn greet_uses_payload_name() {
        let response = MyFirstPlugin::invoke(
            test_request("greet", json!({"name": "Tutorial"}))
        ).unwrap();
        assert!(response.success);
        assert!(response.outputs[0].body.contains("Tutorial"));
    }

    #[test]
    fn unknown_action_returns_error() {
        let result = MyFirstPlugin::invoke(
            test_request("unknown", json!({}))
        );
        assert!(result.is_err());
    }
}
```

Run with:

```bash
cargo test -p my-first-plugin
```

---

## Track B — ABI-Stable Plugin

Use the **AbiStable** track when you need binary compatibility across compiler
versions. Plugins compiled with one Rust version can be loaded by hosts built
with another, as long as the root module layout stays stable. This matters for
long-lived plugins distributed as pre-built binaries.

### 1. Create the crate

Create `plugins/my-abi-plugin/Cargo.toml`:

```toml
[package]
name = "my-abi-plugin"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
abi_stable = "0.11.3"
plugin-abi = { version = "0.1.0", path = "../../crates/plugin-abi" }
plugin-manifest = { version = "0.1.0", path = "../../crates/plugin-manifest" }
plugin-protocol = { version = "0.1.0", path = "../../crates/plugin-protocol" }
serde_json = "1.0.149"
```

Note: ABI-stable plugins depend on `plugin-abi` (which defines the root module
type) and the lower-level `plugin-manifest`/`plugin-protocol` crates directly,
rather than `plugin-sdk`.

Add `"plugins/my-abi-plugin"` to the workspace `members` list.

### 2. Implement the plugin

Create `plugins/my-abi-plugin/src/lib.rs`:

```rust
use abi_stable::{
    export_root_module, prefix_type::PrefixTypeTrait, std_types::RString,
};
use plugin_abi::{AbiPluginModule, AbiPluginModuleRef};
use plugin_manifest::{
    Capability, HostKind, PluginAction, PluginArchitecture,
    PluginManifest, SkillLevel,
};
use plugin_protocol::{OutputKind, PluginRequest, PluginResponse};

fn manifest() -> PluginManifest {
    PluginManifest::new(
        "my-abi-plugin",
        "My ABI-Stable Plugin",
        "0.1.0",
        "A tutorial ABI-stable plugin.",
        PluginArchitecture::AbiStable,
        SkillLevel::Advanced,
    )
    .with_supported_hosts(vec![HostKind::Cli, HostKind::Service])
    .with_capabilities(vec![
        Capability::new("abi-greeting", "Greets through an ABI-stable root module."),
    ])
    .with_tags(["abi-stable", "tutorial"])
    .with_actions(vec![
        PluginAction::new("greet", "Greet", "Return a greeting via ABI-stable FFI.")
            .with_payload_hint(r#"{"name":"Explorer"}"#),
    ])
}

extern "C" fn manifest_json() -> RString {
    RString::from(
        serde_json::to_string(&manifest())
            .expect("manifest serialization should succeed"),
    )
}

extern "C" fn invoke_json(request_json: RString) -> RString {
    let response = match serde_json::from_str::<PluginRequest>(request_json.as_str()) {
        Ok(request) => {
            let name = request
                .payload
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Explorer");

            PluginResponse::ok(
                "my-abi-plugin",
                "greet",
                "Hello from ABI-stable land",
                format!("Greeted '{name}' through an ABI-stable root module."),
            )
            .with_output(OutputKind::Text, "Greeting", format!("Hello, {name}!"))
        }
        Err(e) => PluginResponse::error(
            "my-abi-plugin",
            "decode-request",
            "Failed to decode request",
            e.to_string(),
        ),
    };

    RString::from(
        serde_json::to_string(&response)
            .expect("response serialization should succeed"),
    )
}

#[export_root_module]
pub fn instantiate_root_module() -> AbiPluginModuleRef {
    AbiPluginModule {
        manifest_json,
        invoke_json,
    }
    .leak_into_prefix()
}
```

#### Key differences from Track A

| Aspect              | NativeJson (Track A)                | AbiStable (Track B)                      |
|---------------------|-------------------------------------|------------------------------------------|
| FFI boundary        | Raw `*mut c_char` symbols           | `abi_stable` root module with `RString`  |
| Macro               | `export_plugin!`                    | `#[export_root_module]`                  |
| String type         | `*mut c_char` + free function       | `RString` (ABI-stable `String`)          |
| Compiler compat     | Same compiler version required      | Cross-compiler-version safe              |
| Dependencies        | `plugin-sdk` only                   | `abi_stable` + `plugin-abi`              |

The root module exports two function pointers — `manifest_json` and
`invoke_json` — both working with `RString` instead of raw C strings. The
`#[export_root_module]` attribute generates the ABI-stable entry point that
the host's ABI loader looks up.

### 3. Build and test

```bash
cargo build -p my-abi-plugin
cargo run -p host-cli -- list
cargo run -p host-cli -- run my-abi-plugin greet '{"name":"World"}'
```

The host-cli automatically discovers ABI-stable plugins alongside native ones
and displays them in the unified plugin list.

---

## Track C — WASM Plugin

Use the **Wasm** track when you need sandboxing, portability, or want to run
untrusted code safely. WASM plugins are executed through Wasmtime and have no
direct access to the host filesystem or network.

### 1. Create the plugin directory

```bash
mkdir -p plugins/my-wasm-plugin
```

### 2. Write the manifest

WASM plugins use a JSON manifest file instead of a Rust function. Create
`plugins/my-wasm-plugin/wasm-plugin.json`:

```json
{
  "id": "my-wasm-plugin",
  "name": "My WASM Plugin",
  "version": "0.1.0",
  "description": "A tutorial sandboxed WebAssembly plugin.",
  "architecture": "wasm",
  "skill_level": "advanced",
  "supported_hosts": ["cli", "web", "service"],
  "capabilities": [
    {
      "key": "sandboxed-greeting",
      "description": "Returns a greeting from a sandboxed WASM module."
    }
  ],
  "tags": ["wasm", "sandboxed", "tutorial"],
  "actions": [
    {
      "id": "greet",
      "label": "Greet",
      "description": "Return a greeting from the WebAssembly sandbox.",
      "payload_hint": "{\"name\":\"sandbox\"}"
    }
  ],
  "trust": {
    "level": "restricted",
    "sandbox": "wasm",
    "network": "none",
    "deterministic": true,
    "local_only": true,
    "data_access": ["request-payload-only"],
    "provenance": "bundled-first-party"
  },
  "notes": [
    "This module uses WAT for transparency. Production WASM plugins can be compiled from Rust targeting wasm32."
  ]
}
```

### 3. Write the WAT module

Create `plugins/my-wasm-plugin/module.wat`:

```wat
(module
  (memory (export "memory") 1)
  (global $heap (mut i32) (i32.const 4096))

  ;; Static JSON response embedded in linear memory starting at byte 0
  (data (i32.const 0) "{\"plugin_id\":\"my-wasm-plugin\",\"action_id\":\"greet\",\"title\":\"Hello from WASM\",\"summary\":\"Executed a sandboxed WebAssembly plugin.\",\"success\":true,\"outputs\":[{\"kind\":\"text\",\"title\":\"Greeting\",\"body\":\"Hello from a WebAssembly sandbox!\"}],\"suggested_next_steps\":[\"Compare this with the native JSON plugin output.\"]}")

  ;; Simple bump allocator for the host to write the request into memory
  (func (export "alloc") (param $len i32) (result i32)
    (local $ptr i32)
    global.get $heap
    local.set $ptr
    global.get $heap
    local.get $len
    i32.add
    global.set $heap
    local.get $ptr
  )

  ;; Returns (ptr, len) pointing at the static JSON response
  (func (export "invoke_json") (param $ptr i32) (param $len i32) (result i32 i32)
    i32.const 0
    i32.const 285
  )
)
```

The module exports three things:

- `memory` — shared linear memory the host reads from
- `alloc` — a bump allocator so the host can write the request JSON into memory
- `invoke_json` — takes `(ptr, len)` of the request JSON, returns `(ptr, len)`
  of the response JSON

For this tutorial the response is a static string embedded in linear memory.
A production WASM plugin would parse the request and build a dynamic response.

> **Important:** Update the `i32.const 285` value to match the exact byte
> length of your JSON response string.

### 4. Create a minimal Cargo.toml

```toml
[package]
name = "my-wasm-plugin"
version = "0.1.0"
edition = "2024"

[dependencies]
```

Add `"plugins/my-wasm-plugin"` to the workspace members.

### 5. Build and test

WASM plugins don't need `cargo build` — the WAT file is loaded directly by
the WASM host runtime. Just run:

```bash
cargo run -p host-cli -- list
cargo run -p host-cli -- run my-wasm-plugin greet '{"name":"test"}'
```

Expected output:

```
── Hello from WASM ──
Executed a sandboxed WebAssembly plugin.

  Text ▸ Greeting
  Hello from a WebAssembly sandbox!

Next steps:
  • Compare this with the native JSON plugin output.
```

---

## Comparing the three tracks

| Concern              | NativeJson            | AbiStable                | Wasm                     |
|----------------------|-----------------------|--------------------------|--------------------------|
| Complexity           | Lowest                | Medium                   | Medium                   |
| Sandboxing           | None (in-process)     | None (in-process)        | Full (Wasmtime)          |
| Compiler coupling    | Same compiler version | Cross-compiler safe      | Language-independent     |
| Performance          | Fastest               | Fast (small indirection) | Slight overhead          |
| Distribution         | Shared library        | Shared library           | `.wat`/`.wasm` + JSON    |
| Best for             | Internal tools        | Long-lived binaries      | Untrusted / portable     |

---

## Next steps

- **Inspect existing plugins** — run `cargo run -p host-cli -- inspect hello-world`
  and compare with `inspect abi-stable-greeter` and `inspect wasm-sandboxed`
- **Read the patterns docs** — `docs/patterns/` covers state management, error
  handling, and capability negotiation
- **Try a GUI host** — run the same plugin from `host-tui`, `host-egui`, or
  `host-iced` to see how different hosts render the same response
- **Study advanced manifests** — `plugins/service-hooks/src/lib.rs` shows
  maintenance, compatibility, trust, lifecycle, execution, and capability
  contracts
- **Browse examples** — the `examples/` directory has packaging and advanced
  plugin patterns

