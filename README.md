# Rust Plugin System Playground

This repository is a **course-like workspace** for exploring runtime-loadable plugins in Rust across multiple host surfaces:

- CLI
- TUI
- Desktop GUI with `egui/eframe`
- Desktop GUI with `Iced`
- Desktop GUI with `Dioxus Desktop`
- Web-facing host
- Service/API host

It compares three plugin architecture tracks:

1. **Native dynamic libraries** for direct runtime loading
2. **ABI-stable native plugins** for more durable contracts
3. **WASM sandboxed plugins** for stronger isolation and portability

The goal is not just to show code that works, but to make the trade-offs clear enough that the repository can be used as a public learning resource and a practical snippet zoo.

## Repository map

- `crates/` - shared contracts, loaders, runtimes, SDKs, and host support crates
- `hosts/` - host applications that load plugins at runtime
- `plugins/` - example plugin implementations across difficulty levels
- `docs/` - course material, reference docs, comparisons, and tutorials

## Current focus

The current implementation slice establishes:

- a Cargo workspace
- a shared JSON-based plugin protocol
- a native runtime-loading path
- multiple host apps that consume the same plugin model
- a documentation backbone for the broader course experience
- a practical native plugin catalog that now includes:
  - foundation examples: `hello-world`, `logger`, `formatter`, `transformer`, `command-pack`, `ui-panel`
  - Phase 2 examples: `config-provider`, `filesystem-tools`, `data-pipeline`, `metrics-observer`, `service-hooks`, `tui-tools`
- a Phase 3 host comparison pass with richer host surfaces built on shared host-core helpers such as `default_payload_text()` and `supports_host()`

## Rich host surfaces at a glance

Phase 3 turns the host apps into a real comparison set instead of a binary list:

- `host-tui` - keyboard-first, multi-pane browsing for terminal workflows; great when you want fast plugin/action switching without leaving the terminal
- `host-egui` - inspector/dashboard feel with manifest metadata, action selection, payload editing, and a straightforward output panel
- `host-iced` - explicit state/update desktop host that makes application state transitions easy to follow while comparing plugin metadata and invocation flow
- `host-dioxus-desktop` - reactive desktop surface with plugin and action pickers, payload editing, invocation, and output panels; the crate uses an explicit `[[bin]]` target with `autobins = false`
- `host-web` - browser-facing catalog with manifest badges, action cards, payload composition, and result cards for human-guided exploration
- `host-service` - automation-first HTTP API with catalog, detail, examples, and invocation endpoints for scripts or backend workflows

If you want a guided comparison, start with `docs/hosts/host-matrix.md` and `docs/hosts/rich-host-surfaces.md`.

## Quickstart

```bash
cargo build --workspace
cargo run -p host-cli -- list
cargo run -p host-cli -- run hello-world greet '{"name":"Rustacean"}'
cargo run -p host-cli -- run config-provider merge-layers '{"defaults":{"service":{"port":8080}},"environment":{"service":{"port":8081}},"overrides":{"service":{"host":"127.0.0.1"}}}'
cargo run -p host-cli -- run data-pipeline summarize-field '{"records":[{"duration":12.5},{"duration":7.5}],"field":"duration"}'
cargo run -p host-cli -- run abi-stable-greeter greet '{"name":"Rustacean"}'
cargo run -p host-cli -- run wasm-sandboxed run-demo '{"note":"sandbox"}'
```

Then compare richer hosts:

```bash
cargo run -p host-tui
cargo run -p host-egui
cargo run -p host-iced
cargo run -p host-dioxus-desktop
cargo run -p host-web      # http://127.0.0.1:4000
cargo run -p host-service  # http://127.0.0.1:5000
```

Start in `docs/overview/index.md`, then follow:

- `docs/getting-started/quickstart.md`
- `docs/hosts/host-matrix.md`
- `docs/hosts/rich-host-surfaces.md`
- `docs/plugins/native-json-catalog.md`
- `docs/snippets/cli-recipes.md`

## License

This project is licensed under MIT.
