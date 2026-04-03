# Writing a Host Outside the Workspace

This guide shows you how to build a **host application** for the Rust Plugin
System from scratch in your own Rust project, completely independent of this
repository.

A host application loads plugin dynamic libraries at runtime, invokes their
actions, and renders the results in whatever UI makes sense for your use case
— a CLI, a TUI, a desktop GUI, or an HTTP service.

For the full API reference see [docs.rs/host-core](https://docs.rs/host-core).

---

## Prerequisites

* Rust toolchain (stable) — install via [rustup.rs](https://rustup.rs)
* The crates published from this repository (see
  [publishing/crates-io.md](../publishing/crates-io.md))
* One or more compiled plugin `.so` / `.dylib` / `.dll` files (see
  [external-plugin-author.md](./external-plugin-author.md))

---

## 1. Create a new binary crate

```bash
cargo new my-host
cd my-host
```

---

## 2. Configure `Cargo.toml`

```toml
[package]
name = "my-host"
version = "0.1.0"
edition = "2021"

[dependencies]
host-core       = "0.1"
plugin-manifest = "0.1"
plugin-protocol = "0.1"
anyhow          = "1"
serde_json      = "1"
```

If you also want to load **ABI-stable** or **WASM** plugins, add:

```toml
plugin-abi  = "0.1"   # for abi_stable plugins
plugin-wasm = "0.1"   # for WebAssembly plugins
```

---

## 3. Load plugins and invoke actions

### Minimal example with `host-core`

`host-core::Playground` is the highest-level API.  It handles discovery,
loading, capability assessment, and invocation context setup.

```rust
// src/main.rs
use anyhow::Result;
use host_core::Playground;
use plugin_protocol::HostKind;
use serde_json::json;

fn main() -> Result<()> {
    // By default, Playground::load_default() looks for plugins in the
    // directory set by RUST_PLUGIN_SYSTEM_PLUGIN_DIR, or falls back to
    // "target/debug".  Override with Playground::load("path/to/plugins").
    let playground = Playground::load_default()?;

    if !playground.warnings().is_empty() {
        eprintln!("Loader warnings:");
        for w in playground.warnings() {
            eprintln!("  - {w}");
        }
    }

    // Print a summary of every loaded plugin.
    for summary in playground.summaries() {
        println!(
            "[{}] {} — {} ({} action(s))",
            summary.id, summary.name, summary.description, summary.action_count,
        );
    }

    // Invoke a specific action.
    let response = playground.invoke(
        "my-plugin",
        "greet",
        Some(json!({"name": "World"})),
        HostKind::Cli,
    )?;

    println!("\n{}\n{}", response.title, response.summary);
    for output in &response.outputs {
        println!("[{:?}] {}", output.kind, output.body);
    }

    Ok(())
}
```

### Setting the plugin directory

```bash
# Via environment variable (recommended for production)
RUST_PLUGIN_SYSTEM_PLUGIN_DIR=/usr/lib/my-app/plugins ./my-host

# Or hard-code a path
Playground::load("/usr/lib/my-app/plugins")?;
```

---

## 4. Building a CLI host

Here is a more complete CLI host using [`clap`](https://docs.rs/clap):

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
```

```rust
use anyhow::Result;
use clap::{Parser, Subcommand};
use host_core::Playground;
use plugin_protocol::HostKind;
use serde_json::{Value, json};

#[derive(Parser)]
#[command(name = "my-host", about = "My plugin host")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List all loaded plugins.
    List,
    /// Inspect a plugin's manifest.
    Inspect { plugin_id: String },
    /// Run a plugin action.
    Run {
        plugin_id: String,
        action_id:  String,
        /// JSON payload (optional).
        payload:    Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let playground = Playground::load_default()?;

    match cli.command {
        Command::List => {
            for s in playground.summaries() {
                println!("{:20} {} actions  {}", s.id, s.action_count, s.description);
            }
        }
        Command::Inspect { plugin_id } => {
            let manifest = playground
                .manifests()
                .into_iter()
                .find(|m| m.id == plugin_id)
                .ok_or_else(|| anyhow::anyhow!("plugin not found: {plugin_id}"))?;
            println!("{}", serde_json::to_string_pretty(&manifest)?);
        }
        Command::Run { plugin_id, action_id, payload } => {
            let payload: Option<Value> = payload
                .as_deref()
                .map(serde_json::from_str)
                .transpose()?;
            let response = playground.invoke(&plugin_id, &action_id, payload, HostKind::Cli)?;
            println!("{}\n{}", response.title, response.summary);
            for output in &response.outputs {
                println!("{}", output.body);
            }
        }
    }

    Ok(())
}
```

Build and run:

```bash
cargo build --release
./target/release/my-host list
./target/release/my-host run my-plugin greet '{"name":"Alice"}'
```

---

## 5. Loading ABI-stable and WASM plugins

If you want to load all three plugin architectures, combine the loaders:

```rust
use host_core::Playground;          // native JSON plugins
use plugin_abi::load_plugins_from_directory as load_abi;
use plugin_wasm::load_plugins_from_workspace as load_wasm;

let native_catalog = Playground::load_default()?;
let abi_catalog    = load_abi("target/debug")?;
let wasm_catalog   = load_wasm("plugins")?;

// Print all plugins from all loaders
for p in &native_catalog.as_catalog().plugins { /* … */ }
for p in &abi_catalog.plugins                  { /* … */ }
for p in &wasm_catalog.plugins                 { /* … */ }
```

See the `host-cli` source in this repository for a full example of mixing all
three loaders.

---

## 6. Building an HTTP service host

```toml
[dependencies]
axum   = "0.8"
tokio  = { version = "1", features = ["full"] }
```

```rust
use axum::{Router, Json, extract::{Path, State}};
use host_core::Playground;
use plugin_protocol::HostKind;
use serde_json::Value;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let playground = Arc::new(Playground::load_default()?);

    let app = Router::new()
        .route("/plugins", axum::routing::get(list_plugins))
        .route(
            "/plugins/:plugin_id/actions/:action_id/invoke",
            axum::routing::post(invoke_action),
        )
        .with_state(playground);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn list_plugins(State(pg): State<Arc<Playground>>) -> Json<Vec<String>> {
    Json(pg.summaries().iter().map(|s| s.id.clone()).collect())
}

async fn invoke_action(
    State(pg): State<Arc<Playground>>,
    Path((plugin_id, action_id)): Path<(String, String)>,
    Json(payload): Json<Option<Value>>,
) -> Json<Value> {
    match pg.invoke(&plugin_id, &action_id, payload, HostKind::Service) {
        Ok(r)  => Json(serde_json::to_value(r).unwrap()),
        Err(e) => Json(serde_json::json!({"error": e.to_string()})),
    }
}
```

---

## 7. Capability negotiation

Before invoking a plugin, `Playground::invoke` automatically calls
`assess_host_fit` which checks:

* whether the plugin's `supported_hosts` list includes the current `HostKind`,
* whether the plugin's `CompatibilityContract` matches the host version,
* whether all required capabilities are available.

The result is stored in `PluginResponse.negotiation`.  You can inspect it:

```rust
if let Some(neg) = &response.negotiation {
    match neg.status {
        NegotiationStatus::Ready    => { /* all good */ }
        NegotiationStatus::Degraded => {
            for f in &neg.degraded_features {
                eprintln!("degraded: {}", f.feature);
            }
        }
        NegotiationStatus::Rejected => {
            eprintln!("plugin rejected for this host");
        }
    }
}
```

---

## Next steps

* [Writing a Plugin](./external-plugin-author.md) — build plugins your host can load.
* [Publishing to crates.io](../publishing/crates-io.md) — distribute your host as a library.
* [Publishing via GitHub](../publishing/github-packages.md) — git-based or registry-based distribution.
* [Host Matrix](../hosts/host-matrix.md) — compare the example hosts in this repository.
* [Surface Comparison](../hosts/surface-comparison.md) — TUI vs. GUI vs. web vs. service trade-offs.
