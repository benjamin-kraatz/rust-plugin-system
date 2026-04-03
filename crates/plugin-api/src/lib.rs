//! Low-level FFI helpers for native JSON plugins.
//!
//! This crate defines the symbol names and pointer conversion utilities used by
//! the native dynamic-library plugin ABI.
//!
//! Most plugin authors should use `plugin-sdk` instead of calling these APIs
//! directly.

use std::ffi::{CStr, CString, c_char};

use plugin_manifest::PluginManifest;
use plugin_protocol::{PluginRequest, PluginResponse};

/// Symbol name for the plugin manifest export.
///
/// The trailing `\0` is required because this byte string is passed directly
/// to dynamic symbol lookup APIs that expect C strings.
pub const MANIFEST_SYMBOL: &[u8] = b"plugin_manifest_json\0";
/// Symbol name for the plugin invocation export.
///
/// The trailing `\0` is required because this byte string is passed directly
/// to dynamic symbol lookup APIs that expect C strings.
pub const INVOKE_SYMBOL: &[u8] = b"plugin_invoke_json\0";
/// Symbol name for the plugin string free export.
///
/// The trailing `\0` is required because this byte string is passed directly
/// to dynamic symbol lookup APIs that expect C strings.
pub const FREE_SYMBOL: &[u8] = b"plugin_free_c_string\0";

/// Serializes a manifest and returns an owned C string pointer.
///
/// The returned pointer must eventually be reclaimed by calling
/// [`reclaim_c_string`].
pub fn manifest_to_json_ptr(manifest: &PluginManifest) -> *mut c_char {
    json_string_to_ptr(
        serde_json::to_string(manifest).expect("plugin manifest serialization should succeed"),
    )
}

/// Serializes a response and returns an owned C string pointer.
///
/// The returned pointer must eventually be reclaimed by calling
/// [`reclaim_c_string`].
pub fn response_to_json_ptr(response: &PluginResponse) -> *mut c_char {
    json_string_to_ptr(
        serde_json::to_string(response).expect("plugin response serialization should succeed"),
    )
}

/// Converts JSON text into an owned raw C string pointer.
///
/// This is the lowest-level allocation helper used by higher-level conversion
/// APIs in this crate.
///
/// # Panics
///
/// Panics if `json` contains embedded NUL bytes.
pub fn json_string_to_ptr(json: String) -> *mut c_char {
    CString::new(json)
        .expect("plugin JSON must not contain embedded NUL bytes")
        .into_raw()
}

/// Parses a plugin request from a raw C string pointer.
///
/// # Errors
///
/// Returns an error if the pointer is null, not valid UTF-8, or contains
/// invalid request JSON.
/// # Safety
///
/// `ptr` must be null or point to a valid NUL-terminated C string that remains
/// alive for the duration of this call.
pub unsafe fn request_from_json_ptr(ptr: *const c_char) -> Result<PluginRequest, String> {
    let request_json = unsafe { copy_c_string(ptr) }?;
    serde_json::from_str(&request_json).map_err(|error| error.to_string())
}

/// Copies a borrowed C string pointer into an owned Rust `String`.
///
/// # Errors
///
/// Returns an error when `ptr` is null or when the pointed string is not valid
/// UTF-8.
/// # Safety
///
/// `ptr` must be null or point to a valid NUL-terminated C string that remains
/// alive for the duration of this call.
pub unsafe fn copy_c_string(ptr: *const c_char) -> Result<String, String> {
    if ptr.is_null() {
        return Err("received a null C string pointer".to_owned());
    }

    let value = unsafe { CStr::from_ptr(ptr) };
    value
        .to_str()
        .map(str::to_owned)
        .map_err(|error| error.to_string())
}

/// Reclaims ownership of a raw C string pointer previously allocated by this
/// crate.
/// # Safety
///
/// `ptr` must have been returned by `CString::into_raw` from the same dynamic
/// library instance. Passing any other pointer (or a pointer already reclaimed)
/// is undefined behavior.
pub unsafe fn reclaim_c_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }

    let _ = unsafe { CString::from_raw(ptr) };
}
