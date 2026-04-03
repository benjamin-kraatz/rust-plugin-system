//! Deterministic fixtures and builders for plugin-system tests.
//!
//! This crate is intended for `dev-dependencies` and integration tests. It
//! provides:
//!
//! - Builders for manifest, request, response, and context protocol types
//! - Assertion helpers for common test expectations
//! - Packaging fixture utilities for file-layout tests

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[doc(inline)]
pub use plugin_capabilities::{Capability, HostKind, PluginArchitecture, SkillLevel};
#[doc(inline)]
pub use plugin_manifest::{PluginAction, PluginManifest};
#[doc(inline)]
pub use plugin_protocol::{
    InvocationContext, OutputBlock, OutputKind, PluginRequest, PluginResponse, RuntimeContext,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Current package manifest schema version used by test fixtures.
pub const PACKAGE_SCHEMA_VERSION: u32 = 1;

/// Builder for [`PluginAction`] test fixtures.
#[derive(Debug, Clone)]
pub struct ActionBuilder {
    action: PluginAction,
}

impl ActionBuilder {
    /// Creates a default action fixture.
    pub fn new() -> Self {
        Self {
            action: PluginAction::new(
                "run-demo",
                "Run demo",
                "Exercise a deterministic plugin action.",
            )
            .with_payload_hint(r#"{"input":"fixture"}"#),
        }
    }

    /// Sets action id.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.action.id = id.into();
        self
    }

    /// Sets action label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.action.label = label.into();
        self
    }

    /// Sets action description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.action.description = description.into();
        self
    }

    /// Sets payload hint.
    pub fn payload_hint(mut self, payload_hint: impl Into<String>) -> Self {
        self.action.payload_hint = Some(payload_hint.into());
        self
    }

    /// Removes payload hint.
    pub fn without_payload_hint(mut self) -> Self {
        self.action.payload_hint = None;
        self
    }

    /// Finalizes and returns action.
    pub fn build(self) -> PluginAction {
        self.action
    }
}

impl Default for ActionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for [`PluginManifest`] test fixtures.
#[derive(Debug, Clone)]
pub struct ManifestBuilder {
    manifest: PluginManifest,
}

impl ManifestBuilder {
    /// Creates a default deterministic manifest fixture.
    ///
    /// # Examples
    ///
    /// ```
    /// use plugin_test_kit::ManifestBuilder;
    ///
    /// let manifest = ManifestBuilder::new().id("demo").name("Demo").build();
    /// assert_eq!(manifest.id, "demo");
    /// ```
    pub fn new() -> Self {
        Self {
            manifest: PluginManifest::new(
                "packaging-fixture",
                "Packaging Fixture",
                "0.1.0",
                "A deterministic manifest for shared crate tests.",
                PluginArchitecture::NativeJson,
                SkillLevel::Intermediate,
            )
            .with_supported_hosts(vec![HostKind::Cli, HostKind::Service])
            .with_capabilities(vec![Capability::new(
                "bundle.plan",
                "Produces local bundle metadata for tests.",
            )])
            .with_tags(["testing", "packaging"])
            .with_notes([
                "Used by shared crate tests.",
                "Keeps manifest and protocol fixtures aligned.",
            ]),
        }
    }

