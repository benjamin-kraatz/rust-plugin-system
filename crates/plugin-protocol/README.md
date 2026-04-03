# plugin-protocol

Request/response protocol types for the [Rust Plugin System](https://github.com/benjamin-kraatz/rust-plugin-system).

This crate defines the wire types that travel across the host ↔ plugin
boundary at invocation time.

## Installation

```toml
[dependencies]
plugin-protocol = "0.1"
```

## Quick start

```rust
use plugin_protocol::{PluginRequest, InvocationContext};
use plugin_capabilities::HostKind;
use serde_json::json;

let request = PluginRequest {
    plugin_id: "my-plugin".to_string(),
    action_id: "greet".to_string(),
    payload: Some(json!({"name": "Alice"})),
    context: InvocationContext::for_host(HostKind::Cli),
};
```

## Main types

| Type | Purpose |
|---|---|
| `PluginRequest` | An action invocation sent from host to plugin |
| `PluginResponse` | The result returned by the plugin |
| `InvocationContext` | Per-request ambient metadata |
| `RuntimeContext` | Host capability advertisement |
| `OutputBlock` | A single labelled output fragment (text, JSON, markdown, …) |
| `ExecutionMetadata` | Mode, timing, and job-tracking info attached to a response |
| `NegotiationOutcome` | Capability negotiation result (Ready / Degraded / Rejected) |

## Documentation

Full API documentation is available on [docs.rs](https://docs.rs/plugin-protocol).

See also the [workspace documentation](https://github.com/benjamin-kraatz/rust-plugin-system/tree/main/docs)
for architecture guides, tutorials, and reference material.

## License

MIT
