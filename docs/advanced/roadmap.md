# Advanced Track Roadmap

The workspace already includes crate slots for the next two major architecture tracks:

- `crates/plugin-abi`
- `crates/plugin-wasm`

## ABI-stable native plugins

This track is for:

- compatibility-oriented native extension contracts
- longer-lived plugin ecosystems
- version negotiation and interface evolution

## WASM sandboxed plugins

This track is for:

- stronger runtime isolation
- portability
- safer execution of less-trusted extensions
- web/service-oriented plugin workflows

## Current status

The repository now has all three tracks represented in working form:

- **Native JSON dylib track** through the shared `plugin-loader`, including both the starter plugins and the expanded practical catalog (`config-provider`, `filesystem-tools`, `data-pipeline`, `metrics-observer`, `service-hooks`, `tui-tools`)
- **ABI-stable dylib track** through `crates/plugin-abi` and the `abi-stable-*` plugins
- **WASM sandboxed track** through `crates/plugin-wasm` and WAT-backed modules executed by Wasmtime

The next layer of work is no longer just runtime experimentation. Phase 4 now documents the production-shaped contract layer that sits on top of those tracks.

Use the new reference set for that pass:

- `docs/reference/production-contracts.md`
- `docs/reference/version-compatibility.md`
- `docs/reference/lifecycle-execution.md`
- `docs/reference/trust-capabilities.md`
- `docs/reference/wasm-sandboxing.md`
- `docs/reference/testing-packaging.md`

## Current commands

```bash
cargo run -p host-cli -- inspect service-hooks
cargo run -p host-cli -- run service-hooks preview-delivery '{"service":"billing","event":"deploy.succeeded","attempt":2}'
cargo run -p host-cli -- run abi-stable-greeter plan-upgrade '{"from_host":"0.1.0","to_host":"0.2.0","consumer":"host-cli"}'
cargo run -p host-cli -- run wasm-sandboxed run-demo '{"note":"sandbox"}'
cargo run -p host-cli -- run web-widget render-widget '{"theme":"dark","variant":"incident"}'
```

## Expansion ideas

- Rust-authored `wasm32` plugins to complement the visible WAT modules
- richer ABI-stable module evolution examples across versions
- host-specific rendering layers for advanced track plugins in GUI/web/service hosts
- stronger runtime enforcement for lifecycle, async jobs, and capability policies
- registry, signing, and upgrade workflows on top of the current packaging examples
