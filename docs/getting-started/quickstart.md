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

## 4. Try other hosts

- `cargo run -p host-tui`
- `cargo run -p host-egui`
- `cargo run -p host-iced`
- `cargo run -p host-dioxus-desktop`
- `cargo run -p host-web`
- `cargo run -p host-service`

## 5. Explore the advanced plugin tracks

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

## 6. Understand what you are seeing

The first working slice uses **native dynamic libraries** with a **JSON-over-FFI** boundary:

- hosts discover plugin libraries at runtime
- hosts read a plugin manifest through exported symbols
- hosts send invocation requests as JSON
- plugins return structured responses as JSON

This keeps the ABI small and explicit while still demonstrating real runtime loading.

The repository also includes:

- **ABI-stable native plugins** loaded with `abi_stable`
- **sandboxed WASM plugins** executed with Wasmtime
