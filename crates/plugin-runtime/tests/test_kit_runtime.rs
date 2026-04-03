use plugin_runtime::{PluginSummary, render_response};
use plugin_test_kit::{ActionBuilder, HostKind, ManifestBuilder, OutputKind, ResponseBuilder};

#[test]
fn plugin_summary_uses_manifest_contract_data() {
    let manifest = ManifestBuilder::new()
        .id("hello-world")
        .name("Hello World")
        .description("The simplest runtime-loaded plugin in the playground.")
        .supported_hosts([HostKind::Cli, HostKind::Service])
        .tags(["starter", "native-plugin", "teaching"])
        .add_action(
            ActionBuilder::new()
                .id("greet")
                .label("Greet")
                .description("Say hello to a supplied name.")
                .build(),
        )
        .build();

    let summary = PluginSummary::from(&manifest);
    assert_eq!(summary.id, "hello-world");
    assert_eq!(summary.name, "Hello World");
    assert_eq!(summary.description, manifest.description);
    assert_eq!(summary.action_count, 1);
    assert_eq!(summary.supported_hosts, vec!["CLI", "Service"]);
    assert_eq!(summary.tags, vec!["starter", "native-plugin", "teaching"]);
}

#[test]
fn render_response_matches_runtime_text_layout() {
    let response = ResponseBuilder::ok()
        .plugin_id("hello-world")
        .action_id("greet")
        .title("Hello from a runtime-loaded plugin")
        .summary("The plugin greeted 'Rustacean' for the Cli host.")
        .add_output(OutputKind::Text, "Greeting", "Hello, Rustacean!")
        .add_untitled_output(OutputKind::Json, r#"{"host":"cli"}"#)
        .add_next_step("Try running the same plugin from another host.")
        .add_next_step("Package the compiled library with its manifest.")
        .build();

    let rendered = render_response(&response);
    let expected = concat!(
        "Hello from a runtime-loaded plugin\n",
        "The plugin greeted 'Rustacean' for the Cli host.\n",
        "\n[Text] Greeting\n",
        "Hello, Rustacean!\n",
        "\n[Json] Output\n",
        "{\"host\":\"cli\"}\n",
        "\nNext steps:\n",
        "- Try running the same plugin from another host.\n",
        "- Package the compiled library with its manifest.\n",
    );

    assert_eq!(rendered, expected);
}
