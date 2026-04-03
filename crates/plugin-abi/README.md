# plugin-abi

ABI-stable plugin loader for the [Rust Plugin System](https://github.com/benjamin-kraatz/rust-plugin-system).

This crate loads plugins built with [`abi_stable`](https://docs.rs/abi_stable)
rather than the plain C FFI used by `plugin-loader`.  Because `abi_stable`
enforces layout compatibility across Rust compiler updates you can ship new
plugin versions without recompiling the host.

## Installation

```toml
[dependencies]
plugin-abi = "0.1"
```

## Quick start

```rust,no_run
use plugin_abi::load_plugins_from_directory;

let catalog = load_plugins_from_directory("target/debug").unwrap();
for plugin in &catalog.plugins {
    println!("{} — {}", plugin.manifest().id, plugin.manifest().name);
}
```

## How it works

1. `load_plugins_from_directory` scans a directory for shared libraries
   whose filename contains `"abi_stable"`.
2. Each library is loaded via `abi_stable`'s root-module infrastructure which
   validates the ABI layout at load time.
3. Successfully loaded plugins are wrapped in a `LoadedAbiPlugin` that
   exposes typed `invoke` methods through the stable vtable.

## Documentation

Full API documentation is available on [docs.rs](https://docs.rs/plugin-abi).

See also the [workspace documentation](https://github.com/benjamin-kraatz/rust-plugin-system/tree/main/docs)
for architecture guides, tutorials, and reference material.

## License

MIT
