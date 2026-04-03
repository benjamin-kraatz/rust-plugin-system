# plugin-api

C FFI boundary helpers for the [Rust Plugin System](https://github.com/benjamin-kraatz/rust-plugin-system).

Plugin dynamic libraries and their host loaders communicate via a small set
of C-callable symbols.  This crate provides the constants, serialisation helpers,
and C string management utilities that sit at the FFI boundary.

> **Note:** Plugin authors should use `plugin-sdk` rather than calling this crate
> directly.  Host authors use these constants when loading a plugin library via
> `libloading`.

## Installation

```toml
[dependencies]
plugin-api = "0.1"
```

## Constants

| Constant | Value | Purpose |
|---|---|---|
| `MANIFEST_SYMBOL` | `"plugin_manifest_json\0"` | Symbol exported by every plugin to return its manifest |
| `INVOKE_SYMBOL` | `"plugin_invoke_json\0"` | Symbol exported by every plugin to handle an invocation |
| `FREE_SYMBOL` | `"plugin_free_c_string\0"` | Symbol exported by every plugin to release returned strings |

## Documentation

Full API documentation is available on [docs.rs](https://docs.rs/plugin-api).

See also the [workspace documentation](https://github.com/benjamin-kraatz/rust-plugin-system/tree/main/docs)
for architecture guides, tutorials, and reference material.

## License

MIT
