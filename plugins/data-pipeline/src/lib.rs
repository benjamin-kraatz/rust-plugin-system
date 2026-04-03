use std::collections::BTreeSet;

use plugin_sdk::plugin_manifest::{
    Capability, HostKind, PluginAction, PluginArchitecture, PluginManifest, SkillLevel,
};
use plugin_sdk::plugin_protocol::{OutputKind, PluginRequest, PluginResponse};
use plugin_sdk::{JsonPlugin, export_plugin};
use serde_json::{Map, Value, json};

pub struct DataPipelinePlugin;

impl JsonPlugin for DataPipelinePlugin {
    fn manifest() -> PluginManifest {
        PluginManifest::new(
            "data-pipeline",
            "Data Pipeline",
            "0.1.0",
            "Projects, filters, and summarizes JSON record sets for host workflows.",
            PluginArchitecture::NativeJson,
            SkillLevel::Intermediate,
        )
        .with_supported_hosts(vec![HostKind::Cli, HostKind::Tui, HostKind::Service, HostKind::Web])
        .with_capabilities(vec![
            Capability::new("record-projection", "Selects stable subsets of JSON record fields."),
            Capability::new("record-filtering", "Applies deterministic predicates to JSON record sets."),
            Capability::new("field-summarization", "Computes numeric and categorical summaries from records."),
        ])
        .with_tags(["data", "json", "pipeline", "analytics", "read-only"])
        .with_actions(vec![
            PluginAction::new(
                "project-records",
                "Project records",
                "Keep only selected fields from each record in a dataset.",
            )
            .with_payload_hint(r#"{"records":[{"id":1,"name":"Ada","team":"platform"}],"fields":["id","name"]}"#),
            PluginAction::new(
                "filter-records",
                "Filter records",
                "Filter a dataset using a simple predicate on one field.",
            )
            .with_payload_hint(r#"{"records":[{"status":"active"},{"status":"draft"}],"predicate":{"field":"status","op":"eq","value":"active"}}"#),
            PluginAction::new(
                "summarize-field",
                "Summarize field",
                "Summarize one field across a dataset, including numeric stats when possible.",
            )
            .with_payload_hint(r#"{"records":[{"duration":12.5},{"duration":7.5}],"field":"duration"}"#),
        ])
        .with_notes([
            "All actions operate on payload data only, making the plugin deterministic and safe to replay.",
            "Filtering supports eq, neq, contains, gt, and lt operators for common lightweight data tasks.",
        ])
    }

    fn invoke(request: PluginRequest) -> Result<PluginResponse, String> {
        match request.action_id.as_str() {
            "project-records" => project_records(request),
            "filter-records" => filter_records(request),
            "summarize-field" => summarize_field(request),
            other => Err(format!("unknown action '{other}'")),
        }
    }
}

fn project_records(request: PluginRequest) -> Result<PluginResponse, String> {
    let records = records_from_payload(&request.payload)?;
    let fields = string_list(request.payload.get("fields"))
        .ok_or_else(|| "payload.fields must be an array of strings".to_owned())?;

    let projected = records
        .iter()
        .map(|record| {
            let mut next = Map::new();
            for field in &fields {
                if let Some(value) = record.get(field) {
                    next.insert(field.clone(), value.clone());
                }
            }
            Value::Object(next)
        })
        .collect::<Vec<_>>();

    Ok(PluginResponse::ok(
        "data-pipeline",
        "project-records",
        "Projected records",
        format!(
            "Projected {} record(s) down to {} field(s).",
            projected.len(),
            fields.len()
        ),
    )
    .with_output(
        OutputKind::Json,
        "Projected dataset",
        serde_json::to_string_pretty(&projected).map_err(|error| error.to_string())?,
    )
    .with_next_step(
        "Use filter-records before projection when you want to trim the dataset first.",
    ))
}

fn filter_records(request: PluginRequest) -> Result<PluginResponse, String> {
    let records = records_from_payload(&request.payload)?;
    let predicate = request
        .payload
        .get("predicate")
        .and_then(Value::as_object)
        .ok_or_else(|| "payload.predicate must be an object".to_owned())?;
    let field = predicate
        .get("field")
        .and_then(Value::as_str)
        .ok_or_else(|| "predicate.field must be a string".to_owned())?;
    let op = predicate.get("op").and_then(Value::as_str).unwrap_or("eq");
    let expected = predicate
        .get("value")
        .ok_or_else(|| "predicate.value is required".to_owned())?;

    let filtered = records
        .iter()
        .filter(|record| {
            record
                .get(field)
                .is_some_and(|value| matches_predicate(value, op, expected))
        })
        .cloned()
        .map(Value::Object)
        .collect::<Vec<_>>();

    Ok(PluginResponse::ok(
        "data-pipeline",
        "filter-records",
        "Filtered records",
        format!(
            "Kept {} of {} record(s) where {} {} {}.",
            filtered.len(),
            records.len(),
            field,
            op,
            compact_json(expected)
        ),
    )
    .with_output(
        OutputKind::Json,
        "Filtered dataset",
        serde_json::to_string_pretty(&filtered).map_err(|error| error.to_string())?,
    )
    .with_next_step(
        "Run summarize-field on the filtered dataset to quantify the remaining records.",
    ))
}

fn summarize_field(request: PluginRequest) -> Result<PluginResponse, String> {
    let records = records_from_payload(&request.payload)?;
    let field = request
        .payload
        .get("field")
        .and_then(Value::as_str)
        .ok_or_else(|| "payload.field must be a string".to_owned())?;

    let values = records
        .iter()
        .filter_map(|record| record.get(field).cloned())
        .collect::<Vec<_>>();

    if values.is_empty() {
        return Err(format!("field '{field}' was not present in any records"));
    }

    let numeric_values = values.iter().filter_map(Value::as_f64).collect::<Vec<_>>();
    let distinct = values.iter().map(compact_json).collect::<BTreeSet<_>>();
    let numeric_summary = if numeric_values.is_empty() {
        Value::Null
    } else {
        let sum: f64 = numeric_values.iter().sum();
        let min = numeric_values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = numeric_values
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        json!({
            "sum": sum,
            "min": min,
            "max": max,
            "average": sum / numeric_values.len() as f64,
        })
    };

    let summary = json!({
        "field": field,
        "record_count": records.len(),
        "matched_values": values.len(),
        "null_values": values.iter().filter(|value| value.is_null()).count(),
        "distinct_values": distinct.iter().take(10).cloned().collect::<Vec<_>>(),
        "numeric_summary": numeric_summary,
    });

    Ok(PluginResponse::ok(
        "data-pipeline",
        "summarize-field",
        "Field summary",
        format!("Summarized '{}' across {} record(s).", field, records.len()),
    )
    .with_output(
        OutputKind::Json,
        "Summary",
        serde_json::to_string_pretty(&summary).map_err(|error| error.to_string())?,
    )
    .with_next_step(
        "Use project-records to extract the fields you want to ship to downstream consumers.",
    ))
}

fn records_from_payload(payload: &Value) -> Result<Vec<Map<String, Value>>, String> {
    let records = payload
        .get("records")
        .and_then(Value::as_array)
        .ok_or_else(|| "payload.records must be an array of objects".to_owned())?;

    records
        .iter()
        .map(|value| {
            value
                .as_object()
                .cloned()
                .ok_or_else(|| "payload.records must contain only objects".to_owned())
        })
        .collect()
}

fn string_list(value: Option<&Value>) -> Option<Vec<String>> {
    let items = value?.as_array()?;
    items
        .iter()
        .map(|item| item.as_str().map(str::to_owned))
        .collect()
}

fn matches_predicate(actual: &Value, op: &str, expected: &Value) -> bool {
    match op {
        "eq" => actual == expected,
        "neq" => actual != expected,
        "contains" => match (actual, expected.as_str()) {
            (Value::String(actual), Some(expected)) => actual.contains(expected),
            (Value::Array(items), _) => items.iter().any(|item| item == expected),
            _ => false,
        },
        "gt" => compare_numbers(actual, expected).is_some_and(|(left, right)| left > right),
        "lt" => compare_numbers(actual, expected).is_some_and(|(left, right)| left < right),
        _ => false,
    }
}

fn compare_numbers(actual: &Value, expected: &Value) -> Option<(f64, f64)> {
    Some((actual.as_f64()?, expected.as_f64()?))
}

fn compact_json(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "null".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_numeric_comparisons() {
        assert!(matches_predicate(&json!(10), "gt", &json!(4)));
        assert!(matches_predicate(&json!(2), "lt", &json!(4)));
    }

    #[test]
    fn parses_record_arrays() {
        let records = records_from_payload(&json!({"records":[{"id":1},{"id":2}]})).unwrap();
        assert_eq!(records.len(), 2);
    }
}

export_plugin!(DataPipelinePlugin);
