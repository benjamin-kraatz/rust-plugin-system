# Overview

This playground is designed to answer a deceptively simple question:

> **How can Rust and plugin systems work well together in real applications?**

The repository is intentionally broad. It does not stop at a single host app or a single plugin loading technique. Instead, it explores:

- multiple host surfaces
- multiple runtime-loading strategies
- multiple levels of safety and compatibility
- multiple documentation styles, from tutorial to reference

## Host surfaces

- **CLI** for direct command execution and scripting workflows
- **TUI** for terminal dashboards and keyboard-first workflows
- **Desktop GUI** for plugin-driven panels, inspectors, and actions
- **Web-facing host** for browser-friendly or remotely mediated plugin experiences
- **Service host** for backend hooks, transforms, and automation use cases

## Plugin architecture tracks

### 1. Native dynamic libraries

The most direct path to runtime loading. Great for learning and fast experimentation. The repo uses a JSON-over-FFI pattern to keep the ABI surface small and explicit.

### 2. ABI-stable native plugins

The more production-minded native path. This track explores compatibility and contract evolution more seriously.

### 3. WASM sandboxed plugins

The safest and most portable track in the repo. Great for untrusted extensions, web-adjacent workflows, and stricter runtime boundaries.

## How to use the repo

1. Read the high-level docs in `docs/`.
2. Start with the CLI host and the simplest plugins.
3. Compare the same plugin concepts in TUI and GUI hosts.
4. Explore ABI-stable and WASM tracks once the basics feel natural.

