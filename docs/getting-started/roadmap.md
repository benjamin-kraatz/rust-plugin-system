# Roadmap

## Stage 1: Understand the shared model

Start with the crates that define manifests, protocol messages, and loader behavior.

## Stage 2: Load native plugins

Use the CLI host to discover and invoke native runtime-loaded plugins.

## Stage 3: Compare rich host surfaces

Move to TUI, desktop, web, and service hosts to see how the same plugin ideas adapt across environments.

Recommended reading order:

1. `docs/hosts/host-matrix.md` for the short orientation
2. `docs/hosts/rich-host-surfaces.md` for the detailed comparison
3. `docs/reference/workspace-map.md` to connect the host apps back to shared crates such as `host-core`

Focus on these questions while you compare:

- Which host is optimized for human browsing versus automation?
- How does each framework expose plugin selection, action selection, and payload editing?
- Where do manifest metadata, payload templates, and output framing feel natural or awkward?
- What changes between browser/service surfaces and terminal/desktop surfaces?

## Stage 4: Explore advanced tracks

Use ABI-stable and WASM examples to learn where native dynamic loading becomes fragile and what alternatives look like.
