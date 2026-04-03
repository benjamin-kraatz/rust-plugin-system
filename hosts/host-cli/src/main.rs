use std::path::PathBuf;

use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use host_core::{
    HostFitAssessment, HostFitStatus, Playground, assess_host_fit, build_invocation_context,
    default_plugin_dir, finalize_response, render_response, summarize_action_metadata,
    summarize_manifest_metadata, summarize_response_metadata,
};
use plugin_abi::AbiPluginCatalog;
use plugin_manifest::PluginManifest;
use plugin_protocol::{HostKind, PluginRequest};
use plugin_wasm::WasmPluginCatalog;

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "CLI host for the Rust plugin system playground"
)]
struct Cli {
    /// Directory containing plugin binaries and manifests
    #[arg(long, default_value_os_t = default_plugin_dir())]
    plugin_dir: PathBuf,
    /// Root directory of the workspace (used for WASM plugin discovery)
    #[arg(long, default_value_os_t = std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))]
    workspace_root: PathBuf,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// List all discovered plugins with their architecture and action count
    List,
    /// Show detailed manifest metadata for a specific plugin
    Inspect {
        /// The unique plugin identifier (e.g., "hello-world", "config-provider")
        plugin_id: String,
    },
    /// Invoke a plugin action with an optional JSON payload
    Run {
        /// The unique plugin identifier
        plugin_id: String,
        /// The action to invoke (e.g., "greet", "lookup")
        action_id: String,
        /// Optional JSON payload string (default: "{}")
        payload: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let playground = Playground::load(&cli.plugin_dir)?;
    let abi_catalog = plugin_abi::load_plugins_from_directory(&cli.plugin_dir)?;
    let wasm_catalog = plugin_wasm::load_plugins_from_workspace(&cli.workspace_root)?;
    let cli_context = build_invocation_context(
        HostKind::Cli,
        Some(&cli.workspace_root),
        Some(&cli.plugin_dir),
        Some("interactive"),
        Some(env!("CARGO_PKG_VERSION")),
    );

    if !playground.warnings().is_empty() {
        eprintln!("discovery warnings:");
        for warning in playground.warnings() {
            eprintln!("- {warning}");
        }
        eprintln!();
    }
    print_warnings("abi discovery warnings", &abi_catalog.warnings);
    print_warnings("wasm discovery warnings", &wasm_catalog.warnings);

    match cli.command.unwrap_or(Command::List) {
        Command::List => {
            println!("plugin dir: {}", playground.plugin_dir().display());
            println!("workspace root: {}\n", cli.workspace_root.display());
            print_manifest_group(
                "Native dynamic library plugins",
                &playground.manifests(),
                &cli_context,
            );
            print_manifest_group(
                "ABI-stable plugins",
                &abi_catalog
                    .plugins
                    .iter()
                    .map(|plugin| plugin.manifest().clone())
                    .collect::<Vec<_>>(),
                &cli_context,
            );
            print_manifest_group(
                "WASM sandboxed plugins",
                &wasm_catalog
                    .plugins
                    .iter()
                    .map(|plugin| plugin.manifest().clone())
                    .collect::<Vec<_>>(),
                &cli_context,
            );
        }
        Command::Inspect { plugin_id } => {
            let manifest = find_manifest(&playground, &abi_catalog, &wasm_catalog, &plugin_id)
                .ok_or_else(|| anyhow!("no plugin named '{plugin_id}'"))?;
            print_manifest_details(&manifest, &cli_context);
        }
        Command::Run {
            plugin_id,
            action_id,
            payload,
        } => {
            let payload_text = payload.as_deref().unwrap_or("{}");
            let response = if playground
                .manifests()
                .iter()
                .any(|manifest| manifest.id == plugin_id)
            {
                playground.invoke_text(
                    &plugin_id,
                    &action_id,
                    payload_text,
                    plugin_manifest_host(),
                )?
            } else if let Some(plugin) = abi_catalog
                .plugins
                .iter()
                .find(|plugin| plugin.manifest().id == plugin_id)
            {
                let request = build_request(
                    plugin.manifest(),
                    &plugin_id,
                    &action_id,
                    payload_text,
                    HostKind::Cli,
                    playground.plugin_dir(),
                    &cli.workspace_root,
                );
                let started = std::time::Instant::now();
                let mut response =
                    finalize_response(plugin.manifest(), &request, plugin.invoke(&request)?);
                if let Some(execution) = response.execution.as_mut() {
                    execution.duration_ms =
                        Some(started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64);
                }
                response
            } else if let Some(plugin) = wasm_catalog
                .plugins
                .iter()
                .find(|plugin| plugin.manifest().id == plugin_id)
            {
                let request = build_request(
                    plugin.manifest(),
                    &plugin_id,
                    &action_id,
                    payload_text,
                    HostKind::Cli,
                    playground.plugin_dir(),
                    &cli.workspace_root,
                );
                let started = std::time::Instant::now();
                let mut response =
                    finalize_response(plugin.manifest(), &request, plugin.invoke(&request)?);
                if let Some(execution) = response.execution.as_mut() {
                    execution.duration_ms =
                        Some(started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64);
                }
                response
            } else {
                return Err(anyhow!("no plugin named '{plugin_id}'"));
            };
            for line in summarize_response_metadata(&response) {
                println!("{line}");
            }
            if !response.outputs.is_empty() || !response.summary.is_empty() {
                println!();
            }
            println!("{}", render_response(&response));
        }
    }

    Ok(())
}

fn plugin_manifest_host() -> plugin_protocol::HostKind {
    plugin_protocol::HostKind::Cli
}

fn print_warnings(title: &str, warnings: &[String]) {
    if warnings.is_empty() {
        return;
    }

    eprintln!("{title}:");
    for warning in warnings {
        eprintln!("- {warning}");
    }
    eprintln!();
}

fn print_manifest_group(
    title: &str,
    manifests: &[PluginManifest],
    context: &plugin_protocol::InvocationContext,
) {
    println!("{title}");
    println!("{}", "-".repeat(title.len()));
    for manifest in manifests {
        let assessment = assess_host_fit(manifest, context);
        println!(
            "- {} ({}) [{} actions]",
            manifest.id,
            manifest.name,
            manifest.actions.len()
        );
        println!("  {}", manifest.description);
        println!("  architecture: {:?}", manifest.architecture);
        println!(
            "  hosts: {}",
            manifest
                .supported_hosts
                .iter()
                .map(|host| host.label())
                .collect::<Vec<_>>()
                .join(", ")
        );
        println!("  fit: {}", format_fit_badge(&assessment));
        for line in summarize_manifest_metadata(manifest).into_iter().take(3) {
            println!("  {line}");
        }
        println!();
    }
}

fn print_manifest_details(manifest: &PluginManifest, context: &plugin_protocol::InvocationContext) {
    let assessment = assess_host_fit(manifest, context);
    println!("{} ({})", manifest.name, manifest.id);
    println!("{}", manifest.description);
    println!("architecture: {:?}", manifest.architecture);
    println!("skill level: {:?}", manifest.skill_level);
    println!(
        "hosts: {}",
        manifest
            .supported_hosts
            .iter()
            .map(|host| host.label())
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!(
        "tags: {}",
        if manifest.tags.is_empty() {
            "<none>".to_owned()
        } else {
            manifest.tags.join(", ")
        }
    );
    println!("host fit: {}", format_fit_badge(&assessment));
    println!("host fit details: {}", assessment.summary);
    if let Some(maintenance) = &manifest.maintenance {
        println!("\nmaintenance:");
        println!("  status: {:?}", maintenance.status);
        if let Some(owner) = &maintenance.owner {
            println!("  owner: {owner}");
        }
        if let Some(support_tier) = &maintenance.support_tier {
            println!("  support tier: {support_tier}");
        }
        if let Some(channel) = &maintenance.channel {
            println!("  channel: {channel}");
        }
        if let Some(deprecation) = &maintenance.deprecation {
            println!(
                "  deprecation: {}",
                deprecation
                    .message
                    .as_deref()
                    .unwrap_or("plugin is deprecated")
            );
        }
    }
    for line in summarize_manifest_metadata(manifest) {
        println!("{line}");
    }
    if let Some(compatibility) = &manifest.compatibility {
        println!("\ncompatibility:");
        println!("  strategy: {:?}", compatibility.strategy);
        if let Some(protocol_version) = &compatibility.protocol_version {
            println!("  protocol version: {protocol_version}");
        }
        if let Some(host_version) = &compatibility.host_version {
            println!(
                "  host version window: {} .. {}",
                host_version.minimum.as_deref().unwrap_or("*"),
                host_version.maximum.as_deref().unwrap_or("*")
            );
        }
        if !compatibility.tested_hosts.is_empty() {
            println!("  tested hosts:");
            for tested in &compatibility.tested_hosts {
                match &tested.notes {
                    Some(notes) => {
                        println!("    - {} {} ({notes})", tested.host.label(), tested.version)
                    }
                    None => println!("    - {} {}", tested.host.label(), tested.version),
                }
            }
        }
        for note in &compatibility.notes {
            println!("  note: {note}");
        }
    }
    if let Some(trust) = &manifest.trust {
        println!("\ntrust:");
        println!("  level: {:?}", trust.level);
        println!("  sandbox: {:?}", trust.sandbox);
        println!("  network: {:?}", trust.network);
        println!("  deterministic: {}", trust.deterministic);
        println!("  local only: {}", trust.local_only);
        if !trust.permissions.is_empty() {
            println!("  permissions:");
            for permission in &trust.permissions {
                println!(
                    "    - {} :: {:?}{}",
                    permission.resource,
                    permission.scope,
                    if permission.required {
                        ""
                    } else {
                        " (optional)"
                    }
                );
                if let Some(reason) = &permission.reason {
                    println!("      reason: {reason}");
                }
            }
        }
        if !trust.data_access.is_empty() {
            println!("  data access: {}", trust.data_access.join(", "));
        }
        if let Some(provenance) = &trust.provenance {
            println!("  provenance: {provenance}");
        }
        for note in &trust.notes {
            println!("  note: {note}");
        }
    }
    if let Some(lifecycle) = &manifest.lifecycle {
        println!("\nlifecycle:");
        println!("  state: {:?}", lifecycle.state);
        println!("  stateless: {}", lifecycle.stateless);
        println!(
            "  explicit shutdown: {}",
            lifecycle.requires_explicit_shutdown
        );
        if !lifecycle.hooks.is_empty() {
            println!(
                "  hooks: {}",
                lifecycle
                    .hooks
                    .iter()
                    .map(|hook| format!("{hook:?}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        if let Some(health_probe) = &lifecycle.health_probe {
            println!("  health probe: {health_probe}");
        }
        for note in &lifecycle.notes {
            println!("  note: {note}");
        }
    }
    if let Some(execution) = &manifest.execution {
        println!("\nexecution:");
        println!("  mode: {:?}", execution.default_mode);
        println!("  supports async: {}", execution.supports_async);
        println!("  cancellable: {}", execution.cancellable);
        println!("  idempotent: {}", execution.idempotent);
        println!("  progress reporting: {}", execution.progress_reporting);
        if let Some(timeout_ms) = execution.timeout_ms {
            println!("  timeout ms: {timeout_ms}");
        }
        if let Some(max_concurrency) = execution.max_concurrency {
            println!("  max concurrency: {max_concurrency}");
        }
        if let Some(async_metadata) = &execution.async_metadata {
            println!("  async detached: {}", async_metadata.detached);
            println!("  async streaming: {}", async_metadata.supports_streaming);
            if let Some(completion_timeout_ms) = async_metadata.completion_timeout_ms {
                println!("  async completion timeout ms: {completion_timeout_ms}");
            }
            if let Some(retry_policy) = &async_metadata.retry_policy {
                println!("  retry attempts: {}", retry_policy.max_attempts);
                println!("  retry strategy: {:?}", retry_policy.strategy);
            }
        }
        for note in &execution.notes {
            println!("  note: {note}");
        }
    }
    if let Some(capability_contract) = &manifest.capability_contract {
        println!("\ncapability contract:");
        if !capability_contract.required.is_empty() {
            println!("  required:");
            for requirement in &capability_contract.required {
                println!("    - {}: {}", requirement.key, requirement.detail);
                if let Some(fallback) = &requirement.fallback {
                    println!("      fallback: {fallback}");
                }
            }
        }
        if !capability_contract.optional.is_empty() {
            println!("  optional:");
            for requirement in &capability_contract.optional {
                println!("    - {}: {}", requirement.key, requirement.detail);
                if let Some(fallback) = &requirement.fallback {
                    println!("      fallback: {fallback}");
                }
            }
        }
        if let Some(constraints) = &capability_contract.constraints {
            println!("  constraints:");
            println!("    sandbox: {:?}", constraints.sandbox_level);
            println!("    network: {:?}", constraints.network_access);
            if let Some(max_payload_bytes) = constraints.max_payload_bytes {
                println!("    max payload bytes: {max_payload_bytes}");
            }
            for permission in &constraints.permissions {
                println!(
                    "    permission: {} :: {:?}",
                    permission.resource, permission.scope
                );
            }
        }
        if !capability_contract.degradation.is_empty() {
            println!("  degradation:");
            for rule in &capability_contract.degradation {
                println!(
                    "    - {} ({:?}): {}",
                    rule.feature, rule.severity, rule.behavior
                );
                if !rule.when_missing.is_empty() {
                    println!("      when missing: {}", rule.when_missing.join(", "));
                }
            }
        }
        for note in &capability_contract.notes {
            println!("  note: {note}");
        }
    }
    println!("\nactions:");
    for action in &manifest.actions {
        println!("- {} :: {}", action.id, action.label);
        println!("  {}", action.description);
        if let Some(payload_hint) = &action.payload_hint {
            println!("  payload hint: {payload_hint}");
        }
        for line in summarize_action_metadata(action) {
            println!("  {line}");
        }
        if let Some(contract) = &action.contract
            && let Some(constraints) = &contract.constraints
        {
            println!("  constraints:");
            println!("    sandbox: {:?}", constraints.sandbox_level);
            println!("    network: {:?}", constraints.network_access);
            if let Some(max_payload_bytes) = constraints.max_payload_bytes {
                println!("    max payload bytes: {max_payload_bytes}");
            }
        }
    }
}

fn find_manifest(
    playground: &Playground,
    abi_catalog: &AbiPluginCatalog,
    wasm_catalog: &WasmPluginCatalog,
    plugin_id: &str,
) -> Option<PluginManifest> {
    playground
        .manifests()
        .into_iter()
        .find(|manifest| manifest.id == plugin_id)
        .or_else(|| {
            abi_catalog
                .plugins
                .iter()
                .find(|plugin| plugin.manifest().id == plugin_id)
                .map(|plugin| plugin.manifest().clone())
        })
        .or_else(|| {
            wasm_catalog
                .plugins
                .iter()
                .find(|plugin| plugin.manifest().id == plugin_id)
                .map(|plugin| plugin.manifest().clone())
        })
}

fn build_request(
    manifest: &PluginManifest,
    plugin_id: &str,
    action_id: &str,
    payload_text: &str,
    host: HostKind,
    plugin_dir: &std::path::Path,
    workspace_root: &std::path::Path,
) -> PluginRequest {
    let action = manifest
        .actions
        .iter()
        .find(|action| action.id == action_id);
    let mut context = build_invocation_context(
        host,
        Some(workspace_root),
        Some(plugin_dir),
        Some("interactive"),
        Some(env!("CARGO_PKG_VERSION")),
    );
    context.request_id = Some(format!(
        "{plugin_id}-{action_id}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .unwrap_or_default()
    ));
    context.trace_id = Some(format!("{plugin_id}/{action_id}"));
    context.timeout_ms = action
        .and_then(|action| action.contract.as_ref())
        .and_then(|contract| contract.timeout_ms)
        .or_else(|| {
            manifest
                .execution
                .as_ref()
                .and_then(|execution| execution.timeout_ms)
        });
    context.warnings = manifest
        .maintenance
        .as_ref()
        .and_then(|maintenance| maintenance.deprecation.as_ref())
        .and_then(|deprecation| deprecation.message.clone())
        .into_iter()
        .map(|message| format!("plugin deprecated: {message}"))
        .chain(
            action
                .and_then(|action| action.deprecation.as_ref())
                .and_then(|deprecation| deprecation.message.clone())
                .into_iter()
                .map(|message| format!("action deprecated: {message}")),
        )
        .collect();
    if let Some(runtime) = context.runtime.as_mut() {
        runtime.preferred_mode = action
            .and_then(|action| action.contract.as_ref())
            .map(|contract| contract.execution_mode)
            .or(runtime.preferred_mode);
        runtime.max_timeout_ms = context.timeout_ms.or(runtime.max_timeout_ms);
    }

    PluginRequest {
        plugin_id: plugin_id.to_owned(),
        action_id: action_id.to_owned(),
        payload: parse_payload(payload_text),
        context,
    }
}

fn parse_payload(payload_text: &str) -> serde_json::Value {
    let trimmed = payload_text.trim();
    if trimmed.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_str(trimmed)
            .unwrap_or_else(|_| serde_json::Value::String(payload_text.to_owned()))
    }
}

fn format_fit_badge(assessment: &HostFitAssessment) -> String {
    match assessment.status {
        HostFitStatus::Ready => format!("ready ({})", assessment.summary),
        HostFitStatus::Degraded => format!("degraded ({})", assessment.summary),
        HostFitStatus::Rejected => format!("rejected ({})", assessment.summary),
    }
}
