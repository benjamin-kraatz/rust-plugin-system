//! Runtime protocol types exchanged between hosts and plugins.
//!
//! This crate defines invocation requests, responses, rendering blocks, and
//! negotiation metadata.

#[doc(inline)]
pub use plugin_capabilities::{
    CapabilityAvailability, CapabilityRequirement, DegradationSeverity, ExecutionMode, HostKind,
    LifecycleState,
};
#[doc(inline)]
pub use plugin_manifest::{
    ActionContract, CapabilityContract, CompatibilityContract, DegradationRule, DeprecationNotice,
    ExecutionContract, LifecycleContract, MaintenanceContract, SchemaDescriptor, TrustMetadata,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Context attached to every plugin invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InvocationContext {
    /// Host kind issuing the request.
    pub host: HostKind,
    /// Optional workspace root path.
    pub workspace_root: Option<String>,
    /// Optional plugin directory path used by the host.
    pub plugin_dir: Option<String>,
    /// Optional execution mode label.
    pub mode: Option<String>,
    /// Request identifier used for correlation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Trace identifier used for distributed tracing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    /// Optional timeout budget in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    /// Pre-invocation warnings assembled by the host.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    /// Additional runtime capability context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<RuntimeContext>,
}

impl InvocationContext {
    /// Creates a minimal invocation context for one host kind.
    ///
    /// # Examples
    ///
    /// ```
    /// use plugin_protocol::{HostKind, InvocationContext};
    ///
    /// let context = InvocationContext::for_host(HostKind::Cli)
    ///     .with_request_id("req-123")
    ///     .with_trace_id("trace-abc");
    ///
    /// assert_eq!(context.host, HostKind::Cli);
    /// assert_eq!(context.request_id.as_deref(), Some("req-123"));
    /// ```
    pub fn for_host(host: HostKind) -> Self {
        Self {
            host,
            workspace_root: None,
            plugin_dir: None,
            mode: None,
            request_id: None,
            trace_id: None,
            timeout_ms: None,
            warnings: Vec::new(),
            runtime: None,
        }
    }

    /// Sets the request identifier.
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Sets the trace identifier.
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    /// Sets timeout budget in milliseconds.
    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    /// Appends one warning message.
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }
}

impl Default for InvocationContext {
    fn default() -> Self {
        Self::for_host(HostKind::Any)
    }
}

/// Request payload sent from host to plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRequest {
    /// Target plugin identifier.
    pub plugin_id: String,
    /// Target action identifier.
    pub action_id: String,
    /// Arbitrary JSON payload.
    pub payload: Value,
    /// Invocation metadata and capability context.
    pub context: InvocationContext,
}

/// Output block category for rendered plugin responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputKind {
    /// Plain text output.
    Text,
    /// JSON output.
    Json,
    /// Source-code output.
    Code,
    /// Markdown output.
    Markdown,
}

/// One output section in a plugin response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputBlock {
    /// Output category.
    pub kind: OutputKind,
    /// Optional section title.
    pub title: Option<String>,
    /// Output body content.
    pub body: String,
}

/// Result of a plugin action invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResponse {
    /// Plugin identifier.
    pub plugin_id: String,
    /// Action identifier.
    pub action_id: String,
    /// Human-readable title.
    pub title: String,
    /// Short summary.
    pub summary: String,
    /// Whether the invocation succeeded.
    pub success: bool,
    /// Structured output blocks.
    pub outputs: Vec<OutputBlock>,
    /// Suggested follow-up steps.
    pub suggested_next_steps: Vec<String>,
    /// Correlated request identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Runtime warnings associated with this response.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    /// Optional execution metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: Option<ExecutionMetadata>,
    /// Optional capability negotiation metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub negotiation: Option<NegotiationOutcome>,
}

