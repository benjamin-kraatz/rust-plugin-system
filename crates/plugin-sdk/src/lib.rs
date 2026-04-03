//! Plugin SDK for the **Rust Plugin System**.
//!
//! This is the primary crate plugin authors depend on.  It provides everything
//! needed to build a native JSON dynamic-library plugin:
//!
//! * The [`JsonPlugin`] trait — implement `manifest()` and `invoke()` on your
//!   plugin struct.
//! * The [`export_plugin!`] macro — generates the three C-callable entry points
//!   (`plugin_manifest_json`, `plugin_invoke_json`, `plugin_free_c_string`) that
//!   the host loads at runtime.
//! * Re-exports of `plugin_api`, `plugin_manifest`, and `plugin_protocol` so you
//!   only need a single dependency.
//!
//! # Quick start
//!
//! Add to your plugin's `Cargo.toml`:
//!
//! ```toml
//! [lib]
//! crate-type = ["cdylib"]
//!
//! [dependencies]
//! plugin-sdk = "0.1"
//! serde_json = "1"
//! ```
//!
//! Then implement the trait and call the macro:
//!
//! ```rust,ignore
//! use plugin_sdk::{JsonPlugin, export_plugin};
//! use plugin_sdk::plugin_manifest::{PluginManifest, PluginAction};
//! use plugin_sdk::plugin_protocol::{PluginRequest, PluginResponse};
//! use plugin_capabilities::{HostKind, PluginArchitecture, SkillLevel};
//!
//! pub struct MyPlugin;
//!
//! impl JsonPlugin for MyPlugin {
//!     fn manifest() -> PluginManifest {
//!         PluginManifest {
//!             id: "my-plugin".into(),
//!             name: "My Plugin".into(),
//!             version: "0.1.0".into(),
//!             description: "A sample plugin.".into(),
//!             architecture: PluginArchitecture::NativeJson,
//!             skill_level: SkillLevel::Basic,
//!             supported_hosts: vec![HostKind::Any],
//!             actions: vec![PluginAction {
//!                 id: "greet".into(),
//!                 label: "Greet".into(),
//!                 description: "Returns a greeting.".into(),
//!                 ..Default::default()
//!             }],
//!             ..Default::default()
//!         }
//!     }
//!
//!     fn invoke(request: PluginRequest) -> Result<PluginResponse, String> {
//!         let name = request.payload
//!             .as_ref()
//!             .and_then(|p| p["name"].as_str())
//!             .unwrap_or("World");
//!         Ok(PluginResponse::success(
//!             &request.plugin_id,
//!             &request.action_id,
//!             format!("Hello, {name}!"),
//!             "Greeting delivered.",
//!         ))
//!     }
//! }
//!
//! export_plugin!(MyPlugin);
//! ```

pub use plugin_api;
pub use plugin_manifest;
pub use plugin_protocol;

use plugin_manifest::PluginManifest;
use plugin_protocol::{PluginRequest, PluginResponse};

pub trait JsonPlugin {
    fn manifest() -> PluginManifest;
    fn invoke(request: PluginRequest) -> Result<PluginResponse, String>;
}

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