    /// Sets manifest id.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.manifest.id = id.into();
        self
    }

    /// Sets manifest name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.manifest.name = name.into();
        self
    }

    /// Sets version.
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.manifest.version = version.into();
        self
    }

    /// Sets description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.manifest.description = description.into();
        self
    }

    /// Sets architecture.
    pub fn architecture(mut self, architecture: PluginArchitecture) -> Self {
        self.manifest.architecture = architecture;
        self
    }

    /// Sets skill level.
    pub fn skill_level(mut self, skill_level: SkillLevel) -> Self {
        self.manifest.skill_level = skill_level;
        self
    }

    /// Sets supported hosts.
    pub fn supported_hosts<I>(mut self, supported_hosts: I) -> Self
    where
        I: IntoIterator<Item = HostKind>,
    {
        self.manifest.supported_hosts = supported_hosts.into_iter().collect();
        self
    }

    /// Sets capabilities.
    pub fn capabilities<I>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = Capability>,
    {
        self.manifest.capabilities = capabilities.into_iter().collect();
        self
    }

    /// Sets tags.
    pub fn tags<I, S>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.manifest.tags = tags.into_iter().map(Into::into).collect();
        self
    }

    /// Sets notes.
    pub fn notes<I, S>(mut self, notes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.manifest.notes = notes.into_iter().map(Into::into).collect();
        self
    }

    /// Appends one action.
    pub fn add_action(mut self, action: PluginAction) -> Self {
        self.manifest.actions.push(action);
        self
    }

    /// Replaces all actions.
    pub fn actions<I>(mut self, actions: I) -> Self
    where
        I: IntoIterator<Item = PluginAction>,
    {
        self.manifest.actions = actions.into_iter().collect();
        self
    }

    /// Finalizes and returns manifest.
    pub fn build(self) -> PluginManifest {
        self.manifest
    }
}

impl Default for ManifestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for [`InvocationContext`] fixtures.
#[derive(Debug, Clone)]
pub struct ContextBuilder {
    context: InvocationContext,
}

impl ContextBuilder {
    /// Creates a default CLI context fixture.
    pub fn new() -> Self {
        let mut context = InvocationContext::for_host(HostKind::Cli);
        context.workspace_root = Some("/workspace/playground".to_owned());
        context.plugin_dir = Some("/workspace/playground/target/debug".to_owned());
        context.mode = Some("test".to_owned());
        context.runtime = Some(RuntimeContext::default());

        Self { context }
    }

    /// Sets host kind.
    pub fn host(mut self, host: HostKind) -> Self {
        self.context.host = host;
        self
    }

    /// Sets workspace root.
    pub fn workspace_root(mut self, workspace_root: impl Into<String>) -> Self {
        self.context.workspace_root = Some(workspace_root.into());
        self
    }

    /// Removes workspace root.
    pub fn without_workspace_root(mut self) -> Self {
        self.context.workspace_root = None;
        self
    }

    /// Sets plugin directory.
    pub fn plugin_dir(mut self, plugin_dir: impl Into<String>) -> Self {
        self.context.plugin_dir = Some(plugin_dir.into());
        self
    }

    /// Removes plugin directory.
    pub fn without_plugin_dir(mut self) -> Self {
        self.context.plugin_dir = None;
        self
    }

    /// Sets mode string.
    pub fn mode(mut self, mode: impl Into<String>) -> Self {
        self.context.mode = Some(mode.into());
        self
    }

    /// Removes mode string.
    pub fn without_mode(mut self) -> Self {
        self.context.mode = None;
        self
    }

