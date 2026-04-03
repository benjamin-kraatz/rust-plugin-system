//! Static plugin identity and contract declarations.
//!
//! A [`PluginManifest`] describes what a plugin is, which hosts it targets, and
//! which runtime expectations it declares.

#[doc(inline)]
pub use plugin_capabilities::{
    AsyncMetadata, Capability, CapabilityConstraints, CapabilityRequirement, DegradationSeverity,
    ExecutionMode, HostKind, LifecycleHook, LifecycleState, MaintenanceStatus, NetworkAccess,
    PermissionDescriptor, PluginArchitecture, SandboxLevel, SkillLevel, TrustLevel,
    VersionStrategy,
};
use serde::{Deserialize, Serialize};

/// Per-action execution and capability contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ActionContract {
    /// Default execution mode for this action.
    pub execution_mode: ExecutionMode,
    /// Whether repeated identical invocations are safe.
    pub idempotent: bool,
    /// Whether action mutates workspace state.
    pub mutates_workspace: bool,
    /// Optional timeout budget for this action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    /// Optional async execution details.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub async_metadata: Option<AsyncMetadata>,
    /// Optional capability constraints specific to this action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constraints: Option<CapabilityConstraints>,
}

impl Default for ActionContract {
    fn default() -> Self {
        Self {
            execution_mode: ExecutionMode::Sync,
            idempotent: true,
            mutates_workspace: false,
            timeout_ms: None,
            async_metadata: None,
            constraints: None,
        }
    }
}

impl ActionContract {
    /// Creates a contract with defaults for `execution_mode`.
    pub fn new(execution_mode: ExecutionMode) -> Self {
        Self {
            execution_mode,
            ..Self::default()
        }
    }

    /// Sets timeout budget in milliseconds.
    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    /// Sets async metadata.
    pub fn with_async_metadata(mut self, async_metadata: AsyncMetadata) -> Self {
        self.async_metadata = Some(async_metadata);
        self
    }

    /// Sets capability constraints.
    pub fn with_constraints(mut self, constraints: CapabilityConstraints) -> Self {
        self.constraints = Some(constraints);
        self
    }

    /// Sets idempotence flag.
    pub fn with_idempotent(mut self, idempotent: bool) -> Self {
        self.idempotent = idempotent;
        self
    }

    /// Sets workspace mutation flag.
    pub fn with_workspace_mutation(mut self, mutates_workspace: bool) -> Self {
        self.mutates_workspace = mutates_workspace;
        self
    }
}

/// Schema pointer used for action input/output payloads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SchemaDescriptor {
    /// Schema format, for example `json-schema`.
    pub format: String,
    /// Schema URI or logical reference.
    pub reference: String,
    /// Optional schema version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl SchemaDescriptor {
    /// Creates a schema descriptor.
    pub fn new(format: impl Into<String>, reference: impl Into<String>) -> Self {
        Self {
            format: format.into(),
            reference: reference.into(),
            version: None,
        }
    }

    /// Sets schema version.
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }
}

/// Deprecation metadata for actions or plugin lifecycle policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DeprecationNotice {
    /// Version or date when deprecation started.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
    /// Suggested replacement identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replacement: Option<String>,
    /// Human-readable deprecation message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Planned removal target (version/date).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removal_target: Option<String>,
}

impl DeprecationNotice {
    /// Creates a deprecation notice with message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            since: None,
            replacement: None,
            message: Some(message.into()),
            removal_target: None,
        }
    }

    /// Sets deprecation start marker.
    pub fn with_since(mut self, since: impl Into<String>) -> Self {
        self.since = Some(since.into());
        self
    }

    /// Sets suggested replacement.
    pub fn with_replacement(mut self, replacement: impl Into<String>) -> Self {
        self.replacement = Some(replacement.into());
        self
    }

    /// Sets planned removal target.
    pub fn with_removal_target(mut self, removal_target: impl Into<String>) -> Self {
        self.removal_target = Some(removal_target.into());
        self
    }
}

/// One callable action in a plugin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginAction {
    /// Stable action identifier.
    pub id: String,
    /// Human-readable action label.
    pub label: String,
    /// Human-readable action description.
    pub description: String,
    /// Optional JSON payload hint/example.
    pub payload_hint: Option<String>,
    /// Optional execution contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract: Option<ActionContract>,
    /// Optional input schema metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<SchemaDescriptor>,
    /// Optional output schema metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<SchemaDescriptor>,
    /// Optional deprecation notice for this action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deprecation: Option<DeprecationNotice>,
}

