use plugin_sdk::plugin_manifest::{
    Capability, HostKind, PluginAction, PluginArchitecture, PluginManifest, SkillLevel,
};
use plugin_sdk::plugin_protocol::{OutputKind, PluginRequest, PluginResponse};
use plugin_sdk::{JsonPlugin, export_plugin};
use serde_json::{Map, Value};

pub struct ConfigProviderPlugin;

impl JsonPlugin for ConfigProviderPlugin {
    fn manifest() -> PluginManifest {
        PluginManifest::new(
            "config-provider",
            "Configuration Provider",
            "0.1.0",
            "Merges configuration layers, inspects nested settings, and flattens config maps.",
            PluginArchitecture::NativeJson,
            SkillLevel::Intermediate,
        )
        .with_supported_hosts(vec![HostKind::Cli, HostKind::Tui, HostKind::Service])
        .with_capabilities(vec![
            Capability::new("config-layering", "Builds effective configuration from multiple JSON layers."),
            Capability::new("config-query", "Resolves nested configuration values by dotted paths."),
            Capability::new("config-shape", "Flattens configuration trees into a concise path map."),
        ])
        .with_tags(["config", "settings", "json", "read-only", "utility"])
        .with_actions(vec![
            PluginAction::new(
                "merge-layers",
                "Merge layers",
                "Combine defaults, environment values, and overrides into an effective config.",
            )
            .with_payload_hint(
                r#"{"defaults":{"service":{"port":8080}},"environment":{"service":{"port":8081}},"overrides":{"service":{"host":"127.0.0.1"}}}"#,
            ),
            PluginAction::new(
                "get-value",
                "Get value",
                "Resolve a single configuration value using dotted path lookup.",
            )
            .with_payload_hint(r#"{"config":{"service":{"port":8080}},"path":"service.port"}"#),
            PluginAction::new(
                "flatten-config",
                "Flatten config",
                "Produce a dotted-path view of every leaf value in a configuration document.",
            )
            .with_payload_hint(r#"{"config":{"service":{"port":8080,"host":"localhost"}}}"#),
        ])
        .with_notes([
            "Hosts can feed parsed JSON or TOML/YAML converted into JSON before invoking this plugin.",
            "Layer precedence is defaults < environment < overrides, matching common app configuration flows.",
        ])
    }

    fn invoke(request: PluginRequest) -> Result<PluginResponse, String> {
        match request.action_id.as_str() {
            "merge-layers" => merge_layers(request),
            "get-value" => get_value(request),
            "flatten-config" => flatten_config(request),
            other => Err(format!("unknown action '{other}'")),
        }
    }
}

fn merge_layers(request: PluginRequest) -> Result<PluginResponse, String> {
    let defaults = object_from_payload(&request.payload, "defaults")?;
    let environment = object_from_payload(&request.payload, "environment")?;
    let overrides = object_from_payload(&request.payload, "overrides")?;

    let mut merged = Value::Object(defaults);
    merge_values(&mut merged, Value::Object(environment));
    merge_values(&mut merged, Value::Object(overrides));

    let flattened = flatten_map(&merged);
    let summary = format!(
        "Resolved {} configuration entries for the {:?} host.",
        flattened.len(),
        request.context.host
    );

    Ok(PluginResponse::ok(
        "config-provider",
        "merge-layers",
        "Effective configuration",
        summary,
    )
    .with_output(
        OutputKind::Json,
        "Merged config",
        serde_json::to_string_pretty(&merged).map_err(|error| error.to_string())?,
    )
    .with_output(
        OutputKind::Json,
        "Flattened view",
        serde_json::to_string_pretty(&Value::Object(flattened))
            .map_err(|error| error.to_string())?,
    )
    .with_next_step("Run get-value with a dotted path to inspect a single resolved setting."))
}

fn get_value(request: PluginRequest) -> Result<PluginResponse, String> {
    let config = request
        .payload
        .get("config")
        .ok_or_else(|| "payload.config is required".to_owned())?;
    let path = request
        .payload
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| "payload.path must be a string".to_owned())?;
    let fallback = request.payload.get("fallback").cloned();

    let value = value_at_path(config, path)
        .cloned()
        .or(fallback)
        .ok_or_else(|| format!("no configuration value found for path '{path}'"))?;

    let title = format!("Value for {path}");
    Ok(PluginResponse::ok(
        "config-provider",
        "get-value",
        title.clone(),
        format!("Resolved configuration path '{path}'."),
    )
    .with_output(
        OutputKind::Json,
        title,
        serde_json::to_string_pretty(&value).map_err(|error| error.to_string())?,
    )
    .with_next_step("Run flatten-config to inspect all available dotted paths."))
}

fn flatten_config(request: PluginRequest) -> Result<PluginResponse, String> {
    let config = request
        .payload
        .get("config")
        .ok_or_else(|| "payload.config is required".to_owned())?;
    let flattened = flatten_map(config);
    let keys = flattened.keys().cloned().collect::<Vec<_>>();

    Ok(PluginResponse::ok(
        "config-provider",
        "flatten-config",
        "Flattened configuration",
        format!("Discovered {} leaf settings.", keys.len()),
    )
    .with_output(
        OutputKind::Json,
        "Flattened config",
        serde_json::to_string_pretty(&Value::Object(flattened))
            .map_err(|error| error.to_string())?,
    )
    .with_output(OutputKind::Text, "Paths", keys.join("\n"))
    .with_next_step(
        "Use merge-layers first when you need to flatten the fully resolved configuration.",
    ))
}

fn object_from_payload(payload: &Value, key: &str) -> Result<Map<String, Value>, String> {
    match payload.get(key) {
        Some(Value::Object(object)) => Ok(object.clone()),
        Some(Value::Null) | None => Ok(Map::new()),
        Some(_) => Err(format!("payload.{key} must be an object when provided")),
    }
}

fn merge_values(target: &mut Value, overlay: Value) {
    match (target, overlay) {
        (Value::Object(target_map), Value::Object(overlay_map)) => {
            for (key, overlay_value) in overlay_map {
                match target_map.get_mut(&key) {
                    Some(existing) => merge_values(existing, overlay_value),
                    None => {
                        target_map.insert(key, overlay_value);
                    }
                }
            }
        }
        (target_slot, overlay_value) => *target_slot = overlay_value,
    }
}

fn value_at_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    if path.trim().is_empty() {
        return Some(value);
    }

