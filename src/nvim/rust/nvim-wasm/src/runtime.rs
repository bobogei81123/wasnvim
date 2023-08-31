use std::{
    borrow::Borrow,
    sync::{Mutex, OnceLock},
};

use anyhow::{bail, Context, Result};
use nvim_rs::NvimObject;
use slab::Slab;
use wasmtime::{
    component::{Component, Instance, Linker, TypedFunc},
    Engine, Store,
};

use crate::{
    nvim::api::{nvim_api, nvim_keysets, nvim_types},
    types::{FromWasmType, TryIntoWasmType},
    Guest,
};

/// The global state of the Nvim WASM module.
pub(crate) struct WasmState {
    engine: Engine,
    mutate_state: Mutex<WasmMutateState>,
}

/// The mutating part of the global state.
struct WasmMutateState {
    instances: Slab<Store<NvimHostStore>>,
}

/// The global instance of the Nvim WASM module state.
static WASM_STATE: OnceLock<WasmState> = OnceLock::new();

/// Initializes the global WASM state.
pub(crate) fn init_wasm_state() {
    WASM_STATE
        .set(WasmState::new())
        .map_err(|_| ())
        .expect("Failed to initialize wasm state");
}

/// Returns the global WASM state.
pub(crate) fn state() -> &'static WasmState {
    WASM_STATE.get().expect("Wasm state is not initialized")
}

const WASM_CLIENT_CALLBACK_INTERFACE: &str = "nvim:api/client-callback-impl";

impl WasmState {
    /// Returns the config for creating Wasmtime engine.
    fn wasmtime_config() -> wasmtime::Config {
        let mut config = wasmtime::Config::new();
        config.wasm_component_model(true);
        config
    }

    fn new() -> Self {
        let engine = Engine::new(&Self::wasmtime_config()).expect("Failed to create wasm engine");
        WasmState {
            engine,
            mutate_state: Mutex::new(WasmMutateState {
                instances: Slab::new(),
            }),
        }
    }

    pub(crate) fn load_wasm_file(&self, file_path: &str) -> Result<InstanceId> {
        // TODO: It will be helpful to cache the compiled component here.
        let engine = &self.engine;
        let component = Component::from_file(engine, file_path)
            .with_context(|| format!("Failed to load the WASM file {}", file_path))?;

        let mut linker = Linker::new(engine);
        Guest::add_to_linker(&mut linker, |state| state)
            .expect("Failed to add the host bindings to WASM linker");

        let mut store = Store::new(engine, NvimHostStore::new());
        let mut mutate_state = state().mutate_state.lock().expect(MUTEX_POISONED_ERR);
        let mutate_state = &mut *mutate_state;
        // This should rarely happen. No one loads 2^31 WASM files...
        if mutate_state.instances.len() >= i32::MAX as usize {
            bail!(
                "Cannot load new WASM file because the number of instances has reached the limit."
            );
        }
        let (_, instance) = Guest::instantiate(&mut store, &component, &linker)
            .with_context(|| format!("Failed to instantiate the WASM file {}", file_path))?;
        let instance_id = mutate_state.instances.insert(store) as InstanceId;
        mutate_state.instances[instance_id as usize]
            .data_mut()
            .set(InstanceData {
                instance,
                instance_id,
            });

        Ok(instance_id)
    }

    pub(crate) fn call_instance_func<NO>(
        &self,
        instance_id: InstanceId,
        func_name: &str,
        args: &[NO],
    ) -> Result<NvimObject>
    where
        NO: Borrow<NvimObject>,
    {
        if instance_id < 0 {
            bail!("Instance ID should be non-negative, got {instance_id}")
        }
        let mut mutate_state = self.mutate_state.lock().expect(MUTEX_POISONED_ERR);
        let store = mutate_state.get_instance_store_mut(instance_id)?;
        let instance = store.data().get().instance.clone();

        let func: TypedFunc<(Vec<nvim_api::Object>,), (nvim_api::Object,)> = instance
            .get_func(&mut *store, func_name)
            .with_context(|| format!("Cannot find function {func_name} in instance {instance_id}"))?
            .typed(&mut *store)
            .with_context(|| {
                format!("The function {func_name} is not a function of type list<object> -> object")
            })?;
        let context = store.data().conversion_context();
        let args = args
            .iter()
            .map(|obj| Ok(obj.borrow().clone().try_into_wasm_type(&context)?))
            .collect::<Result<Vec<_>>>()?;

        let (result,) = func.call(&mut *store, (args,)).with_context(|| {
            format!(
            "The function call to {func_name} trapped (an runtime exception is raised) or failed"
        )
        })?;
        func.post_return(&mut *store)
            .with_context(|| format!("The function call to {func_name} failed at post_return"))?;
        Ok(NvimObject::from_wasm_type(result, &context))
    }

