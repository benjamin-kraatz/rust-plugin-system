pub use plugin_capabilities::{
    CapabilityAvailability, CapabilityRequirement, DegradationSeverity, ExecutionMode, HostKind,
    LifecycleState,
};
pub use plugin_manifest::{
    ActionContract, CapabilityContract, CompatibilityContract, DegradationRule, DeprecationNotice,
    ExecutionContract, LifecycleContract, MaintenanceContract, SchemaDescriptor, TrustMetadata,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InvocationContext {
    pub host: HostKind,
    pub workspace_root: Option<String>,
    pub plugin_dir: Option<String>,
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<RuntimeContext>,
}

impl InvocationContext {
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

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRequest {
    pub plugin_id: String,
    pub action_id: String,
    pub payload: Value,
    pub context: InvocationContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputKind {
    Text,
    Json,
    Code,
    Markdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputBlock {
    pub kind: OutputKind,
    pub title: Option<String>,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResponse {
    pub plugin_id: String,
    pub action_id: String,
    pub title: String,
    pub summary: String,
    pub success: bool,
    pub outputs: Vec<OutputBlock>,
    pub suggested_next_steps: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: Option<ExecutionMetadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub negotiation: Option<NegotiationOutcome>,
}

impl PluginResponse {
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

    pub fn with_next_step(mut self, next_step: impl Into<String>) -> Self {
        self.suggested_next_steps.push(next_step.into());
        self
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    pub fn with_execution(mut self, execution: ExecutionMetadata) -> Self {
        self.execution = Some(execution);
        self
    }

    pub fn with_negotiation(mut self, negotiation: NegotiationOutcome) -> Self {
        self.negotiation = Some(negotiation);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RuntimeContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_version: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub available_capabilities: Vec<CapabilityAvailability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_mode: Option<ExecutionMode>,
    pub deterministic: bool,
    pub local_only: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ExecutionMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<ExecutionMode>,
    pub supports_async: bool,
    pub cancellable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<LifecycleState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job: Option<JobMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct JobMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<JobState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum JobState {
    Queued,
    Running,
    #[default]
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NegotiationOutcome {
    pub status: NegotiationStatus,
    pub summary: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub granted_capabilities: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub missing_required: Vec<CapabilityRequirement>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub missing_optional: Vec<CapabilityRequirement>,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum NegotiationStatus {
    #[default]
    Ready,
    Degraded,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DegradedFeature {
    pub feature: String,
    pub reason: String,
    pub severity: DegradationSeverity,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<String>,
}
