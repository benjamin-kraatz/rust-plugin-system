//! Host-side orchestration entry point for the plugin system.
//!
//! [`Playground`] is the main API for loading native plugins and invoking their
//! actions with host-aware runtime context.

use std::{
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::{Result, anyhow};
use plugin_loader::{LoadedPlugin, PluginCatalog, load_plugins_from_directory};
use plugin_manifest::{PluginAction, PluginManifest};
use plugin_protocol::{
    CapabilityAvailability, DegradedFeature, ExecutionMetadata, HostKind, InvocationContext,
    NegotiationOutcome, NegotiationStatus, PluginRequest, PluginResponse, RuntimeContext,
};
use plugin_runtime::{PluginSummary, render_response as render_plugin_response};
use semver::Version;
use serde_json::Value;

/// Host runtime facade for loading and invoking plugins.
pub struct Playground {
    plugins: Vec<LoadedPlugin>,
    warnings: Vec<String>,
    plugin_dir: PathBuf,
}

impl Playground {
    /// Loads plugins from the default plugin directory.
    ///
    /// The default can be overridden with the
    /// `RUST_PLUGIN_SYSTEM_PLUGIN_DIR` environment variable.
    ///
    /// # Errors
    ///
    /// Returns an error when the plugin directory cannot be scanned.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use host_core::Playground;
    ///
    /// let host = Playground::load_default()?;
    /// println!("loaded {} plugin(s)", host.manifests().len());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn load_default() -> Result<Self> {
        Self::load(default_plugin_dir())
    }

    /// Loads plugins from a specific directory.
    ///
    /// # Errors
    ///
    /// Returns an error when directory scanning fails.
    pub fn load(plugin_dir: impl AsRef<Path>) -> Result<Self> {
        let plugin_dir = plugin_dir.as_ref().to_path_buf();
        let PluginCatalog { plugins, warnings } = load_plugins_from_directory(&plugin_dir)?;

        Ok(Self {
            plugins,
            warnings,
            plugin_dir,
        })
    }

    /// Returns the plugin directory used by this host instance.
    pub fn plugin_dir(&self) -> &Path {
        &self.plugin_dir
    }

    /// Returns non-fatal plugin loading warnings.
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    /// Returns cloned manifests for all loaded plugins.
    pub fn manifests(&self) -> Vec<PluginManifest> {
        self.plugins
            .iter()
            .map(|plugin| plugin.manifest().clone())
            .collect()
    }

    /// Returns presentation summaries for loaded plugins.
    pub fn summaries(&self) -> Vec<PluginSummary> {
        self.plugins
            .iter()
            .map(|plugin| PluginSummary::from(plugin.manifest()))
            .collect()
    }

    /// Invokes one plugin action with textual payload input.
    ///
    /// If `payload_text` is valid JSON it is parsed as JSON, otherwise it is
    /// passed as a string payload.
    ///
    /// # Errors
    ///
    /// Returns an error when plugin/action lookup fails or invocation fails.
    pub fn invoke_text(
        &self,
        plugin_id: &str,
        action_id: &str,
        payload_text: &str,
        host: HostKind,
    ) -> Result<PluginResponse> {
        let payload = parse_payload(payload_text);
        self.invoke(plugin_id, action_id, payload, host)
    }

    /// Invokes one plugin action with a structured JSON payload.
    ///
    /// # Errors
    ///
    /// Returns an error when plugin lookup fails or invocation fails.
    pub fn invoke(
        &self,
        plugin_id: &str,
        action_id: &str,
        payload: Value,
        host: HostKind,
    ) -> Result<PluginResponse> {
        let plugin = self
            .plugins
            .iter()
            .find(|plugin| plugin.manifest().id == plugin_id)
            .ok_or_else(|| anyhow!("no loaded plugin named '{plugin_id}'"))?;
        let action = plugin
            .manifest()
            .actions
            .iter()
            .find(|action| action.id == action_id);

        let mut request = PluginRequest {
            plugin_id: plugin_id.to_owned(),
            action_id: action_id.to_owned(),
            payload,
            context: build_invocation_context(
                host,
                std::env::current_dir().ok().as_deref(),
                Some(&self.plugin_dir),
                Some("interactive"),
                None,
            ),
        };
        request.context.request_id = Some(make_request_id(plugin_id, action_id));
        request.context.trace_id = Some(format!("{plugin_id}/{action_id}"));
        request.context.timeout_ms = action
            .and_then(|action| action.contract.as_ref())
            .and_then(|contract| contract.timeout_ms)
            .or_else(|| {
                plugin
                    .manifest()
                    .execution
                    .as_ref()
                    .and_then(|execution| execution.timeout_ms)
            });
        request.context.warnings = manifest_warnings(plugin.manifest(), action);
        if let Some(runtime) = request.context.runtime.as_mut() {
            runtime.preferred_mode = action
                .and_then(|action| action.contract.as_ref())
                .map(|contract| contract.execution_mode)
                .or(runtime.preferred_mode);
            runtime.max_timeout_ms = request.context.timeout_ms.or(runtime.max_timeout_ms);
        }

        let started = Instant::now();
        plugin.invoke(&request).map(|response| {
            let mut response = finalize_response(plugin.manifest(), &request, response);
            if let Some(execution) = response.execution.as_mut() {
                execution.duration_ms =
                    Some(started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64);
            }
            response
        })
    }
}

