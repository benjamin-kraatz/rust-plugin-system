use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub use plugin_capabilities::{Capability, HostKind, PluginArchitecture, SkillLevel};
pub use plugin_manifest::{PluginAction, PluginManifest};
pub use plugin_protocol::{
    InvocationContext, OutputBlock, OutputKind, PluginRequest, PluginResponse, RuntimeContext,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const PACKAGE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone)]
pub struct ActionBuilder {
    action: PluginAction,
}

impl ActionBuilder {
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

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.action.id = id.into();
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.action.label = label.into();
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.action.description = description.into();
        self
    }

    pub fn payload_hint(mut self, payload_hint: impl Into<String>) -> Self {
        self.action.payload_hint = Some(payload_hint.into());
        self
    }

    pub fn without_payload_hint(mut self) -> Self {
        self.action.payload_hint = None;
        self
    }

    pub fn build(self) -> PluginAction {
        self.action
    }
}

impl Default for ActionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ManifestBuilder {
    manifest: PluginManifest,
}

impl ManifestBuilder {
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

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.manifest.id = id.into();
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.manifest.name = name.into();
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.manifest.version = version.into();
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.manifest.description = description.into();
        self
    }

    pub fn architecture(mut self, architecture: PluginArchitecture) -> Self {
        self.manifest.architecture = architecture;
        self
    }

    pub fn skill_level(mut self, skill_level: SkillLevel) -> Self {
        self.manifest.skill_level = skill_level;
        self
    }

    pub fn supported_hosts<I>(mut self, supported_hosts: I) -> Self
    where
        I: IntoIterator<Item = HostKind>,
    {
        self.manifest.supported_hosts = supported_hosts.into_iter().collect();
        self
    }

    pub fn capabilities<I>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = Capability>,
    {
        self.manifest.capabilities = capabilities.into_iter().collect();
        self
    }

    pub fn tags<I, S>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.manifest.tags = tags.into_iter().map(Into::into).collect();
        self
    }

    pub fn notes<I, S>(mut self, notes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.manifest.notes = notes.into_iter().map(Into::into).collect();
        self
    }

    pub fn add_action(mut self, action: PluginAction) -> Self {
        self.manifest.actions.push(action);
        self
    }

    pub fn actions<I>(mut self, actions: I) -> Self
    where
        I: IntoIterator<Item = PluginAction>,
    {
        self.manifest.actions = actions.into_iter().collect();
        self
    }

    pub fn build(self) -> PluginManifest {
        self.manifest
    }
}

impl Default for ManifestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ContextBuilder {
    context: InvocationContext,
}

impl ContextBuilder {
    pub fn new() -> Self {
        let mut context = InvocationContext::for_host(HostKind::Cli);
        context.workspace_root = Some("/workspace/playground".to_owned());
        context.plugin_dir = Some("/workspace/playground/target/debug".to_owned());
        context.mode = Some("test".to_owned());
        context.runtime = Some(RuntimeContext::default());

        Self { context }
    }

    pub fn host(mut self, host: HostKind) -> Self {
        self.context.host = host;
        self
    }

    pub fn workspace_root(mut self, workspace_root: impl Into<String>) -> Self {
        self.context.workspace_root = Some(workspace_root.into());
        self
    }

    pub fn without_workspace_root(mut self) -> Self {
        self.context.workspace_root = None;
        self
    }

    pub fn plugin_dir(mut self, plugin_dir: impl Into<String>) -> Self {
        self.context.plugin_dir = Some(plugin_dir.into());
        self
    }

    pub fn without_plugin_dir(mut self) -> Self {
        self.context.plugin_dir = None;
        self
    }

    pub fn mode(mut self, mode: impl Into<String>) -> Self {
        self.context.mode = Some(mode.into());
        self
    }

    pub fn without_mode(mut self) -> Self {
        self.context.mode = None;
        self
    }

