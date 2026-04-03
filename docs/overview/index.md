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
- **Web-facing host** for browser-friendly plugin discovery and payload composition
- **Service host** for backend hooks, transforms, and automation use cases

## Phase 3: rich host comparison

The current host pass is no longer just "run the same binary with a different framework." The Phase 3 hosts expose the same plugin model with noticeably different UX choices:

- `host-tui` emphasizes keyboard-first panes, inline payload editing, templates, and fast output review
- `host-egui` feels like an inspector/dashboard for manifest metadata and action payloads
- `host-iced` highlights explicit state/update flow and more deliberate application structure
- `host-dioxus-desktop` presents a reactive desktop action studio for selection, payload editing, and invocation
- `host-web` focuses on browser-guided discovery, payload composition, and result cards
- `host-service` exposes the same ideas as a structured JSON API for automation

Across those hosts, the shared baseline is now clearer:

- plugin selection and action selection
- payload hints loaded through `host-core::default_payload_text()`
- host-fit checks via `host-core::supports_host()`
- manifest metadata such as tags, capabilities, notes, supported hosts, architecture, and skill level
- output/result framing that helps compare the same action across terminal, desktop, browser, and service surfaces

For the guided comparison, read:

- `docs/hosts/host-matrix.md`
- `docs/hosts/surface-comparison.md`

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
3. Use `docs/plugins/native-json-catalog.md` to walk from the foundation plugins into the expanded Phase 2 practical catalog.
4. Compare the Phase 3 TUI, desktop, web, and service hosts with `docs/hosts/surface-comparison.md`.
5. Explore the Phase 4 contract layer with `docs/reference/production-contracts.md`.
6. Use the ABI-stable and WASM tracks to connect those contracts back to real runtime trade-offs.
