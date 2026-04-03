use plugin_capabilities::{RetryBackoffStrategy, RetryPolicy};
use plugin_sdk::plugin_manifest::{
    ActionContract, AsyncMetadata, Capability, CapabilityConstraints, CapabilityContract,
    CapabilityRequirement, CompatibilityContract, DegradationRule, DegradationSeverity,
    ExecutionContract, ExecutionMode, HostKind, LifecycleContract, LifecycleHook, LifecycleState,
    MaintenanceContract, MaintenanceStatus, NetworkAccess, PluginAction, PluginArchitecture,
    PluginManifest, SandboxLevel, SchemaDescriptor, SkillLevel, TestedHost, TrustLevel,
    TrustMetadata, VersionRange, VersionStrategy,
};
use plugin_sdk::plugin_protocol::{OutputKind, PluginRequest, PluginResponse};
use plugin_sdk::{JsonPlugin, export_plugin};
use serde_json::{Value, json};

pub struct ServiceHooksPlugin;

impl JsonPlugin for ServiceHooksPlugin {
    fn manifest() -> PluginManifest {
        PluginManifest::new(
            "service-hooks",
            "Service Hooks",
            "0.1.0",
            "Plans deterministic service-hook rollouts and webhook delivery previews.",
            PluginArchitecture::NativeJson,
            SkillLevel::Intermediate,
        )
        .with_supported_hosts(vec![
            HostKind::Cli,
            HostKind::Tui,
            HostKind::Web,
            HostKind::Service,
        ])
        .with_capabilities(vec![
            Capability::new(
                "hook-rollout-planning",
                "Builds a predictable service-hook configuration plan from event inputs.",
            ),
            Capability::new(
                "delivery-preview",
                "Renders a safe webhook delivery preview without making any network calls.",
            ),
            Capability::new(
                "rollout-negotiation",
                "Publishes lifecycle and host-capability expectations for rollout tooling.",
            ),
        ])
        .with_tags(["service-hooks", "webhooks", "operations", "integration", "advanced"])
        .with_actions(vec![
            PluginAction::new(
                "plan-hook-set",
                "Plan hook set",
                "Generate deterministic hook endpoints, retries, and ownership hints.",
            )
            .with_payload_hint(
                r#"{"service":"billing","environment":"staging","events":["deploy.succeeded","incident.opened"],"target_base_url":"https://hooks.internal.example"}"#,
            )
            .with_contract(
                ActionContract::new(ExecutionMode::Sync)
                    .with_timeout_ms(1_500)
                    .with_constraints(
                        CapabilityConstraints::new()
                            .with_required([CapabilityRequirement::new(
                                "stdout-json",
                                "Hosts must surface the generated rollout plan.",
                            )])
                            .with_optional([CapabilityRequirement::new(
                                "markdown-output",
                                "Rollout checklists render best as markdown.",
                            )])
                            .with_max_payload_bytes(16_384)
                            .with_network_access(NetworkAccess::None)
                            .with_sandbox_level(SandboxLevel::HostMediated),
                    ),
            )
            .with_input_schema(
                SchemaDescriptor::new(
                    "json-schema",
                    "docs/schemas/service-hooks/plan-hook-set-input.schema.json",
                )
                .with_version("1.0.0"),
            )
            .with_output_schema(
                SchemaDescriptor::new(
                    "json-schema",
                    "docs/schemas/service-hooks/plan-hook-set-output.schema.json",
                )
                .with_version("1.0.0"),
            ),
            PluginAction::new(
                "preview-delivery",
                "Preview delivery",
                "Show a webhook request preview with headers, payload, and retry timing.",
            )
            .with_payload_hint(
                r#"{"service":"billing","event":"deploy.succeeded","attempt":2,"target_base_url":"https://hooks.internal.example"}"#,
            )
            .with_contract(
                ActionContract::new(ExecutionMode::Async)
                    .with_timeout_ms(2_500)
                    .with_async_metadata(
                        AsyncMetadata::asynchronous()
                            .with_streaming(true)
                            .with_completion_timeout_ms(20_000)
                            .with_retry_policy(
                                RetryPolicy::new(3)
                                    .with_initial_backoff_ms(500)
                                    .with_max_backoff_ms(5_000)
                                    .with_strategy(RetryBackoffStrategy::Exponential)
                                    .with_retry_on(["delivery-preview-timeout", "host-retry"]),
                            ),
                    )
                    .with_constraints(
                        CapabilityConstraints::new()
                            .with_required([CapabilityRequirement::new(
                                "stdout-json",
                                "Hosts must surface the delivery envelope JSON.",
                            )])
                            .with_optional([
                                CapabilityRequirement::new(
                                    "code-output",
                                    "HTTP request previews render best as verbatim code blocks.",
                                )
                                .with_fallback(
                                    "Hosts can fall back to JSON-only previews when code blocks are unavailable.",
                                ),
                                CapabilityRequirement::new(
                                    "async-jobs",
                                    "Async-capable hosts can treat preview generation as a background job.",
                                )
                                .with_fallback(
                                    "Foreground execution is still valid because the preview is deterministic.",
                                ),
                            ])
                            .with_max_payload_bytes(8_192)
                            .with_network_access(NetworkAccess::None)
                            .with_sandbox_level(SandboxLevel::HostMediated),
                    ),
            )
            .with_input_schema(
                SchemaDescriptor::new(
                    "json-schema",
                    "docs/schemas/service-hooks/preview-delivery-input.schema.json",
                )
                .with_version("1.0.0"),
            )
            .with_output_schema(
                SchemaDescriptor::new(
                    "json-schema",
                    "docs/schemas/service-hooks/preview-delivery-output.schema.json",
                )
                .with_version("1.0.0"),
            ),
        ])
        .with_notes([
            "Useful for CLI demos, service orchestration smoke tests, and web panels that need structured hook metadata.",
            "Every action is read-only and emits preview data only.",
            "Phase 4 uses this plugin to demonstrate lifecycle, async metadata, trust signaling, and capability degradation in one runnable flow.",
        ])
        .with_maintenance(
            MaintenanceContract::new(MaintenanceStatus::Active)
                .with_owner("platform-integrations")
                .with_support_tier("course-demo")
                .with_channel("stable"),
        )
        .with_compatibility(
            CompatibilityContract::new(VersionStrategy::Semver)
                .with_protocol_version("0.1.0")
                .with_host_version(
                    VersionRange::new()
                        .with_minimum("0.1.0")
                        .with_maximum("0.3.0"),
                )
                .with_tested_hosts(vec![
                    TestedHost::new(HostKind::Cli, "0.1.0")
                        .with_notes("Foreground rollout planning is the default operator workflow."),
                    TestedHost::new(HostKind::Web, "0.1.0")
                        .with_notes("Browser hosts can render preview cards with async orchestration."),
                    TestedHost::new(HostKind::Service, "0.1.0")
                        .with_notes("Service hosts can reuse the JSON envelopes for health and rollout APIs."),
                ])
                .with_notes([
                    "The payload and output shapes are additive so rollout tooling can tolerate new metadata fields.",
                    "Retry-preview semantics stay stable across host releases within the declared window.",
                ]),
        )
        .with_trust(
            TrustMetadata::new(
                TrustLevel::Reviewed,
                SandboxLevel::HostMediated,
                NetworkAccess::None,
            )
            .with_data_access(["request-payload-only", "host-generated rollout context"])
            .with_provenance("bundled-first-party")
            .with_notes([
                "The plugin does not contact live webhook endpoints.",
                "Signature and retry artifacts are synthetic so hosts can replay them safely.",
            ]),
        )
        .with_lifecycle(
            LifecycleContract::new(LifecycleState::Ready)
                .with_hooks(vec![
                    LifecycleHook::Load,
                    LifecycleHook::Invoke,
                    LifecycleHook::HealthCheck,
                    LifecycleHook::Shutdown,
                ])
                .with_health_probe(
                    "Run preview-delivery with a canned deploy.succeeded payload and verify the generated retry envelope.",
                )
                .with_notes([
                    "The demo remains stateless, but hosts can still wire lifecycle hooks into rollout supervisors.",
                    "Shutdown hooks are metadata-only in this sample and exist to show production cleanup affordances.",
                ]),
        )
        .with_execution(
            ExecutionContract::new(ExecutionMode::Sync)
                .with_async_support(true)
                .with_cancellable(true)
                .with_idempotent(true)
                .with_progress_reporting(true)
                .with_timeout_ms(3_000)
                .with_max_concurrency(6)
                .with_async_metadata(
                    AsyncMetadata::asynchronous()
                        .with_streaming(true)
                        .with_completion_timeout_ms(20_000)
                        .with_retry_policy(
                            RetryPolicy::new(3)
                                .with_initial_backoff_ms(500)
                                .with_max_backoff_ms(5_000)
                                .with_strategy(RetryBackoffStrategy::Exponential)
                                .with_retry_on(["delivery-preview-timeout", "host-retry"]),
                        ),
                )
                .with_notes([
                    "Hosts may degrade async preview generation to foreground execution without changing output semantics.",
                    "Progress updates are optional metadata for hosts that visualize rollout preparation stages.",
                ]),
        )
        .with_capability_contract(
            CapabilityContract::new()
                .with_required(vec![CapabilityRequirement::new(
                    "stdout-json",
                    "Hosts must render or forward the structured rollout and delivery metadata.",
                )])
                .with_optional(vec![
                    CapabilityRequirement::new(
                        "markdown-output",
                        "Rollout checklists read best as markdown summaries.",
                    )
                    .with_fallback(
                        "The JSON plan remains authoritative when markdown rendering is unavailable.",
                    ),
                    CapabilityRequirement::new(
                        "code-output",
                        "HTTP previews read best in code blocks.",
                    )
                    .with_fallback(
                        "Hosts can fall back to JSON-only request envelopes.",
                    ),
                    CapabilityRequirement::new(
                        "async-jobs",
                        "Async-capable hosts can queue delivery previews as background work.",
                    )
                    .with_fallback(
                        "Foreground execution stays safe because previews are deterministic and side-effect free.",
                    ),
                    CapabilityRequirement::new(
                        "health-hooks",
                        "Service-style hosts can wire the health probe into operational checks.",
                    )
                    .with_fallback(
                        "Other hosts can document the health probe without executing it automatically.",
                    ),
                ])
                .with_degradation(vec![
                    DegradationRule::new(
                        "rollout-checklists",
                        "If markdown-output is unavailable the plugin returns JSON only and omits checklist formatting.",
                        DegradationSeverity::Low,
                    )
                    .when_missing(["markdown-output"]),
                    DegradationRule::new(
                        "http-preview-rendering",
                        "If code-output is unavailable the host should surface the JSON delivery envelope instead of verbatim HTTP previews.",
                        DegradationSeverity::Medium,
                    )
                    .when_missing(["code-output"]),
                    DegradationRule::new(
                        "background-preview-jobs",
                        "If async-jobs is unavailable the host should run preview-delivery in the foreground.",
                        DegradationSeverity::Low,
                    )
                    .when_missing(["async-jobs"]),
                ])
                .with_notes([
                    "Capability negotiation is intentionally declarative so host-cli inspect output stays readable.",
                ]),
        )
    }

