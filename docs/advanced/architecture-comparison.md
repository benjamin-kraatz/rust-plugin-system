# Architecture Comparison

This repository now exposes three runtime-loading strategies side by side.

| Track | Current mechanism | Strength | Weakness |
| --- | --- | --- | --- |
| Native JSON dylib | `cdylib` + `libloading` + JSON-over-FFI | Simple and explicit | ABI is manually constrained |
| ABI-stable dylib | `abi_stable` root modules | Better compatibility story for native plugins | More machinery and interface discipline |
| WASM sandboxed | Wasmtime loading WAT/WASM modules | Isolation and portability | Lower-level data exchange and a different runtime model |

## When to start with each

### Native JSON dylib

Start here when you want to understand the basic mechanics of runtime loading without too much framework overhead.

### ABI-stable dylib

Move here when you want a more serious native plugin contract that can tolerate evolution better.

### WASM sandboxed

Use this when isolation, portability, or trust boundaries matter more than direct native integration.

## Repo examples

- Native JSON dylib: `hello-world`, `formatter`, `transformer`
- ABI-stable dylib: `abi-stable-greeter`, `abi-stable-command-pack`
- WASM sandboxed: `wasm-sandboxed`, `web-widget`


## Phase 4 follow-through

The advanced tracks are easier to compare now that the shared crates expose explicit production metadata. After reading this page, continue with:

- `docs/reference/version-compatibility.md`
- `docs/reference/lifecycle-execution.md`
- `docs/reference/trust-capabilities.md`
- `docs/reference/wasm-sandboxing.md`

Those docs explain which parts of the production story are shared across all tracks and which parts are only concretely enforced on the Wasm path today.