impl PluginAction {
    /// Creates an action descriptor.
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            description: description.into(),
            payload_hint: None,
            contract: None,
            input_schema: None,
            output_schema: None,
            deprecation: None,
        }
    }

    /// Sets payload hint.
    pub fn with_payload_hint(mut self, payload_hint: impl Into<String>) -> Self {
        self.payload_hint = Some(payload_hint.into());
        self
    }

    /// Sets action contract.
    pub fn with_contract(mut self, contract: ActionContract) -> Self {
        self.contract = Some(contract);
        self
    }

    /// Sets input schema.
    pub fn with_input_schema(mut self, input_schema: SchemaDescriptor) -> Self {
        self.input_schema = Some(input_schema);
        self
    }

    /// Sets output schema.
    pub fn with_output_schema(mut self, output_schema: SchemaDescriptor) -> Self {
        self.output_schema = Some(output_schema);
        self
    }

    /// Sets action deprecation notice.
    pub fn with_deprecation(mut self, deprecation: DeprecationNotice) -> Self {
        self.deprecation = Some(deprecation);
        self
    }
}

/// Top-level plugin descriptor consumed by hosts and loaders.
///
/// # Examples
///
/// ```
/// use plugin_manifest::{HostKind, PluginArchitecture, PluginManifest, SkillLevel};
///
/// let manifest = PluginManifest::new(
///     "hello-world",
///     "Hello World",
///     "0.1.0",
///     "Minimal plugin",
///     PluginArchitecture::NativeJson,
///     SkillLevel::Basic,
/// )
/// .with_supported_hosts(vec![HostKind::Cli, HostKind::Service])
/// .with_tags(["example", "starter"]);
///
/// assert_eq!(manifest.id, "hello-world");
/// assert_eq!(manifest.supported_hosts.len(), 2);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Stable plugin identifier.
    pub id: String,
    /// Human-readable plugin name.
    pub name: String,
    /// Plugin version.
    pub version: String,
    /// Plugin summary description.
    pub description: String,
    /// Runtime architecture used to load the plugin.
    pub architecture: PluginArchitecture,
    /// Intended user skill level.
    pub skill_level: SkillLevel,
    /// Supported host kinds.
    pub supported_hosts: Vec<HostKind>,
    /// Declared capabilities.
    pub capabilities: Vec<Capability>,
    /// Search/filter tags.
    pub tags: Vec<String>,
    /// Callable actions.
    pub actions: Vec<PluginAction>,
    /// Free-form notes.
    pub notes: Vec<String>,
    /// Optional maintenance policy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maintenance: Option<MaintenanceContract>,
    /// Optional compatibility policy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<CompatibilityContract>,
    /// Optional trust metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trust: Option<TrustMetadata>,
    /// Optional lifecycle contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<LifecycleContract>,
    /// Optional execution defaults and behavior.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: Option<ExecutionContract>,
    /// Optional capability negotiation contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_contract: Option<CapabilityContract>,
}

impl PluginManifest {
    /// Creates a manifest with conservative defaults.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
        architecture: PluginArchitecture,
        skill_level: SkillLevel,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: version.into(),
            description: description.into(),
            architecture,
            skill_level,
            supported_hosts: vec![HostKind::Any],
            capabilities: Vec::new(),
            tags: Vec::new(),
            actions: Vec::new(),
            notes: Vec::new(),
            maintenance: None,
            compatibility: None,
            trust: None,
            lifecycle: None,
            execution: None,
            capability_contract: None,
        }
    }

    /// Sets supported hosts.
    pub fn with_supported_hosts(mut self, supported_hosts: Vec<HostKind>) -> Self {
        self.supported_hosts = supported_hosts;
        self
    }

    /// Sets plugin capabilities.
    pub fn with_capabilities(mut self, capabilities: Vec<Capability>) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Sets tags.
    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags = tags.into_iter().map(Into::into).collect();
        self
    }

    /// Sets actions.
    pub fn with_actions(mut self, actions: Vec<PluginAction>) -> Self {
        self.actions = actions;
        self
    }

    /// Sets notes.
    pub fn with_notes(mut self, notes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.notes = notes.into_iter().map(Into::into).collect();
        self
    }

    /// Sets maintenance contract.
    pub fn with_maintenance(mut self, maintenance: MaintenanceContract) -> Self {
        self.maintenance = Some(maintenance);
        self
    }

    /// Sets compatibility contract.
    pub fn with_compatibility(mut self, compatibility: CompatibilityContract) -> Self {
        self.compatibility = Some(compatibility);
        self
    }

    /// Sets trust metadata.
    pub fn with_trust(mut self, trust: TrustMetadata) -> Self {
        self.trust = Some(trust);
        self
    }

    /// Sets lifecycle contract.
    pub fn with_lifecycle(mut self, lifecycle: LifecycleContract) -> Self {
        self.lifecycle = Some(lifecycle);
        self
    }

    /// Sets execution contract.
    pub fn with_execution(mut self, execution: ExecutionContract) -> Self {
        self.execution = Some(execution);
        self
    }

    /// Sets capability contract.
    pub fn with_capability_contract(mut self, capability_contract: CapabilityContract) -> Self {
        self.capability_contract = Some(capability_contract);
        self
    }
}