    fn invoke(request: PluginRequest) -> Result<PluginResponse, String> {
        match request.action_id.as_str() {
            "plan-hook-set" => plan_hook_set(request),
            "preview-delivery" => preview_delivery(request),
            other => Err(format!("unknown action '{other}'")),
        }
    }
}

fn plan_hook_set(request: PluginRequest) -> Result<PluginResponse, String> {
    let service = string_field(&request.payload, "service", "billing");
    let environment = string_field(&request.payload, "environment", "staging");
    let target_base_url = string_field(
        &request.payload,
        "target_base_url",
        "https://hooks.internal.example",
    );
    let retry_limit = u64_field(&request.payload, "retry_limit", 3).clamp(1, 6);
    let events = list_of_strings(&request.payload, "events", &["deploy.succeeded".to_owned()]);

    let hooks = events
        .iter()
        .map(|event| {
            let slug = event_slug(event);
            json!({
                "event": event,
                "endpoint": format!("{target_base_url}/v1/{service}/{environment}/{slug}"),
                "method": "POST",
                "timeout_ms": 2_500,
                "retry_limit": retry_limit,
                "idempotency_key": format!("{service}:{environment}:{slug}"),
                "owner": format!("{service}-ops"),
            })
        })
        .collect::<Vec<_>>();

    let plan_json = json!({
        "service": service,
        "environment": environment,
        "hook_count": hooks.len(),
        "hooks": hooks,
    });
    let pretty_json =
        serde_json::to_string_pretty(&plan_json).map_err(|error| error.to_string())?;

    let checklist = format!(
        "### Hook rollout plan\n- Service: **{service}**\n- Environment: **{environment}**\n- Hooks planned: **{}**\n- Retry limit: **{retry_limit}**\n- Base URL: `{target_base_url}`",
        events.len()
    );

    Ok(PluginResponse::ok(
        "service-hooks",
        "plan-hook-set",
        "Hook plan ready",
        format!(
            "Prepared {} deterministic service hooks for {service} in {environment}.",
            events.len()
        ),
    )
    .with_output(OutputKind::Json, "Hook plan", pretty_json)
    .with_output(OutputKind::Markdown, "Rollout checklist", checklist)
    .with_next_step("Use preview-delivery with one of the planned events to inspect payload shape and retry timing."))
}

