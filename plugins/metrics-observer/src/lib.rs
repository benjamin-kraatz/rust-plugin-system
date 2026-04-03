use plugin_sdk::plugin_manifest::{
    Capability, CapabilityContract, CapabilityRequirement, CompatibilityContract, DegradationRule,
    DegradationSeverity, ExecutionContract, ExecutionMode, HostKind, LifecycleContract,
    LifecycleHook, LifecycleState, NetworkAccess, PluginAction, PluginArchitecture, PluginManifest,
    SandboxLevel, SkillLevel, TestedHost, TrustLevel, TrustMetadata, VersionRange, VersionStrategy,
};
use plugin_sdk::plugin_protocol::{OutputKind, PluginRequest, PluginResponse};
use plugin_sdk::{JsonPlugin, export_plugin};
use serde_json::{Value, json};

pub struct MetricsObserverPlugin;

impl JsonPlugin for MetricsObserverPlugin {
    fn manifest() -> PluginManifest {
        PluginManifest::new(
            "metrics-observer",
            "Metrics Observer",
            "0.1.0",
            "Builds deterministic metric rollups and SLO budget summaries for host surfaces.",
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
                "metric-rollups",
                "Summarizes request volume, latency, and saturation into host-friendly outputs.",
            ),
            Capability::new(
                "slo-budget-evaluation",
                "Calculates a deterministic error-budget status for service reviews.",
            ),
        ])
        .with_tags(["observability", "metrics", "slo", "operations"])
        .with_actions(vec![
            PluginAction::new(
                "summarize-signals",
                "Summarize signals",
                "Create a compact request, latency, and saturation summary.",
            )
            .with_payload_hint(
                r#"{"service":"checkout","window_minutes":15,"samples":{"requests":1800,"errors":18,"p95_ms":245,"saturation_pct":61}}"#,
            ),
            PluginAction::new(
                "evaluate-slo",
                "Evaluate SLO",
                "Estimate remaining error budget for a request window.",
            )
            .with_payload_hint(
                r#"{"service":"checkout","objective_pct":99.5,"window_requests":1800,"window_errors":18}"#,
            ),
        ])
        .with_notes([
            "Designed for demos where hosts need structured observability output without touching real metrics backends.",
            "Outputs stay deterministic so they are safe for CLI smoke tests and web previews.",
        ])
        .with_compatibility(
            CompatibilityContract::new(VersionStrategy::Semver)
                .with_protocol_version("0.1.0")
                .with_host_version(
                    VersionRange::new()
                        .with_minimum("0.1.0")
                        .with_maximum("0.3.0"),
                )
                .with_tested_hosts(vec![
                    TestedHost::new(HostKind::Cli, "0.1.0"),
                    TestedHost::new(HostKind::Web, "0.1.0"),
                    TestedHost::new(HostKind::Service, "0.1.0"),
                ])
                .with_notes([
                    "The plugin follows additive response evolution and keeps deterministic output shapes stable.",
                ]),
        )
        .with_trust(
            TrustMetadata::new(TrustLevel::Reviewed, SandboxLevel::HostMediated, NetworkAccess::None)
                .with_data_access(["request-payload-only"])
                .with_provenance("bundled-first-party")
                .with_notes([
                    "No network or wall-clock access is required.",
                    "Hosts can safely replay requests for tests and previews.",
                ]),
        )
        .with_lifecycle(
            LifecycleContract::new(LifecycleState::Ready)
                .with_hooks(vec![
                    LifecycleHook::Load,
                    LifecycleHook::Invoke,
                    LifecycleHook::HealthCheck,
                ])
                .with_health_probe("re-run summarize-signals with a canned payload")
                .with_notes(["All state is derived from the invocation payload."]),
        )
        .with_execution(
            ExecutionContract::new(ExecutionMode::Sync)
                .with_async_support(true)
                .with_cancellable(true)
                .with_idempotent(true)
                .with_timeout_ms(2_000)
                .with_max_concurrency(8)
                .with_notes([
                    "Async hosts may schedule larger rollups as background jobs without changing the response shape.",
                ]),
        )
        .with_capability_contract(
            CapabilityContract::new()
                .with_required(vec![CapabilityRequirement::new(
                    "stdout-json",
                    "Hosts must be able to render or forward structured metric snapshots.",
                )])
                .with_optional(vec![CapabilityRequirement::new(
                    "markdown-output",
                    "Operator-facing summaries render best as markdown.",
                )
                .with_fallback("Hosts can fall back to the JSON block if markdown rendering is unavailable.")])
                .with_degradation(vec![DegradationRule::new(
                    "operator-summary",
                    "If markdown-output is unavailable the plugin still returns structured JSON only.",
                    DegradationSeverity::Low,
                )
                .when_missing(["markdown-output"])])
                .with_notes([
                    "Capability negotiation is intentionally host-local and deterministic.",
                ]),
        )
    }

    fn invoke(request: PluginRequest) -> Result<PluginResponse, String> {
        match request.action_id.as_str() {
            "summarize-signals" => summarize_signals(request),
            "evaluate-slo" => evaluate_slo(request),
            other => Err(format!("unknown action '{other}'")),
        }
    }
}

