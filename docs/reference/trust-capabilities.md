# Trust, Security, and Capability Negotiation

Phase 4 splits production concerns into two layers:

- **trust metadata** tells the host what kind of plugin it is dealing with
- **capability negotiation** tells the host whether it can satisfy the plugin's declared expectations

They are related, but not the same.

## Trust and security metadata

`TrustMetadata` describes:

- trust level: `low`, `reviewed`, `restricted`, `privileged`
- sandbox level: `none`, `process`, `wasm`, `host-mediated`
- network policy: `none`, `loopback`, `allowlisted`, `full`
- whether results are deterministic
- whether execution is local-only
- scoped permissions
- declared data-access patterns
- provenance and notes

`host-core` turns that metadata into warnings such as:

- plugin may access non-local resources
- plugin declares elevated permissions
- plugin may produce non-deterministic results

## What is actually enforced today

This repo is careful not to overclaim.

Today:

- **Wasm plugins** run inside a real Wasmtime runtime through `crates/plugin-wasm`
- **native JSON plugins** and **ABI-stable plugins** still run as native code in-process once loaded
- trust metadata for native plugins is descriptive metadata plus host warnings, not a hard permission sandbox

So the safest reading is:

- `trust` fields explain expected operating conditions
- hosts can display warnings and make policy decisions
- only the Wasm track currently demonstrates a concrete sandbox boundary in the runtime itself

## Capability negotiation

`CapabilityContract` describes:

- required capabilities
- optional capabilities
- optional runtime constraints such as permission scopes, payload-size limits, network access, and sandbox level
- degradation rules that explain what happens when optional features are missing
- notes for host authors

`host-core::negotiate_capabilities()` compares that contract with `InvocationContext.runtime.available_capabilities` and produces a `NegotiationOutcome`.

The current statuses are:

- `Ready` - required capabilities are present and no degradation is needed
- `Degraded` - execution can proceed, but optional capabilities are missing or degradation rules apply
- `Rejected` - host kind, host version, or required capabilities do not satisfy the contract

## What "rejected" means in this repo

`Rejected` is a **host-fit assessment**, not an unconditional hard stop.

The current hosts surface the rejection in summaries and warnings, but they do not universally block invocation just because a plugin declared a narrower host set or a missing capability. That is deliberate: the repo wants you to see the contract data and the trade-off, not hide the plugin from view.

## Concrete examples

- `metrics-observer` requires `stdout-json`, treats `markdown-output` as optional, and declares a low-severity degradation rule for the markdown summary
- `abi-stable-command-pack` requires `code-output` and degrades to raw text when it is unavailable
- `wasm-sandboxed` optionally benefits from a `sandboxed-runtime` capability so hosts can label the isolation boundary more explicitly

## Authoring guidance

Use trust metadata when you want to answer:

- should an operator treat this plugin as privileged?
- does it depend on network access?
- does it need scoped file or process permissions?
- is it deterministic enough for replay, previews, or tests?

Use capability negotiation when you want to answer:

- what must the host provide?
- what can be omitted with graceful degradation?
- what fallback should the host or operator expect?

## Related docs

- [`production-contracts.md`](./production-contracts.md)
- [`version-compatibility.md`](./version-compatibility.md)
- [`lifecycle-execution.md`](./lifecycle-execution.md)
- [`wasm-sandboxing.md`](./wasm-sandboxing.md)
- [`testing-packaging.md`](./testing-packaging.md)
