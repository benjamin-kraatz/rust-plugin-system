use plugin_sdk::plugin_manifest::{
    Capability, HostKind, PluginAction, PluginArchitecture, PluginManifest, SkillLevel,
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
        ])
        .with_tags(["service-hooks", "webhooks", "operations", "integration"])
        .with_actions(vec![
            PluginAction::new(
                "plan-hook-set",
                "Plan hook set",
                "Generate deterministic hook endpoints, retries, and ownership hints.",
            )
            .with_payload_hint(
                r#"{"service":"billing","environment":"staging","events":["deploy.succeeded","incident.opened"],"target_base_url":"https://hooks.internal.example"}"#,
            ),
            PluginAction::new(
                "preview-delivery",
                "Preview delivery",
                "Show a webhook request preview with headers, payload, and retry timing.",
            )
            .with_payload_hint(
                r#"{"service":"billing","event":"deploy.succeeded","attempt":2,"target_base_url":"https://hooks.internal.example"}"#,
            ),
        ])
        .with_notes([
            "Useful for CLI demos, service orchestration smoke tests, and web panels that need structured hook metadata.",
            "Every action is read-only and emits preview data only.",
        ])
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

    #[test]
    fn event_slug_normalizes_punctuation() {
        assert_eq!(event_slug("deploy.succeeded"), "deploy-succeeded");
    }

    #[test]
    fn later_attempts_escalate() {
        assert_eq!(delivery_status(3), "escalate");
    }
}
