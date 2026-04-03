use std::fs;
use std::path::{Path, PathBuf};

use plugin_sdk::plugin_manifest::{
    Capability, HostKind, PluginAction, PluginArchitecture, PluginManifest, SkillLevel,
};
use plugin_sdk::plugin_protocol::{OutputKind, PluginRequest, PluginResponse};
use plugin_sdk::{JsonPlugin, export_plugin};
use serde_json::{Value, json};

pub struct FilesystemToolsPlugin;

impl JsonPlugin for FilesystemToolsPlugin {
    fn manifest() -> PluginManifest {
        PluginManifest::new(
            "filesystem-tools",
            "Filesystem Tools",
            "0.1.0",
            "Inspects workspace files and directories with read-only, path-safe actions.",
            PluginArchitecture::NativeJson,
            SkillLevel::Intermediate,
        )
        .with_supported_hosts(vec![HostKind::Cli, HostKind::Tui, HostKind::Service])
        .with_capabilities(vec![
            Capability::new("directory-discovery", "Lists workspace directories without modifying them."),
            Capability::new("file-preview", "Reads text files with bounded output sizes."),
            Capability::new("path-inspection", "Reports metadata for files and directories inside the workspace."),
        ])
        .with_tags(["filesystem", "workspace", "read-only", "inspection", "utility"])
        .with_actions(vec![
            PluginAction::new(
                "list-directory",
                "List directory",
                "Recursively list files and directories under a workspace-relative path.",
            )
            .with_payload_hint(r#"{"path":"plugins","max_depth":2,"include_hidden":false,"max_entries":25}"#),
            PluginAction::new(
                "read-text-file",
                "Read text file",
                "Preview a text file within the workspace without changing it.",
            )
            .with_payload_hint(r#"{"path":"README.md","max_bytes":2048}"#),
            PluginAction::new(
                "describe-path",
                "Describe path",
                "Return metadata for a file or directory inside the workspace.",
            )
            .with_payload_hint(r#"{"path":"crates/plugin-sdk/src/lib.rs"}"#),
        ])
        .with_notes([
            "All requested paths are constrained to the invocation workspace root when one is supplied by the host.",
            "Directory walks skip hidden entries by default and never mutate files.",
        ])
    }

    fn invoke(request: PluginRequest) -> Result<PluginResponse, String> {
        match request.action_id.as_str() {
            "list-directory" => list_directory(request),
            "read-text-file" => read_text_file(request),
            "describe-path" => describe_path(request),
            other => Err(format!("unknown action '{other}'")),
        }
    }
}

fn list_directory(request: PluginRequest) -> Result<PluginResponse, String> {
    let root = workspace_root(&request)?;
    let path = request
        .payload
        .get("path")
        .and_then(Value::as_str)
        .unwrap_or(".");
    let target = resolve_workspace_path(&root, path)?;
    let max_depth = request
        .payload
        .get("max_depth")
        .and_then(Value::as_u64)
        .unwrap_or(1)
        .min(5) as usize;
    let max_entries = request
        .payload
        .get("max_entries")
        .and_then(Value::as_u64)
        .unwrap_or(50)
        .clamp(1, 250) as usize;
    let include_hidden = request
        .payload
        .get("include_hidden")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    if !target.is_dir() {
        return Err(format!("{} is not a directory", target.display()));
    }

    let mut entries = Vec::new();
    collect_entries(
        &root,
        &target,
        0,
        max_depth,
        include_hidden,
        max_entries,
        &mut entries,
    )?;

    Ok(PluginResponse::ok(
        "filesystem-tools",
        "list-directory",
        "Workspace directory listing",
        format!(
            "Collected {} entries from {}.",
            entries.len(),
            display_path(&root, &target)
        ),
    )
    .with_output(
        OutputKind::Json,
        "Entries",
        serde_json::to_string_pretty(&entries).map_err(|error| error.to_string())?,
    )
    .with_output(
        OutputKind::Text,
        "Paths",
        entries
            .iter()
            .map(|entry| {
                format!(
                    "{} [{}]",
                    entry["path"].as_str().unwrap_or("?"),
                    entry["kind"].as_str().unwrap_or("unknown")
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .with_next_step("Run describe-path for any item you want to inspect in more detail."))
}

fn read_text_file(request: PluginRequest) -> Result<PluginResponse, String> {
    let root = workspace_root(&request)?;
    let path = request
        .payload
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| "payload.path must be a string".to_owned())?;
    let target = resolve_workspace_path(&root, path)?;
    let max_bytes = request
        .payload
        .get("max_bytes")
        .and_then(Value::as_u64)
        .unwrap_or(4096)
        .clamp(64, 16384) as usize;

    if !target.is_file() {
        return Err(format!("{} is not a file", target.display()));
    }

    let bytes = fs::read(&target)
        .map_err(|error| format!("failed to read {}: {error}", target.display()))?;
    let truncated = bytes.len() > max_bytes;
    let preview = String::from_utf8_lossy(&bytes[..bytes.len().min(max_bytes)]).into_owned();
    let metadata = fs::metadata(&target)
        .map_err(|error| format!("failed to read metadata for {}: {error}", target.display()))?;

    Ok(PluginResponse::ok(
        "filesystem-tools",
        "read-text-file",
        "File preview",
        format!(
            "Read {} byte(s) from {}{}.",
            metadata.len(),
            display_path(&root, &target),
            if truncated {
                " (truncated preview)"
            } else {
                ""
            }
        ),
    )
    .with_output(OutputKind::Code, "Contents", preview)
    .with_output(
        OutputKind::Json,
        "Metadata",
        serde_json::to_string_pretty(&json!({
            "path": display_path(&root, &target),
            "bytes": metadata.len(),
            "preview_bytes": bytes.len().min(max_bytes),
            "truncated": truncated,
            "readonly": metadata.permissions().readonly(),
        }))
        .map_err(|error| error.to_string())?,
    )
    .with_next_step(
        "Use list-directory to discover neighboring files or describe-path for directory metadata.",
    ))
}

fn describe_path(request: PluginRequest) -> Result<PluginResponse, String> {
    let root = workspace_root(&request)?;
    let path = request
        .payload
        .get("path")
        .and_then(Value::as_str)
        .unwrap_or(".");
    let target = resolve_workspace_path(&root, path)?;
    let metadata = fs::metadata(&target)
        .map_err(|error| format!("failed to stat {}: {error}", target.display()))?;

    let directory_entries = if metadata.is_dir() {
        Some(
            fs::read_dir(&target)
                .map_err(|error| format!("failed to read directory {}: {error}", target.display()))?
                .filter_map(Result::ok)
                .count(),
        )
    } else {
        None
    };

    Ok(PluginResponse::ok(
        "filesystem-tools",
        "describe-path",
        "Path metadata",
        format!("Inspected {}.", display_path(&root, &target)),
    )
    .with_output(
        OutputKind::Json,
        "Metadata",
        serde_json::to_string_pretty(&json!({
            "path": display_path(&root, &target),
            "absolute_path": target.display().to_string(),
            "kind": if metadata.is_dir() { "directory" } else if metadata.is_file() { "file" } else { "other" },
            "bytes": metadata.len(),
            "readonly": metadata.permissions().readonly(),
            "entry_count": directory_entries,
        }))
        .map_err(|error| error.to_string())?,
    )
    .with_next_step("Use read-text-file for file previews or list-directory for recursive discovery."))
}

fn workspace_root(request: &PluginRequest) -> Result<PathBuf, String> {
    let base = request
        .context
        .workspace_root
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    fs::canonicalize(&base).map_err(|error| {
        format!(
            "failed to resolve workspace root {}: {error}",
            base.display()
        )
    })
}

fn resolve_workspace_path(root: &Path, path: &str) -> Result<PathBuf, String> {
    let candidate = if path.trim().is_empty() || path == "." {
        root.to_path_buf()
    } else {
        let path = Path::new(path);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        }
    };

    let resolved = fs::canonicalize(&candidate)
        .map_err(|error| format!("failed to resolve {}: {error}", candidate.display()))?;

    if !resolved.starts_with(root) {
        return Err(format!(
            "requested path {} is outside the workspace root {}",
            resolved.display(),
            root.display()
        ));
    }

    Ok(resolved)
}

fn collect_entries(
    root: &Path,
    current: &Path,
    depth: usize,
    max_depth: usize,
    include_hidden: bool,
    max_entries: usize,
    entries: &mut Vec<Value>,
) -> Result<(), String> {
    if entries.len() >= max_entries {
        return Ok(());
    }

    let mut dir_entries = fs::read_dir(current)
        .map_err(|error| format!("failed to read directory {}: {error}", current.display()))?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    dir_entries.sort_by_key(|entry| entry.file_name());

    for entry in dir_entries {
        if entries.len() >= max_entries {
            break;
        }

        let name = entry.file_name().to_string_lossy().into_owned();
        if !include_hidden && name.starts_with('.') {
            continue;
        }

        let file_type = entry.file_type().map_err(|error| {
            format!(
                "failed to read file type for {}: {error}",
                entry.path().display()
            )
        })?;
        let path = entry.path();
        let relative = display_path(root, &path);
        let size = entry.metadata().ok().map(|metadata| metadata.len());

        entries.push(json!({
            "path": relative,
            "kind": if file_type.is_dir() { "directory" } else if file_type.is_file() { "file" } else if file_type.is_symlink() { "symlink" } else { "other" },
            "bytes": size,
            "depth": depth + 1,
        }));

        if file_type.is_dir() && depth + 1 < max_depth {
            collect_entries(
                root,
                &path,
                depth + 1,
                max_depth,
                include_hidden,
                max_entries,
                entries,
            )?;
        }
    }

    Ok(())
}

fn display_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .ok()
        .map(|relative| {
            if relative.as_os_str().is_empty() {
                ".".to_owned()
            } else {
                relative.display().to_string()
            }
        })
        .unwrap_or_else(|| path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_path_returns_workspace_relative_value() {
        let root = Path::new("/workspace");
        let child = Path::new("/workspace/plugins/example");
        assert_eq!(display_path(root, child), "plugins/example");
    }
}

export_plugin!(FilesystemToolsPlugin);
