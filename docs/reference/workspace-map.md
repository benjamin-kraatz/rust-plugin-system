# Workspace Map

## Shared crates

- `plugin-api` - low-level FFI helpers and exported symbol names
- `plugin-manifest` - plugin metadata structures
- `plugin-protocol` - request and response message types
- `plugin-loader` - native plugin discovery and runtime loading
- `plugin-runtime` - host-side execution helpers and summaries
- `plugin-sdk` - helper utilities for plugin authors
- `plugin-abi` - ABI-stable track support
- `plugin-wasm` - WASM track support
- `host-core` - shared host-side logic used by multiple hosts

## Host apps

- `host-cli`
- `host-tui`
- `host-egui`
- `host-iced`
- `host-dioxus-desktop`
- `host-web`
- `host-service`

## Plugin crates

The `plugins/` directory contains loadable examples and host/domain-specific experiments.
