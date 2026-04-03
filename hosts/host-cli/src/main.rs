use std::path::PathBuf;

use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use host_core::{Playground, default_plugin_dir, render_response};
use plugin_abi::AbiPluginCatalog;
use plugin_manifest::PluginManifest;
use plugin_protocol::{HostKind, InvocationContext, PluginRequest};
use plugin_wasm::WasmPluginCatalog;

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "CLI host for the Rust plugin system playground"
)]
struct Cli {
    #[arg(long, default_value_os_t = default_plugin_dir())]
    plugin_dir: PathBuf,
    #[arg(long, default_value_os_t = std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))]
    workspace_root: PathBuf,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    List,
    Inspect {
        plugin_id: String,
    },
    Run {
        plugin_id: String,
        action_id: String,
        payload: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let playground = Playground::load(&cli.plugin_dir)?;
    let abi_catalog = plugin_abi::load_plugins_from_directory(&cli.plugin_dir)?;
    let wasm_catalog = plugin_wasm::load_plugins_from_workspace(&cli.workspace_root)?;

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
            print_manifest_group("Native dynamic library plugins", &playground.manifests());
            print_manifest_group(
                "ABI-stable plugins",
                &abi_catalog
                    .plugins
                    .iter()
                    .map(|plugin| plugin.manifest().clone())
                    .collect::<Vec<_>>(),
            );
            print_manifest_group(
                "WASM sandboxed plugins",
                &wasm_catalog
                    .plugins
                    .iter()
                    .map(|plugin| plugin.manifest().clone())
                    .collect::<Vec<_>>(),
            );
        }
        Command::Inspect { plugin_id } => {
            let manifest = find_manifest(&playground, &abi_catalog, &wasm_catalog, &plugin_id)
                .ok_or_else(|| anyhow!("no plugin named '{plugin_id}'"))?;
            print_manifest_details(&manifest);
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
                plugin.invoke(&build_request(
                    &plugin_id,
                    &action_id,
                    payload_text,
                    HostKind::Cli,
                    playground.plugin_dir(),
                    &cli.workspace_root,
                ))?
            } else if let Some(plugin) = wasm_catalog
                .plugins
                .iter()
                .find(|plugin| plugin.manifest().id == plugin_id)
            {
                plugin.invoke(&build_request(
                    &plugin_id,
                    &action_id,
                    payload_text,
                    HostKind::Cli,
                    playground.plugin_dir(),
                    &cli.workspace_root,
                ))?
            } else {
                return Err(anyhow!("no plugin named '{plugin_id}'"));
            };
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

fn print_manifest_group(title: &str, manifests: &[PluginManifest]) {
    println!("{title}");
    println!("{}", "-".repeat(title.len()));
    for manifest in manifests {
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
        println!();
    }
}

fn print_manifest_details(manifest: &PluginManifest) {
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
    println!("\nactions:");
    for action in &manifest.actions {
        println!("- {} :: {}", action.id, action.label);
        println!("  {}", action.description);
        if let Some(payload_hint) = &action.payload_hint {
            println!("  payload hint: {payload_hint}");
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
    plugin_id: &str,
    action_id: &str,
    payload_text: &str,
    host: HostKind,
    plugin_dir: &std::path::Path,
    workspace_root: &std::path::Path,
) -> PluginRequest {
    PluginRequest {
        plugin_id: plugin_id.to_owned(),
        action_id: action_id.to_owned(),
        payload: parse_payload(payload_text),
        context: InvocationContext {
            host,
            workspace_root: workspace_root.to_str().map(str::to_owned),
            plugin_dir: plugin_dir.to_str().map(str::to_owned),
            mode: Some("interactive".to_owned()),
        },
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
