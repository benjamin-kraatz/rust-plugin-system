# plugin-wasm

WebAssembly plugin sandbox for the [Rust Plugin System](https://github.com/benjamin-kraatz/rust-plugin-system).

This crate loads and executes plugins compiled to WebAssembly (`.wasm`) or
WebAssembly Text format (`.wat`) using [Wasmtime](https://wasmtime.dev/).
Each invocation runs inside a fresh Wasmtime `Store`, providing strong memory
isolation between the host and the plugin.

## Installation

```toml
[dependencies]
plugin-wasm = "0.1"
```

## Quick start

```rust,no_run
use plugin_wasm::load_plugins_from_workspace;

let catalog = load_plugins_from_workspace("plugins").unwrap();
for plugin in &catalog.plugins {
    println!("{} — {}", plugin.manifest().id, plugin.manifest().name);
}
```

## Plugin layout

A WASM plugin lives in a directory that contains:

* `wasm-plugin.json` — the plugin's `PluginManifest` serialised as JSON.
* A `.wasm` or `.wat` module file exporting `alloc`, `invoke_json`, and
  `free` symbols.

## Documentation

Full API documentation is available on [docs.rs](https://docs.rs/plugin-wasm).

See also the [workspace documentation](https://github.com/benjamin-kraatz/rust-plugin-system/tree/main/docs)
for architecture guides, tutorials, and reference material.

## License

MIT
