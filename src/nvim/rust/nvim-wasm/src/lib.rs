use std::{
    ffi::{c_char, CStr, CString},
    sync::{Mutex, OnceLock},
};

use anyhow::{bail, Context, Result};
use nvim::api::{nvim_api, nvim_keysets, nvim_types};
use nvim_rs::{slice_from_ffi_ref, types::NvimObject};
use slab::Slab;
use types::{FromWasmType, TryIntoWasmType};
use wasmtime::{
    component::{Component, Instance, Linker, TypedFunc},
    Engine, Store,
};

mod types;

/// Initializes the Nvim WASM module.
///
/// This function must be called before any other functions defined in this module.
///
/// # Panics
///
/// Panics when failing to create the wasm engine.
#[no_mangle]
pub extern "C" fn wasm_rs_init() {
    let config = wasm_config();
    init_wasm_state(&config);
}

/// Loads the WASM binary to the global store and returns the instance ID.
///
/// # Safety
/// The `file_path` pointer must be a valid UTF-8 CString.
//
// TODO: The requirement of `file_path` being a valid unicode string is probably over-restricted.
// See what the convention of file path is for Neovim.
#[no_mangle]
pub unsafe extern "C" fn wasm_load_file(
    file_path: *const c_char,
    errmsg: *mut *const c_char,
) -> i32 {
    let file_path = unsafe { CStr::from_ptr(file_path) }
        .to_str()
        .expect("File path is not a valid utf-8 string");
    let result = wasm_load_file_impl(file_path);

    unwrap_or_set_error_and_return(result, errmsg, -1)
}

/// Calls a function exported by a WASM instance.
///
/// # Arguments
/// * `instance_id` - The instance ID returned by `wasm_load_file`.
/// * `func_name` - The function name.
/// * `args` - The arguments passed as a Neovim API array.
/// * `errmsg` - If errored, a string describing the error will be stored.
///
/// # Safety
/// All the pointers argument should be non-null and `errmsg` should point to a valid `Error`
/// struct.
#[no_mangle]
pub unsafe extern "C" fn wasm_call_func(
    instance_id: i32,
    func_name: *const c_char,
    args: nvim_sys::Array,
    errmsg: *mut *const c_char,
) -> nvim_sys::Object {
    let func_name = CStr::from_ptr(func_name)
        .to_str()
        .expect("Function name is not a valid utf-8 string");
    let args = slice_from_ffi_ref(&args);
    let result = wasm_call_func_impl(instance_id, func_name, args);

    unwrap_or_set_error_and_return(result, errmsg, NvimObject::nil()).into_ffi()
}

unsafe fn unwrap_or_set_error_and_return<T>(
    result: Result<T>,
    errmsg: *mut *const c_char,
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

/// The global state of the Nvim WASM module.
struct WasmState {
    engine: Engine,
    mutate_state: Mutex<WasmMutateState>,
}

struct WasmMutateState {
    store: Store<NvimHost>,
    linker: Linker<NvimHost>,
    instances: Slab<Instance>,
}

/// The global instance of the Nvim WASM module state.
static WASM_STATE: OnceLock<WasmState> = OnceLock::new();

/// Returns the global WASM state.
fn state() -> &'static WasmState {
    WASM_STATE.get().expect("Wasm state is not initialized")
}

/// Returns the config for creating the WASM engine.
fn wasm_config() -> wasmtime::Config {
    let mut config = wasmtime::Config::new();
    config.wasm_component_model(true);
    config
}

fn init_wasm_state(config: &wasmtime::Config) {
    let engine = Engine::new(config).expect("Failed to create wasm engine");
    let store = Store::new(&engine, NvimHost);
    let mut linker = Linker::new(&engine);
    Plugin::add_to_linker(&mut linker, |state| state)
        .expect("Failed to add the host bindings to WASM linker");
    WASM_STATE
        .set(WasmState {
            engine,
            mutate_state: Mutex::new(WasmMutateState {
                store,
                linker,
                instances: Slab::new(),
            }),
        })
        .map_err(|_| ())
        .expect("Failed to initialize wasm state");
}

const MUTEX_POISONED_ERR: &str = "Mutex is poisoned";

fn wasm_load_file_impl(file_path: &str) -> Result<i32> {
    // TODO: It will be helpful to cache the compiled component here.
    let component = Component::from_file(&state().engine, file_path)
        .with_context(|| format!("Failed to load the WASM file {}", file_path))?;

    let mut mutate_state = state().mutate_state.lock().expect(MUTEX_POISONED_ERR);
    let mutate_state = &mut *mutate_state;
    // This should rarely happen. No one loads 2^31 WASM files...
    if mutate_state.instances.len() >= i32::MAX as usize {
        bail!("Cannot load new WASM file because the number of instances has reached the limit.");
    }
    let (_, instance) =
        Plugin::instantiate(&mut mutate_state.store, &component, &mutate_state.linker)
            .with_context(|| format!("Failed to instantiate the WASM file {}", file_path))?;

    Ok(mutate_state.instances.insert(instance) as i32)
}

fn wasm_call_func_impl(
    instance_id: i32,
    func_name: &str,
    args: &[NvimObject],
) -> Result<NvimObject> {
    if instance_id < 0 {
        bail!("Instance ID should be non-negative, got {instance_id}")
    }
    let mut mutate_state = state().mutate_state.lock().expect(MUTEX_POISONED_ERR);
    let instance = *mutate_state
        .instances
        .get(instance_id as usize)
        .with_context(|| format!("Cannot find instance with ID = {instance_id}"))?;

    let func: TypedFunc<(Vec<nvim_api::Object>,), (nvim_api::Object,)> = instance
        .get_func(&mut mutate_state.store, func_name)
        .with_context(|| format!("Cannot find function {func_name} in instance {instance_id}"))?
        .typed(&mut mutate_state.store)
        .with_context(|| {
            format!("The function {func_name} is not a function of type list<Object> -> Object")
        })?;
    let args = args
        .iter()
        .map(|obj| Ok(obj.clone().try_into_wasm_type()?))
        .collect::<Result<Vec<_>>>()?;

    let (result,) = func.call(&mut mutate_state.store, (args,)).with_context(|| {
      format!("The function call to {func_name} trapped (an runtime exception is raised) or failed")
    })?;
    Ok(NvimObject::from_wasm_type(result))
}

// This generates all the types and interface defined in the wit file.
wasmtime::component::bindgen!("plugin");

/// Implements the host bindings.
///
/// See `wit/nvim.wit` for the definition of the host bindings.
struct NvimHost;

include!(concat!(env!("OUT_DIR"), "/api_impl.rs"));

impl nvim_types::Host for NvimHost {}
impl nvim_keysets::Host for NvimHost {}