fn summarize_signals(request: PluginRequest) -> Result<PluginResponse, String> {
    let service = string_field(&request.payload, "service", "checkout");
    let window_minutes = u64_field(&request.payload, "window_minutes", 15).max(1);
    let samples = request
        .payload
        .get("samples")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    let requests = object_u64(&samples, "requests", 1800);
    let errors = object_u64(&samples, "errors", 18).min(requests);
    let p95_ms = object_u64(&samples, "p95_ms", 245);
    let saturation_pct = object_u64(&samples, "saturation_pct", 61).min(100);
    let throughput_rpm = requests as f64 / window_minutes as f64;
    let error_rate_pct = percentage(errors, requests);
    let health = health_band(error_rate_pct, p95_ms, saturation_pct);

    let summary_json = json!({
        "service": service,
        "window_minutes": window_minutes,
        "requests": requests,
        "errors": errors,
        "error_rate_pct": round2(error_rate_pct),
        "p95_ms": p95_ms,
        "saturation_pct": saturation_pct,
        "throughput_rpm": round2(throughput_rpm),
        "health": health,
        "host": format!("{:?}", request.context.host),
    });

    let summary_markdown = format!(
        "### {service} signal summary\n- Health: **{health}**\n- Request volume: **{requests}** in {window_minutes} min (~{:.2} rpm)\n- Error rate: **{:.2}%** ({errors} errors)\n- Latency: **p95 {p95_ms} ms**\n- Saturation: **{saturation_pct}%**",
        throughput_rpm, error_rate_pct
    );

    let pretty_json =
        serde_json::to_string_pretty(&summary_json).map_err(|error| error.to_string())?;

    Ok(PluginResponse::ok(
        "metrics-observer",
        "summarize-signals",
        "Metrics snapshot ready",
        format!(
            "Summarized {service} metrics for the {:?} host with a {health} health band.",
            request.context.host
        ),
    )
    .with_output(OutputKind::Json, "Metric snapshot", pretty_json)
    .with_output(OutputKind::Markdown, "Operator summary", summary_markdown)
    .with_next_step("Compare this rollup with logger output to align logs, metrics, and alerts."))
}

fn evaluate_slo(request: PluginRequest) -> Result<PluginResponse, String> {
    let service = string_field(&request.payload, "service", "checkout");
    let objective_pct = f64_field(&request.payload, "objective_pct", 99.5).clamp(0.0, 100.0);
    let window_requests = u64_field(&request.payload, "window_requests", 1800);
    let window_errors = u64_field(&request.payload, "window_errors", 18).min(window_requests);

    let achieved_pct = if window_requests == 0 {
        100.0
    } else {
        100.0 - percentage(window_errors, window_requests)
    };
    let allowed_errors =
        ((window_requests as f64) * ((100.0 - objective_pct) / 100.0)).floor() as u64;
    let remaining_errors = allowed_errors.saturating_sub(window_errors);
    let budget_status = if window_requests == 0 || window_errors <= allowed_errors / 2 {
        "healthy"
    } else if window_errors <= allowed_errors {
        "watch"
    } else {
        "breached"
    };

    let budget_json = json!({
        "service": service,
        "objective_pct": round2(objective_pct),
        "achieved_pct": round2(achieved_pct),
        "window_requests": window_requests,
        "window_errors": window_errors,
        "allowed_errors": allowed_errors,
        "remaining_errors": remaining_errors,
        "budget_status": budget_status,
    });

    let pretty_json =
        serde_json::to_string_pretty(&budget_json).map_err(|error| error.to_string())?;
    let message = format!(
        "{service} achieved {:.2}% against a {:.2}% objective; {} error budget slots remain.",
        achieved_pct, objective_pct, remaining_errors
    );

    Ok(PluginResponse::ok(
        "metrics-observer",
        "evaluate-slo",
        "SLO budget evaluated",
        message.clone(),
    )
    .with_output(OutputKind::Json, "SLO budget", pretty_json)
    .with_output(
        OutputKind::Text,
        "Budget verdict",
        format!("status={budget_status} allowed_errors={allowed_errors} remaining_errors={remaining_errors}"),
    )
    .with_next_step("Feed the same request counts into service-hooks to preview incident or rollout automation."))
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

fn f64_field(payload: &Value, key: &str, default: f64) -> f64 {
    payload.get(key).and_then(Value::as_f64).unwrap_or(default)
}

fn object_u64(map: &serde_json::Map<String, Value>, key: &str, default: u64) -> u64 {
    map.get(key).and_then(Value::as_u64).unwrap_or(default)
}

fn percentage(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        (numerator as f64 / denominator as f64) * 100.0
    }
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn health_band(error_rate_pct: f64, p95_ms: u64, saturation_pct: u64) -> &'static str {
    if error_rate_pct >= 5.0 || p95_ms >= 1_000 || saturation_pct >= 90 {
        "critical"
    } else if error_rate_pct >= 2.0 || p95_ms >= 400 || saturation_pct >= 75 {
        "watch"
    } else {
        "healthy"
    }
}

export_plugin!(MetricsObserverPlugin);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_band_prefers_healthy_values() {
        assert_eq!(health_band(0.5, 220, 45), "healthy");
    }

    #[test]
    fn health_band_flags_critical_values() {
        assert_eq!(health_band(6.0, 220, 45), "critical");
    }
}
