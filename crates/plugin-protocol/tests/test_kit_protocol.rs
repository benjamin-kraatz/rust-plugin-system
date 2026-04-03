use plugin_protocol::{PluginRequest, PluginResponse};
use plugin_test_kit::{
    HostKind, OutputKind, RequestBuilder, ResponseBuilder, assert_next_steps,
    assert_output_contains, assert_payload_eq, assert_response_error, assert_response_ok,
};
use serde_json::json;

#[test]
fn request_builder_round_trips_context_and_payload() {
    let request = RequestBuilder::new()
        .plugin_id("hello-world")
        .action_id("greet")
        .host(HostKind::Web)
        .payload(json!({"name":"Rustacean","channel":"stable"}))
        .build();

    let encoded = serde_json::to_value(&request).expect("request should serialize");
    assert_eq!(encoded["context"]["host"], "web");
    assert_eq!(encoded["payload"]["name"], "Rustacean");

    let decoded: PluginRequest = serde_json::from_value(encoded).expect("request should decode");
    assert_eq!(decoded.context.host, HostKind::Web);
    assert_payload_eq(&decoded, json!({"name":"Rustacean","channel":"stable"}));
}

#[test]
fn response_builder_tracks_outputs_and_next_steps() {
    let response = ResponseBuilder::ok()
        .plugin_id("hello-world")
        .action_id("greet")
        .title("Bundle ready")
        .summary("Prepared local release metadata.")
        .add_output(
            OutputKind::Json,
            "Release metadata",
            r#"{"version":"1.2.3","channel":"stable"}"#,
        )
        .add_next_step("Attach the release metadata to the bundle.")
        .add_next_step("Copy the compiled library into the package.")
        .build();

    assert_response_ok(&response);
    assert_output_contains(&response, Some("Release metadata"), r#""channel":"stable""#);
    assert_next_steps(
        &response,
        &[
            "Attach the release metadata to the bundle.",
            "Copy the compiled library into the package.",
        ],
    );

    let encoded = serde_json::to_value(&response).expect("response should serialize");
    assert_eq!(encoded["outputs"][0]["kind"], "json");

    let decoded: PluginResponse =
        serde_json::from_value(encoded).expect("response should deserialize");
    assert_eq!(decoded.outputs.len(), 1);
    assert_eq!(decoded.suggested_next_steps.len(), 2);
}

#[test]
fn error_builder_marks_failures() {
    let response = ResponseBuilder::error()
        .summary("Bundle checksum mismatch.")
        .build();

    assert_response_error(&response);
    assert!(!response.success);
}
