# plugin-loader

Dynamic library loader for the [Rust Plugin System](https://github.com/benjamin-kraatz/rust-plugin-system).

This crate discovers and loads native JSON dynamic-library plugins (`.so` /
`.dylib` / `.dll`) from a directory at runtime using
[libloading](https://docs.rs/libloading).

## Installation

```toml
[dependencies]
plugin-loader = "0.1"
```

## Quick start

```rust,no_run
use plugin_loader::load_plugins_from_directory;

let catalog = load_plugins_from_directory("target/debug").unwrap();
for plugin in &catalog.plugins {
    println!("{} — {}", plugin.manifest().id, plugin.manifest().name);
}
```

## How it works

1. `load_plugins_from_directory` scans a directory for files that look like
   dynamic libraries.
2. For each file it calls the `plugin_manifest_json` symbol to obtain the
   plugin's `PluginManifest`.
3. Successfully loaded plugins are wrapped in a `LoadedPlugin` which exposes
   typed `invoke` methods.
4. The collected plugins are returned in a `PluginCatalog` together with
   any non-fatal loading warnings.

## Documentation

Full API documentation is available on [docs.rs](https://docs.rs/plugin-loader).

See also the [workspace documentation](https://github.com/benjamin-kraatz/rust-plugin-system/tree/main/docs)
for architecture guides, tutorials, and reference material.

## License

MIT
