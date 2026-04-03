# plugin-manifest

Plugin manifest structures for the [Rust Plugin System](https://github.com/benjamin-kraatz/rust-plugin-system).

A `PluginManifest` is the machine-readable "identity card" of a plugin.  It
describes every action the plugin exposes together with production metadata such
as versioning, maintenance status, capability contracts, trust level, and
lifecycle hooks.

## Installation

```toml
[dependencies]
plugin-manifest = "0.1"
```

## Quick start

```rust
use plugin_manifest::{PluginManifest, PluginAction};
use plugin_capabilities::{HostKind, PluginArchitecture, SkillLevel};

let manifest = PluginManifest {
    id: "my-plugin".to_string(),
    name: "My Plugin".to_string(),
    version: "0.1.0".to_string(),
    description: "A sample plugin.".to_string(),
    architecture: PluginArchitecture::NativeJson,
    skill_level: SkillLevel::Basic,
    supported_hosts: vec![HostKind::Any],
    actions: vec![PluginAction {
        id: "greet".to_string(),
        label: "Greet".to_string(),
        description: "Returns a greeting.".to_string(),
        ..Default::default()
    }],
    ..Default::default()
};
```

## Main types

| Type | Purpose |
|---|---|
| `PluginManifest` | Top-level manifest describing a plugin |
| `PluginAction` | A single callable action with its metadata |
| `ActionContract` | Execution guarantees (idempotence, timeout, …) |
| `MaintenanceContract` | Ownership, support tier, and deprecation notices |
| `CompatibilityContract` | Protocol and host version compatibility |
| `CapabilityContract` | Required and optional capability declarations |
| `TrustMetadata` | Trust level, sandbox, and network access |
| `LifecycleContract` | Lifecycle hooks and current state |
| `ExecutionContract` | Concurrency and async execution details |

## Documentation

Full API documentation is available on [docs.rs](https://docs.rs/plugin-manifest).

See also the [workspace documentation](https://github.com/benjamin-kraatz/rust-plugin-system/tree/main/docs)
for architecture guides, tutorials, and reference material.

## License

MIT
