# Error Handling

Plugins communicate errors through the same `PluginResponse` envelope used for
successful results. This guide covers how errors flow from plugin code across
the FFI boundary to the host, and the common patterns you should follow.

---

## The error response format

A failed response sets `success: false` and uses `title`/`summary` to describe
what went wrong:

```rust
PluginResponse::error(
    "my-plugin",         // plugin_id
    "decode-request",    // action_id (or the action that failed)
    "Failed to decode request",  // title shown to the user
    error.to_string(),   // summary with details
)
```

The `PluginResponse::error` constructor mirrors `PluginResponse::ok` but sets
`success` to `false`:

```rust
pub fn error(
    plugin_id: impl Into<String>,
    action_id: impl Into<String>,
    title: impl Into<String>,
    summary: impl Into<String>,
) -> Self {
    Self {
        success: false,
        outputs: Vec::new(),
        suggested_next_steps: Vec::new(),
        // ... remaining fields
    }
}
```

You can still attach outputs and next steps to error responses — for example,
including diagnostic JSON or a suggested fix.

---

## Error patterns in native JSON plugins

### Returning `Err` from the `JsonPlugin` trait

The simplest error path: return `Err(String)` from `invoke`. The
`export_plugin!` macro wraps it in `PluginResponse::error` automatically:

```rust
impl JsonPlugin for MyPlugin {
    fn invoke(request: PluginRequest) -> Result<PluginResponse, String> {
        match request.action_id.as_str() {
            "greet" => Ok(/* ... */),
            other => Err(format!("unknown action '{other}'")),
        }
    }
}
```

The macro-generated FFI function converts `Err` into:

```rust
PluginResponse::error(plugin_id, action_id, "Plugin invocation failed", error)
```

### Returning an explicit error response

For richer error reporting, return `Ok(PluginResponse::error(...))` instead:

```rust
fn get_value(request: PluginRequest) -> Result<PluginResponse, String> {
    let config = request
        .payload
        .get("config")
        .ok_or_else(|| "payload.config is required".to_owned())?;
    let path = request
        .payload
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| "payload.path must be a string".to_owned())?;
    // ...
}
```

In the `config-provider` plugin, the `?` operator propagates validation errors
as `Err(String)`, which the macro converts into an error response.

---

## Error handling in ABI-stable plugins

ABI-stable plugins do not use the `export_plugin!` macro, so they must handle
deserialization errors explicitly in their `invoke_json` function:

```rust
extern "C" fn invoke_json(request_json: RString) -> RString {
    let response = match serde_json::from_str::<PluginRequest>(request_json.as_str()) {
        Ok(request) => invoke(request),
        Err(error) => PluginResponse::error(
            "abi-stable-greeter",
            "decode-request",
            "Failed to decode request",
            error.to_string(),
        ),
    };
    RString::from(
        serde_json::to_string(&response)
            .expect("response serialization should succeed"),
    )
}
```

The inner `invoke` function handles unknown actions the same way:

```rust
fn invoke(request: PluginRequest) -> PluginResponse {
    match request.action_id.as_str() {
        "greet" => greet(request),
        other => PluginResponse::error(
            "abi-stable-greeter",
            other,
            "Unknown action",
            format!("unknown action '{other}'"),
        ),
    }
}
```

---

## Common error patterns

### Invalid payload

Validate required fields early and return descriptive messages:

```rust
let config = request
    .payload
    .get("config")
    .ok_or_else(|| "payload.config is required".to_owned())?;
```

### Missing or unknown action

Always include a catch-all arm in your action match:

```rust
match request.action_id.as_str() {
    "greet" => greet(request),
    "inspect" => inspect(request),
    other => Err(format!("unknown action '{other}'")),
}
```

### Plugin load failure

Plugin load errors are handled by the host, not the plugin. The `plugin-loader`
crate returns warnings when a shared library cannot be loaded or is missing
expected symbols. The `host-cli` prints these as discovery warnings:

```rust
// In host-cli main.rs
for warning in playground.warnings() {
    eprintln!("  ⚠ {warning}");
}
```

Common causes: missing `cdylib` artifact, symbol name mismatch, incompatible
architecture.

---

## Error propagation across the FFI boundary

The FFI boundary between host and plugin uses JSON strings, so errors never
propagate as Rust panics or exceptions. The contract is:

1. The plugin **always** returns a valid JSON `PluginResponse`.
2. If the plugin panics, the `export_plugin!` macro does not catch it — this
   will abort the host process. **Never panic in plugin code.** Use `Result`
   and the `?` operator instead.
3. If JSON deserialization of the request fails, the plugin returns an error
   response with `action_id: "decode-request"`.
4. The host reads `response.success` to decide how to render the result.

### Host-side error rendering

Hosts check `response.success` and render accordingly. The `plugin-runtime`
crate's `render_response` function handles both success and error responses.
Error responses are displayed with their title, summary, and any attached
outputs — just like success responses but marked as failures.

---

## Warnings (non-fatal issues)

Both `InvocationContext` and `PluginResponse` carry a `warnings: Vec<String>`
field. Use warnings for non-fatal issues that should be surfaced but don't
prevent the plugin from returning a result:

```rust
let mut response = PluginResponse::ok(/* ... */);

if request.context.timeout_ms.is_some_and(|t| t < 100) {
    response = response.with_warning("Timeout budget is very low; results may be incomplete.");
}
```

The host merges context warnings into the response during `finalize_response`
in `host-core`, so plugin authors don't need to copy them manually.

---

## Best practices

1. **Never panic** — always return `Result` or an error `PluginResponse`
2. **Be specific** — include the field name, expected type, or action id in
   error messages
3. **Use `Err(String)`** for simple cases — the macro handles the conversion
4. **Use `PluginResponse::error`** when you need to attach diagnostic outputs
5. **Validate early** — check required fields before doing any work
6. **Use warnings** for degraded-but-functional results
