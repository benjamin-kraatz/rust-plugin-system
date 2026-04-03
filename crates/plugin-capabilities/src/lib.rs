//! Shared capability vocabulary for manifests and runtime negotiation.
//!
//! This crate defines stable, serializable data types used across plugin
//! authoring, host runtime negotiation, and protocol metadata.

use serde::{Deserialize, Serialize};

/// Host runtime category supported by a plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum HostKind {
    /// Command-line host.
    Cli,
    /// Text user-interface host.
    Tui,
    /// `egui` or `eframe`-style GUI host.
    Egui,
    /// Iced GUI host.
    Iced,
    /// Dioxus desktop host.
    Dioxus,
    /// Browser/web host.
    Web,
    /// Background service host.
    Service,
    /// Any host category.
    #[default]
    Any,
}

impl HostKind {
    /// Returns a display label suitable for user-facing summaries.
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

/// Runtime architecture used to load and execute a plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginArchitecture {
    /// Native dynamic library with JSON-over-FFI calls.
    NativeJson,
    /// ABI-stable Rust module loaded through `abi_stable`.
    AbiStable,
    /// Wasm module executed in a sandboxed runtime.
    Wasm,
}

/// Suggested expertise level for safe and effective plugin usage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SkillLevel {
    /// Beginner-friendly usage.
    Basic,
    /// Requires moderate familiarity.
    Intermediate,
    /// Requires advanced domain understanding.
    Advanced,
    /// Intended for expert users.
    Expert,
}

/// Named capability surfaced by a plugin or host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capability {
    /// Capability identifier, for example `stdout-json`.
    pub key: String,
    /// Human-readable description of what the capability means.
    pub description: String,
}

impl Capability {
    /// Creates a capability descriptor.
    pub fn new(key: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            description: description.into(),
        }
    }
}

/// Version compatibility strategy between host and plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum VersionStrategy {
    /// Semantic-version compatibility.
    #[default]
    Semver,
    /// Exact version match only.
    Exact,
    /// Host and plugin versions move in lockstep.
    Lockstep,
    /// Compatibility is decided by host-defined policy.
    HostDefined,
}

/// Declared trust level for a plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum TrustLevel {
    /// Minimal review and trust.
    Low,
    /// Reviewed and approved for normal use.
    #[default]
    Reviewed,
    /// Restricted usage with elevated safeguards.
    Restricted,
    /// Privileged plugin with broad trust.
    Privileged,
}

/// Isolation model used while executing plugin code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SandboxLevel {
    /// No sandboxing.
    None,
    /// Isolated process boundary.
    Process,
    /// Wasm runtime sandbox.
    #[default]
    Wasm,
    /// Host-mediated and policy-driven sandbox.
    HostMediated,
}

/// Network reachability policy declared by a plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkAccess {
    /// No network access.
    #[default]
    None,
    /// Loopback access only.
    Loopback,
    /// Network access limited to allowlisted endpoints.
    Allowlisted,
    /// Full network access.
    Full,
}

/// Lifecycle hook points a plugin may participate in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum LifecycleHook {
    /// Installation-time hook.
    Install,
    /// Load-time hook.
    Load,
    /// Initialization hook before first invocation.
    Initialize,
    /// Invocation hook.
    #[default]
    Invoke,
    /// Health check hook.
    HealthCheck,
    /// Suspend hook.
    Suspend,
    /// Resume hook.
    Resume,
    /// Shutdown hook.
    Shutdown,
}

/// Runtime lifecycle status reported by a plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum LifecycleState {
    /// Registered but not loaded.
    Registered,
    /// Loaded into host runtime.
    Loaded,
    /// Initializing.
    Initializing,
    /// Ready for normal invocation.
    #[default]
    Ready,
    /// Running in degraded state.
    Degraded,
    /// Draining and not accepting new work.
    Draining,
    /// Stopped.
    Stopped,
}

/// Preferred execution mode for an action or runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionMode {
    /// Foreground, synchronous completion.
    #[default]
    Sync,
    /// Asynchronous execution with possible detached completion.
    Async,
}

/// Severity of degradation when capabilities are missing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum DegradationSeverity {
    /// Minor degradation.
    #[default]
    Low,
    /// Noticeable degradation.
    Medium,
    /// Significant degradation.
    High,
    /// Critical degradation.
    Critical,
}

