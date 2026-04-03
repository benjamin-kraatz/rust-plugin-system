# Minimal Plugin Loader

The simplest possible host: load a single native-JSON plugin via
`libloading` and exercise its three FFI entry-points.

## What it demonstrates

1. Opening a `.dylib` / `.so` at runtime with `libloading`.
2. Calling `plugin_manifest_json()` to inspect the plugin.
3. Calling `plugin_invoke_json()` to run the first declared action.
4. Calling `plugin_free_c_string()` to hand memory back to the plugin.

## How to run

```bash
# Build a plugin first
cargo build -p hello-world

# Run the loader (macOS)
cargo run -p example-minimal-loader -- target/debug/libhello_world.dylib

# On Linux use .so instead
cargo run -p example-minimal-loader -- target/debug/libhello_world.so
```