    /// Finalizes and returns context.
    pub fn build(self) -> InvocationContext {
        self.context
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for [`PluginRequest`] fixtures.
#[derive(Debug, Clone)]
pub struct RequestBuilder {
    request: PluginRequest,
}

impl RequestBuilder {
    /// Creates a default request fixture.
    ///
    /// # Examples
    ///
    /// ```
    /// use plugin_test_kit::RequestBuilder;
    ///
    /// let request = RequestBuilder::new().plugin_id("demo").build();
    /// assert_eq!(request.plugin_id, "demo");
    /// ```
    pub fn new() -> Self {
        Self {
            request: PluginRequest {
                plugin_id: "packaging-fixture".to_owned(),
                action_id: "run-demo".to_owned(),
                payload: json!({"input":"fixture","channel":"stable"}),
                context: ContextBuilder::new().build(),
            },
        }
    }

    /// Sets plugin id.
    pub fn plugin_id(mut self, plugin_id: impl Into<String>) -> Self {
        self.request.plugin_id = plugin_id.into();
        self
    }

    /// Sets action id.
    pub fn action_id(mut self, action_id: impl Into<String>) -> Self {
        self.request.action_id = action_id.into();
        self
    }

    /// Sets payload JSON.
    pub fn payload(mut self, payload: Value) -> Self {
        self.request.payload = payload;
        self
    }

    /// Sets full context.
    pub fn context(mut self, context: InvocationContext) -> Self {
        self.request.context = context;
        self
    }

    /// Sets host kind on embedded context.
    pub fn host(mut self, host: HostKind) -> Self {
        self.request.context.host = host;
        self
    }

    /// Finalizes and returns request.
    pub fn build(self) -> PluginRequest {
        self.request
    }
}

impl Default for RequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for [`PluginResponse`] fixtures.
#[derive(Debug, Clone)]
pub struct ResponseBuilder {
    response: PluginResponse,
}

impl ResponseBuilder {
    /// Creates a successful response fixture.
    pub fn ok() -> Self {
        Self {
            response: PluginResponse::ok(
                "packaging-fixture",
                "run-demo",
                "Fixture response",
                "Prepared deterministic fixture output.",
            ),
        }
    }

    /// Creates an error response fixture.
    pub fn error() -> Self {
        Self {
            response: PluginResponse::error(
                "packaging-fixture",
                "run-demo",
                "Fixture error",
                "Fixture validation failed.",
            ),
        }
    }

    /// Sets plugin id.
    pub fn plugin_id(mut self, plugin_id: impl Into<String>) -> Self {
        self.response.plugin_id = plugin_id.into();
        self
    }

    /// Sets action id.
    pub fn action_id(mut self, action_id: impl Into<String>) -> Self {
        self.response.action_id = action_id.into();
        self
    }

    /// Sets title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.response.title = title.into();
        self
    }

    /// Sets summary.
    pub fn summary(mut self, summary: impl Into<String>) -> Self {
        self.response.summary = summary.into();
        self
    }

    /// Adds one titled output block.
    pub fn add_output(
        mut self,
        kind: OutputKind,
        title: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        self.response.outputs.push(OutputBlock {
            kind,
            title: Some(title.into()),
            body: body.into(),
        });
        self
    }

    /// Adds one untitled output block.
    pub fn add_untitled_output(mut self, kind: OutputKind, body: impl Into<String>) -> Self {
        self.response.outputs.push(OutputBlock {
            kind,
            title: None,
            body: body.into(),
        });
        self
    }

    /// Adds one next-step message.
    pub fn add_next_step(mut self, next_step: impl Into<String>) -> Self {
        self.response.suggested_next_steps.push(next_step.into());
        self
    }

    /// Finalizes and returns response.
    pub fn build(self) -> PluginResponse {
        self.response
    }
}

/// Returns a deterministic two-action manifest fixture.
pub fn sample_manifest() -> PluginManifest {
    ManifestBuilder::new()
        .add_action(
            ActionBuilder::new()
                .id("run-demo")
                .label("Run demo")
                .description("Generate a deterministic fixture payload.")
                .payload_hint(r#"{"input":"fixture","channel":"stable"}"#)
                .build(),
        )
        .add_action(
            ActionBuilder::new()
                .id("verify-bundle")
                .label("Verify bundle")
                .description("Check a local bundle layout.")
                .payload_hint(r#"{"bundle_root":"./bundle"}"#)
                .build(),
        )
        .build()
}

/// Returns a default request fixture.
pub fn sample_request() -> PluginRequest {
    RequestBuilder::new().build()
}

/// Returns a default successful response fixture.
pub fn sample_response() -> PluginResponse {
    ResponseBuilder::ok()
        .add_output(OutputKind::Json, "Bundle plan", r#"{"channel":"stable"}"#)
        .add_next_step("Copy the compiled plugin library into the bundle.")
        .build()
}

/// Asserts that a manifest contains an action id.
pub fn assert_manifest_has_action(manifest: &PluginManifest, action_id: &str) {
    assert!(
        manifest.actions.iter().any(|action| action.id == action_id),
        "expected manifest '{}' to contain action '{action_id}', available actions: {:?}",
        manifest.id,
        manifest
            .actions
            .iter()
            .map(|action| action.id.as_str())
            .collect::<Vec<_>>()
    );
}

/// Asserts that supported hosts match exactly.
pub fn assert_manifest_hosts(manifest: &PluginManifest, expected_hosts: &[HostKind]) {
    assert_eq!(
        manifest.supported_hosts, expected_hosts,
        "supported hosts for manifest '{}' did not match",
        manifest.id
    );
}

/// Asserts payload equality for a request.
pub fn assert_payload_eq(request: &PluginRequest, expected_payload: Value) {
    assert_eq!(
        request.payload, expected_payload,
        "payload for request '{}:{}' did not match",
        request.plugin_id, request.action_id
    );
}

/// Asserts response success.
pub fn assert_response_ok(response: &PluginResponse) {
    assert!(
        response.success,
        "expected response '{}:{}' to succeed, summary: {}",
        response.plugin_id, response.action_id, response.summary
    );
}

/// Asserts response failure.
pub fn assert_response_error(response: &PluginResponse) {
    assert!(
        !response.success,
        "expected response '{}:{}' to fail",
        response.plugin_id, response.action_id
    );
}

/// Asserts an output body contains a substring.
pub fn assert_output_contains(response: &PluginResponse, output_title: Option<&str>, needle: &str) {
    let matching_output =
        response
            .outputs
            .iter()
            .find(|output| match (output_title, &output.title) {
                (Some(expected_title), Some(actual_title)) => expected_title == actual_title,
                (None, None) => true,
                _ => false,
            });

    let output = matching_output.unwrap_or_else(|| {
        panic!(
            "expected response '{}:{}' to contain output {:?}, available outputs: {:?}",
            response.plugin_id,
            response.action_id,
            output_title,
            response
                .outputs
                .iter()
                .map(|output| output.title.as_deref().unwrap_or("<untitled>"))
                .collect::<Vec<_>>()
        )
    });

    assert!(
        output.body.contains(needle),
        "expected output {:?} to contain '{needle}', body was '{}'",
        output.title,
        output.body
    );
}

/// Asserts suggested next steps match exactly.
pub fn assert_next_steps(response: &PluginResponse, expected_steps: &[&str]) {
    let actual_steps = response
        .suggested_next_steps
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert_eq!(
        actual_steps, expected_steps,
        "next steps for response '{}:{}' did not match",
        response.plugin_id, response.action_id
    );
}

/// Runtime packaging mode for bundle fixtures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PackageRuntime {
    /// Native JSON dynamic library package.
    NativeJson,
    /// ABI-stable dynamic library package.
    AbiStable,
    /// Wasm package.
    Wasm,
}

/// Artifact category for package manifests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactKind {
    /// Native/ABI dynamic library artifact.
    DynamicLibrary,
    /// Wasm module artifact.
    WasmModule,
    /// Embedded plugin manifest snapshot.
    ManifestSnapshot,
    /// Release metadata document.
    ReleaseMetadata,
}

/// One artifact entry in a package manifest fixture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactManifest {
    /// Relative artifact path.
    pub path: String,
    /// Artifact category.
    pub kind: ArtifactKind,
    /// Whether artifact must exist for validation.
    pub required: bool,
}

/// Release metadata fixture attached to package manifests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseMetadata {
    /// Release channel.
    pub channel: String,
    /// Build target triple.
    pub target: String,
    /// Supported hosts for this package release.
    pub hosts: Vec<HostKind>,
    /// Optional installation hint text.
    pub installer_hint: Option<String>,
}

