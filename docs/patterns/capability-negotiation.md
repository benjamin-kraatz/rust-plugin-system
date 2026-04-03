# Capability Negotiation

Capability negotiation lets plugins declare what they need from a host and lets
hosts decide whether they can run a plugin fully, in a degraded mode, or not at
all. This guide explains the structs, the negotiation flow, and how to use them.

---

## Why capabilities matter

Not every host supports every feature. A CLI host can render markdown and code
blocks; a minimal service host may only support raw JSON. Capabilities let
plugins declare these requirements up front so hosts can:

- **Accept** — all requirements met, full functionality
- **Degrade** — optional capabilities missing, reduced functionality
- **Reject** — required capabilities missing, plugin cannot run

This avoids runtime surprises and lets hosts display meaningful compatibility
information before the user invokes a plugin.

---

## Declaring capabilities in the manifest

### The `CapabilityContract` struct

Plugins declare their capability needs in `PluginManifest.capability_contract`:

```rust
pub struct CapabilityContract {
    pub required: Vec<CapabilityRequirement>,   // must be present
    pub optional: Vec<CapabilityRequirement>,   // nice to have
    pub constraints: Option<CapabilityConstraints>, // sandbox/network limits
    pub degradation: Vec<DegradationRule>,       // what happens when optional caps are missing
    pub notes: Vec<String>,
}
```

### The `CapabilityRequirement` struct

Each requirement names a capability key and explains why it is needed:

```rust
pub struct CapabilityRequirement {
    pub key: String,        // e.g. "stdout-json", "code-output"
    pub detail: String,     // human-readable explanation
    pub fallback: Option<String>,  // what to do if unavailable (optional caps)
}
```

### The `DegradationRule` struct

Degradation rules describe how the plugin behaves when specific capabilities
are missing:

```rust
pub struct DegradationRule {
    pub feature: String,           // name of the degraded feature
    pub behavior: String,          // description of degraded behaviour
    pub severity: DegradationSeverity,  // Low, Medium, High, Critical
    pub when_missing: Vec<String>, // capability keys that trigger this rule
}
```

### Example: `abi-stable-greeter`

```rust
.with_capability_contract(
    CapabilityContract::new()
        .with_required(vec![
            CapabilityRequirement::new(
                "stdout-json",
                "Hosts should be able to surface structured compatibility envelopes.",
            ),
        ])
        .with_optional(vec![
            CapabilityRequirement::new(
                "code-output",
                "Command snippets and ABI notes read best in code blocks.",
            )
            .with_fallback(
                "Hosts can fall back to plain text guidance when code blocks are unavailable.",
            ),
        ])
        .with_degradation(vec![
            DegradationRule::new(
                "formatted-rollout-guidance",
                "Without code-output the host should render upgrade guidance as plain text.",
                DegradationSeverity::Low,
            )
            .when_missing(["code-output"]),
        ]),
)
```

This tells the host: *I require `stdout-json`; I would like `code-output` but
can fall back to plain text with low-severity degradation.*

---

## Host-side: advertising available capabilities

Hosts declare what they support through `RuntimeContext.available_capabilities`,
which is a list of `CapabilityAvailability` values:

```rust
pub struct CapabilityAvailability {
    pub key: String,
    pub available: bool,
    pub detail: Option<String>,
}
```

The `host-core` crate builds this list in `default_host_capabilities` based on
the `HostKind`. For example, a CLI host might advertise:

```rust
vec![
    CapabilityAvailability::available("stdout-json", "CLI can render JSON"),
    CapabilityAvailability::available("code-output", "CLI supports code blocks"),
    CapabilityAvailability::available("markdown-output", "CLI renders markdown"),
]
```

---

## The negotiation flow

Negotiation happens in `host-core::assess_host_fit` and follows this sequence:

```
Plugin manifest                     Host runtime context
       │                                    │
       │  CapabilityContract                │  available_capabilities
       │  (required + optional + rules)     │  (Vec<CapabilityAvailability>)
       │                                    │
       └────────────┐          ┌────────────┘
                    ▼          ▼
              negotiate_capabilities()
                    │
                    ▼
            NegotiationOutcome
            ├── status: Ready / Degraded / Rejected
            ├── granted_capabilities
            ├── missing_required
            ├── missing_optional
            └── degraded_features
```

### Step 1 — Collect available capabilities

```rust
let available_capabilities = context.runtime
    .as_ref()
    .map(|runtime| runtime.available_capabilities.iter()
        .filter(|cap| cap.available)
        .map(|cap| cap.key.as_str())
        .collect::<HashSet<_>>())
    .unwrap_or_default();
```

### Step 2 — Classify requirements

The negotiator checks each requirement against the available set:

- **Granted** — both required and optional capabilities that are present
- **Missing required** — required capabilities the host lacks → **Rejected**
- **Missing optional** — optional capabilities the host lacks → may **Degrade**

### Step 3 — Evaluate degradation rules

Each `DegradationRule` fires when **all** of its `when_missing` capabilities
are absent. Fired rules produce `DegradedFeature` entries in the outcome:

```rust
pub struct DegradedFeature {
    pub feature: String,
    pub reason: String,
    pub severity: DegradationSeverity,
    pub fallback: Option<String>,
}
```

### Step 4 — Determine status

```rust
if !host_supported || !version_supported || !missing_required.is_empty() {
    NegotiationStatus::Rejected
} else if !missing_optional.is_empty() || !degraded_features.is_empty() {
    NegotiationStatus::Degraded
} else {
    NegotiationStatus::Ready
}
```

### Step 5 — Attach to response

The `finalize_response` function in `host-core` attaches the
`NegotiationOutcome` to every `PluginResponse`, so both the host UI and the
plugin output carry negotiation results.

---

## The `HostFitAssessment`

The top-level assessment combines host support, version compatibility, and
capability negotiation:

```rust
pub struct HostFitAssessment {
    pub status: HostFitStatus,    // Ready, Degraded, or Rejected
    pub summary: String,          // human-readable explanation
    pub version_summary: Option<String>,
    pub negotiation: NegotiationOutcome,
}
```

Hosts use `assess_host_fit` before displaying a plugin in their UI:

```rust
let assessment = assess_host_fit(&manifest, &context);
match assessment.status {
    HostFitStatus::Ready    => /* show as fully supported */,
    HostFitStatus::Degraded => /* show with warnings */,
    HostFitStatus::Rejected => /* hide or show as unsupported */,
}
```

---

## Best practices

1. **Keep required capabilities minimal** — only mark a capability as required
   if the plugin truly cannot produce useful output without it.
2. **Provide fallbacks** — set `fallback` on optional requirements so hosts
   know what the user experience will be in degraded mode.
3. **Use degradation rules** — make the trade-off explicit. Name the feature
   that degrades, describe the behaviour change, and set an honest severity.
4. **Test with different hosts** — run `cargo run -p host-cli -- inspect <plugin>`
   to see the negotiation outcome, then try a GUI host or service host.
5. **Declare constraints** — if your plugin needs network access or specific
   sandbox levels, use `CapabilityConstraints` to make this visible.
