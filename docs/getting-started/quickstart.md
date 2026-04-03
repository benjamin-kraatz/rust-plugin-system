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

## 3. Try other hosts

- `cargo run -p host-tui`
- `cargo run -p host-egui`
- `cargo run -p host-iced`
- `cargo run -p host-dioxus-desktop`
- `cargo run -p host-web`
- `cargo run -p host-service`

## 4. Explore the advanced plugin tracks

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

## 5. Understand what you are seeing

The first working slice uses **native dynamic libraries** with a **JSON-over-FFI** boundary:

- hosts discover plugin libraries at runtime
- hosts read a plugin manifest through exported symbols
- hosts send invocation requests as JSON
- plugins return structured responses as JSON

This keeps the ABI small and explicit while still demonstrating real runtime loading.

The repository also includes:

- **ABI-stable native plugins** loaded with `abi_stable`
- **sandboxed WASM plugins** executed with Wasmtime
