use std::path::PathBuf;

use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use host_core::{Playground, default_plugin_dir, render_response};

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "CLI host for the Rust plugin system playground"
)]
struct Cli {
    #[arg(long, default_value_os_t = default_plugin_dir())]
    plugin_dir: PathBuf,
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

    if !playground.warnings().is_empty() {
        eprintln!("discovery warnings:");
        for warning in playground.warnings() {
            eprintln!("- {warning}");
        }
        eprintln!();
    }

    match cli.command.unwrap_or(Command::List) {
        Command::List => {
            println!("plugin dir: {}\n", playground.plugin_dir().display());
            for manifest in playground.manifests() {
                println!(
                    "- {} ({}) [{} actions]",
                    manifest.id,
                    manifest.name,
                    manifest.actions.len()
                );
                println!("  {}", manifest.description);
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
        Command::Inspect { plugin_id } => {
            let manifest = playground
                .manifests()
                .into_iter()
                .find(|manifest| manifest.id == plugin_id)
                .ok_or_else(|| anyhow!("no plugin named '{plugin_id}'"))?;

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
            for action in manifest.actions {
                println!("- {} :: {}", action.id, action.label);
                println!("  {}", action.description);
                if let Some(payload_hint) = action.payload_hint {
                    println!("  payload hint: {payload_hint}");
                }
            }
        }
        Command::Run {
            plugin_id,
            action_id,
            payload,
        } => {
            let response = playground.invoke_text(
                &plugin_id,
                &action_id,
                payload.as_deref().unwrap_or("{}"),
                plugin_manifest_host(),
            )?;
            println!("{}", render_response(&response));
        }
    }

    Ok(())
}

fn plugin_manifest_host() -> plugin_protocol::HostKind {
    plugin_protocol::HostKind::Cli
}
