//! C FFI boundary helpers for the **Rust Plugin System**.
//!
//! Plugin dynamic libraries and their host loaders communicate via a small set
//! of C-callable symbols.  This crate provides:
//!
//! * The canonical symbol name constants ([`MANIFEST_SYMBOL`], [`INVOKE_SYMBOL`],
//!   [`FREE_SYMBOL`]).
//! * Serialisation helpers ([`manifest_to_json_ptr`], [`response_to_json_ptr`])
//!   that turn Rust values into heap-allocated C strings the host can read.
//! * Deserialisation helpers ([`request_from_json_ptr`]) that parse the host's
//!   request JSON back into a typed [`PluginRequest`].
//! * Memory helpers ([`copy_c_string`], [`reclaim_c_string`]) for safe ownership
//!   transfer across the FFI boundary.
//!
//! Plugin authors generally do **not** call these directly — use the
//! `export_plugin!` macro from `plugin-sdk` instead.  Host authors use these
//! constants when loading a plugin library via `libloading`.
//!
//! # Safety
//!
//! All functions that accept raw pointers are marked `unsafe`.  Callers must
//! guarantee that the pointer is either null or points to a valid
//! NUL-terminated C string for the duration of the call.

use std::ffi::{CStr, CString, c_char};

use plugin_manifest::PluginManifest;
use plugin_protocol::{PluginRequest, PluginResponse};

pub const MANIFEST_SYMBOL: &[u8] = b"plugin_manifest_json\0";
pub const INVOKE_SYMBOL: &[u8] = b"plugin_invoke_json\0";
pub const FREE_SYMBOL: &[u8] = b"plugin_free_c_string\0";

pub fn manifest_to_json_ptr(manifest: &PluginManifest) -> *mut c_char {
    json_string_to_ptr(
        serde_json::to_string(manifest).expect("plugin manifest serialization should succeed"),
    )
}

pub fn response_to_json_ptr(response: &PluginResponse) -> *mut c_char {
    json_string_to_ptr(
        serde_json::to_string(response).expect("plugin response serialization should succeed"),
    )
}

pub fn json_string_to_ptr(json: String) -> *mut c_char {
    CString::new(json)
        .expect("plugin JSON must not contain embedded NUL bytes")
        .into_raw()
}

/// # Safety
///
/// The pointer must either be null or point to a valid NUL-terminated C string.
pub unsafe fn request_from_json_ptr(ptr: *const c_char) -> Result<PluginRequest, String> {
    let request_json = unsafe { copy_c_string(ptr) }?;
    serde_json::from_str(&request_json).map_err(|error| error.to_string())
}

/// # Safety
///
/// The pointer must either be null or point to a valid NUL-terminated C string.
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

/// # Safety
///
/// The pointer must have been produced by `CString::into_raw` in the same dynamic library.
pub unsafe fn reclaim_c_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }

    let _ = unsafe { CString::from_raw(ptr) };
}