/// Access scope requested for a resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum PermissionScope {
    /// No access required.
    #[default]
    None,
    /// Read access.
    Read,
    /// Write access.
    Write,
    /// Execute access.
    Execute,
    /// Administrative access.
    Admin,
}

/// Operational maintenance phase of a plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum MaintenanceStatus {
    /// Early-stage and unstable.
    Experimental,
    /// Actively maintained.
    #[default]
    Active,
    /// Maintenance and fixes only.
    MaintenanceOnly,
    /// Deprecated and pending removal.
    Deprecated,
    /// Retired and no longer supported.
    Retired,
}

/// Backoff strategy applied between retry attempts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RetryBackoffStrategy {
    /// Constant delay.
    #[default]
    Fixed,
    /// Linear delay growth.
    Linear,
    /// Exponential delay growth.
    Exponential,
}

/// Requirement for one capability key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CapabilityRequirement {
    /// Capability key that must be provided by the host.
    pub key: String,
    /// Explanation of why this capability is needed.
    pub detail: String,
    /// Optional fallback behavior if unavailable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<String>,
}

impl CapabilityRequirement {
    /// Creates a capability requirement.
    pub fn new(key: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            detail: detail.into(),
            fallback: None,
        }
    }

    /// Adds fallback guidance when the capability is unavailable.
    pub fn with_fallback(mut self, fallback: impl Into<String>) -> Self {
        self.fallback = Some(fallback.into());
        self
    }
}

/// Permission requirement attached to an action or plugin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PermissionDescriptor {
    /// Resource name or scope target.
    pub resource: String,
    /// Requested access level.
    pub scope: PermissionScope,
    /// Optional business or safety justification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Whether the permission is mandatory.
    pub required: bool,
}

impl PermissionDescriptor {
    /// Creates a required permission descriptor.
    pub fn new(resource: impl Into<String>, scope: PermissionScope) -> Self {
        Self {
            resource: resource.into(),
            scope,
            reason: None,
            required: true,
        }
    }

    /// Marks this permission as optional.
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    /// Adds explanatory context for reviewers and hosts.
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }
}

/// Availability state of one host capability during negotiation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CapabilityAvailability {
    /// Capability key.
    pub key: String,
    /// Whether the capability is currently available.
    pub available: bool,
    /// Optional detail about support level or reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl CapabilityAvailability {
    /// Creates an available capability record.
    pub fn available(key: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            available: true,
            detail: Some(detail.into()),
        }
    }

    /// Creates an unavailable capability record.
    pub fn unavailable(key: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            available: false,
            detail: Some(detail.into()),
        }
    }
}

/// Retry behavior for asynchronous or remote operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RetryPolicy {
    /// Maximum number of attempts, including the initial try.
    pub max_attempts: u32,
    /// Initial delay before a retry.
    pub initial_backoff_ms: u64,
    /// Upper bound for computed backoff delay.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_backoff_ms: Option<u64>,
    /// Backoff growth strategy.
    pub strategy: RetryBackoffStrategy,
    /// Error categories eligible for retry.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub retry_on: Vec<String>,
}

impl RetryPolicy {
    /// Creates a retry policy with fixed strategy and no retry categories.
    ///
    /// # Examples
    ///
    /// ```
    /// use plugin_capabilities::{RetryBackoffStrategy, RetryPolicy};
    ///
    /// let policy = RetryPolicy::new(3)
    ///     .with_initial_backoff_ms(100)
    ///     .with_max_backoff_ms(1_000)
    ///     .with_strategy(RetryBackoffStrategy::Exponential)
    ///     .with_retry_on(["timeout", "temporary-unavailable"]);
    ///
    /// assert_eq!(policy.max_attempts, 3);
    /// assert_eq!(policy.retry_on.len(), 2);
    /// ```
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            initial_backoff_ms: 0,
            max_backoff_ms: None,
            strategy: RetryBackoffStrategy::Fixed,
            retry_on: Vec::new(),
        }
    }

    /// Sets the initial retry delay.
    pub fn with_initial_backoff_ms(mut self, initial_backoff_ms: u64) -> Self {
        self.initial_backoff_ms = initial_backoff_ms;
        self
    }

    /// Sets an upper bound for retry delay.
    pub fn with_max_backoff_ms(mut self, max_backoff_ms: u64) -> Self {
        self.max_backoff_ms = Some(max_backoff_ms);
        self
    }

    /// Sets the retry backoff strategy.
    pub fn with_strategy(mut self, strategy: RetryBackoffStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Sets retryable error categories.
    pub fn with_retry_on(mut self, retry_on: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.retry_on = retry_on.into_iter().map(Into::into).collect();
        self
    }
}

