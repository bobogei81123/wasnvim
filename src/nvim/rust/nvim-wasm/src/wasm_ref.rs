use std::{
    ffi::{c_char, CStr},
    rc::Rc,
};

use anyhow::Result;
use nvim_rs::{
    slice_from_ffi_ref, IntoObject, NvimError, NvimObject, NvimObjectEnum, NvimResult, NvimString,
};
use nvim_sys::{Error, WasmRef};

use crate::runtime::{self, InstanceId};

#[derive(PartialEq, Eq)]
pub(crate) struct WasmRefInner {
    pub(crate) instance_id: InstanceId,
    pub(crate) ref_: u32,
}

pub(crate) unsafe fn from_sys(wasmref: WasmRef) -> Option<&'static WasmRefInner> {
    (wasmref as *const WasmRefInner).as_ref()
}

impl WasmRefInner {
    fn call(self, name: &str, args: &[NvimObject]) -> Result<NvimObject> {
        runtime::state().call_instance_callback(self.instance_id, self.ref_, args)
    }
}

impl Drop for WasmRefInner {
    fn drop(&mut self) {
        runtime::state().drop_instance_callback(self.instance_id, self.ref_);
    }
}

pub unsafe extern "C" fn wasmref_call(
    wasmref: WasmRef,
    name: *const c_char,
    args: nvim_sys::Array,
    err: *mut Error,
) -> nvim_sys::Object {
    let Some(wasmref) = from_sys(wasmref) else {
        return;
    };
    let result = if name.is_null() {
        runtime::state().call_instance_callback(
            wasmref.instance_id,
            wasmref.ref_,
            slice_from_ffi_ref(&args),
        )
    } else {
        let name = CStr::from_ptr(name)
            .to_str()
            .expect("Function name is not a valid UTF-8 string");
        let name = NvimString::new(name).into_object();
        let args = std::iter::once(&name)
            .chain(slice_from_ffi_ref(&args).iter())
            .collect::<Vec<_>>();
        runtime::state().call_instance_callback(wasmref.instance_id, wasmref.ref_, &args)
    };

    match result {
        Err(e) => {
            let err = unsafe { err.as_mut() }.map(NvimResult::from_ffi_mut);
            if let Some(err) = err {
                *err = NvimResult::from_result(Err(NvimError {
                    msg: format!("{err:#}"),
                    kind: nvim_rs::NvimErrorKind::Exception,
                }))
            } else {
                nvim_rs::emsg(&format!(
                    "Failed during the execution of the callback: {err:#}"
                ));
            }
        }
        Ok(ret) => ret.into_ffi(),
    }
}

pub extern "C" fn wasmref_new(instance_id: i32, ref_: u32) -> WasmRef {
    if instance_id < 0 || ref_ == 0 {
        return std::ptr::null();
    }

    Rc::into_raw(Rc::new(WasmRefInner { instance_id, ref_ })) as WasmRef
}

pub unsafe extern "C" fn wasmref_eq(r1: WasmRef, r2: WasmRef) -> bool {
    let r1 = unsafe { from_sys(r1) };
    let r2 = unsafe { from_sys(r2) };
    match (r1, r2) {
        (None, None) => true,
        (None, _) | (_, None) => false,
        (Some(r1), Some(r2)) => r1 == r2,
    }
}

pub unsafe extern "C" fn wasmref_into_object(wasmref: WasmRef) -> nvim_sys::Object {
    NvimObject::from_enum(NvimObjectEnum::WasmRef(wasmref)).into_ffi()
}

pub unsafe extern "C" fn wasmref_clone(wasmref: WasmRef) -> WasmRef {
    let rc = Rc::from_raw(wasmref);
    let cloned = Rc::into_raw(Rc::clone(&rc));
    std::mem::forget(rc);
    cloned
}

pub unsafe extern "C" fn wasmref_drop(wasmref: WasmRef) {
    if wasmref == std::ptr::null() {
        return;
    }
    let rc = Rc::from_raw(wasmref);
}
