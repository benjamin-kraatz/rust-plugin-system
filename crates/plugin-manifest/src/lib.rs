pub use plugin_capabilities::{
    AsyncMetadata, Capability, CapabilityConstraints, CapabilityRequirement, DegradationSeverity,
    ExecutionMode, HostKind, LifecycleHook, LifecycleState, MaintenanceStatus, NetworkAccess,
    PermissionDescriptor, PluginArchitecture, SandboxLevel, SkillLevel, TrustLevel,
    VersionStrategy,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ActionContract {
    pub execution_mode: ExecutionMode,
    pub idempotent: bool,
    pub mutates_workspace: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub async_metadata: Option<AsyncMetadata>,
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
    pub fn new(execution_mode: ExecutionMode) -> Self {
        Self {
            execution_mode,
            ..Self::default()
        }
    }

    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    pub fn with_async_metadata(mut self, async_metadata: AsyncMetadata) -> Self {
        self.async_metadata = Some(async_metadata);
        self
    }

    pub fn with_constraints(mut self, constraints: CapabilityConstraints) -> Self {
        self.constraints = Some(constraints);
        self
    }

    pub fn with_idempotent(mut self, idempotent: bool) -> Self {
        self.idempotent = idempotent;
        self
    }

    pub fn with_workspace_mutation(mut self, mutates_workspace: bool) -> Self {
        self.mutates_workspace = mutates_workspace;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SchemaDescriptor {
    pub format: String,
    pub reference: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl SchemaDescriptor {
    pub fn new(format: impl Into<String>, reference: impl Into<String>) -> Self {
        Self {
            format: format.into(),
            reference: reference.into(),
            version: None,
        }
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DeprecationNotice {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replacement: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removal_target: Option<String>,
}

impl DeprecationNotice {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            since: None,
            replacement: None,
            message: Some(message.into()),
            removal_target: None,
        }
    }

    pub fn with_since(mut self, since: impl Into<String>) -> Self {
        self.since = Some(since.into());
        self
    }

    pub fn with_replacement(mut self, replacement: impl Into<String>) -> Self {
        self.replacement = Some(replacement.into());
        self
    }

    pub fn with_removal_target(mut self, removal_target: impl Into<String>) -> Self {
        self.removal_target = Some(removal_target.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginAction {
    pub id: String,
    pub label: String,
    pub description: String,
    pub payload_hint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract: Option<ActionContract>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<SchemaDescriptor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<SchemaDescriptor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deprecation: Option<DeprecationNotice>,
}

impl PluginAction {
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

    pub fn with_payload_hint(mut self, payload_hint: impl Into<String>) -> Self {
        self.payload_hint = Some(payload_hint.into());
        self
    }

    pub fn with_contract(mut self, contract: ActionContract) -> Self {
        self.contract = Some(contract);
        self
    }

    pub fn with_input_schema(mut self, input_schema: SchemaDescriptor) -> Self {
        self.input_schema = Some(input_schema);
        self
    }

    pub fn with_output_schema(mut self, output_schema: SchemaDescriptor) -> Self {
        self.output_schema = Some(output_schema);
        self
    }

    pub fn with_deprecation(mut self, deprecation: DeprecationNotice) -> Self {
        self.deprecation = Some(deprecation);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub architecture: PluginArchitecture,
    pub skill_level: SkillLevel,
    pub supported_hosts: Vec<HostKind>,
    pub capabilities: Vec<Capability>,
    pub tags: Vec<String>,
    pub actions: Vec<PluginAction>,
    pub notes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maintenance: Option<MaintenanceContract>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<CompatibilityContract>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trust: Option<TrustMetadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<LifecycleContract>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: Option<ExecutionContract>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_contract: Option<CapabilityContract>,
}

impl PluginManifest {
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

    pub fn with_supported_hosts(mut self, supported_hosts: Vec<HostKind>) -> Self {
        self.supported_hosts = supported_hosts;
        self
    }

    pub fn with_capabilities(mut self, capabilities: Vec<Capability>) -> Self {
        self.capabilities = capabilities;
        self
    }

    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags = tags.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_actions(mut self, actions: Vec<PluginAction>) -> Self {
        self.actions = actions;
        self
    }

    pub fn with_notes(mut self, notes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.notes = notes.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_maintenance(mut self, maintenance: MaintenanceContract) -> Self {
        self.maintenance = Some(maintenance);
        self
    }

    pub fn with_compatibility(mut self, compatibility: CompatibilityContract) -> Self {
        self.compatibility = Some(compatibility);
        self
    }

    pub fn with_trust(mut self, trust: TrustMetadata) -> Self {
        self.trust = Some(trust);
        self
    }

    pub fn with_lifecycle(mut self, lifecycle: LifecycleContract) -> Self {
        self.lifecycle = Some(lifecycle);
        self
    }

    pub fn with_execution(mut self, execution: ExecutionContract) -> Self {
        self.execution = Some(execution);
        self
    }

    pub fn with_capability_contract(mut self, capability_contract: CapabilityContract) -> Self {
        self.capability_contract = Some(capability_contract);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MaintenanceContract {
    pub status: MaintenanceStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub support_tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
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
    pub fn new(status: MaintenanceStatus) -> Self {
        Self {
            status,
            ..Self::default()
        }
    }

    pub fn with_owner(mut self, owner: impl Into<String>) -> Self {
        self.owner = Some(owner.into());
        self
    }

    pub fn with_support_tier(mut self, support_tier: impl Into<String>) -> Self {
        self.support_tier = Some(support_tier.into());
        self
    }

    pub fn with_channel(mut self, channel: impl Into<String>) -> Self {
        self.channel = Some(channel.into());
        self
    }

    pub fn with_deprecation(mut self, deprecation: DeprecationNotice) -> Self {
        self.deprecation = Some(deprecation);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CompatibilityContract {
    pub strategy: VersionStrategy,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_version: Option<VersionRange>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tested_hosts: Vec<TestedHost>,
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
    pub fn new(strategy: VersionStrategy) -> Self {
        Self {
            strategy,
            ..Self::default()
        }
    }

    pub fn with_protocol_version(mut self, protocol_version: impl Into<String>) -> Self {
        self.protocol_version = Some(protocol_version.into());
        self
    }

    pub fn with_host_version(mut self, host_version: VersionRange) -> Self {
        self.host_version = Some(host_version);
        self
    }

    pub fn with_tested_hosts(mut self, tested_hosts: Vec<TestedHost>) -> Self {
        self.tested_hosts = tested_hosts;
        self
    }

    pub fn with_notes(mut self, notes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.notes = notes.into_iter().map(Into::into).collect();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct VersionRange {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<String>,
}

impl VersionRange {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_minimum(mut self, minimum: impl Into<String>) -> Self {
        self.minimum = Some(minimum.into());
        self
    }

    pub fn with_maximum(mut self, maximum: impl Into<String>) -> Self {
        self.maximum = Some(maximum.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TestedHost {
    pub host: HostKind,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl TestedHost {
    pub fn new(host: HostKind, version: impl Into<String>) -> Self {
        Self {
            host,
            version: version.into(),
            notes: None,
        }
    }

    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TrustMetadata {
    pub level: TrustLevel,
    pub sandbox: SandboxLevel,
    pub network: NetworkAccess,
    pub deterministic: bool,
    pub local_only: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<PermissionDescriptor>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub data_access: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<String>,
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
    pub fn new(level: TrustLevel, sandbox: SandboxLevel, network: NetworkAccess) -> Self {
        Self {
            level,
            sandbox,
            network,
            ..Self::default()
        }
    }

    pub fn with_data_access(
        mut self,
        data_access: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.data_access = data_access.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_permissions(
        mut self,
        permissions: impl IntoIterator<Item = PermissionDescriptor>,
    ) -> Self {
        self.permissions = permissions.into_iter().collect();
        self
    }

    pub fn with_provenance(mut self, provenance: impl Into<String>) -> Self {
        self.provenance = Some(provenance.into());
        self
    }

    pub fn with_notes(mut self, notes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.notes = notes.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_deterministic(mut self, deterministic: bool) -> Self {
        self.deterministic = deterministic;
        self
    }

    pub fn with_local_only(mut self, local_only: bool) -> Self {
        self.local_only = local_only;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LifecycleContract {
    pub state: LifecycleState,
    pub stateless: bool,
    pub requires_explicit_shutdown: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub hooks: Vec<LifecycleHook>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_probe: Option<String>,
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
    pub fn new(state: LifecycleState) -> Self {
        Self {
            state,
            ..Self::default()
        }
    }

    pub fn with_hooks(mut self, hooks: Vec<LifecycleHook>) -> Self {
        self.hooks = hooks;
        self
    }

    pub fn with_health_probe(mut self, health_probe: impl Into<String>) -> Self {
        self.health_probe = Some(health_probe.into());
        self
    }

    pub fn with_notes(mut self, notes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.notes = notes.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_stateless(mut self, stateless: bool) -> Self {
        self.stateless = stateless;
        self
    }

    pub fn with_explicit_shutdown(mut self, requires_explicit_shutdown: bool) -> Self {
        self.requires_explicit_shutdown = requires_explicit_shutdown;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ExecutionContract {
    pub default_mode: ExecutionMode,
    pub supports_async: bool,
    pub cancellable: bool,
    pub idempotent: bool,
    pub progress_reporting: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_concurrency: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub async_metadata: Option<AsyncMetadata>,
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
    pub fn new(default_mode: ExecutionMode) -> Self {
        Self {
            default_mode,
            ..Self::default()
        }
    }

    pub fn with_async_support(mut self, supports_async: bool) -> Self {
        self.supports_async = supports_async;
        self
    }

    pub fn with_cancellable(mut self, cancellable: bool) -> Self {
        self.cancellable = cancellable;
        self
    }

    pub fn with_idempotent(mut self, idempotent: bool) -> Self {
        self.idempotent = idempotent;
        self
    }

    pub fn with_progress_reporting(mut self, progress_reporting: bool) -> Self {
        self.progress_reporting = progress_reporting;
        self
    }

    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    pub fn with_max_concurrency(mut self, max_concurrency: u32) -> Self {
        self.max_concurrency = Some(max_concurrency);
        self
    }

    pub fn with_async_metadata(mut self, async_metadata: AsyncMetadata) -> Self {
        self.async_metadata = Some(async_metadata);
        self
    }

    pub fn with_notes(mut self, notes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.notes = notes.into_iter().map(Into::into).collect();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CapabilityContract {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<CapabilityRequirement>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub optional: Vec<CapabilityRequirement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constraints: Option<CapabilityConstraints>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub degradation: Vec<DegradationRule>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

impl CapabilityContract {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_required(mut self, required: Vec<CapabilityRequirement>) -> Self {
        self.required = required;
        self
    }

    pub fn with_optional(mut self, optional: Vec<CapabilityRequirement>) -> Self {
        self.optional = optional;
        self
    }

    pub fn with_constraints(mut self, constraints: CapabilityConstraints) -> Self {
        self.constraints = Some(constraints);
        self
    }

    pub fn with_degradation(mut self, degradation: Vec<DegradationRule>) -> Self {
        self.degradation = degradation;
        self
    }

    pub fn with_notes(mut self, notes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.notes = notes.into_iter().map(Into::into).collect();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DegradationRule {
    pub feature: String,
    pub behavior: String,
    pub severity: DegradationSeverity,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub when_missing: Vec<String>,
}

impl DegradationRule {
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

    pub fn when_missing(
        mut self,
        when_missing: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.when_missing = when_missing.into_iter().map(Into::into).collect();
        self
    }
}
