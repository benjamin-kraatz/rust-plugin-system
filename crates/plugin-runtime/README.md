# plugin-runtime

Plugin runtime utilities for the [Rust Plugin System](https://github.com/benjamin-kraatz/rust-plugin-system).

This crate contains lightweight helpers that are useful in both host
applications and tests.

## Installation

```toml
[dependencies]
plugin-runtime = "0.1"
```

## Main types

| Type | Purpose |
|---|---|
| `PluginSummary` | A compact view of a plugin derived from its `PluginManifest`, suitable for list displays and log messages |
| `render_response` | Formats a `PluginResponse` into a human-readable string for terminal output |

## Documentation

Full API documentation is available on [docs.rs](https://docs.rs/plugin-runtime).

See also the [workspace documentation](https://github.com/benjamin-kraatz/rust-plugin-system/tree/main/docs)
for architecture guides, tutorials, and reference material.

## License

MIT
