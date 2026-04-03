# Native JSON Plugin Model

The first implemented plugin track in this repository uses a deliberately small native ABI:

- `plugin_manifest_json`
- `plugin_invoke_json`
- `plugin_free_c_string`

Each symbol works with UTF-8 JSON strings across the FFI boundary.

## Why this approach?

Naively sharing Rust traits or rich Rust structs directly across runtime-loaded libraries is fragile. A JSON boundary is less fancy, but it is:

- easier to explain
- easier to debug
- easier to visualize in docs
- easier to adapt across many host types

That makes it ideal for the “foundational course” layer of the repo.

## Trade-offs

### Strengths

- Real runtime loading
- Clear boundary between host and plugin
- Easy inspection and logging
- The same conceptual model works in CLI, TUI, GUI, web, and service hosts

### Weaknesses

- Less type-rich than direct Rust APIs
- More serialization overhead
- Not the final word on production-grade native plugin compatibility

## Why keep this if ABI-stable and WASM tracks exist?

Because it teaches the problem space cleanly. Once you understand this model, the motivation for ABI-stable native plugins and sandboxed WASM plugins becomes much more concrete.

