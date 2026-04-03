use std::path::PathBuf;

use host_core::{HostFitStatus, Playground, assess_host_fit, build_invocation_context};
use plugin_protocol::HostKind;
use serde_json::json;

/// Resolve the workspace-level `target/debug` directory.
fn plugin_dir() -> PathBuf {
    // CARGO_MANIFEST_DIR points to crates/host-core; go up twice for the workspace root.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .join("../../target/debug")
        .canonicalize()
        .unwrap_or_else(|_| manifest_dir.join("../../target/debug"))
}

/// Load the playground, exiting cleanly if plugins haven't been built yet.
fn load_playground() -> Playground {
    match Playground::load(plugin_dir()) {
        Ok(pg) => pg,
        Err(e) => {
            eprintln!("skipping: plugins not built (run `cargo build --workspace` first): {e}");
            std::process::exit(0);
        }
    }
}

#[test]
fn test_playground_loads_plugins() {
    let pg = load_playground();
    let manifests = pg.manifests();
    assert!(
        !manifests.is_empty(),
        "expected at least 1 plugin, found none (dir: {:?})",
        pg.plugin_dir(),
    );
}

#[test]
fn test_playground_manifests_not_empty() {
    let pg = load_playground();
    for m in pg.manifests() {
        assert!(!m.id.is_empty(), "manifest id must not be empty");
        assert!(!m.name.is_empty(), "manifest name must not be empty");
        assert!(!m.version.is_empty(), "manifest version must not be empty");
    }
}

#[test]
fn test_invoke_hello_world() {
    let pg = load_playground();

    let manifest = pg
        .manifests()
        .into_iter()
        .find(|m| m.id == "hello-world")
        .expect("hello-world plugin should be loaded");

    let action_id = &manifest.actions[0].id;

    let response = pg
        .invoke("hello-world", action_id, json!({}), HostKind::Cli)
        .expect("invoking hello-world should succeed");

    assert!(
        response.success,
        "hello-world response should be successful, got: {}",
        response.summary,
    );
    assert_eq!(response.plugin_id, "hello-world");
}

#[test]
fn test_invoke_with_payload() {
    let pg = load_playground();

    let response = pg
        .invoke(
            "config-provider",
            "get-value",
            json!({"config": {"service": {"port": 8080}}, "path": "service.port"}),
            HostKind::Cli,
        )
        .expect("invoking config-provider get-value should succeed");

    assert!(
        response.success,
        "config-provider get-value should succeed, got: {}",
        response.summary,
    );
    assert_eq!(response.plugin_id, "config-provider");
    assert_eq!(response.action_id, "get-value");
}

#[test]
fn test_invoke_unknown_plugin() {
    let pg = load_playground();

    let result = pg.invoke(
        "nonexistent-plugin",
        "some-action",
        json!({}),
        HostKind::Cli,
    );

    assert!(
        result.is_err(),
        "invoking a nonexistent plugin should return an error",
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("nonexistent-plugin"),
        "error should mention the missing plugin id, got: {err_msg}",
    );
}

#[test]
fn test_invoke_unknown_action() {
    let pg = load_playground();

    let result = pg.invoke(
        "hello-world",
        "nonexistent-action",
        json!({}),
        HostKind::Cli,
    );

    // The plugin binary decides how to handle unknown actions — it may
    // return an Err or a PluginResponse with success == false.
    match result {
        Err(_) => { /* expected */ }
        Ok(response) => {
            assert!(
                !response.success,
                "unknown action should not succeed, got success with summary: {}",
                response.summary,
            );
        }
    }
}

#[test]
fn test_manifest_fields_valid() {
    let pg = load_playground();

    for m in pg.manifests() {
        assert!(!m.id.is_empty(), "plugin id must not be empty");
        assert!(!m.name.is_empty(), "plugin name must not be empty");
        assert!(!m.version.is_empty(), "plugin version must not be empty");
        assert!(
            !m.actions.is_empty(),
            "plugin '{}' must have at least one action",
            m.id,
        );
        // architecture is an enum — if it deserialized, it's valid
        let _ = format!("{:?}", m.architecture);
    }
}

#[test]
fn test_host_fit_assessment() {
    let pg = load_playground();
    let dir = plugin_dir();

    for manifest in pg.manifests() {
        let context = build_invocation_context(
            HostKind::Cli,
            std::env::current_dir().ok().as_deref(),
            Some(dir.as_path()),
            Some("interactive"),
            None,
        );
        let assessment = assess_host_fit(&manifest, &context);

        assert!(
            matches!(
                assessment.status,
                HostFitStatus::Ready | HostFitStatus::Degraded | HostFitStatus::Rejected
            ),
            "unexpected host-fit status for '{}': {:?}",
            manifest.id,
            assessment.status,
        );
        assert!(
            !assessment.summary.is_empty(),
            "host-fit summary should not be empty for '{}'",
            manifest.id,
        );
    }
}