/// Additional metadata for async-capable execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AsyncMetadata {
    /// Declared execution mode.
    pub execution_mode: ExecutionMode,
    /// Whether work may continue after the initial response is returned.
    pub detached: bool,
    /// Whether incremental/streamed output is supported.
    pub supports_streaming: bool,
    /// Optional deadline for full completion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_timeout_ms: Option<u64>,
    /// Optional retry behavior for asynchronous processing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_policy: Option<RetryPolicy>,
}

impl AsyncMetadata {
    /// Creates default async metadata with asynchronous execution mode enabled.
    pub fn asynchronous() -> Self {
        Self {
            execution_mode: ExecutionMode::Async,
            detached: false,
            supports_streaming: false,
            completion_timeout_ms: None,
            retry_policy: None,
        }
    }

    /// Sets completion timeout for async work.
    pub fn with_completion_timeout_ms(mut self, completion_timeout_ms: u64) -> Self {
        self.completion_timeout_ms = Some(completion_timeout_ms);
        self
    }

    /// Attaches a retry policy.
    pub fn with_retry_policy(mut self, retry_policy: RetryPolicy) -> Self {
        self.retry_policy = Some(retry_policy);
        self
    }

    /// Enables or disables streaming support.
    pub fn with_streaming(mut self, supports_streaming: bool) -> Self {
        self.supports_streaming = supports_streaming;
        self
    }
}

/// Negotiation constraints and policy for capability-aware execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CapabilityConstraints {
    /// Required capabilities.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<CapabilityRequirement>,
    /// Optional capabilities.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub optional: Vec<CapabilityRequirement>,
    /// Scoped permission requests.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<PermissionDescriptor>,
    /// Optional payload size limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_payload_bytes: Option<u64>,
    /// Declared network access policy.
    pub network_access: NetworkAccess,
    /// Declared sandbox requirement.
    pub sandbox_level: SandboxLevel,
}

impl CapabilityConstraints {
    /// Creates an empty constraints set.
    ///
    /// # Examples
    ///
    /// ```
    /// use plugin_capabilities::{
    ///     CapabilityConstraints, CapabilityRequirement, NetworkAccess,
    ///     PermissionDescriptor, PermissionScope, SandboxLevel,
    /// };
    ///
    /// let constraints = CapabilityConstraints::new()
    ///     .with_required([CapabilityRequirement::new("stdout-json", "Need JSON output")])
    ///     .with_permissions([
    ///         PermissionDescriptor::new("workspace", PermissionScope::Read)
    ///             .with_reason("Read local project metadata"),
    ///     ])
    ///     .with_network_access(NetworkAccess::None)
    ///     .with_sandbox_level(SandboxLevel::Wasm)
    ///     .with_max_payload_bytes(64 * 1024);
    ///
    /// assert_eq!(constraints.required.len(), 1);
    /// assert_eq!(constraints.permissions.len(), 1);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets required capabilities.
    pub fn with_required(
        mut self,
        required: impl IntoIterator<Item = CapabilityRequirement>,
    ) -> Self {
        self.required = required.into_iter().collect();
        self
    }

    /// Sets optional capabilities.
    pub fn with_optional(
        mut self,
        optional: impl IntoIterator<Item = CapabilityRequirement>,
    ) -> Self {
        self.optional = optional.into_iter().collect();
        self
    }

    /// Sets permission requirements.
    pub fn with_permissions(
        mut self,
        permissions: impl IntoIterator<Item = PermissionDescriptor>,
    ) -> Self {
        self.permissions = permissions.into_iter().collect();
        self
    }

    /// Sets the maximum payload size.
    pub fn with_max_payload_bytes(mut self, max_payload_bytes: u64) -> Self {
        self.max_payload_bytes = Some(max_payload_bytes);
        self
    }

    /// Sets network access policy.
    pub fn with_network_access(mut self, network_access: NetworkAccess) -> Self {
        self.network_access = network_access;
        self
    }

    /// Sets sandbox requirement.
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
