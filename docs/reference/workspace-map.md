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
- `host-core` - shared host-side logic used by multiple hosts, including payload template loading via `default_payload_text()` and host-fit checks via `supports_host()`

## Host apps

- `host-cli` - baseline command-line inspection and invocation
- `host-tui` - keyboard-first multi-pane terminal comparison host with inline payload editing
- `host-egui` - inspector/dashboard desktop host
- `host-iced` - explicit state/update desktop host
- `host-dioxus-desktop` - reactive desktop host using an explicit Cargo `[[bin]]` target with `autobins = false`
- `host-web` - browser-facing comparison host on `127.0.0.1:4000`
- `host-service` - automation-oriented API host on `127.0.0.1:5000`

## Docs to pair with the code

- `docs/getting-started/quickstart.md` - first-run commands
- `docs/hosts/host-matrix.md` - short comparison table for all hosts
- `docs/hosts/surface-comparison.md` - detailed Phase 3 host comparison
- `docs/plugins/native-json-catalog.md` - native plugin catalog walkthrough

## Plugin crates

The `plugins/` directory contains loadable examples and host/domain-specific experiments.
