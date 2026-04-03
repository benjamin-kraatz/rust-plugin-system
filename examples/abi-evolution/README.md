# ABI Evolution Example

Demonstrates how `abi_stable`'s prefix-type system allows you to evolve a
plugin interface across versions **without** breaking binary compatibility.

## What it demonstrates

- Defining a V1 and V2 interface struct using `#[sabi(kind(Prefix(...)))]`.
- Using `#[sabi(last_prefix_field)]` to mark the extension point.
- How a newer host can safely load an older plugin (missing fields → `None`).
- How an older host can safely load a newer plugin (extra fields ignored).

## How to run

```bash
cargo run -p example-abi-evolution
```

No plugin library is needed – this example prints a narrative explanation
with real struct sizes from your current platform.

## See also

- `plugins/abi-stable-greeter/` – a real ABI-stable plugin
- `crates/plugin-abi/` – the shared ABI module definition