impl ReleaseMetadata {
    /// Creates release metadata.
    pub fn new(
        channel: impl Into<String>,
        target: impl Into<String>,
        hosts: Vec<HostKind>,
    ) -> Self {
        Self {
            channel: channel.into(),
            target: target.into(),
            hosts,
            installer_hint: None,
        }
    }

    /// Sets installation hint.
    pub fn with_installer_hint(mut self, installer_hint: impl Into<String>) -> Self {
        self.installer_hint = Some(installer_hint.into());
        self
    }
}

/// On-disk package manifest fixture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageManifest {
    /// Schema version.
    pub schema_version: u32,
    /// Package name.
    pub package_name: String,
    /// Runtime packaging mode.
    pub runtime: PackageRuntime,
    /// Entrypoint artifact path.
    pub entrypoint: String,
    /// Embedded plugin manifest.
    pub plugin: PluginManifest,
    /// Declared artifacts.
    pub artifacts: Vec<ArtifactManifest>,
    /// Release metadata.
    pub release: ReleaseMetadata,
}

impl PackageManifest {
    /// Returns canonical manifest snapshot file name for this runtime mode.
    pub fn manifest_file_name(&self) -> &'static str {
        match self.runtime {
            PackageRuntime::Wasm => "wasm-plugin.json",
            PackageRuntime::NativeJson | PackageRuntime::AbiStable => "plugin-manifest.json",
        }
    }

    /// Returns required artifact paths that do not exist under `root`.
    pub fn missing_required_artifacts(&self, root: impl AsRef<Path>) -> Vec<PathBuf> {
        self.artifacts
            .iter()
            .filter(|artifact| artifact.required)
            .map(|artifact| root.as_ref().join(&artifact.path))
            .filter(|path| !path.exists())
            .collect()
    }
}

