# plugin-capabilities

Capability and constraint types for the [Rust Plugin System](https://github.com/benjamin-kraatz/rust-plugin-system).

This crate defines the shared vocabulary that plugins and hosts use to describe
what a plugin can do, what it requires, and under which conditions it may run.

## Installation

```toml
[dependencies]
plugin-capabilities = "0.1"
```

## Main types

| Type | Purpose |
|---|---|
| `HostKind` | Which host surface a plugin targets (CLI, TUI, egui, web, …) |
| `PluginArchitecture` | How the plugin is delivered (native JSON, ABI-stable, WASM) |
| `SkillLevel` | Indicative complexity of the plugin |
| `Capability` | A named feature a plugin advertises or a host provides |
| `CapabilityConstraints` | Resource and permission boundaries for a single action |
| `CapabilityRequirement` | Whether a capability is required, optional, or has a fallback |
| `TrustLevel` | How much trust has been granted to a plugin |
| `SandboxLevel` | Isolation level the plugin runs inside |
| `NetworkAccess` | Network permission the plugin may use |
| `LifecycleHook` | Points in the lifecycle a plugin subscribes to |
| `LifecycleState` | Runtime state a plugin is currently in |
| `ExecutionMode` | Synchronous vs. asynchronous execution |
| `RetryPolicy` | Back-off configuration for retried invocations |

## Documentation

Full API documentation is available on [docs.rs](https://docs.rs/plugin-capabilities).

See also the [workspace documentation](https://github.com/benjamin-kraatz/rust-plugin-system/tree/main/docs)
for architecture guides, tutorials, and reference material.

## License

MIT