/// Resolves the default plugin directory.
///
/// Uses `RUST_PLUGIN_SYSTEM_PLUGIN_DIR` when present, otherwise `target/debug`.
pub fn default_plugin_dir() -> PathBuf {
    std::env::var_os("RUST_PLUGIN_SYSTEM_PLUGIN_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/debug"))
}

/// Renders a plugin response into plain text for terminal/UI display.
pub fn render_response(response: &PluginResponse) -> String {
    render_plugin_response(response)
}

/// Returns a default textual payload for an action.
///
/// If the action includes a JSON payload hint, it is pretty-printed when
/// possible.
pub fn default_payload_text(action: &PluginAction) -> String {
    action
        .payload_hint
        .as_deref()
        .map(pretty_json_or_raw)
        .unwrap_or_else(|| "{}".to_owned())
}

/// Returns whether a manifest declares support for `host`.
pub fn supports_host(manifest: &PluginManifest, host: HostKind) -> bool {
    manifest.supported_hosts.contains(&HostKind::Any) || manifest.supported_hosts.contains(&host)
}

/// Builds an invocation context suitable for host-driven plugin invocation.
pub fn build_invocation_context(
    host: HostKind,
    workspace_root: Option<&Path>,
    plugin_dir: Option<&Path>,
    mode: Option<&str>,
    host_version: Option<&str>,
) -> InvocationContext {
    InvocationContext {
        host,
        workspace_root: workspace_root
            .and_then(|path| path.to_str())
            .map(str::to_owned),
        plugin_dir: plugin_dir.and_then(|path| path.to_str()).map(str::to_owned),
        mode: mode.map(str::to_owned),
        request_id: None,
        trace_id: None,
        timeout_ms: None,
        warnings: Vec::new(),
        runtime: Some(default_runtime_context(host, host_version)),
    }
}

/// Builds default runtime context for a given host kind.
pub fn default_runtime_context(host: HostKind, host_version: Option<&str>) -> RuntimeContext {
    RuntimeContext {
        host_version: host_version.map(str::to_owned),
        available_capabilities: default_host_capabilities(host),
        preferred_mode: Some(default_execution_mode(host)),
        deterministic: true,
        local_only: true,
        max_timeout_ms: Some(30_000),
    }
}

