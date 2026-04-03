# Workspace Map

## Shared crates

- `plugin-api` - low-level FFI helpers and exported symbol names
- `plugin-manifest` - plugin metadata structures
- `plugin-protocol` - request and response message types
- `plugin-loader` - native plugin discovery and runtime loading
- `plugin-runtime` - host-side execution helpers and response rendering for execution and negotiation metadata
- `plugin-sdk` - helper utilities for plugin authors
- `plugin-test-kit` - deterministic builders, assertions, and package fixtures for Phase 4 tests
- `plugin-abi` - ABI-stable track support
- `plugin-wasm` - WASM track support through a Wasmtime-based JSON bridge
- `host-core` - shared host-side logic used by multiple hosts, including payload template loading, host-fit assessment, capability negotiation, warnings, and synthesized execution metadata

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
- `docs/reference/production-contracts.md` - entry point for the Phase 4 contract and packaging layer
- `docs/reference/version-compatibility.md` - compatibility windows and version strategy notes
- `docs/reference/lifecycle-execution.md` - lifecycle, execution, and async semantics
- `docs/reference/trust-capabilities.md` - trust metadata plus capability negotiation and degradation
- `docs/reference/wasm-sandboxing.md` - Wasmtime runtime model and current sandbox scope
- `docs/reference/testing-packaging.md` - plugin-test-kit usage and packaging examples

## Plugin crates

The `plugins/` directory contains loadable examples and host/domain-specific experiments.
