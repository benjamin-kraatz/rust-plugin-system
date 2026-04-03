//! High-level SDK for authoring native JSON plugins.
//!
//! If you are building a plugin crate, this is the primary entry point.
//! Typical workflow:
//!
//! 1. Build a [`plugin_manifest::PluginManifest`].
//! 2. Implement [`JsonPlugin`].
//! 3. Export ABI symbols with [`export_plugin!`].
//!
//! # Examples
//!
//! ```no_run
//! use plugin_sdk::{JsonPlugin, export_plugin};
//! use plugin_sdk::plugin_manifest::{PluginArchitecture, PluginManifest, SkillLevel};
//! use plugin_sdk::plugin_protocol::{PluginRequest, PluginResponse};
//!
//! struct HelloPlugin;
//!
//! impl JsonPlugin for HelloPlugin {
//!     fn manifest() -> PluginManifest {
//!         PluginManifest::new(
//!             "hello-world",
//!             "Hello World",
//!             "0.1.0",
//!             "Minimal plugin example",
//!             PluginArchitecture::NativeJson,
//!             SkillLevel::Basic,
//!         )
//!     }
//!
//!     fn invoke(request: PluginRequest) -> Result<PluginResponse, String> {
//!         Ok(PluginResponse::ok(
//!             request.plugin_id,
//!             request.action_id,
//!             "Invocation finished",
//!             "Plugin executed successfully",
//!         ))
//!     }
//! }
//!
//! export_plugin!(HelloPlugin);
//! ```

#[doc(inline)]
pub use plugin_api;
#[doc(inline)]
pub use plugin_manifest;
#[doc(inline)]
pub use plugin_protocol;

use plugin_manifest::PluginManifest;
use plugin_protocol::{PluginRequest, PluginResponse};

/// Trait implemented by native JSON plugins.
///
/// Hosts discover and invoke plugins through generated ABI exports. The trait
/// keeps plugin implementation code fully Rust-native while the SDK handles ABI
/// wiring.
pub trait JsonPlugin {
    /// Returns the static manifest for this plugin type.
    fn manifest() -> PluginManifest;
    /// Executes one request and returns a structured response.
    ///
    /// Return `Err` for operational failures. The SDK turns the error into a
    /// standardized `PluginResponse::error` payload for hosts.
    fn invoke(request: PluginRequest) -> Result<PluginResponse, String>;
}

/// Exports the required C ABI symbols for a plugin type.
///
/// The generated functions are:
///
/// - `plugin_manifest_json`
/// - `plugin_invoke_json`
/// - `plugin_free_c_string`
///
/// These symbols are loaded by host-side runtime loaders.
#[macro_export]
macro_rules! export_plugin {
    ($plugin_ty:ty) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn plugin_manifest_json() -> *mut ::std::ffi::c_char {
            let manifest = <$plugin_ty as $crate::JsonPlugin>::manifest();
            $crate::plugin_api::manifest_to_json_ptr(&manifest)
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn plugin_invoke_json(
            request_json: *const ::std::ffi::c_char,
        ) -> *mut ::std::ffi::c_char {
            let request = match unsafe { $crate::plugin_api::request_from_json_ptr(request_json) } {
                Ok(request) => request,
                Err(error) => {
                    let response = $crate::plugin_protocol::PluginResponse::error(
                        "unknown-plugin",
                        "decode-request",
                        "Failed to decode request",
                        error,
                    );
                    return $crate::plugin_api::response_to_json_ptr(&response);
                }
            };

            let plugin_id = request.plugin_id.clone();
            let action_id = request.action_id.clone();

            match <$plugin_ty as $crate::JsonPlugin>::invoke(request) {
                Ok(response) => $crate::plugin_api::response_to_json_ptr(&response),
                Err(error) => {
                    let response = $crate::plugin_protocol::PluginResponse::error(
                        plugin_id,
                        action_id,
                        "Plugin invocation failed",
                        error,
                    );
                    $crate::plugin_api::response_to_json_ptr(&response)
                }
            }
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn plugin_free_c_string(ptr: *mut ::std::ffi::c_char) {
            unsafe { $crate::plugin_api::reclaim_c_string(ptr) };
        }
    };
}