/// Maintenance and support commitments for a plugin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MaintenanceContract {
    /// Current maintenance phase.
    pub status: MaintenanceStatus,
    /// Optional owner or team.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// Optional support tier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub support_tier: Option<String>,
    /// Optional release/support channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    /// Optional deprecation policy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deprecation: Option<DeprecationNotice>,
}

impl Default for MaintenanceContract {
    fn default() -> Self {
        Self {
            status: MaintenanceStatus::Active,
            owner: None,
            support_tier: None,
            channel: None,
            deprecation: None,
        }
    }
}

impl MaintenanceContract {
    /// Creates a maintenance contract.
    pub fn new(status: MaintenanceStatus) -> Self {
        Self {
            status,
            ..Self::default()
        }
    }

    /// Sets owner.
    pub fn with_owner(mut self, owner: impl Into<String>) -> Self {
        self.owner = Some(owner.into());
        self
    }

    /// Sets support tier.
    pub fn with_support_tier(mut self, support_tier: impl Into<String>) -> Self {
        self.support_tier = Some(support_tier.into());
        self
    }

    /// Sets channel.
    pub fn with_channel(mut self, channel: impl Into<String>) -> Self {
        self.channel = Some(channel.into());
        self
    }

    /// Sets deprecation notice.
    pub fn with_deprecation(mut self, deprecation: DeprecationNotice) -> Self {
        self.deprecation = Some(deprecation);
        self
    }
}

/// Version and host compatibility policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CompatibilityContract {
    /// Compatibility strategy.
    pub strategy: VersionStrategy,
    /// Optional protocol version requirement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<String>,
    /// Optional host version range.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_version: Option<VersionRange>,
    /// Explicit tested host/version pairs.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tested_hosts: Vec<TestedHost>,
    /// Additional compatibility notes.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

impl Default for CompatibilityContract {
    fn default() -> Self {
        Self {
            strategy: VersionStrategy::Semver,
            protocol_version: None,
            host_version: None,
            tested_hosts: Vec::new(),
            notes: Vec::new(),
        }
    }
}

impl CompatibilityContract {
    /// Creates compatibility settings.
    pub fn new(strategy: VersionStrategy) -> Self {
        Self {
            strategy,
            ..Self::default()
        }
    }

    /// Sets protocol version.
    pub fn with_protocol_version(mut self, protocol_version: impl Into<String>) -> Self {
        self.protocol_version = Some(protocol_version.into());
        self
    }

    /// Sets host version range.
    pub fn with_host_version(mut self, host_version: VersionRange) -> Self {
        self.host_version = Some(host_version);
        self
    }

    /// Sets tested host list.
    pub fn with_tested_hosts(mut self, tested_hosts: Vec<TestedHost>) -> Self {
        self.tested_hosts = tested_hosts;
        self
    }

    /// Sets compatibility notes.
    pub fn with_notes(mut self, notes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.notes = notes.into_iter().map(Into::into).collect();
        self
    }
}

/// Inclusive host version range.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct VersionRange {
    /// Inclusive minimum version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<String>,
    /// Inclusive maximum version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<String>,
}

impl VersionRange {
    /// Creates an empty range.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets minimum version.
    pub fn with_minimum(mut self, minimum: impl Into<String>) -> Self {
        self.minimum = Some(minimum.into());
        self
    }

    /// Sets maximum version.
    pub fn with_maximum(mut self, maximum: impl Into<String>) -> Self {
        self.maximum = Some(maximum.into());
        self
    }
}

