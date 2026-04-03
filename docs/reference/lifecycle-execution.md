# Lifecycle and Execution Semantics

Phase 4 adds two related manifest layers:

- `LifecycleContract` describes how a plugin wants to be managed
- `ExecutionContract` describes how a plugin expects to run

Together they let hosts expose richer metadata without inventing their own ad hoc fields.

## Lifecycle contracts

`LifecycleContract` models:

- `state` - `registered`, `loaded`, `initializing`, `ready`, `degraded`, `draining`, or `stopped`
- `stateless` - whether invocations should be treated as independent
- `requires_explicit_shutdown` - whether the host should clean up intentionally
- `hooks` - `install`, `load`, `initialize`, `invoke`, `health-check`, `suspend`, `resume`, `shutdown`
- `health_probe` - an operator-facing hint for validating the plugin
- `notes` - extra context

Example:

- `metrics-observer` declares `load`, `invoke`, and `health-check`
- `wasm-sandboxed` declares `load` and `invoke`

## Execution contracts

`ExecutionContract` models:

- default mode: `sync` or `async`
- whether async execution is supported at all
- whether requests are cancellable
- whether calls are idempotent
- whether the plugin can report progress
- timeout hints
- max concurrency hints
- optional async details such as detached execution, streaming support, completion timeout, and retry policy

Action-level `ActionContract` can further refine execution mode, idempotency, workspace mutation, timeouts, async metadata, and capability constraints for a specific action.

## What hosts do with it today

`crates/host-core` converts manifest declarations into response-side `ExecutionMetadata`.

That metadata currently includes:

- execution mode
- async support
- cancellable flag
- timeout
- measured duration
- lifecycle state
- optional progress message
- reserved `job` metadata field

The default execution mode is host-shaped:

- CLI and TUI default to `sync`
- web, service, and desktop hosts default to `async`

That preference comes from `host-core::default_runtime_context()`.

## Important caveats

The current repo treats lifecycle and async data as **contracts and hints**, not as a global supervisor.

That means:

- hooks are declared in the manifest, but there is no central lifecycle engine that invokes them automatically
- `supports_async = true` does not create a background worker by itself
- `timeout_ms` is surfaced to the host and copied into response metadata, but it is not universally enforced by a scheduler
- `JobMetadata` exists in the protocol, but the current hosts do not yet populate job IDs or progress streams

A good way to read the current implementation is:

- the manifest says what a plugin expects
- the host can adapt its UX and warnings around that expectation
- the protocol has room for richer runtime orchestration later

## What `host-cli` shows

Use:

```bash
cargo run -p host-cli -- inspect metrics-observer
cargo run -p host-cli -- run metrics-observer summarize-signals '{"service":"checkout","window_minutes":15,"samples":{"requests":1800,"errors":18,"p95_ms":245,"saturation_pct":61}}'
```

The inspect output shows the declared lifecycle and execution contract.
The run output shows synthesized execution metadata such as mode, timeout, duration, async capability, and lifecycle state.

## Authoring guidance

Declare lifecycle data when a host should know whether a plugin is:

- safe to treat as stateless
- expected to expose health checks
- expected to clean up explicitly
- in a degraded or draining state

Declare execution data when a host should know whether an action is:

- safe to queue or retry
- capable of background execution
- likely to mutate the workspace
- sensitive to timeout or concurrency limits

## Related docs

- [`production-contracts.md`](./production-contracts.md)
- [`version-compatibility.md`](./version-compatibility.md)
- [`trust-capabilities.md`](./trust-capabilities.md)
- [`wasm-sandboxing.md`](./wasm-sandboxing.md)