    path.split('.')
        .filter(|segment| !segment.is_empty())
        .try_fold(value, |current, segment| match current {
            Value::Object(map) => map.get(segment),
            Value::Array(items) => segment
                .parse::<usize>()
                .ok()
                .and_then(|index| items.get(index)),
            _ => None,
        })
}

fn flatten_map(value: &Value) -> Map<String, Value> {
    let mut flattened = Map::new();
    flatten_into(None, value, &mut flattened);
    flattened
}

fn flatten_into(prefix: Option<String>, value: &Value, flattened: &mut Map<String, Value>) {
    match value {
        Value::Object(map) if !map.is_empty() => {
            for (key, child) in map {
                let next_prefix = prefix
                    .as_ref()
                    .map(|existing| format!("{existing}.{key}"))
                    .unwrap_or_else(|| key.clone());
                flatten_into(Some(next_prefix), child, flattened);
            }
        }
        Value::Array(items) if !items.is_empty() => {
            for (index, child) in items.iter().enumerate() {
                let next_prefix = prefix
                    .as_ref()
                    .map(|existing| format!("{existing}.{index}"))
                    .unwrap_or_else(|| index.to_string());
                flatten_into(Some(next_prefix), child, flattened);
            }
        }
        _ => {
            let key = prefix.unwrap_or_else(|| "value".to_owned());
            flattened.insert(key, value.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn merges_objects_recursively() {
        let mut target = json!({"service":{"port":8080,"host":"localhost"}});
        merge_values(&mut target, json!({"service":{"port":9090,"tls":true}}));
        assert_eq!(target["service"]["port"], json!(9090));
        assert_eq!(target["service"]["host"], json!("localhost"));
        assert_eq!(target["service"]["tls"], json!(true));
    }

    #[test]
    fn resolves_dotted_paths() {
        let value = json!({"service":{"endpoints":[{"name":"health"}]}});
        assert_eq!(
            value_at_path(&value, "service.endpoints.0.name"),
            Some(&json!("health"))
        );
    }
}

export_plugin!(ConfigProviderPlugin);