/// One tested host/version tuple.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TestedHost {
    /// Host kind tested.
    pub host: HostKind,
    /// Tested host version string.
    pub version: String,
    /// Optional test notes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl TestedHost {
    /// Creates a tested-host descriptor.
    pub fn new(host: HostKind, version: impl Into<String>) -> Self {
        Self {
            host,
            version: version.into(),
            notes: None,
        }
    }

    /// Sets optional notes.
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }
}

/// Trust and security posture metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TrustMetadata {
    /// Declared trust level.
    pub level: TrustLevel,
    /// Declared sandbox model.
    pub sandbox: SandboxLevel,
    /// Declared network policy.
    pub network: NetworkAccess,
    /// Whether behavior is expected to be deterministic.
    pub deterministic: bool,
    /// Whether behavior is expected to stay local.
    pub local_only: bool,
    /// Required permissions.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<PermissionDescriptor>,
    /// Named data domains touched by the plugin.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub data_access: Vec<String>,
    /// Optional provenance statement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<String>,
    /// Additional trust notes.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

impl Default for TrustMetadata {
    fn default() -> Self {
        Self {
            level: TrustLevel::Reviewed,
            sandbox: SandboxLevel::Wasm,
            network: NetworkAccess::None,
            deterministic: true,
            local_only: true,
            permissions: Vec::new(),
            data_access: Vec::new(),
            provenance: None,
            notes: Vec::new(),
        }
    }
}

impl TrustMetadata {
    /// Creates trust metadata with defaults for unspecified fields.
    pub fn new(level: TrustLevel, sandbox: SandboxLevel, network: NetworkAccess) -> Self {
        Self {
            level,
            sandbox,
            network,
            ..Self::default()
        }
    }

    /// Sets data access declarations.
    pub fn with_data_access(
        mut self,
        data_access: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.data_access = data_access.into_iter().map(Into::into).collect();
        self
    }

    /// Sets permissions.
    pub fn with_permissions(
        mut self,
        permissions: impl IntoIterator<Item = PermissionDescriptor>,
    ) -> Self {
        self.permissions = permissions.into_iter().collect();
        self
    }

    /// Sets provenance metadata.
    pub fn with_provenance(mut self, provenance: impl Into<String>) -> Self {
        self.provenance = Some(provenance.into());
        self
    }

    /// Sets notes.
    pub fn with_notes(mut self, notes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.notes = notes.into_iter().map(Into::into).collect();
        self
    }

    /// Sets deterministic flag.
    pub fn with_deterministic(mut self, deterministic: bool) -> Self {
        self.deterministic = deterministic;
        self
    }

    /// Sets local-only flag.
    pub fn with_local_only(mut self, local_only: bool) -> Self {
        self.local_only = local_only;
        self
    }
}

/// Declared lifecycle behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LifecycleContract {
    /// Current lifecycle state.
    pub state: LifecycleState,
    /// Whether plugin is stateless across invocations.
    pub stateless: bool,
    /// Whether explicit shutdown handling is required.
    pub requires_explicit_shutdown: bool,
    /// Supported lifecycle hooks.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub hooks: Vec<LifecycleHook>,
    /// Optional health probe hook/action reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_probe: Option<String>,
    /// Additional lifecycle notes.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

impl Default for LifecycleContract {
    fn default() -> Self {
        Self {
            state: LifecycleState::Ready,
            stateless: true,
            requires_explicit_shutdown: false,
            hooks: vec![LifecycleHook::Invoke],
            health_probe: None,
            notes: Vec::new(),
        }
    }
}

impl LifecycleContract {
    /// Creates lifecycle metadata with defaults.
    pub fn new(state: LifecycleState) -> Self {
        Self {
            state,
            ..Self::default()
        }
    }

    /// Sets supported hooks.
    pub fn with_hooks(mut self, hooks: Vec<LifecycleHook>) -> Self {
        self.hooks = hooks;
        self
    }

    /// Sets health probe identifier.
    pub fn with_health_probe(mut self, health_probe: impl Into<String>) -> Self {
        self.health_probe = Some(health_probe.into());
        self
    }

    /// Sets notes.
    pub fn with_notes(mut self, notes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.notes = notes.into_iter().map(Into::into).collect();
        self
    }

    /// Sets stateless behavior flag.
    pub fn with_stateless(mut self, stateless: bool) -> Self {
        self.stateless = stateless;
        self
    }

    /// Sets explicit shutdown requirement flag.
    pub fn with_explicit_shutdown(mut self, requires_explicit_shutdown: bool) -> Self {
        self.requires_explicit_shutdown = requires_explicit_shutdown;
        self
    }
}