fn preview_delivery(request: PluginRequest) -> Result<PluginResponse, String> {
    let service = string_field(&request.payload, "service", "billing");
    let event = string_field(&request.payload, "event", "deploy.succeeded");
    let target_base_url = string_field(
        &request.payload,
        "target_base_url",
        "https://hooks.internal.example",
    );
    let attempt = u64_field(&request.payload, "attempt", 1).max(1);
    let environment = string_field(&request.payload, "environment", "staging");
    let slug = event_slug(&event);
    let backoff_seconds = attempt.saturating_sub(1) * 30;
    let request_id = format!("{service}-{environment}-{slug}-{:02}", attempt);
    let endpoint = format!("{target_base_url}/v1/{service}/{environment}/{slug}");

    let preview_json = json!({
        "endpoint": endpoint,
        "method": "POST",
        "attempt": attempt,
        "backoff_seconds": backoff_seconds,
        "headers": {
            "content-type": "application/json",
            "x-hook-event": event,
            "x-hook-request-id": request_id,
            "x-hook-signature": format!("demo-{service}-{slug}-{:02}", attempt),
        },
        "body": {
            "service": service,
            "environment": environment,
            "event": event,
            "status": delivery_status(attempt),
            "attempt": attempt,
        }
    });
    let pretty_json =
        serde_json::to_string_pretty(&preview_json).map_err(|error| error.to_string())?;

    let http_preview = format!(
        "POST {endpoint}\ncontent-type: application/json\nx-hook-event: {event}\nx-hook-request-id: {request_id}\nx-hook-signature: demo-{service}-{slug}-{attempt:02}\n\n{{\"service\":\"{service}\",\"environment\":\"{environment}\",\"event\":\"{event}\",\"status\":\"{}\",\"attempt\":{attempt}}}",
        delivery_status(attempt)
    );

    Ok(PluginResponse::ok(
        "service-hooks",
        "preview-delivery",
        "Delivery preview ready",
        format!(
            "Prepared attempt {attempt} for {service}::{event} with {backoff_seconds}s backoff.",
        ),
    )
    .with_output(OutputKind::Json, "Delivery envelope", pretty_json)
    .with_output(OutputKind::Code, "HTTP preview", http_preview)
    .with_next_step("Pair this preview with metrics-observer to model alert-triggered hooks and rollout notifications."))
}

