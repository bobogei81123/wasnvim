use std::{
    ffi::{c_char, CStr},
    sync::OnceLock,
};

use anyhow::Context;
use nvim_rs::types::{NvimArray, NvimObject, NvimString};
use wasmtime::{
    component::{Component, Linker},
    Engine, Store,
};

/// Initializes the Nvim WASM module.
///
/// This function must be called before any other functions defined in this module.
///
/// # Panics
///
/// Panics when failing to create the wasm engine.
#[no_mangle]
pub extern "C" fn wasm_rs_init() {
    let mut config = wasmtime::Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config).expect("Failed to create wasm engine");
    WASM_STATE
        .set(WasmState {
            engine,
            state: NvimHost,
        })
        .map_err(|_| ())
        .expect("Failed to initialize wasm state");
}

/// Runs the wasm component binary with the given file path.
///
/// # Safety
///
/// The `file_path` pointer must be a valid CString.
#[no_mangle]
pub unsafe extern "C" fn wasm_rs_run(file_path: *const c_char) {
    let file_path = unsafe { CStr::from_ptr(file_path) };
    if let Err(err) = wasm_run_impl(file_path.to_str().unwrap()) {
        nvim_rs::emsg(&format!("{:?}", err));
    }
}

/// The global state of the Nvim WASM module.
struct WasmState {
    engine: Engine,
    #[allow(dead_code)]
    state: NvimHost,
}

/// The global instance of the Nvim WASM module state.
static WASM_STATE: OnceLock<WasmState> = OnceLock::new();

fn engine() -> &'static Engine {
    &WASM_STATE
        .get()
        .expect("Wasm state is not initialized")
        .engine
}

pub fn wasm_run_impl(file_path: &str) -> anyhow::Result<()> {
    let component = Component::from_file(engine(), file_path)
        .with_context(|| format!("Failed to load component from {}", file_path))?;

    let mut linker = Linker::new(engine());
    Plugin::add_to_linker(&mut linker, |state: &mut NvimHost| state)
        .context("Failed to create wasm linker")?;

    let mut store = Store::new(engine(), NvimHost);
    let (bindings, _) = Plugin::instantiate(&mut store, &component, &linker)
        .context("Failed to instantiate plugin")?;

    bindings
        .call_run(&mut store)
        .with_context(|| "Failed to run the wasm plugin".to_string())
}

// This generates `PluginImports` trait that contains all the required host-bindings.
wasmtime::component::bindgen!("plugin");
/// Implements the host bindings.
///
/// See `wit/nvim.wit` for the definition of the host bindings.
struct NvimHost;

impl From<Object> for NvimObject {
    fn from(value: Object) -> Self {
        match value {
            Object::Nil => NvimObject::Nil,
            Object::Boolean(b) => NvimObject::Boolean(b),
            Object::Integer(i) => NvimObject::Integer(i),
            Object::Float(f) => NvimObject::Float(f),
            Object::String(s) => NvimObject::String(NvimString::new(s).unwrap()),
        }
    }
}

impl From<NvimObject> for Object {
    fn from(value: NvimObject) -> Self {
        match value {
            NvimObject::Nil => Object::Nil,
            NvimObject::Boolean(b) => Object::Boolean(b),
            NvimObject::Integer(i) => Object::Integer(i),
            NvimObject::Float(f) => Object::Float(f),
            NvimObject::String(s) => Object::String(s.into_string()),
            _ => unimplemented!(),
        }
    }
}

impl PluginImports for NvimHost {
    fn nvim_exec(&mut self, cmd: String) -> wasmtime::Result<Result<(), String>> {
        Ok(nvim_rs::nvim_command(cmd).map_err(|err| format!("{}", err)))
    }

    fn nvim_call_function(
        &mut self,
        name: String,
        args: Vec<Object>,
    ) -> wasmtime::Result<Result<Object, String>> {
        let mut args_vec = NvimArray::new();
        for arg in args {
            args_vec.push(arg.into());
        }
        Ok(nvim_rs::nvim_call_function(name, args_vec)
            .map(Object::from)
            .map_err(|err| format!("{}", err)))
    }
}
