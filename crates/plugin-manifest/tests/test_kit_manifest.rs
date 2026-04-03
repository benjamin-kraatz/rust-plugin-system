use plugin_manifest::PluginManifest;
use plugin_test_kit::{
    ActionBuilder, Capability, HostKind, ManifestBuilder, PluginArchitecture, SkillLevel,
    assert_manifest_has_action, assert_manifest_hosts,
};

#[test]
fn manifest_builder_round_trips_with_repo_conventions() {
    let manifest = ManifestBuilder::new()
        .id("release-planner")
        .name("Release Planner")
        .version("1.2.3")
        .description("Plans deterministic packaging steps.")
        .architecture(PluginArchitecture::NativeJson)
        .skill_level(SkillLevel::Advanced)
        .supported_hosts([HostKind::Cli, HostKind::Service])
        .capabilities([
            Capability::new("bundle.plan", "Builds a local bundle plan."),
            Capability::new("release.notes", "Produces release metadata snippets."),
        ])
        .tags(["packaging", "release", "testing"])
        .notes(["Used for shared crate tests."])
        .add_action(
            ActionBuilder::new()
                .id("plan-release")
                .label("Plan release")
                .description("Create release metadata for a local bundle.")
                .payload_hint(r#"{"channel":"stable","version":"1.2.3"}"#)
                .build(),
        )
        .build();

    assert_manifest_has_action(&manifest, "plan-release");
    assert_manifest_hosts(&manifest, &[HostKind::Cli, HostKind::Service]);

    let encoded = serde_json::to_value(&manifest).expect("manifest should serialize");
    assert_eq!(encoded["architecture"], "native-json");
    assert_eq!(encoded["skill_level"], "advanced");
    assert_eq!(encoded["supported_hosts"][0], "cli");

    let decoded: PluginManifest =
        serde_json::from_value(encoded).expect("manifest should deserialize");
    assert_eq!(decoded, manifest);
}
