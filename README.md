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
- `examples/` - packaging, distribution, and bundle layout assets for local release workflows
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

- `host-tui` - keyboard-first terminal host with separate plugin panes, action panes, inline payload editing, templates, and output review
- `host-egui` - inspector/dashboard host for browsing manifest metadata, action templates, payload editing, and output panels
- `host-iced` - explicit state/update desktop host where plugin selection, action selection, template loading, and invocation are part of the app flow
- `host-dioxus-desktop` - reactive desktop action studio with plugin/action selection, payload editing, invocation, and output panels; the crate uses an explicit `[[bin]]` target with `autobins = false`
- `host-web` - browser-facing catalog with manifest badges, action browser, payload composer, and result cards for human-guided comparison
- `host-service` - automation-first HTTP API with discovery, examples, detail endpoints, and `POST /plugins/{plugin_id}/actions/{action_id}/invoke` as the canonical invoke path

If you want the guided comparison, start with `docs/hosts/host-matrix.md` and `docs/hosts/surface-comparison.md`.

### Host × Architecture Matrix

| Host                | Native JSON | ABI-Stable | WASM  | UI Type            |
| ------------------- | :---------: | :--------: | :---: | ------------------ |
| host-cli            |      ✅      |     ✅      |   ✅   | Terminal           |
| host-tui            |      ✅      |     —      |   —   | Terminal (Ratatui) |
| host-egui           |      ✅      |     —      |   —   | Desktop (egui)     |
| host-iced           |      ✅      |     —      |   —   | Desktop (Iced)     |
| host-dioxus-desktop |      ✅      |     —      |   —   | Desktop (Dioxus)   |
| host-web            |      ✅      |     —      |   —   | Browser (Axum SSR) |
| host-service        |      ✅      |     —      |   —   | API (Axum JSON)    |

> Only `host-cli` loads all three plugin tracks directly. The other hosts use `host-core::Playground`, which loads native JSON plugins via `plugin-loader`.

### Plugin Catalog

| Plugin                  | Architecture | Skill Level  | Actions |
| ----------------------- | ------------ | ------------ | :-----: |
| hello-world             | Native JSON  | Basic        |    1    |
| logger                  | Native JSON  | Basic        |    1    |
| formatter               | Native JSON  | Basic        |    1    |
| transformer             | Native JSON  | Intermediate |    1    |
| command-pack            | Native JSON  | Intermediate |    1    |
| ui-panel                | Native JSON  | Intermediate |    1    |
| config-provider         | Native JSON  | Intermediate |    3    |
| filesystem-tools        | Native JSON  | Intermediate |    3    |
| data-pipeline           | Native JSON  | Intermediate |    3    |
| metrics-observer        | Native JSON  | Intermediate |    2    |
| service-hooks           | Native JSON  | Intermediate |    2    |
| tui-tools               | Native JSON  | Intermediate |    2    |
| abi-stable-greeter      | ABI-Stable   | Advanced     |    2    |
| abi-stable-command-pack | ABI-Stable   | Advanced     |    1    |
| wasm-sandboxed          | WASM         | Advanced     |    3    |
| web-widget              | WASM         | Advanced     |    3    |

## Quickstart

```bash
cargo build --workspace
cargo run -p host-cli -- list
cargo run -p host-cli -- run hello-world greet '{"name":"Rustacean"}'
cargo run -p host-cli -- run config-provider merge-layers '{"defaults":{"service":{"port":8080}},"environment":{"service":{"port":8081}},"overrides":{"service":{"host":"127.0.0.1"}}}'
cargo run -p host-cli -- run data-pipeline summarize-field '{"records":[{"duration":12.5},{"duration":7.5}],"field":"duration"}'
cargo run -p host-cli -- run abi-stable-greeter greet '{"name":"Rustacean"}'
cargo run -p host-cli -- run abi-stable-greeter plan-upgrade '{"from_host":"0.1.0","to_host":"0.2.0","consumer":"host-cli"}'
cargo run -p host-cli -- inspect service-hooks
cargo run -p host-cli -- run service-hooks preview-delivery '{"service":"billing","event":"deploy.succeeded","attempt":2}'
cargo run -p host-cli -- run wasm-sandboxed run-demo '{"note":"sandbox"}'
cargo run -p host-cli -- run web-widget render-widget '{"theme":"dark","variant":"incident"}'
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
- `docs/hosts/surface-comparison.md`
- `docs/plugins/native-json-catalog.md`
- `docs/snippets/cli-recipes.md`

## Publish new versions

Make sure that the token from Kellnr is available in the environment as `CARGO_REGISTRIES_DZWEI_REGISTRY_TOKEN`.

```bash
export CARGO_REGISTRIES_DZWEI_REGISTRY_TOKEN=yourtokentokellnr
LEVEL=$(./scripts/detect-release-level.sh)
cargo release "$LEVEL" --execute --no-publish
./scripts/publish-shared-crates.sh
```

## License

This project is licensed under MIT.