    pub(crate) fn call_instance_callback<NO>(
        &self,
        instance_id: i32,
        wasm_ref: u32,
        args: &[NO],
    ) -> Result<NvimObject>
    where
        NO: Borrow<NvimObject>,
    {
        if instance_id < 0 {
            bail!("Instance ID should be non-negative, got {instance_id}")
        }
        let mut mutate_state = self.mutate_state.lock().expect(MUTEX_POISONED_ERR);
        let store = mutate_state.get_instance_store_mut(instance_id)?;
        let instance = store.data().get().instance.clone();

        let func = instance
            .exports(&mut *store)
            .instance(WASM_CLIENT_CALLBACK_INTERFACE)
            .with_context(|| {
                format!(
                    "Instance {instance_id} that owns the callback does not export \
                         `{WASM_CLIENT_CALLBACK_INTERFACE}` interface"
                )
            })?
            .func("call-callback")
            .with_context(|| {
                format!("Cannot find function `call-callback` in instance {instance_id}")
            })?;

        let context = store.data().conversion_context();
        let args = args
            .iter()
            .map(|obj| Ok(obj.borrow().clone().try_into_wasm_type(&context)?))
            .collect::<Result<Vec<_>>>()?;
        let func: TypedFunc<(u32, Vec<nvim_api::Object>), (nvim_api::Object,)> =
            func.typed(&mut *store).with_context(|| {
                format!(
                    "The function call-callback is not a function of type \
                        callback-ref, list<object> -> ()"
                )
            })?;

        let (result,) = func.call(&mut *store, (wasm_ref, args,)).with_context(||format!(
                "The function call to call-callback trapped (an runtime exception is raised) or failed"
        ))?;
        func.post_return(&mut *store)
            .with_context(|| format!("The function call to call-callback failed at post_return"))?;
        Ok(NvimObject::from_wasm_type(result, &context))
    }

    pub(crate) fn drop_instance_callback(&self, instance_id: i32, wasm_ref: u32) -> Result<()> {
        if instance_id < 0 {
            bail!("Instance ID should be non-negative, got {instance_id}")
        }
        let mut mutate_state = self.mutate_state.lock().expect(MUTEX_POISONED_ERR);
        let store = mutate_state.get_instance_store_mut(instance_id)?;
        let instance = store.data().get().instance.clone();

        let func = instance
            .exports(&mut *store)
            .instance(WASM_CLIENT_CALLBACK_INTERFACE)
            .with_context(|| {
                format!(
                    "Instance {instance_id} that owns the callback does not export \
                         `{WASM_CLIENT_CALLBACK_INTERFACE}` interface"
                )
            })?
            .func("drop-callback")
            .with_context(|| {
                format!("Cannot find function `drop-callback` in instance {instance_id}")
            })?;

        let func: TypedFunc<(u32,), ()> = func.typed(&mut *store).with_context(|| {
            format!("The function drop-callback is not a function of type callback-ref -> ()")
        })?;
        func.call(&mut *store, (wasm_ref,)).with_context(||format!(
                "The function call to drop-callback trapped (an runtime exception is raised) or failed"
        ))?;
        func.post_return(&mut *store)
            .with_context(|| format!("The function call to drop-callback failed at post_return"))?;
        Ok(())
    }
}

impl WasmMutateState {
    fn get_instance_store_mut(&mut self, instance_id: InstanceId) -> Result<&mut Store<NvimHostStore>> {
        Ok(self
            .instances
            .get_mut(instance_id as usize)
            .with_context(|| format!("Cannot find instance with ID = {instance_id}"))?)
    }
}

/// Represents a WASM host store.
///
/// Currently, we adopts a one-instance-per-store model.
struct NvimHostStore {
    data: Option<InstanceData>,
}

pub type InstanceId = i32;

struct InstanceData {
    instance: Instance,
    instance_id: InstanceId,
}

impl NvimHostStore {
    fn new() -> Self {
        Self { data: None }
    }

    fn set(&mut self, instance_data: InstanceData) {
        self.data = Some(instance_data);
    }

    fn get(&self) -> &InstanceData {
        self.data.as_ref().expect("Instance data is not set.")
    }

    fn conversion_context(&self) -> crate::types::Context {
        crate::types::Context {
            current_instance_id: 0,
        }
    }
}

// Implements the host bindings.
//
// See `wit/nvim.wit` for the definition of the host bindings.
include!(concat!(env!("OUT_DIR"), "/api_impl.rs"));

impl nvim_types::Host for NvimHostStore {}
impl nvim_keysets::Host for NvimHostStore {}

const MUTEX_POISONED_ERR: &str = "Mutex is poisoned";
