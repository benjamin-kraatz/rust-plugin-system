//! Plugin Discovery Example
//!
//! Scans the default plugin directory, groups plugins by architecture and
//! skill level, and prints a summary table.

use std::collections::BTreeMap;

use anyhow::Result;
use host_core::Playground;
use plugin_manifest::{HostKind, PluginArchitecture, PluginManifest, SkillLevel};

fn arch_label(a: &PluginArchitecture) -> &'static str {
    match a {
        PluginArchitecture::NativeJson => "Native-JSON",
        PluginArchitecture::AbiStable => "ABI-Stable",
        PluginArchitecture::Wasm => "WASM",
    }
}

fn skill_label(s: &SkillLevel) -> &'static str {
    match s {
        SkillLevel::Basic => "Basic",
        SkillLevel::Intermediate => "Intermediate",
        SkillLevel::Advanced => "Advanced",
        SkillLevel::Expert => "Expert",
    }
}

fn supports_host(manifest: &PluginManifest, host: &HostKind) -> bool {
    manifest
        .supported_hosts
        .iter()
        .any(|h| matches!(h, HostKind::Any) || std::mem::discriminant(h) == std::mem::discriminant(host))
}

fn main() -> Result<()> {
    let playground = Playground::load_default()?;

    // Print warnings.
    for w in playground.warnings() {
        eprintln!("⚠  {w}");
    }

    let manifests = playground.manifests();
    if manifests.is_empty() {
        println!("No plugins discovered. Build some first:");
        println!("  cargo build -p hello-world -p logger -p formatter");
        return Ok(());
    }

    println!("=== Plugin Discovery ===\n");
    println!("Plugin directory: {}\n", playground.plugin_dir().display());

    // ── Summary table ──
    println!("{:<24} {:<14} {:<14} Hosts", "Plugin", "Architecture", "Skill");
    println!("{}", "─".repeat(72));
    for m in &manifests {
        let hosts: Vec<String> = m.supported_hosts.iter().map(|h| format!("{h:?}")).collect();
        println!(
            "{:<24} {:<14} {:<14} {}",
            m.name,
            arch_label(&m.architecture),
            skill_label(&m.skill_level),
            hosts.join(", "),
        );
    }

    // ── Group by architecture ──
    println!("\n--- By Architecture ---");
    let mut by_arch: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for m in &manifests {
        by_arch.entry(arch_label(&m.architecture)).or_default().push(&m.name);
    }
    for (arch, names) in &by_arch {
        println!("  {arch}: {}", names.join(", "));
    }

    // ── Group by skill level ──
    println!("\n--- By Skill Level ---");
    let mut by_skill: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for m in &manifests {
        by_skill.entry(skill_label(&m.skill_level)).or_default().push(&m.name);
    }
    for (skill, names) in &by_skill {
        println!("  {skill}: {}", names.join(", "));
    }

    // ── Filtering demo ──
    println!("\n--- Filter: Advanced+ plugins that support Web ---");
    let advanced_web: Vec<&str> = manifests
        .iter()
        .filter(|m| {
            matches!(m.skill_level, SkillLevel::Advanced | SkillLevel::Expert)
                && supports_host(m, &HostKind::Web)
        })
        .map(|m| m.name.as_str())
        .collect();

    if advanced_web.is_empty() {
        println!("  (none found – try building more plugins)");
    } else {
        for name in &advanced_web {
            println!("  • {name}");
        }
    }

    println!("\nDiscovered {} plugin(s) total.", manifests.len());

    Ok(())
}