/// Finalizes a plugin response with host-derived metadata.
///
/// This function fills negotiation results, request identifiers, inherited
/// warnings, and default execution metadata when missing.
pub fn finalize_response(
    manifest: &PluginManifest,
    request: &PluginRequest,
    mut response: PluginResponse,
) -> PluginResponse {
    let assessment = assess_host_fit(manifest, &request.context);
    if response.negotiation.is_none() {
        response.negotiation = Some(assessment.negotiation.clone());
    }
    if response.request_id.is_none() {
        response.request_id = request.context.request_id.clone();
    }
    for warning in &request.context.warnings {
        if !response.warnings.iter().any(|existing| existing == warning) {
            response.warnings.push(warning.clone());
        }
    }
    let mut execution = response
        .execution
        .take()
        .unwrap_or_else(|| default_execution_metadata(manifest, request));
    if execution.timeout_ms.is_none() {
        execution.timeout_ms = request.context.timeout_ms;
    }
    if execution.duration_ms.is_none() {
        execution.duration_ms = Some(0);
    }
    response.execution = Some(execution);
    response
}

/// Assesses whether a plugin manifest is fit for a given host context.
///
/// The result combines host-kind support, version compatibility, and capability
/// negotiation.
pub fn assess_host_fit(
    manifest: &PluginManifest,
    context: &InvocationContext,
) -> HostFitAssessment {
    let host_supported = supports_host(manifest, context.host);
    let version_assessment = compatibility_summary(manifest, context);
    let version_supported = version_assessment
        .as_ref()
        .is_none_or(|(_, version_supported)| *version_supported);
    let version_summary = version_assessment.map(|(summary, _)| summary);
    let negotiation = negotiate_capabilities(
        manifest,
        context,
        host_supported,
        version_supported,
        &version_summary,
    );
    let status = if !host_supported
        || !version_supported
        || matches!(negotiation.status, NegotiationStatus::Rejected)
    {
        HostFitStatus::Rejected
    } else if matches!(negotiation.status, NegotiationStatus::Degraded) {
        HostFitStatus::Degraded
    } else {
        HostFitStatus::Ready
    };

    let mut summary_parts = Vec::new();
    summary_parts.push(if host_supported {
        format!("host {} is supported", context.host.label())
    } else {
        format!(
            "host {} is outside the declared support set",
            context.host.label()
        )
    });
    if let Some(version_summary) = &version_summary {
        summary_parts.push(version_summary.clone());
    }
    if !negotiation.summary.is_empty() {
        summary_parts.push(negotiation.summary.clone());
    }

    HostFitAssessment {
        status,
        summary: summary_parts.join("; "),
        version_summary,
        negotiation,
    }
}

/// Coarse host fit classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostFitStatus {
    /// Host and runtime context satisfy plugin requirements.
    Ready,
    /// Invocation can proceed but with degraded behavior.
    Degraded,
    /// Invocation should be rejected.
    Rejected,
}

/// Detailed host-fit assessment result.
#[derive(Debug, Clone)]
pub struct HostFitAssessment {
    /// Top-level fit status.
    pub status: HostFitStatus,
    /// Human-readable summary.
    pub summary: String,
    /// Optional version compatibility summary.
    pub version_summary: Option<String>,
    /// Capability negotiation outcome.
    pub negotiation: NegotiationOutcome,
}

