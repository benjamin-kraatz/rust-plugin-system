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
