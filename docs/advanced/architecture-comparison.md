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
