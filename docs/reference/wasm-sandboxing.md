# Wasm Sandboxing in This Repo

The Wasm track is the most isolated runtime path in the playground, but it is intentionally small enough to inspect end to end.

## What the loader expects

`crates/plugin-wasm` scans `plugins/` for directories that contain:

- `wasm-plugin.json`
- `module.wasm` or `module.wat`

`load_plugin_from_dir()` reads the manifest first, then loads the Wasm or WAT module next to it.

## How invocation works

`LoadedWasmPlugin::invoke()` uses Wasmtime directly:

1. create a fresh `Engine`
2. compile or load the module
3. instantiate it with no host imports
4. find exported `memory`
5. find exported `alloc(len) -> ptr`
6. find exported `invoke_json(ptr, len) -> (ptr, len)`
7. serialize the `PluginRequest` to JSON
8. copy the request bytes into Wasm memory
9. call `invoke_json`
10. read the response bytes back out of memory and deserialize `PluginResponse`

That makes the boundary easy to understand: the current Wasm runtime is a JSON-over-memory protocol hosted by Wasmtime.

## What the demo plugins actually do

The visible Wasm examples are intentionally simple.

- `plugins/wasm-sandboxed/module.wat` returns a static JSON response from linear memory
- `plugins/web-widget/module.wat` follows the same teaching-oriented style

There are no host calls for filesystem, network, or clock access in the current demo runtime.

## What the sandbox means here

In this repo, the Wasm sandbox gives you:

- isolated linear memory inside the Wasmtime instance
- a narrow exported function surface
- a runtime boundary that is more constrained than the native plugin tracks

It does **not** currently add:

- a custom capability broker
- host imports for safe filesystem or network mediation
- persistent Wasm instances across requests
- async job orchestration inside the Wasm runtime

The `wasm-sandboxed` manifest is therefore accurate when it says the module is sandboxed and has no outbound I/O.

## Packaging and distribution

The Wasm packaging examples live in:

- `examples/packaging/wasm/wasm-sandboxed-bundle/`
- `examples/packaging/package-release.sh`

The bundle keeps `wasm-plugin.json`, `module.wat`, and `release.json` together. That matches the loader's expectation that the manifest and module live side by side.

## When to use this track

Use the Wasm track when you want to explore:

- stronger runtime isolation than native plugins
- portable plugin bundles
- web or service-flavored plugin boundaries
- host-managed contracts for less-trusted extensions

Use the native tracks when you need tighter integration and are comfortable owning more of the trust boundary yourself.

## Related docs

- [`production-contracts.md`](./production-contracts.md)
- [`trust-capabilities.md`](./trust-capabilities.md)
- [`lifecycle-execution.md`](./lifecycle-execution.md)
- [`testing-packaging.md`](./testing-packaging.md)
- [`../advanced/architecture-comparison.md`](../advanced/architecture-comparison.md)
