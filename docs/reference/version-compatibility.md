# Version and Compatibility Strategy

The Phase 4 manifest model adds a dedicated `CompatibilityContract` so plugins can describe how carefully they expect hosts to evolve around them.

## The contract shape

`crates/plugin-manifest` exposes:

- `VersionStrategy` - `semver`, `exact`, `lockstep`, or `host-defined`
- `protocol_version` - the plugin protocol revision the manifest was authored against
- `host_version` - an optional minimum/maximum host-version window
- `tested_hosts` - explicit host/version pairs that were exercised
- `notes` - free-form guidance for operators and plugin authors

You can see a concrete example in `plugins/metrics-observer/src/lib.rs` and `plugins/wasm-sandboxed/wasm-plugin.json`.

## What hosts do with it today

`crates/host-core` uses compatibility data during host-fit assessment.

When a host provides `InvocationContext.runtime.host_version`, the current logic:

1. parses the declared minimum/maximum version range with `semver`
2. checks whether the host version falls inside that window
3. prefers a `tested_hosts` note when there is an exact host/version match
4. folds the result into the `NegotiationOutcome` summary

That means the current enforcement is practical but intentionally small:

- the **version window** is checked
- the **tested host list** improves the summary
- the **strategy enum** is surfaced as metadata, but it is not yet used to switch between different resolution algorithms

## Declared strategy vs current behavior

Use the strategies as documentation of intent:

- `semver` - additive evolution is expected
- `exact` - plugin and host should match exactly
- `lockstep` - plugin and host should move together
- `host-defined` - the host owns the policy

Important caveat: the current host-side implementation still evaluates the declared min/max window the same way regardless of the chosen strategy. The strategy is meaningful documentation, but not yet a full policy engine.

## How to inspect it

Run:

```bash
cargo run -p host-cli -- inspect metrics-observer
cargo run -p host-cli -- inspect abi-stable-command-pack
cargo run -p host-cli -- inspect wasm-sandboxed
```

`host-cli` prints:

- strategy
- protocol version
- host-version window
- tested hosts
- compatibility notes

## Authoring guidance

Use compatibility metadata when you want to explain:

- which host releases you have actually exercised
- whether response shapes are expected to evolve additively
- whether a plugin must stay pinned to a host release line
- whether a plugin should only be adopted by specific host surfaces

A good pattern is the one used by `metrics-observer`:

- declare `semver`
- record a protocol version
- add a conservative host-version window
- list the host/version pairs that were actually tested
- write a note describing the compatibility promise in plain language

## What this does not claim

This repo does **not** yet include:

- a registry that resolves plugin dependencies
- automatic upgrade or downgrade flows
- ABI migration tooling
- multi-version dispatch based on `VersionStrategy`

So treat compatibility data as **host-readable contract metadata** plus a **basic host-version check**, not a full packaging ecosystem.

## Related docs

- [`production-contracts.md`](./production-contracts.md)
- [`lifecycle-execution.md`](./lifecycle-execution.md)
- [`trust-capabilities.md`](./trust-capabilities.md)
- [`testing-packaging.md`](./testing-packaging.md)