/// Summarizes high-level manifest metadata into readable lines.
pub fn summarize_manifest_metadata(manifest: &PluginManifest) -> Vec<String> {
    let mut lines = Vec::new();

    if let Some(maintenance) = &manifest.maintenance {
        let mut parts = vec![format!("maintenance {:?}", maintenance.status)];
        if let Some(owner) = &maintenance.owner {
            parts.push(format!("owner {owner}"));
        }
        if let Some(support_tier) = &maintenance.support_tier {
            parts.push(format!("tier {support_tier}"));
        }
        lines.push(parts.join(" · "));
    }

    if let Some(trust) = &manifest.trust {
        let mut parts = vec![
            format!("trust {:?}", trust.level),
            format!("sandbox {:?}", trust.sandbox),
            format!("network {:?}", trust.network),
        ];
        if !trust.permissions.is_empty() {
            parts.push(format!("{} permission(s)", trust.permissions.len()));
        }
        lines.push(parts.join(" · "));
    }

    if let Some(compatibility) = &manifest.compatibility {
        let mut parts = vec![format!("compatibility {:?}", compatibility.strategy)];
        if let Some(protocol_version) = &compatibility.protocol_version {
            parts.push(format!("protocol {protocol_version}"));
        }
        if let Some(host_version) = &compatibility.host_version {
            parts.push(format!(
                "host {}..{}",
                host_version.minimum.as_deref().unwrap_or("*"),
                host_version.maximum.as_deref().unwrap_or("*")
            ));
        }
        lines.push(parts.join(" · "));
    }

    if let Some(execution) = &manifest.execution {
        let mut parts = vec![format!("execution {:?}", execution.default_mode)];
        if execution.supports_async {
            parts.push("async".to_owned());
        }
        if let Some(timeout_ms) = execution.timeout_ms {
            parts.push(format!("timeout {timeout_ms}ms"));
        }
        lines.push(parts.join(" · "));
    }

    if let Some(capability_contract) = &manifest.capability_contract {
        let mut parts = vec![format!(
            "capabilities {} required / {} optional",
            capability_contract.required.len(),
            capability_contract.optional.len()
        )];
        if let Some(constraints) = &capability_contract.constraints {
            if !constraints.permissions.is_empty() {
                parts.push(format!(
                    "{} scoped permission(s)",
                    constraints.permissions.len()
                ));
            }
            if let Some(max_payload_bytes) = constraints.max_payload_bytes {
                parts.push(format!("max payload {max_payload_bytes} bytes"));
            }
        }
        if !capability_contract.degradation.is_empty() {
            parts.push(format!(
                "{} degradation rule(s)",
                capability_contract.degradation.len()
            ));
        }
        lines.push(parts.join(" · "));
    }

    lines
}

/// Summarizes action metadata into readable lines.
pub fn summarize_action_metadata(action: &PluginAction) -> Vec<String> {
    let mut lines = Vec::new();

    if let Some(contract) = &action.contract {
        let mut parts = vec![format!("mode {:?}", contract.execution_mode)];
        if let Some(timeout_ms) = contract.timeout_ms {
            parts.push(format!("timeout {timeout_ms}ms"));
        }
        if contract.mutates_workspace {
            parts.push("mutates workspace".to_owned());
        }
        if !contract.idempotent {
            parts.push("non-idempotent".to_owned());
        }
        lines.push(parts.join(" · "));

        if let Some(async_metadata) = &contract.async_metadata {
            let mut parts = vec!["async metadata".to_owned()];
            if async_metadata.detached {
                parts.push("detached".to_owned());
            }
            if async_metadata.supports_streaming {
                parts.push("streaming".to_owned());
            }
            if let Some(completion_timeout_ms) = async_metadata.completion_timeout_ms {
                parts.push(format!("completion timeout {completion_timeout_ms}ms"));
            }
            if let Some(retry_policy) = &async_metadata.retry_policy {
                parts.push(format!(
                    "retry {} attempt(s) via {:?}",
                    retry_policy.max_attempts, retry_policy.strategy
                ));
            }
            lines.push(parts.join(" · "));
        }
    }

    if let Some(input_schema) = &action.input_schema {
        lines.push(format!(
            "input schema {} {}",
            input_schema.format, input_schema.reference
        ));
    }
    if let Some(output_schema) = &action.output_schema {
        lines.push(format!(
            "output schema {} {}",
            output_schema.format, output_schema.reference
        ));
    }
    if let Some(deprecation) = &action.deprecation {
        lines.push(format!(
            "deprecated {}",
            deprecation
                .message
                .as_deref()
                .unwrap_or("this action is deprecated")
        ));
    }

    lines
}

