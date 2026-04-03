use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum HostKind {
    Cli,
    Tui,
    Egui,
    Iced,
    Dioxus,
    Web,
    Service,
    #[default]
    Any,
}

impl HostKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Cli => "CLI",
            Self::Tui => "TUI",
            Self::Egui => "egui/eframe",
            Self::Iced => "Iced",
            Self::Dioxus => "Dioxus Desktop",
            Self::Web => "Web",
            Self::Service => "Service",
            Self::Any => "Any host",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginArchitecture {
    NativeJson,
    AbiStable,
    Wasm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SkillLevel {
    Basic,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capability {
    pub key: String,
    pub description: String,
}

impl Capability {
    pub fn new(key: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            description: description.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum VersionStrategy {
    #[default]
    Semver,
    Exact,
    Lockstep,
    HostDefined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum TrustLevel {
    Low,
    #[default]
    Reviewed,
    Restricted,
    Privileged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SandboxLevel {
    None,
    Process,
    #[default]
    Wasm,
    HostMediated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkAccess {
    #[default]
    None,
    Loopback,
    Allowlisted,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum LifecycleHook {
    Install,
    Load,
    Initialize,
    #[default]
    Invoke,
    HealthCheck,
    Suspend,
    Resume,
    Shutdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum LifecycleState {
    Registered,
    Loaded,
    Initializing,
    #[default]
    Ready,
    Degraded,
    Draining,
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionMode {
    #[default]
    Sync,
    Async,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum DegradationSeverity {
    #[default]
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum PermissionScope {
    #[default]
    None,
    Read,
    Write,
    Execute,
    Admin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum MaintenanceStatus {
    Experimental,
    #[default]
    Active,
    MaintenanceOnly,
    Deprecated,
    Retired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RetryBackoffStrategy {
    #[default]
    Fixed,
    Linear,
    Exponential,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CapabilityRequirement {
    pub key: String,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<String>,
}

impl CapabilityRequirement {
    pub fn new(key: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            detail: detail.into(),
            fallback: None,
        }
    }

    pub fn with_fallback(mut self, fallback: impl Into<String>) -> Self {
        self.fallback = Some(fallback.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PermissionDescriptor {
    pub resource: String,
    pub scope: PermissionScope,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub required: bool,
}

impl PermissionDescriptor {
    pub fn new(resource: impl Into<String>, scope: PermissionScope) -> Self {
        Self {
            resource: resource.into(),
            scope,
            reason: None,
            required: true,
        }
    }

    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CapabilityAvailability {
    pub key: String,
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl CapabilityAvailability {
    pub fn available(key: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            available: true,
            detail: Some(detail.into()),
        }
    }

    pub fn unavailable(key: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            available: false,
            detail: Some(detail.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_backoff_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_backoff_ms: Option<u64>,
    pub strategy: RetryBackoffStrategy,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub retry_on: Vec<String>,
}

impl RetryPolicy {
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            initial_backoff_ms: 0,
            max_backoff_ms: None,
            strategy: RetryBackoffStrategy::Fixed,
            retry_on: Vec::new(),
        }
    }

    pub fn with_initial_backoff_ms(mut self, initial_backoff_ms: u64) -> Self {
        self.initial_backoff_ms = initial_backoff_ms;
        self
    }

    pub fn with_max_backoff_ms(mut self, max_backoff_ms: u64) -> Self {
        self.max_backoff_ms = Some(max_backoff_ms);
        self
    }

    pub fn with_strategy(mut self, strategy: RetryBackoffStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn with_retry_on(mut self, retry_on: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.retry_on = retry_on.into_iter().map(Into::into).collect();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AsyncMetadata {
    pub execution_mode: ExecutionMode,
    pub detached: bool,
    pub supports_streaming: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_timeout_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_policy: Option<RetryPolicy>,
}

impl AsyncMetadata {
    pub fn asynchronous() -> Self {
        Self {
            execution_mode: ExecutionMode::Async,
            detached: false,
            supports_streaming: false,
            completion_timeout_ms: None,
            retry_policy: None,
        }
    }

    pub fn with_completion_timeout_ms(mut self, completion_timeout_ms: u64) -> Self {
        self.completion_timeout_ms = Some(completion_timeout_ms);
        self
    }

    pub fn with_retry_policy(mut self, retry_policy: RetryPolicy) -> Self {
        self.retry_policy = Some(retry_policy);
        self
    }

    pub fn with_streaming(mut self, supports_streaming: bool) -> Self {
        self.supports_streaming = supports_streaming;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CapabilityConstraints {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<CapabilityRequirement>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub optional: Vec<CapabilityRequirement>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<PermissionDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_payload_bytes: Option<u64>,
    pub network_access: NetworkAccess,
    pub sandbox_level: SandboxLevel,
}

impl CapabilityConstraints {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_required(
        mut self,
        required: impl IntoIterator<Item = CapabilityRequirement>,
    ) -> Self {
        self.required = required.into_iter().collect();
        self
    }

    pub fn with_optional(
        mut self,
        optional: impl IntoIterator<Item = CapabilityRequirement>,
    ) -> Self {
        self.optional = optional.into_iter().collect();
        self
    }

    pub fn with_permissions(
        mut self,
        permissions: impl IntoIterator<Item = PermissionDescriptor>,
    ) -> Self {
        self.permissions = permissions.into_iter().collect();
        self
    }

    pub fn with_max_payload_bytes(mut self, max_payload_bytes: u64) -> Self {
        self.max_payload_bytes = Some(max_payload_bytes);
        self
    }

    pub fn with_network_access(mut self, network_access: NetworkAccess) -> Self {
        self.network_access = network_access;
        self
    }

    pub fn with_sandbox_level(mut self, sandbox_level: SandboxLevel) -> Self {
        self.sandbox_level = sandbox_level;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CapabilityDegradation {
    pub key: String,
    pub severity: DegradationSeverity,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<String>,
}

impl CapabilityDegradation {
    pub fn new(
        key: impl Into<String>,
        severity: DegradationSeverity,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            severity,
            reason: reason.into(),
            fallback: None,
        }
    }

    pub fn with_fallback(mut self, fallback: impl Into<String>) -> Self {
        self.fallback = Some(fallback.into());
        self
    }
}