    pub fn build(self) -> InvocationContext {
        self.context
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct RequestBuilder {
    request: PluginRequest,
}

impl RequestBuilder {
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

    pub fn plugin_id(mut self, plugin_id: impl Into<String>) -> Self {
        self.request.plugin_id = plugin_id.into();
        self
    }

    pub fn action_id(mut self, action_id: impl Into<String>) -> Self {
        self.request.action_id = action_id.into();
        self
    }

    pub fn payload(mut self, payload: Value) -> Self {
        self.request.payload = payload;
        self
    }

    pub fn context(mut self, context: InvocationContext) -> Self {
        self.request.context = context;
        self
    }

    pub fn host(mut self, host: HostKind) -> Self {
        self.request.context.host = host;
        self
    }

    pub fn build(self) -> PluginRequest {
        self.request
    }
}

impl Default for RequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ResponseBuilder {
    response: PluginResponse,
}

impl ResponseBuilder {
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

    pub fn plugin_id(mut self, plugin_id: impl Into<String>) -> Self {
        self.response.plugin_id = plugin_id.into();
        self
    }

    pub fn action_id(mut self, action_id: impl Into<String>) -> Self {
        self.response.action_id = action_id.into();
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.response.title = title.into();
        self
    }

    pub fn summary(mut self, summary: impl Into<String>) -> Self {
        self.response.summary = summary.into();
        self
    }

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

    pub fn add_untitled_output(mut self, kind: OutputKind, body: impl Into<String>) -> Self {
        self.response.outputs.push(OutputBlock {
            kind,
            title: None,
            body: body.into(),
        });
        self
    }

    pub fn add_next_step(mut self, next_step: impl Into<String>) -> Self {
        self.response.suggested_next_steps.push(next_step.into());
        self
    }

    pub fn build(self) -> PluginResponse {
        self.response
    }
}

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
                .payload_hint(r#"{"bundle_root":"examples/packaging/native-json-hello-world"}"#)
                .build(),
        )
        .build()
}

pub fn sample_request() -> PluginRequest {
    RequestBuilder::new().build()
}

pub fn sample_response() -> PluginResponse {
    ResponseBuilder::ok()
        .add_output(OutputKind::Json, "Bundle plan", r#"{"channel":"stable"}"#)
        .add_next_step("Copy the compiled plugin library into the bundle.")
        .build()
}

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

pub fn assert_manifest_hosts(manifest: &PluginManifest, expected_hosts: &[HostKind]) {
    assert_eq!(
        manifest.supported_hosts, expected_hosts,
        "supported hosts for manifest '{}' did not match",
        manifest.id
    );
}

pub fn assert_payload_eq(request: &PluginRequest, expected_payload: Value) {
    assert_eq!(
        request.payload, expected_payload,
        "payload for request '{}:{}' did not match",
        request.plugin_id, request.action_id
    );
}

pub fn assert_response_ok(response: &PluginResponse) {
    assert!(
        response.success,
        "expected response '{}:{}' to succeed, summary: {}",
        response.plugin_id, response.action_id, response.summary
    );
}

pub fn assert_response_error(response: &PluginResponse) {
    assert!(
        !response.success,
        "expected response '{}:{}' to fail",
        response.plugin_id, response.action_id
    );
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PackageRuntime {
    NativeJson,
    AbiStable,
    Wasm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactKind {
    DynamicLibrary,
    WasmModule,
    ManifestSnapshot,
    ReleaseMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactManifest {
    pub path: String,
    pub kind: ArtifactKind,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseMetadata {
    pub channel: String,
    pub target: String,
    pub hosts: Vec<HostKind>,
    pub installer_hint: Option<String>,
}

impl ReleaseMetadata {
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

    pub fn with_installer_hint(mut self, installer_hint: impl Into<String>) -> Self {
        self.installer_hint = Some(installer_hint.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageManifest {
    pub schema_version: u32,
    pub package_name: String,
    pub runtime: PackageRuntime,
    pub entrypoint: String,
    pub plugin: PluginManifest,
    pub artifacts: Vec<ArtifactManifest>,
    pub release: ReleaseMetadata,
}

impl PackageManifest {
    pub fn manifest_file_name(&self) -> &'static str {
        match self.runtime {
            PackageRuntime::Wasm => "wasm-plugin.json",
            PackageRuntime::NativeJson | PackageRuntime::AbiStable => "plugin-manifest.json",
        }
    }

    pub fn missing_required_artifacts(&self, root: impl AsRef<Path>) -> Vec<PathBuf> {
        self.artifacts
            .iter()
            .filter(|artifact| artifact.required)
            .map(|artifact| root.as_ref().join(&artifact.path))
            .filter(|path| !path.exists())
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct PackageFixture {
    manifest: PackageManifest,
    extra_files: BTreeMap<PathBuf, String>,
}

impl PackageFixture {
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

    pub fn with_text_file(
        mut self,
        relative_path: impl Into<PathBuf>,
        contents: impl Into<String>,
    ) -> Self {
        self.extra_files
            .insert(relative_path.into(), contents.into());
        self
    }

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

#[derive(Debug, Clone)]
pub struct WrittenPackageFixture {
    pub root: PathBuf,
    pub package_manifest_path: PathBuf,
    pub manifest_snapshot_path: PathBuf,
    pub release_metadata_path: PathBuf,
}

#[cfg(test)]
mod tests {
    use abi_stable::std_types::RString;
    use abi_stable_greeter::instantiate_root_module;
    use plugin_api::{copy_c_string, json_string_to_ptr, reclaim_c_string};
    use plugin_sdk::JsonPlugin;
    use plugin_wasm::{load_plugin_from_dir, load_plugins_from_workspace};
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
            "lib/libfixture_plugin.dylib",
            ReleaseMetadata::new(
                "stable",
                "x86_64-apple-darwin",
                vec![HostKind::Cli, HostKind::Service],
            )
            .with_installer_hint("Point RUST_PLUGIN_SYSTEM_PLUGIN_DIR at the lib directory."),
        )
        .with_required_entrypoint()
        .with_text_file("lib/libfixture_plugin.dylib", "fixture binary placeholder");

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

    #[test]
    fn native_json_exports_round_trip_requests() {
        let manifest_ptr = hello_world::plugin_manifest_json();
        let manifest_json = unsafe { copy_c_string(manifest_ptr.cast_const()) }.unwrap();
        unsafe { hello_world::plugin_free_c_string(manifest_ptr) };
        let manifest: PluginManifest = serde_json::from_str(&manifest_json).unwrap();

        let request = RequestBuilder::new()
            .plugin_id("hello-world")
            .action_id("greet")
            .payload(json!({"name":"Phase 4"}))
            .host(HostKind::Cli)
            .build();
        let request_ptr = json_string_to_ptr(serde_json::to_string(&request).unwrap());
        let response_ptr = unsafe { hello_world::plugin_invoke_json(request_ptr.cast_const()) };
        unsafe { reclaim_c_string(request_ptr) };

        let response_json = unsafe { copy_c_string(response_ptr.cast_const()) }.unwrap();
        unsafe { hello_world::plugin_free_c_string(response_ptr) };
        let response: PluginResponse = serde_json::from_str(&response_json).unwrap();

        assert_eq!(manifest.id, hello_world::HelloWorldPlugin::manifest().id);
        assert_response_ok(&response);
        assert_output_contains(&response, Some("Greeting"), "Phase 4");
    }

    #[test]
    fn abi_stable_module_round_trips_requests() {
        let module = instantiate_root_module();
        let manifest_json = (module.manifest_json())();
        let manifest: PluginManifest = serde_json::from_str(manifest_json.as_str()).unwrap();

        let request = RequestBuilder::new()
            .plugin_id("abi-stable-greeter")
            .action_id("greet")
            .payload(json!({"name":"Bundle Tester"}))
            .host(HostKind::Service)
            .build();
        let response_json =
            (module.invoke_json())(RString::from(serde_json::to_string(&request).unwrap()));
        let response: PluginResponse = serde_json::from_str(response_json.as_str()).unwrap();

        assert_eq!(manifest.architecture, PluginArchitecture::AbiStable);
        assert_response_ok(&response);
        assert_output_contains(&response, Some("Greeting"), "Bundle Tester");
    }

    #[test]
    fn wasm_workspace_fixtures_load_and_invoke_deterministically() {
        let workspace = workspace_root();
        let catalog = load_plugins_from_workspace(&workspace).unwrap();
        let plugin_ids = catalog
            .plugins
            .iter()
            .map(|plugin| plugin.manifest().id.clone())
            .collect::<Vec<_>>();

        assert!(catalog.warnings.is_empty());
        assert!(
            plugin_ids
                .iter()
                .any(|plugin_id| plugin_id == "wasm-sandboxed")
        );
        assert!(plugin_ids.iter().any(|plugin_id| plugin_id == "web-widget"));

        let plugin = load_plugin_from_dir(&workspace.join("plugins/wasm-sandboxed")).unwrap();
        let response = plugin
            .invoke(
                &RequestBuilder::new()
                    .plugin_id("wasm-sandboxed")
                    .action_id("run-demo")
                    .payload(json!({"note":"phase4"}))
                    .host(HostKind::Web)
                    .build(),
            )
            .unwrap();

        assert_response_ok(&response);
        assert_output_contains(&response, Some("Sandbox"), "WebAssembly module");
    }

    #[test]
    fn example_packaging_assets_match_bundle_schema() {
        for relative_path in [
            "examples/packaging/native-json/hello-world-bundle",
            "examples/packaging/native-json/service-hooks-bundle",
            "examples/packaging/abi-stable/abi-stable-greeter-bundle",
            "examples/packaging/wasm/wasm-sandboxed-bundle",
            "examples/packaging/wasm/web-widget-bundle",
        ] {
            let bundle_root = workspace_root().join(relative_path);
            let package_manifest = read_package_manifest(bundle_root.join("package.json"));

            assert_eq!(package_manifest.schema_version, PACKAGE_SCHEMA_VERSION);
            assert!(
                package_manifest
                    .missing_required_artifacts(&bundle_root)
                    .is_empty()
            );
            assert!(bundle_root.join("release.json").exists());

            if matches!(package_manifest.runtime, PackageRuntime::Wasm) {
                assert!(bundle_root.join("module.wat").exists());
            }
        }
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