/// Summarizes response metadata into readable lines.
pub fn summarize_response_metadata(response: &PluginResponse) -> Vec<String> {
    let mut lines = Vec::new();

    if let Some(request_id) = &response.request_id {
        lines.push(format!("request id: {request_id}"));
    }

    if let Some(execution) = &response.execution {
        let mut parts = Vec::new();
        if let Some(mode) = execution.mode {
            parts.push(format!("mode {mode:?}"));
        }
        if let Some(timeout_ms) = execution.timeout_ms {
            parts.push(format!("timeout {timeout_ms}ms"));
        }
        if let Some(duration_ms) = execution.duration_ms {
            parts.push(format!("duration {duration_ms}ms"));
        }
        if let Some(lifecycle_state) = execution.lifecycle_state {
            parts.push(format!("lifecycle {lifecycle_state:?}"));
        }
        if execution.supports_async {
            parts.push("async-capable".to_owned());
        }
        if !parts.is_empty() {
            lines.push(format!("execution: {}", parts.join(" · ")));
        }
    }

    if let Some(negotiation) = &response.negotiation {
        let mut parts = vec![format!("negotiation: {:?}", negotiation.status)];
        if !negotiation.summary.is_empty() {
            parts.push(negotiation.summary.clone());
        }
        lines.push(parts.join(" · "));
    }

    if !response.warnings.is_empty() {
        lines.push(format!("warnings: {}", response.warnings.join(" | ")));
    }

    lines
}

fn parse_payload(payload_text: &str) -> Value {
    let trimmed = payload_text.trim();
    if trimmed.is_empty() {
        Value::Null
    } else {
        serde_json::from_str(trimmed).unwrap_or_else(|_| Value::String(payload_text.to_owned()))
    }
}

fn make_request_id(plugin_id: &str, action_id: &str) -> String {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    format!("{plugin_id}:{action_id}:{millis}")
}

