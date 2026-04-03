# State Management

Plugins in this system are designed to be stateless by default. Each invocation
receives a self-contained `PluginRequest` and returns a `PluginResponse` with no
implicit shared memory between calls. This guide covers the patterns available
when you need state.

---

## Stateless plugins (the default)

Most plugins derive all behaviour from the request payload and invocation
context. The `config-provider` plugin is a good example — every action receives
its full input in the payload:

```rust
fn merge_layers(request: PluginRequest) -> Result<PluginResponse, String> {
    let defaults = object_from_payload(&request.payload, "defaults")?;
    let environment = object_from_payload(&request.payload, "environment")?;
    let overrides = object_from_payload(&request.payload, "overrides")?;

    let mut merged = Value::Object(defaults);
    merge_values(&mut merged, Value::Object(environment));
    merge_values(&mut merged, Value::Object(overrides));
    // ... build response from merged config
}
```

No global or thread-local state is touched. The function is pure: same input
always produces the same output. This is the recommended pattern for most
plugins because it keeps the FFI boundary simple and avoids concurrency hazards.

---

## Configuration injection via payload

Rather than reading config files or environment variables directly, plugins
receive configuration through the request payload. Hosts prepare the payload
before invoking the plugin, which keeps the plugin sandboxable and testable.

The `config-provider` plugin demonstrates this explicitly: the host passes
`defaults`, `environment`, and `overrides` objects in the payload, and the
plugin merges them without knowing where the values came from.

```json
{
  "defaults": { "service": { "port": 8080 } },
  "environment": { "service": { "port": 8081 } },
  "overrides": { "service": { "host": "127.0.0.1" } }
}
```

For plugins that need host-specific configuration, use `InvocationContext`:

```rust
let host = request.context.host;           // HostKind::Cli, etc.
let workspace = &request.context.workspace_root;  // Option<String>
let plugin_dir = &request.context.plugin_dir;     // Option<String>
```

This keeps the plugin deterministic — the same context and payload always
produce the same response.

---

## Context-derived state

The `InvocationContext` carries runtime information that plugins can use to
adapt behaviour without maintaining internal state:

```rust
pub struct InvocationContext {
    pub host: HostKind,
    pub workspace_root: Option<String>,
    pub plugin_dir: Option<String>,
    pub mode: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub timeout_ms: Option<u64>,
    pub warnings: Vec<String>,
    pub runtime: Option<RuntimeContext>,
}
```

The `service-hooks` plugin uses context fields to tailor webhook delivery
previews per host kind without storing anything between invocations.

---

## Thread-local state (advanced)

When a native plugin needs mutable state within a single invocation (e.g.
accumulating intermediate results), standard Rust patterns apply because the
plugin runs in the host process. However, because multiple hosts could load
the same shared library, any cross-invocation state must be `Send + Sync` or
use `thread_local!`:

```rust
use std::cell::RefCell;

thread_local! {
    static CALL_COUNT: RefCell<u64> = const { RefCell::new(0) };
}

fn invoke(request: PluginRequest) -> Result<PluginResponse, String> {
    CALL_COUNT.with(|count| *count.borrow_mut() += 1);
    // ...
}
```

> **Warning:** Thread-local state does not survive across host restarts, is
> invisible to other threads, and is not available in WASM plugins at all.
> Prefer payload-driven state whenever possible.

---

## State across invocations (current limitations)

The plugin system does not currently provide a built-in persistence layer.
Each invocation is independent. If you need state across calls, consider:

1. **Host-managed state** — the host stores results and feeds them back to
   the plugin via the payload on subsequent calls.
2. **Filesystem** — native plugins can read/write files if the manifest
   declares `mutates_workspace: true` in the action contract. The
   `workspace_root` field in `InvocationContext` provides the path.
3. **External services** — plugins with `NetworkAccess::Outbound` or
   `NetworkAccess::Full` (declared in `TrustMetadata`) can call APIs.

Future patterns under consideration:

- A host-provided key-value store injected through `RuntimeContext`
- Session-scoped state that the host manages and serialises between calls
- A `PluginState` return field that hosts persist and re-inject

---

## Guideline summary

| Pattern                     | When to use                           | Caveats                          |
|-----------------------------|---------------------------------------|----------------------------------|
| Stateless (payload-driven)  | Default — most plugins                | None                             |
| Context-derived             | Adapting to host kind or workspace    | Context may be incomplete        |
| Thread-local                | In-process counters or caches         | Not portable to WASM             |
| Host-managed persistence    | Multi-step workflows                  | Requires host cooperation        |
| Filesystem / external APIs  | Durable state                         | Must declare in manifest trust   |
