use std::{
    borrow::Borrow,
    ffi::{c_char, CStr, CString},
    sync::{Mutex, OnceLock},
};

use anyhow::{bail, Context, Result};
use nvim::api::{nvim_api, nvim_keysets, nvim_types};
use nvim_rs::{slice_from_ffi_ref, types::NvimObject, IntoObject, NvimArray, NvimString};
use runtime::{init_wasm_state, InstanceId};
use slab::Slab;
use types::{FromWasmType, TryIntoWasmType};
use wasmtime::{
    component::{Component, Instance, Linker, ResourceAny, TypedFunc},
    Engine, Store,
};

mod runtime;
mod types;
pub mod wasmref;

/// Initializes the Nvim WASM module.
///
/// This function must be called before any other functions defined in this module.
///
/// # Panics
///
/// Panics when failing to create the wasm engine.
#[no_mangle]
pub extern "C" fn wasm_init() {
    init_wasm_state();
}

/// Loads the WASM binary to the global store and returns the instance ID.
///
/// # Safety
/// The `file_path` pointer must be a valid C-string.
//
// TODO: The requirement of `file_path` being a valid unicode string is probably over-restricted.
// See what the convention of file path is for Neovim.
#[no_mangle]
pub unsafe extern "C" fn wasm_load_file(
    file_path: *const c_char,
    errmsg: *mut *const c_char,
) -> i32 {
    assert!(!file_path.is_null());
    let errmsg = errmsg.as_mut().expect("expect errmsg to be non-null");
    let file_path = unsafe { CStr::from_ptr(file_path) }
        .to_str()
        .expect("File path is not a valid utf-8 string");
    let result = runtime::state().load_wasm_file(file_path);

    unwrap_or_set_error_and_return(result, errmsg, -1)
}

/// Calls a function exported by a WASM instance.
///
/// The function should be in the "root" of the WASM component (i.e., in the outer "world").
///
/// # Arguments
/// * `instance_id` - The instance ID returned by `wasm_load_file`.
/// * `func_name` - The function name.
/// * `args` - The arguments passed as a Neovim API array.
/// * `errmsg` - If errored, a string describing the error will be stored.
///
/// # Safety
/// * `func_name` should point to a valid C-string.
/// * `errmsg` should point to a valid `Error` struct.
#[no_mangle]
pub unsafe extern "C" fn wasm_call_func(
    instance_id: InstanceId,
    func_name: *const c_char,
    args: nvim_sys::Array,
    errmsg: *mut *const c_char,
) -> nvim_sys::Object {
    assert!(!func_name.is_null());
    let errmsg = errmsg.as_mut().expect("expect errmsg to be non-null");
    let func_name = CStr::from_ptr(func_name)
        .to_str()
        .expect("Function name is not a valid utf-8 string");
    let args = slice_from_ffi_ref(&args);
    let result = runtime::state().call_instance_func(instance_id, func_name, args);

    unwrap_or_set_error_and_return(result, errmsg, NvimObject::nil()).into_ffi()
}

/// Calls a WASM callback given the ref.
///
/// # Arguments
/// * `wasmref` - The WASM ref pointing to the callback. It contains the instance ID and the
///   callback reference.
/// * `name` - The name of the function or the event triggering the callback. If not null, then an
///   extra string of this name will be passed as the first argument to the callback.
/// * `args` - The arguments passed as a Neovim API array.
///
/// # Safety
/// * If `name` is not null, it should point to a valid C-string.
#[no_mangle]
pub unsafe extern "C" fn wasm_call_wasmref(
    wasmref: nvim_sys::WasmRef,
    name: *const c_char,
    args: nvim_sys::Array,
) -> nvim_sys::Object {
    let mut name_appended_args = Vec::<&NvimObject>::new();
    let callback_name_obj = name.as_ref().map(|s| {
        NvimString::new(
            CStr::from_ptr(s)
                .to_str()
                .expect("Callback name is not a valid utf-8 string")
                .to_owned(),
        )
        .into_object()
    });
    if let Some(ref callback_name) = callback_name_obj {
        name_appended_args.push(callback_name);
    }
    let args = slice_from_ffi_ref(&args);
    for arg in args {
        name_appended_args.push(arg);
    }
    match runtime::state().call_instance_callback(wasmref.instance_id, wasmref.ref_, args) {
        Ok(result) => result.into_ffi(),
        Err(err) => {
            nvim_rs::emsg(&format!(
                "Failed during the execution of the callback: {err:#}"
            ));
            NvimObject::nil().into_ffi()
        }
    }
}

unsafe fn unwrap_or_set_error_and_return<T>(
    result: Result<T>,
    errmsg: &mut *const c_char,
    err_val: T,
) -> T {
    match result {
        Ok(result) => {
            unsafe {
                *errmsg = std::ptr::null_mut();
            }
            result
        }
        Err(err) => {
            unsafe {
                *errmsg = CString::new(format!("{err:#}")).unwrap().into_raw();
            }
            err_val
        }
    }
}

const WASM_CLIENT_CALLBACK_INTERFACE: &str = "nvim:api/client-callback-impl";

// This generates all the types and interface defined in the wit file.
wasmtime::component::bindgen!("guest");
