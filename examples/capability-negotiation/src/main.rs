//! Capability Negotiation Example
//!
//! Loads every plugin via `Playground::load_default()` and shows how the
//! host/plugin capability-matching flow works for different host kinds.

use anyhow::Result;
use host_core::{Playground, assess_host_fit};
use plugin_capabilities::HostKind;
use plugin_protocol::InvocationContext;

/// All host kinds we want to test against.
const HOST_KINDS: &[HostKind] = &[
    HostKind::Cli,
    HostKind::Egui,
    HostKind::Web,
    HostKind::Service,
];

fn main() -> Result<()> {
    let playground = Playground::load_default()?;

    // Show any discovery warnings first.
    for w in playground.warnings() {
        eprintln!("⚠  {w}");
    }

    let manifests = playground.manifests();
    if manifests.is_empty() {
        println!("No plugins found. Build some first:");
        println!("  cargo build -p hello-world -p logger -p formatter");
        return Ok(());
    }

    println!("=== Capability Negotiation ===\n");
    println!(
        "Assessing {} plugin(s) against {} host kind(s).\n",
        manifests.len(),
        HOST_KINDS.len()
    );

    for manifest in &manifests {
        println!("─── {} ({}) ───", manifest.name, manifest.id);

        // Show declared capabilities.
        if manifest.capabilities.is_empty() {
            println!("  capabilities: (none declared)");
        } else {
            let caps: Vec<&str> = manifest
                .capabilities
                .iter()
                .map(|c| c.key.as_str())
                .collect();
            println!("  capabilities: {}", caps.join(", "));
        }

        // Show supported hosts.
        let hosts: Vec<String> = manifest
            .supported_hosts
            .iter()
            .map(|h| format!("{h:?}"))
            .collect();
        println!("  supported hosts: {}", hosts.join(", "));

        // Show capability contract constraints if present.
        if let Some(cc) = &manifest.capability_contract {
            if !cc.required.is_empty() {
                let keys: Vec<&str> = cc.required.iter().map(|r| r.key.as_str()).collect();
                println!("  required capabilities: {}", keys.join(", "));
            }
            if !cc.optional.is_empty() {
                let keys: Vec<&str> = cc.optional.iter().map(|r| r.key.as_str()).collect();
                println!("  optional capabilities: {}", keys.join(", "));
            }
            if !cc.degradation.is_empty() {
                println!("  degradation rules: {}", cc.degradation.len());
            }
        }

        // Assess fitness for each host kind.
        println!();
        for &host in HOST_KINDS {
            let context = InvocationContext {
                host,
                ..Default::default()
            };

            let assessment = assess_host_fit(manifest, &context);
            let icon = match assessment.status {
                host_core::HostFitStatus::Ready => "✅",
                host_core::HostFitStatus::Degraded => "⚠️",
                host_core::HostFitStatus::Rejected => "❌",
            };
            println!(
                "  {:?}: {icon} {:?} – {}",
                host, assessment.status, assessment.summary
            );

            // Show negotiation details when interesting.
            let neg = &assessment.negotiation;
            if !neg.missing_required.is_empty() {
                let keys: Vec<&str> = neg
                    .missing_required
                    .iter()
                    .map(|r| r.key.as_str())
                    .collect();
                println!("              missing required: {}", keys.join(", "));
            }
            if !neg.degraded_features.is_empty() {
                for df in &neg.degraded_features {
                    println!(
                        "              degraded: {} → {}",
                        df.feature,
                        df.fallback.as_deref().unwrap_or("(none)")
                    );
                }
            }
        }
        println!();
    }

    Ok(())
}