/// File-system package fixture builder and writer.
#[derive(Debug, Clone)]
pub struct PackageFixture {
    manifest: PackageManifest,
    extra_files: BTreeMap<PathBuf, String>,
}

impl PackageFixture {
    /// Creates a native JSON package fixture.
    pub fn native_json(
        package_name: impl Into<String>,
        plugin: PluginManifest,
        entrypoint: impl Into<String>,
        release: ReleaseMetadata,
    ) -> Self {
        Self::new(
            package_name,
            PackageRuntime::NativeJson,
            plugin,
            entrypoint,
            ArtifactKind::DynamicLibrary,
            release,
        )
    }

    /// Creates an ABI-stable package fixture.
    pub fn abi_stable(
        package_name: impl Into<String>,
        plugin: PluginManifest,
        entrypoint: impl Into<String>,
        release: ReleaseMetadata,
    ) -> Self {
        Self::new(
            package_name,
            PackageRuntime::AbiStable,
            plugin,
            entrypoint,
            ArtifactKind::DynamicLibrary,
            release,
        )
    }

    /// Creates a Wasm package fixture.
    pub fn wasm(
        package_name: impl Into<String>,
        plugin: PluginManifest,
        entrypoint: impl Into<String>,
        release: ReleaseMetadata,
    ) -> Self {
        Self::new(
            package_name,
            PackageRuntime::Wasm,
            plugin,
            entrypoint,
            ArtifactKind::WasmModule,
            release,
        )
    }

    fn new(
        package_name: impl Into<String>,
        runtime: PackageRuntime,
        plugin: PluginManifest,
        entrypoint: impl Into<String>,
        entrypoint_kind: ArtifactKind,
        release: ReleaseMetadata,
    ) -> Self {
        let entrypoint = entrypoint.into();
        let mut manifest = PackageManifest {
            schema_version: PACKAGE_SCHEMA_VERSION,
            package_name: package_name.into(),
            runtime,
            entrypoint: entrypoint.clone(),
            plugin,
            artifacts: vec![ArtifactManifest {
                path: entrypoint,
                kind: entrypoint_kind,
                required: false,
            }],
            release,
        };
        manifest.artifacts.push(ArtifactManifest {
            path: manifest.manifest_file_name().to_owned(),
            kind: ArtifactKind::ManifestSnapshot,
            required: true,
        });
        manifest.artifacts.push(ArtifactManifest {
            path: "release.json".to_owned(),
            kind: ArtifactKind::ReleaseMetadata,
            required: true,
        });

        Self {
            manifest,
            extra_files: BTreeMap::new(),
        }
    }

    /// Marks the entrypoint artifact as required.
    pub fn with_required_entrypoint(mut self) -> Self {
        if let Some(artifact) = self
            .manifest
            .artifacts
            .iter_mut()
            .find(|artifact| artifact.path == self.manifest.entrypoint)
        {
            artifact.required = true;
        }
        self
    }

    /// Adds an additional text file to the fixture output.
    pub fn with_text_file(
        mut self,
        relative_path: impl Into<PathBuf>,
        contents: impl Into<String>,
    ) -> Self {
        self.extra_files
            .insert(relative_path.into(), contents.into());
        self
    }