fn string_field(payload: &Value, key: &str, default: &str) -> String {
    payload
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or(default)
        .to_owned()
}

fn u64_field(payload: &Value, key: &str, default: u64) -> u64 {
    payload.get(key).and_then(Value::as_u64).unwrap_or(default)
}

fn list_of_strings(payload: &Value, key: &str, default: &[String]) -> Vec<String> {
    payload
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|value| value.as_str().map(str::to_owned))
                .collect::<Vec<_>>()
        })
        .filter(|items| !items.is_empty())
        .unwrap_or_else(|| default.to_vec())
}

fn event_slug(event: &str) -> String {
    event
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn delivery_status(attempt: u64) -> &'static str {
    if attempt >= 3 { "escalate" } else { "deliver" }
}

export_plugin!(ServiceHooksPlugin);

#[cfg(test)]
mod tests {
    use super::*;
    use plugin_sdk::plugin_protocol::InvocationContext;

    #[test]
    fn event_slug_normalizes_punctuation() {
        assert_eq!(event_slug("deploy.succeeded"), "deploy-succeeded");
    }

    #[test]
    fn later_attempts_escalate() {
        assert_eq!(delivery_status(3), "escalate");
    }

    #[test]
    fn manifest_exposes_phase4_contracts() {
        let manifest = ServiceHooksPlugin::manifest();
        let preview = manifest
            .actions
            .iter()
            .find(|action| action.id == "preview-delivery")
            .expect("preview action should exist");

        assert!(manifest.compatibility.is_some());
        assert!(manifest.trust.is_some());
        assert!(manifest.lifecycle.is_some());
        assert!(
            manifest
                .execution
                .as_ref()
                .is_some_and(|execution| execution.supports_async)
        );
        assert_eq!(
            preview
                .contract
                .as_ref()
                .map(|contract| contract.execution_mode),
            Some(ExecutionMode::Async)
        );
        assert!(
            preview
                .contract
                .as_ref()
                .and_then(|contract| contract.async_metadata.as_ref())
                .is_some()
        );
    }

    #[test]
    fn preview_delivery_returns_http_preview() {
        let response = ServiceHooksPlugin::invoke(PluginRequest {
            plugin_id: "service-hooks".to_owned(),
            action_id: "preview-delivery".to_owned(),
            payload: json!({
                "service": "billing",
                "event": "deploy.succeeded",
                "attempt": 2,
                "environment": "prod"
            }),
            context: InvocationContext::for_host(HostKind::Cli),
        })
        .expect("preview should succeed");

        assert!(response.outputs.iter().any(|output| {
            output.title.as_deref() == Some("HTTP preview")
                && output.body.contains(
                    "POST https://hooks.internal.example/v1/billing/prod/deploy-succeeded",
                )
        }));
    }
}
