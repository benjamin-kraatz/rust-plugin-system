//! ABI Evolution Example
//!
//! Demonstrates how `abi_stable`'s prefix-type mechanism lets you extend a
//! plugin interface **without breaking** already-compiled plugins.
//!
//! The key idea: the `#[sabi(last_prefix_field)]` attribute marks where a
//! version of the struct ends.  A newer host can add fields *after* that
//! marker.  When it loads an older plugin whose struct is shorter, the extra
//! fields simply return `None` – no crash, no ABI mismatch.

use abi_stable::StableAbi;
use abi_stable::std_types::RString;

// ---------------------------------------------------------------------------
// Version 1 of the plugin interface
// ---------------------------------------------------------------------------
// In the first release the host only asks for a manifest and an invoke fn.

/// V1 interface – the "original" contract.
///
/// `#[sabi(last_prefix_field)]` on `invoke_json` tells abi_stable that
/// any field *after* this one was added in a later version and may be
/// absent when loading an older plugin.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = PluginModuleV1Ref)))]
pub struct PluginModuleV1 {
    pub manifest_json: extern "C" fn() -> RString,

    #[sabi(last_prefix_field)]
    pub invoke_json: extern "C" fn(RString) -> RString,
}

// ---------------------------------------------------------------------------
// Version 2 of the plugin interface
// ---------------------------------------------------------------------------
// A later release adds an optional `capabilities_json` query so the host
// can discover fine-grained capabilities without parsing the full manifest.
// Crucially the struct layout *starts* the same as V1.

/// V2 interface – backwards-compatible extension.
///
/// The first two fields are identical to V1.  The new
/// `capabilities_json` is placed *after* the old `last_prefix_field`.
/// This field becomes the new `last_prefix_field`.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = PluginModuleV2Ref)))]
pub struct PluginModuleV2 {
    pub manifest_json: extern "C" fn() -> RString,
    pub invoke_json: extern "C" fn(RString) -> RString,

    /// NEW in V2 – returns a JSON array of capability keys.
    #[sabi(last_prefix_field)]
    pub capabilities_json: extern "C" fn() -> RString,
}

fn main() {
    println!("=== ABI Evolution Demo ===\n");

    println!("This example illustrates how abi_stable's prefix-type system");
    println!("lets a host extend its plugin ABI without breaking older plugins.\n");

    // Explain the V1 → V2 scenario.
    println!("--- V1 interface ---");
    println!("Fields: manifest_json, invoke_json (#[sabi(last_prefix_field)])");
    println!(
        "Size  : {} bytes (on this platform)\n",
        std::mem::size_of::<PluginModuleV1>()
    );

    println!("--- V2 interface ---");
    println!("Fields: manifest_json, invoke_json, capabilities_json (#[sabi(last_prefix_field)])");
    println!(
        "Size  : {} bytes (on this platform)\n",
        std::mem::size_of::<PluginModuleV2>()
    );

    println!("Scenario A: V2 host loads a V1 plugin");
    println!("  → manifest_json  : ✅ available");
    println!("  → invoke_json    : ✅ available");
    println!("  → capabilities_json: ❌ returns None (field did not exist in V1)");
    println!("  → The host can gracefully degrade – no crash.\n");

    println!("Scenario B: V1 host loads a V2 plugin");
    println!("  → manifest_json  : ✅ available");
    println!("  → invoke_json    : ✅ available");
    println!("  → capabilities_json: ignored (old host doesn't know about it)");
    println!("  → Works perfectly – the extra field is simply unused.\n");

    println!("Scenario C: V2 host loads a V2 plugin");
    println!("  → All three fields available – full functionality.\n");

    // Show the real abi-stable-greeter plugin manifest if available.
    println!("--- Real plugin example ---");
    println!("The `abi-stable-greeter` plugin in this workspace uses the");
    println!("`plugin-abi` crate which defines a similar prefix-type module:");
    println!("  AbiPluginModule {{");
    println!("      #[sabi(last_prefix_field)]");
    println!("      manifest_json: extern \"C\" fn() -> RString,");
    println!("      invoke_json:   extern \"C\" fn(RString) -> RString,");
    println!("  }}");
    println!();
    println!("Build it with:  cargo build -p abi-stable-greeter");
    println!("Then load it with the minimal-loader or host-cli to see it in action.");
}
