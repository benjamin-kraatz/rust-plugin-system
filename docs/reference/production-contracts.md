# Production Contracts Guide

Phase 4 turns the playground from a runtime-loading demo into a repo that also explains the contracts you would need in a longer-lived plugin ecosystem.

Use this guide as the hub for the production-shaped layer:

1. [`version-compatibility.md`](./version-compatibility.md) - how manifests declare protocol versions, host-version windows, and tested hosts
2. [`lifecycle-execution.md`](./lifecycle-execution.md) - how lifecycle, execution mode, async metadata, timeouts, and response metadata fit together
3. [`trust-capabilities.md`](./trust-capabilities.md) - how trust metadata, permissions, capability negotiation, and feature degradation are modeled
4. [`wasm-sandboxing.md`](./wasm-sandboxing.md) - what the current Wasmtime-based sandbox actually does
5. [`testing-packaging.md`](./testing-packaging.md) - how to validate contracts with `plugin-test-kit` and how the example bundle layouts work

## Suggested reading order

If you already understand the Phase 1-3 material, read the Phase 4 layer in this order:

```bash
cargo run -p host-cli -- inspect metrics-observer
cargo run -p host-cli -- inspect abi-stable-command-pack
cargo run -p host-cli -- inspect wasm-sandboxed
```

Those three plugins are the most concrete examples of the new contract surface:

- `metrics-observer` shows compatibility, trust metadata, lifecycle hooks, execution metadata, and capability degradation in one native plugin
- `abi-stable-command-pack` shows the same ideas on the ABI-stable track
- `wasm-sandboxed` shows how the sandboxed track describes isolation and capability expectations

## What Phase 4 adds

The shared crates now model:

- compatibility strategy and tested-host notes through `CompatibilityContract`
- lifecycle state and hooks through `LifecycleContract`
- execution defaults plus async hints through `ExecutionContract` and response-side `ExecutionMetadata`
- trust, sandbox, network, and permission metadata through `TrustMetadata`
- capability requirements and degradation rules through `CapabilityContract`
- reusable fixtures, builders, and package manifests through `plugin-test-kit`

Hosts surface that data in two ways:

- `host-cli inspect <plugin>` prints the declared manifest contracts
- host-side helpers in `crates/host-core` turn those declarations into host-fit summaries, warnings, execution metadata, and capability negotiation outcomes

## Important caveat

This layer is intentionally honest about the difference between **declared contract metadata** and **runtime enforcement**.

Today the repo demonstrates:

- metadata modeling in manifests
- host-side assessment in `host-core`
- response metadata rendering in `plugin-runtime` and `host-cli`
- a real sandboxed runtime only for the Wasm track via `plugin-wasm`

It does **not** yet provide a global registry, dependency solver, permission broker, async job queue, or lifecycle supervisor. When the docs say a plugin is "async-capable" or "requires explicit shutdown," that is a contract the host can react to, not a universal runtime guarantee.

## Where to connect this back to the course

- Start at [`../overview/index.md`](../overview/index.md) for the big-picture course map
- Use [`../getting-started/quickstart.md`](../getting-started/quickstart.md) for runnable commands
- Use [`../advanced/architecture-comparison.md`](../advanced/architecture-comparison.md) to compare native, ABI-stable, and Wasm trade-offs
- Use [`../reference/workspace-map.md`](./workspace-map.md) to connect the docs to the workspace layout