/// Global execution defaults and behavior guarantees.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ExecutionContract {
    /// Default execution mode.
    pub default_mode: ExecutionMode,
    /// Whether async execution is supported.
    pub supports_async: bool,
    /// Whether invocations can be cancelled.
    pub cancellable: bool,
    /// Whether operations are idempotent.
    pub idempotent: bool,
    /// Whether progress reporting is supported.
    pub progress_reporting: bool,
    /// Optional timeout budget.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    /// Optional maximum concurrent jobs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_concurrency: Option<u32>,
    /// Optional async metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub async_metadata: Option<AsyncMetadata>,
    /// Additional notes.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

impl Default for ExecutionContract {
    fn default() -> Self {
        Self {
            default_mode: ExecutionMode::Sync,
            supports_async: false,
            cancellable: false,
            idempotent: true,
            progress_reporting: false,
            timeout_ms: None,
            max_concurrency: None,
            async_metadata: None,
            notes: Vec::new(),
        }
    }
}

impl ExecutionContract {
    /// Creates an execution contract.
    pub fn new(default_mode: ExecutionMode) -> Self {
        Self {
            default_mode,
            ..Self::default()
        }
    }

    /// Sets async support flag.
    pub fn with_async_support(mut self, supports_async: bool) -> Self {
        self.supports_async = supports_async;
        self
    }

    /// Sets cancellable flag.
    pub fn with_cancellable(mut self, cancellable: bool) -> Self {
        self.cancellable = cancellable;
        self
    }

    /// Sets idempotent flag.
    pub fn with_idempotent(mut self, idempotent: bool) -> Self {
        self.idempotent = idempotent;
        self
    }

    /// Sets progress reporting flag.
    pub fn with_progress_reporting(mut self, progress_reporting: bool) -> Self {
        self.progress_reporting = progress_reporting;
        self
    }

    /// Sets timeout in milliseconds.
    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    /// Sets max concurrency.
    pub fn with_max_concurrency(mut self, max_concurrency: u32) -> Self {
        self.max_concurrency = Some(max_concurrency);
        self
    }

    /// Sets async metadata.
    pub fn with_async_metadata(mut self, async_metadata: AsyncMetadata) -> Self {
        self.async_metadata = Some(async_metadata);
        self
    }

    /// Sets notes.
    pub fn with_notes(mut self, notes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.notes = notes.into_iter().map(Into::into).collect();
        self
    }
}

/// Capability negotiation contract at plugin level.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CapabilityContract {
    /// Required capabilities.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<CapabilityRequirement>,
    /// Optional capabilities.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub optional: Vec<CapabilityRequirement>,
    /// Additional capability constraints.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constraints: Option<CapabilityConstraints>,
    /// Feature degradation policy.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub degradation: Vec<DegradationRule>,
    /// Additional notes.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

impl CapabilityContract {
    /// Creates an empty capability contract.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets required capabilities.
    pub fn with_required(mut self, required: Vec<CapabilityRequirement>) -> Self {
        self.required = required;
        self
    }

    /// Sets optional capabilities.
    pub fn with_optional(mut self, optional: Vec<CapabilityRequirement>) -> Self {
        self.optional = optional;
        self
    }

    /// Sets constraints.
    pub fn with_constraints(mut self, constraints: CapabilityConstraints) -> Self {
        self.constraints = Some(constraints);
        self
    }

    /// Sets degradation rules.
    pub fn with_degradation(mut self, degradation: Vec<DegradationRule>) -> Self {
        self.degradation = degradation;
        self
    }

    /// Sets notes.
    pub fn with_notes(mut self, notes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.notes = notes.into_iter().map(Into::into).collect();
        self
    }
}

/// Degradation behavior for one feature when capabilities are missing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DegradationRule {
    /// Feature identifier.
    pub feature: String,
    /// Behavior applied when degraded.
    pub behavior: String,
    /// Severity classification.
    pub severity: DegradationSeverity,
    /// Capability keys whose absence triggers this rule.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub when_missing: Vec<String>,
}

impl DegradationRule {
    /// Creates a degradation rule.
    pub fn new(
        feature: impl Into<String>,
        behavior: impl Into<String>,
        severity: DegradationSeverity,
    ) -> Self {
        Self {
            feature: feature.into(),
            behavior: behavior.into(),
            severity,
            when_missing: Vec::new(),
        }
    }

    /// Sets capability keys that trigger this degradation.
    pub fn when_missing(
        mut self,
        when_missing: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.when_missing = when_missing.into_iter().map(Into::into).collect();
        self
    }
}
