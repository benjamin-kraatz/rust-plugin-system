# plugin-sdk

Plugin SDK for the [Rust Plugin System](https://github.com/benjamin-kraatz/rust-plugin-system).

This is the primary crate plugin authors depend on.  It provides everything
needed to build a native JSON dynamic-library plugin:

* The `JsonPlugin` trait — implement `manifest()` and `invoke()` on your plugin struct.
* The `export_plugin!` macro — generates the three C-callable entry points that the
  host loads at runtime.
* Re-exports of `plugin_api`, `plugin_manifest`, and `plugin_protocol` so you only
  need a single dependency.

## Installation

Add to your plugin's `Cargo.toml`:

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
plugin-sdk = "0.1"
serde_json = "1"
```

## Quick start

```rust,ignore
use plugin_sdk::{JsonPlugin, export_plugin};
use plugin_sdk::plugin_manifest::{PluginManifest, PluginAction};
use plugin_sdk::plugin_protocol::{PluginRequest, PluginResponse};
use plugin_capabilities::{HostKind, PluginArchitecture, SkillLevel};

pub struct MyPlugin;

impl JsonPlugin for MyPlugin {
    fn manifest() -> PluginManifest {
        PluginManifest {
            id: "my-plugin".into(),
            name: "My Plugin".into(),
            version: "0.1.0".into(),
            description: "A sample plugin.".into(),
            architecture: PluginArchitecture::NativeJson,
            skill_level: SkillLevel::Basic,
            supported_hosts: vec![HostKind::Any],
            actions: vec![PluginAction {
                id: "greet".into(),
                label: "Greet".into(),
                description: "Returns a greeting.".into(),
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    fn invoke(request: PluginRequest) -> Result<PluginResponse, String> {
        let name = request.payload
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
}

export_plugin!(MyPlugin);
```

## Complete guide

See [`docs/guides/external-plugin-author.md`](https://github.com/benjamin-kraatz/rust-plugin-system/blob/main/docs/guides/external-plugin-author.md)
for a full walkthrough including project setup, testing, and packaging.

## Documentation

Full API documentation is available on [docs.rs](https://docs.rs/plugin-sdk).

## License

MIT
