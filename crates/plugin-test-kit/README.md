# plugin-test-kit

Testing utilities for the [Rust Plugin System](https://github.com/benjamin-kraatz/rust-plugin-system).

`plugin-test-kit` provides fluent builder types that make it easy to
construct test fixtures for plugin manifests and actions without having to
fill in every optional field by hand.

## Installation

```toml
[dev-dependencies]
plugin-test-kit = "0.1"
```

## Quick start

```rust
use plugin_test_kit::{ManifestBuilder, ActionBuilder};

let manifest = ManifestBuilder::new("my-plugin", "My Plugin", "0.1.0")
    .description("A test plugin")
    .action(
        ActionBuilder::new("greet")
            .label("Greet")
            .description("Returns a greeting")
            .build(),
    )
    .build();

assert_eq!(manifest.id, "my-plugin");
assert_eq!(manifest.actions.len(), 1);
```

## Main types

| Type | Purpose |
|---|---|
| `ManifestBuilder` | Builds a `PluginManifest` with sensible defaults |
| `ActionBuilder` | Builds a single `PluginAction` |

## Documentation

Full API documentation is available on [docs.rs](https://docs.rs/plugin-test-kit).

See also the [workspace documentation](https://github.com/benjamin-kraatz/rust-plugin-system/tree/main/docs)
for architecture guides, tutorials, and reference material.

## License

MIT