    /// Writes fixture files to disk and returns discovered output paths.
    pub fn write_to(&self, root: impl AsRef<Path>) -> io::Result<WrittenPackageFixture> {
        let root = root.as_ref();
        fs::create_dir_all(root)?;

        let package_manifest_path = root.join("package.json");
        let manifest_snapshot_path = root.join(self.manifest.manifest_file_name());
        let release_metadata_path = root.join("release.json");

        fs::write(
            &package_manifest_path,
            serde_json::to_string_pretty(&self.manifest).map_err(io::Error::other)?,
        )?;
        fs::write(
            &manifest_snapshot_path,
            serde_json::to_string_pretty(&self.manifest.plugin).map_err(io::Error::other)?,
        )?;
        fs::write(
            &release_metadata_path,
            serde_json::to_string_pretty(&self.manifest.release).map_err(io::Error::other)?,
        )?;

        for (relative_path, contents) in &self.extra_files {
            let path = root.join(relative_path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, contents)?;
        }

        Ok(WrittenPackageFixture {
            root: root.to_path_buf(),
            package_manifest_path,
            manifest_snapshot_path,
            release_metadata_path,
        })
    }
}

/// Paths produced when a [`PackageFixture`] is written.
#[derive(Debug, Clone)]
pub struct WrittenPackageFixture {
    /// Root output directory.
    pub root: PathBuf,
    /// Path to `package.json`.
    pub package_manifest_path: PathBuf,
    /// Path to plugin manifest snapshot file.
    pub manifest_snapshot_path: PathBuf,
    /// Path to `release.json`.
    pub release_metadata_path: PathBuf,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn builders_create_deterministic_protocol_fixtures() {
        let manifest = sample_manifest();
        let request = RequestBuilder::new()
            .plugin_id("fixture-plugin")
            .action_id("verify")
            .payload(json!({"bundle":"native-json"}))
            .host(HostKind::Service)
            .build();
        let response = ResponseBuilder::ok()
            .plugin_id("fixture-plugin")
            .action_id("verify")
            .summary("Verified bundle layout.")
            .add_output(OutputKind::Json, "Bundle", r#"{"ok":true}"#)
            .add_next_step("Publish the package metadata.")
            .build();

        assert_manifest_has_action(&manifest, "verify-bundle");
        assert_payload_eq(&request, json!({"bundle":"native-json"}));
        assert_response_ok(&response);
        assert_output_contains(&response, Some("Bundle"), "\"ok\":true");
        assert_next_steps(&response, &["Publish the package metadata."]);
    }

    #[test]
    fn package_fixture_writes_expected_bundle_layout() {
        let output_root = workspace_root().join("target/plugin-test-kit/package-fixture");
        let _ = fs::remove_dir_all(&output_root);

        let fixture = PackageFixture::native_json(
            "fixture-plugin-macos",
            sample_manifest(),
            "dist/libfixture_plugin.dylib",
            ReleaseMetadata::new(
                "stable",
                "x86_64-apple-darwin",
                vec![HostKind::Cli, HostKind::Service],
            )
            .with_installer_hint("Point RUST_PLUGIN_SYSTEM_PLUGIN_DIR at the lib directory."),
        )
        .with_required_entrypoint()
        .with_text_file("dist/libfixture_plugin.dylib", "fixture binary placeholder");

        let written = fixture.write_to(&output_root).unwrap();
        let package_manifest = read_package_manifest(&written.package_manifest_path);

        assert_eq!(package_manifest.package_name, "fixture-plugin-macos");
        assert!(
            package_manifest
                .missing_required_artifacts(&written.root)
                .is_empty()
        );
        assert!(written.manifest_snapshot_path.exists());
        assert!(written.release_metadata_path.exists());
    }

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    fn read_package_manifest(path: impl AsRef<Path>) -> PackageManifest {
        let manifest_json = fs::read_to_string(path).unwrap();
        serde_json::from_str(&manifest_json).unwrap()
    }
}
