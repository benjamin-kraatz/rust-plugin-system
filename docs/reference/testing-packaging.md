# Testing Contracts and Packaging Plugins

Phase 4 adds two practical pieces around the new contract model:

- `plugin-test-kit` for deterministic tests and fixtures
- packaging examples that show what a local release bundle looks like for each runtime track

## Testing with `plugin-test-kit`

`crates/plugin-test-kit` is a shared helper crate for Phase 4 tests.

It includes builders for:

- manifests with `ManifestBuilder` and `ActionBuilder`
- invocation contexts with `ContextBuilder`
- requests with `RequestBuilder`
- responses with `ResponseBuilder`

It also includes assertion helpers such as:

- `assert_manifest_has_action`
- `assert_manifest_hosts`
- `assert_payload_eq`
- `assert_response_ok`
- `assert_response_error`
- `assert_output_contains`
- `assert_next_steps`

See the real integration tests in:

- `crates/plugin-manifest/tests/test_kit_manifest.rs`
- `crates/plugin-protocol/tests/test_kit_protocol.rs`
- `crates/plugin-runtime/tests/test_kit_runtime.rs`

Those tests are a good template for validating that contract data survives serialization and that rendered output stays stable.

## Package fixtures in tests

The same crate also models package metadata through:

- `PackageManifest`
- `ReleaseMetadata`
- `PackageFixture`

`PackageFixture::{native_json, abi_stable, wasm}` lets tests write deterministic bundle layouts to disk and then assert which artifacts are required.

That is how the repo keeps packaging examples, manifest snapshots, and release metadata aligned.

## Packaging examples in the repo

The repo contains two useful packaging views.

### 1. A manual native install skeleton

`examples/packaging/native-json-hello-world/` shows a hand-authored local bundle layout plus an `install-local.sh` helper.

Use it to understand the simplest native distribution story:

- compile the plugin
- copy the library beside its manifest
- install into a directory a host can discover
- run `host-cli list` or `host-cli inspect` to validate the result

### 2. Per-runtime example bundles

`examples/packaging/{native-json,abi-stable,wasm}/` contains Phase 4-style bundle directories with:

- `package.json`
- a manifest snapshot (`plugin-manifest.json` or `wasm-plugin.json`)
- `release.json`
- the runtime entrypoint path expected by the bundle

`examples/packaging/package-release.sh` builds the native examples and assembles all three bundle styles under `target/example-bundles`.

## What the package metadata means

The example `package.json` files capture:

- schema version
- package name
- runtime kind
- bundle entrypoint
- embedded plugin manifest snapshot
- artifact list
- release metadata such as channel, target, supported hosts, and installer hint

This is intentionally local and file-oriented. The repo does **not** yet include a package registry, signing service, or dependency graph.

## Recommended validation loop

```bash
cargo test --workspace
cargo run -p host-cli -- inspect metrics-observer
bash examples/packaging/package-release.sh
cargo run -p host-cli -- inspect hello-world
```

If you are authoring your own plugin, copy the style of the example bundles and then adapt the entrypoint, manifest snapshot, and release metadata to your plugin.

## Related docs

- [`production-contracts.md`](./production-contracts.md)
- [`version-compatibility.md`](./version-compatibility.md)
- [`trust-capabilities.md`](./trust-capabilities.md)
- [`wasm-sandboxing.md`](./wasm-sandboxing.md)