fn manifest_warnings(manifest: &PluginManifest, action: Option<&PluginAction>) -> Vec<String> {
    let mut warnings = Vec::new();

    if let Some(maintenance) = &manifest.maintenance {
        match maintenance.status {
            plugin_manifest::MaintenanceStatus::Experimental => warnings.push(format!(
                "{} is marked experimental and may change without notice.",
                manifest.name
            )),
            plugin_manifest::MaintenanceStatus::MaintenanceOnly => {
                warnings.push(format!("{} is in maintenance-only mode.", manifest.name))
            }
            plugin_manifest::MaintenanceStatus::Deprecated => {
                warnings.push(format!("{} is deprecated.", manifest.name))
            }
            plugin_manifest::MaintenanceStatus::Retired => warnings.push(format!(
                "{} is retired and should only be used for archival or migration scenarios.",
                manifest.name
            )),
            plugin_manifest::MaintenanceStatus::Active => {}
        }

        if let Some(deprecation) = &maintenance.deprecation {
            warnings.push(render_deprecation_warning(
                &manifest.name,
                deprecation.message.as_deref(),
                deprecation.replacement.as_deref(),
                deprecation.removal_target.as_deref(),
            ));
        }
    }

    if let Some(trust) = &manifest.trust {
        if !trust.deterministic {
            warnings.push(format!(
                "{} may produce non-deterministic results across runs.",
                manifest.name
            ));
        }
        if !trust.local_only {
            warnings.push(format!(
                "{} may access non-local resources or services.",
                manifest.name
            ));
        }
        if trust.network != plugin_manifest::NetworkAccess::None {
            warnings.push(format!(
                "{} declares network access policy {:?}.",
                manifest.name, trust.network
            ));
        }
        if !trust.permissions.is_empty() {
            warnings.push(format!(
                "{} declares elevated permissions: {}.",
                manifest.name,
                trust
                    .permissions
                    .iter()
                    .map(|permission| format!("{} ({:?})", permission.resource, permission.scope))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }

    if let Some(lifecycle) = &manifest.lifecycle {
        if lifecycle.requires_explicit_shutdown {
            warnings.push(format!(
                "{} requires explicit shutdown handling to clean up safely.",
                manifest.name
            ));
        }
        if !lifecycle.stateless {
            warnings.push(format!(
                "{} maintains internal state between invocations.",
                manifest.name
            ));
        }
        if !matches!(lifecycle.state, plugin_manifest::LifecycleState::Ready) {
            warnings.push(format!(
                "{} currently reports lifecycle state {:?}.",
                manifest.name, lifecycle.state
            ));
        }
    }

    if manifest
        .execution
        .as_ref()
        .is_some_and(|execution| execution.supports_async)
    {
        warnings.push(format!(
            "{} supports async execution; hosts may degrade to foreground execution.",
            manifest.name
        ));
    }

    if manifest
        .capability_contract
        .as_ref()
        .is_some_and(|contract| !contract.optional.is_empty() || !contract.degradation.is_empty())
    {
        warnings.push(format!(
            "{} may degrade features based on host capability negotiation.",
            manifest.name
        ));
    }

    if let Some(action) = action {
        if let Some(deprecation) = &action.deprecation {
            warnings.push(render_deprecation_warning(
                &format!("action {}", action.id),
                deprecation.message.as_deref(),
                deprecation.replacement.as_deref(),
                deprecation.removal_target.as_deref(),
            ));
        }

        if let Some(contract) = &action.contract {
            if contract.mutates_workspace {
                warnings.push(format!(
                    "Action {} mutates the workspace or local files.",
                    action.id
                ));
            }
            if contract.async_metadata.is_some() {
                warnings.push(format!(
                    "Action {} publishes async execution metadata.",
                    action.id
                ));
            }
            if let Some(constraints) = &contract.constraints {
                if !constraints.required.is_empty() {
                    warnings.push(format!(
                        "Action {} requires host capabilities: {}.",
                        action.id,
                        constraints
                            .required
                            .iter()
                            .map(|requirement| requirement.key.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
                if !constraints.permissions.is_empty() {
                    warnings.push(format!(
                        "Action {} requires permissions: {}.",
                        action.id,
                        constraints
                            .permissions
                            .iter()
                            .map(|permission| {
                                format!("{} ({:?})", permission.resource, permission.scope)
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
                if constraints.max_payload_bytes.is_some() {
                    warnings.push(format!(
                        "Action {} enforces payload size limits.",
                        action.id
                    ));
                }
            }
        }
    }

    warnings
}

fn render_deprecation_warning(
    target: &str,
    message: Option<&str>,
    replacement: Option<&str>,
    removal_target: Option<&str>,
) -> String {
    let mut warning = format!("{target} is deprecated");
    if let Some(message) = message {
        warning.push_str(&format!(": {message}"));
    }
    if let Some(replacement) = replacement {
        warning.push_str(&format!(" Use {replacement} instead."));
    } else {
        warning.push('.');
    }
    if let Some(removal_target) = removal_target {
        warning.push_str(&format!(" Planned removal target: {removal_target}."));
    }
    warning
}

fn pretty_json_or_raw(input: &str) -> String {
    serde_json::from_str::<Value>(input)
        .ok()
        .and_then(|value| serde_json::to_string_pretty(&value).ok())
        .unwrap_or_else(|| input.to_owned())
}

fn default_host_capabilities(host: HostKind) -> Vec<CapabilityAvailability> {
    match host {
        HostKind::Cli => vec![
            CapabilityAvailability::available("stdout-text", "Plain-text terminal rendering."),
            CapabilityAvailability::available("stdout-json", "Structured JSON can be printed."),
            CapabilityAvailability::available(
                "markdown-output",
                "Markdown sections render readably.",
            ),
            CapabilityAvailability::available("code-output", "Code blocks can be shown inline."),
            CapabilityAvailability::available("workspace-root", "Host provides a workspace root."),
            CapabilityAvailability::available(
                "local-only",
                "Execution stays on the local machine.",
            ),
            CapabilityAvailability::available("sync-execution", "Immediate foreground execution."),
        ],
        HostKind::Tui => vec![
            CapabilityAvailability::available(
                "panel-layout",
                "Structured panel layout is available.",
            ),
            CapabilityAvailability::available("stdout-json", "Structured JSON can be shown."),
            CapabilityAvailability::available(
                "markdown-output",
                "Markdown summaries are supported.",
            ),
            CapabilityAvailability::available("sync-execution", "Immediate foreground execution."),
        ],
        HostKind::Egui | HostKind::Iced | HostKind::Dioxus => vec![
            CapabilityAvailability::available("rich-ui", "Host can render richer interactive UI."),
            CapabilityAvailability::available("stdout-json", "Structured JSON can be rendered."),
            CapabilityAvailability::available("async-jobs", "Host can track async jobs."),
        ],
        HostKind::Web => vec![
            CapabilityAvailability::available("rich-ui", "Host can render browser-oriented UI."),
            CapabilityAvailability::available("stdout-json", "Structured JSON can be rendered."),
            CapabilityAvailability::available("async-jobs", "Host can track async jobs."),
            CapabilityAvailability::available(
                "sandboxed-runtime",
                "Host expects sandboxed execution.",
            ),
        ],
        HostKind::Service => vec![
            CapabilityAvailability::available(
                "stdout-json",
                "Structured JSON responses are supported.",
            ),
            CapabilityAvailability::available("async-jobs", "Host can track async jobs."),
            CapabilityAvailability::available(
                "health-hooks",
                "Lifecycle health hooks are supported.",
            ),
            CapabilityAvailability::available(
                "sync-execution",
                "Immediate execution is available.",
            ),
        ],
        HostKind::Any => Vec::new(),
    }
}

fn default_execution_mode(host: HostKind) -> plugin_protocol::ExecutionMode {
    match host {
        HostKind::Web | HostKind::Service | HostKind::Egui | HostKind::Iced | HostKind::Dioxus => {
            plugin_protocol::ExecutionMode::Async
        }
        HostKind::Cli | HostKind::Tui | HostKind::Any => plugin_protocol::ExecutionMode::Sync,
    }
}

fn default_execution_metadata(
    manifest: &PluginManifest,
    request: &PluginRequest,
) -> ExecutionMetadata {
    let execution = manifest.execution.as_ref();
    let runtime = request.context.runtime.as_ref();

    ExecutionMetadata {
        mode: runtime
            .and_then(|runtime| runtime.preferred_mode)
            .or_else(|| execution.map(|execution| execution.default_mode)),
        supports_async: execution.is_some_and(|execution| execution.supports_async),
        cancellable: execution.is_some_and(|execution| execution.cancellable),
        timeout_ms: execution
            .and_then(|execution| execution.timeout_ms)
            .or_else(|| runtime.and_then(|runtime| runtime.max_timeout_ms)),
        duration_ms: None,
        lifecycle_state: manifest.lifecycle.as_ref().map(|lifecycle| lifecycle.state),
        progress_message: execution
            .filter(|execution| execution.progress_reporting)
            .map(|_| {
                "Host can expect deterministic progress updates when the plugin emits them."
                    .to_owned()
            }),
        job: None,
    }
}

fn negotiate_capabilities(
    manifest: &PluginManifest,
    context: &InvocationContext,
    host_supported: bool,
    version_supported: bool,
    version_summary: &Option<String>,
) -> NegotiationOutcome {
    let available_capabilities = context
        .runtime
        .as_ref()
        .map(|runtime| {
            runtime
                .available_capabilities
                .iter()
                .filter(|capability| capability.available)
                .map(|capability| capability.key.as_str())
                .collect::<std::collections::HashSet<_>>()
        })
        .unwrap_or_default();

    let mut outcome = NegotiationOutcome::default();
    let mut degraded_features = Vec::new();

    if let Some(contract) = &manifest.capability_contract {
        outcome.granted_capabilities = contract
            .required
            .iter()
            .chain(contract.optional.iter())
            .filter(|requirement| available_capabilities.contains(requirement.key.as_str()))
            .map(|requirement| requirement.key.clone())
            .collect();

        outcome.missing_required = contract
            .required
            .iter()
            .filter(|requirement| !available_capabilities.contains(requirement.key.as_str()))
            .cloned()
            .collect();

        outcome.missing_optional = contract
            .optional
            .iter()
            .filter(|requirement| !available_capabilities.contains(requirement.key.as_str()))
            .cloned()
            .collect();

        degraded_features = contract
            .degradation
            .iter()
            .filter(|rule| {
                rule.when_missing
                    .iter()
                    .all(|required| !available_capabilities.contains(required.as_str()))
            })
            .map(|rule| DegradedFeature {
                feature: rule.feature.clone(),
                reason: rule.behavior.clone(),
                severity: rule.severity,
                fallback: Some(rule.behavior.clone()),
            })
            .collect();
    }

    outcome.degraded_features = degraded_features;

    outcome.status =
        if !host_supported || !version_supported || !outcome.missing_required.is_empty() {
            NegotiationStatus::Rejected
        } else if !outcome.missing_optional.is_empty() || !outcome.degraded_features.is_empty() {
            NegotiationStatus::Degraded
        } else {
            NegotiationStatus::Ready
        };

    let mut summary_parts = Vec::new();
    if let Some(version_summary) = version_summary {
        summary_parts.push(version_summary.clone());
    }
    if !host_supported {
        summary_parts.push("host kind does not match the plugin manifest".to_owned());
    }
    if !version_supported {
        summary_parts.push("host version is outside the declared compatibility window".to_owned());
    }
    if !outcome.missing_required.is_empty() {
        summary_parts.push(format!(
            "missing required capabilities: {}",
            outcome
                .missing_required
                .iter()
                .map(|requirement| requirement.key.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if !outcome.missing_optional.is_empty() {
        summary_parts.push(format!(
            "optional capabilities unavailable: {}",
            outcome
                .missing_optional
                .iter()
                .map(|requirement| requirement.key.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if !outcome.degraded_features.is_empty() {
        summary_parts.push(format!(
            "degraded features: {}",
            outcome
                .degraded_features
                .iter()
                .map(|feature| feature.feature.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if summary_parts.is_empty() {
        summary_parts.push("host satisfies the declared runtime contract".to_owned());
    }
    outcome.summary = summary_parts.join("; ");

    outcome
}

fn compatibility_summary(
    manifest: &PluginManifest,
    context: &InvocationContext,
) -> Option<(String, bool)> {
    let compatibility = manifest.compatibility.as_ref()?;
    let host_version = context.runtime.as_ref()?.host_version.as_deref()?;

    let host_range = compatibility.host_version.as_ref()?;
    let host_version = Version::parse(host_version).ok()?;

    let minimum = host_range
        .minimum
        .as_deref()
        .and_then(|value| Version::parse(value).ok());
    let maximum = host_range
        .maximum
        .as_deref()
        .and_then(|value| Version::parse(value).ok());

    let above_minimum = minimum
        .as_ref()
        .is_none_or(|minimum| &host_version >= minimum);
    let below_maximum = maximum
        .as_ref()
        .is_none_or(|maximum| &host_version <= maximum);

    let tested = compatibility
        .tested_hosts
        .iter()
        .find(|tested| tested.host == context.host && tested.version == host_version.to_string());

    Some(match (above_minimum, below_maximum, tested) {
        (true, true, Some(tested)) => match &tested.notes {
            Some(notes) => (
                format!(
                    "tested against {} {} ({notes})",
                    tested.host.label(),
                    tested.version
                ),
                true,
            ),
            None => (
                format!("tested against {} {}", tested.host.label(), tested.version),
                true,
            ),
        },
        (true, true, None) => (
            format!(
                "host version {} satisfies the declared {:?} compatibility window",
                host_version, compatibility.strategy
            ),
            true,
        ),
        _ => (
            format!(
                "host version {} falls outside the declared compatibility window",
                host_version
            ),
            false,
        ),
    })
}
