# Quickstart

## 1. Build the workspace

The host apps load plugin dynamic libraries from `target/debug`, so build the workspace first:

```bash
cargo build --workspace
```

## 2. Explore from the CLI host

List discovered plugins:

```bash
cargo run -p host-cli -- list
```

Inspect a plugin:

```bash
cargo run -p host-cli -- inspect hello-world
```

Invoke a plugin action:

```bash
cargo run -p host-cli -- run formatter pretty-json '{"project":"rust-plugin-system"}'
```

## 3. Walk the native plugin catalog in phases

Start with the foundation examples:

```bash
cargo run -p host-cli -- inspect hello-world
cargo run -p host-cli -- run hello-world greet '{"name":"Rustacean"}'
cargo run -p host-cli -- run transformer slugify '{"text":"Rust Plugin Systems Course Module"}'
```

Then move into the expanded Phase 2 practical plugins:

```bash
cargo run -p host-cli -- inspect config-provider
cargo run -p host-cli -- run config-provider merge-layers '{"defaults":{"service":{"port":8080}},"environment":{"service":{"port":8081}},"overrides":{"service":{"host":"127.0.0.1"}}}'
cargo run -p host-cli -- run filesystem-tools list-directory '{"path":"plugins","max_depth":1,"max_entries":8}'
cargo run -p host-cli -- run data-pipeline summarize-field '{"records":[{"duration":12.5},{"duration":7.5}],"field":"duration"}'
cargo run -p host-cli -- run metrics-observer evaluate-slo '{"service":"checkout","objective_pct":99.5,"window_requests":1800,"window_errors":18}'
cargo run -p host-cli -- run service-hooks preview-delivery '{"service":"billing","event":"deploy.succeeded","attempt":2,"target_base_url":"https://hooks.internal.example"}'
cargo run -p host-cli -- run tui-tools draft-status-line '{"mode":"NORMAL","workspace":"rust-plugin-system","branch":"main","dirty":false,"pending_tasks":2,"focus":"editor"}'
```

For the full course-style catalog, continue with `docs/plugins/native-json-catalog.md`.

## 4. Compare the rich hosts

These hosts all load the same manifests and actions, but expose very different UX:

- `cargo run -p host-tui` - keyboard-first terminal UI with plugin panes, action panes, inline payload editing, templates, and output review
- `cargo run -p host-egui` - inspector/dashboard desktop UI for manifest metadata, action browsing, payload editing, and output panels
- `cargo run -p host-iced` - explicit state/update desktop UI where selection, template loading, and invocation feel like a deliberate application workflow
- `cargo run -p host-dioxus-desktop` - reactive desktop action studio with plugin/action selection, payload editing, invocation output, and an explicit Cargo `[[bin]]` target with `autobins = false`
- `cargo run -p host-web` - browser host at `http://127.0.0.1:4000` with a plugin catalog, manifest details, action browser, payload composer, and result cards
- `cargo run -p host-service` - API host at `http://127.0.0.1:5000` with automation endpoints for discovery, examples, details, and invocation

## 5. Compare web and service surfaces

The two networked hosts intentionally split into human-facing and automation-facing workflows:

- `host-web` is for browsing manifests, loading payload hints into a composer, formatting payload JSON, and reviewing result cards in the browser
- `host-service` is for scripts or backend flows that want stable JSON responses from:
  - `GET /`
  - `GET /health`
  - `GET /catalog`
  - `GET /examples`
  - `GET /plugins`
  - `GET /plugins/{plugin_id}`
  - `GET /plugins/{plugin_id}/actions/{action_id}`
  - `POST /plugins/{plugin_id}/actions/{action_id}/invoke`

See `docs/hosts/surface-comparison.md` for the full host comparison.

## 6. Explore the advanced plugin tracks

Try the ABI-stable plugin examples:

```bash
cargo run -p host-cli -- inspect abi-stable-greeter
cargo run -p host-cli -- run abi-stable-greeter greet '{"name":"Rustacean"}'
```

Try the sandboxed WASM plugin examples:

```bash
cargo run -p host-cli -- inspect wasm-sandboxed
cargo run -p host-cli -- run wasm-sandboxed run-demo '{"note":"sandbox"}'
cargo run -p host-cli -- run web-widget render-widget '{"theme":"dark"}'
```

## 7. Understand what you are seeing

The first working slice uses **native dynamic libraries** with a **JSON-over-FFI** boundary:

- hosts discover plugin libraries at runtime
- hosts read a plugin manifest through exported symbols
- hosts send invocation requests as JSON
- plugins return structured responses as JSON

This keeps the ABI small and explicit while still demonstrating real runtime loading.

The repository also includes:

- **ABI-stable native plugins** loaded with `abi_stable`
- **sandboxed WASM plugins** executed with Wasmtime

## 8. Read the Phase 4 contract layer

Once the commands above feel natural, use these docs as the production-minded reference path:

- `docs/reference/production-contracts.md` - guided entry point for the Phase 4 material
- `docs/reference/version-compatibility.md` - compatibility windows, protocol versions, tested hosts, and strategy caveats
- `docs/reference/lifecycle-execution.md` - lifecycle hooks, execution mode, async hints, timeouts, and response metadata
- `docs/reference/trust-capabilities.md` - trust levels, security metadata, capability negotiation, and degradation semantics
- `docs/reference/wasm-sandboxing.md` - what the current Wasmtime sandbox does and does not provide
- `docs/reference/testing-packaging.md` - `plugin-test-kit`, package fixtures, and example bundle flows

If you are building outside this workspace, continue with:

- `docs/guides/external-host-plugin.md` - create your own host and plugin project
- `docs/guides/publishing.md` - publish the shared crates to crates.io or GitHub Packages

A practical Phase 4 inspection loop is:

```bash
cargo run -p host-cli -- inspect metrics-observer
cargo run -p host-cli -- inspect abi-stable-command-pack
cargo run -p host-cli -- inspect wasm-sandboxed
```
