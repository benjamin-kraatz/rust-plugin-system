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

The first implementation slice establishes:

- a Cargo workspace
- a shared JSON-based plugin protocol
- a native runtime-loading path
- multiple host apps that consume the same plugin model
- a documentation backbone for the broader course experience

Start in `docs/overview/index.md`.

## License

This project is licensed under MIT.