impl PluginResponse {
    /// Constructs a successful response shell.
    ///
    /// # Examples
    ///
    /// ```
    /// use plugin_protocol::PluginResponse;
    ///
    /// let response = PluginResponse::ok("demo", "run", "Done", "All checks passed");
    /// assert!(response.success);
    /// ```
    pub fn ok(
        plugin_id: impl Into<String>,
        action_id: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            action_id: action_id.into(),
            title: title.into(),
            summary: summary.into(),
            success: true,
            outputs: Vec::new(),
            suggested_next_steps: Vec::new(),
            request_id: None,
            warnings: Vec::new(),
            execution: None,
            negotiation: None,
        }
    }

    /// Constructs an error response shell.
    ///
    /// # Examples
    ///
    /// ```
    /// use plugin_protocol::PluginResponse;
    ///
    /// let response = PluginResponse::error("demo", "run", "Failed", "Validation error");
    /// assert!(!response.success);
    /// ```
    pub fn error(
        plugin_id: impl Into<String>,
        action_id: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            action_id: action_id.into(),
            title: title.into(),
            summary: summary.into(),
            success: false,
            outputs: Vec::new(),
            suggested_next_steps: Vec::new(),
            request_id: None,
            warnings: Vec::new(),
            execution: None,
            negotiation: None,
        }
    }

    /// Appends one titled output block.
    pub fn with_output(
        mut self,
        kind: OutputKind,
        title: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        self.outputs.push(OutputBlock {
            kind,
            title: Some(title.into()),
            body: body.into(),
        });
        self
    }

    /// Appends one suggested next step.
    pub fn with_next_step(mut self, next_step: impl Into<String>) -> Self {
        self.suggested_next_steps.push(next_step.into());
        self
    }

    /// Sets correlated request id.
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Appends one warning message.
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    /// Sets execution metadata.
    pub fn with_execution(mut self, execution: ExecutionMetadata) -> Self {
        self.execution = Some(execution);
        self
    }

    /// Sets negotiation metadata.
    pub fn with_negotiation(mut self, negotiation: NegotiationOutcome) -> Self {
        self.negotiation = Some(negotiation);
        self
    }
}

/// Host runtime capabilities and execution preferences.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RuntimeContext {
    /// Optional host version string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_version: Option<String>,
    /// Capability availability snapshot for current host session.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub available_capabilities: Vec<CapabilityAvailability>,
    /// Preferred execution mode requested by host.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_mode: Option<ExecutionMode>,
    /// Whether host expects deterministic behavior.
    pub deterministic: bool,
    /// Whether host expects local-only effects.
    pub local_only: bool,
    /// Optional maximum timeout budget accepted by host.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_timeout_ms: Option<u64>,
}

/// Execution details attached to a response.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ExecutionMetadata {
    /// Effective execution mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<ExecutionMode>,
    /// Whether plugin supports async execution.
    pub supports_async: bool,
    /// Whether requests are cancellable.
    pub cancellable: bool,
    /// Timeout budget in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    /// Total execution duration in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    /// Reported lifecycle state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<LifecycleState>,
    /// Human-readable progress text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_message: Option<String>,
    /// Optional async job metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job: Option<JobMetadata>,
}

/// Async job tracking details.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct JobMetadata {
    /// Stable job identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>,
    /// Current job state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<JobState>,
    /// Optional progress indicator.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<String>,
}

/// Lifecycle state of asynchronous job execution.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum JobState {
    /// Waiting for execution.
    Queued,
    /// Currently running.
    Running,
    /// Completed.
    #[default]
    Completed,
}

/// Result of host/plugin capability negotiation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NegotiationOutcome {
    /// Overall negotiation status.
    pub status: NegotiationStatus,
    /// Human-readable summary.
    pub summary: String,
    /// Capability keys granted by host.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub granted_capabilities: Vec<String>,
    /// Required capabilities not provided by host.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub missing_required: Vec<CapabilityRequirement>,
    /// Optional capabilities not provided by host.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub missing_optional: Vec<CapabilityRequirement>,
    /// Features that were downgraded.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub degraded_features: Vec<DegradedFeature>,
}

impl Default for NegotiationOutcome {
    fn default() -> Self {
        Self {
            status: NegotiationStatus::Ready,
            summary: String::new(),
            granted_capabilities: Vec::new(),
            missing_required: Vec::new(),
            missing_optional: Vec::new(),
            degraded_features: Vec::new(),
        }
    }
}

/// Top-level outcome classification for negotiation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum NegotiationStatus {
    /// Negotiation satisfied all requirements.
    #[default]
    Ready,
    /// Invocation can proceed but with degradation.
    Degraded,
    /// Invocation should be rejected.
    Rejected,
}

/// Description of one degraded feature.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DegradedFeature {
    /// Feature name.
    pub feature: String,
    /// Reason for degradation.
    pub reason: String,
    /// Degradation severity.
    pub severity: DegradationSeverity,
    /// Optional fallback behavior.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<String>,
}
