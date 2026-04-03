# host-core

Shared host orchestration for the [Rust Plugin System](https://github.com/benjamin-kraatz/rust-plugin-system).

`host-core` is the crate that host application authors depend on.  It ties
together the loader, runtime, and protocol crates into a single high-level
`Playground` façade that any host surface (CLI, TUI, GUI, web, service) can
use without reimplementing plugin discovery, invocation context setup, or
capability negotiation.

## Installation

```toml
[dependencies]
host-core = "0.1"
```

## Quick start

```rust,no_run
use host_core::Playground;
use plugin_protocol::HostKind;
use serde_json::json;

let playground = Playground::load_default().expect("failed to load plugins");

// List all loaded plugins
for summary in playground.summaries() {
    println!("{}: {}", summary.id, summary.description);
}

// Invoke a plugin action
let response = playground
    .invoke("my-plugin", "greet", Some(json!({"name": "Alice"})), HostKind::Cli)
    .expect("invocation failed");
println!("{}", response.summary);
```

## Plugin directory

By default `Playground::load_default` looks for plugins in the directory
pointed to by the `RUST_PLUGIN_SYSTEM_PLUGIN_DIR` environment variable, falling
back to `target/debug` when the variable is not set.

## Complete guide

See [`docs/guides/external-host-author.md`](https://github.com/benjamin-kraatz/rust-plugin-system/blob/main/docs/guides/external-host-author.md)
for a full walkthrough including project setup, configuration, and deployment.

## Documentation

Full API documentation is available on [docs.rs](https://docs.rs/host-core).

## License

MIT
