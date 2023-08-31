use std::{ffi::c_char, rc::Rc};

use nvim_rs::{slice_from_ffi_ref, NvimObject};
use nvim_sys::Error;

use crate::runtime::{self, InstanceId};

type WasmRef = nvim_sys::WasmRef;

pub unsafe extern "C" fn wasmref_drop(wasmref: WasmRef) {
    runtime::state().drop_instance_callback(wasmref.instance_id, wasmref.ref_);
}

pub unsafe extern "C" fn wasmref_is_none(wasmref: WasmRef) -> bool {
    wasmref.instance_id < 0 && wasmref.ref_ != 0
}

pub unsafe extern "C" fn wasmref_call(
    wasmref: WasmRef,
    name: *const c_char,
    args: nvim_sys::Array,
) {
    if !wasmref_is_none(wasmref) {
        runtime::state().call_instance_callback(
            wasmref.instance_id,
            wasmref.ref_,
            slice_from_ffi_ref(&args),
        );
    }
}

type RcWasmRef = *const WasmRef;

pub unsafe extern "C" fn rc_wasmref_call(
    rc_wasmref: RcWasmRef,
    name: *const c_char,
    args: nvim_sys::Array,
) {
    wasmref_call(*rc_wasmref, name, args);
}

pub unsafe extern "C" fn rc_wasmref_from_object(
    obj: nvim_sys::Object,
    what: *const c_char,
    err: *mut Error,
) -> RcWasmRef {
    todo!();
}

pub unsafe extern "C" fn rc_wasmref_into_object(rc_wasmref: RcWasmRef) -> nvim_sys::Object {
    let rc = Rc::from_raw(rc_wasmref);
    NvimObject::from_enum(nvim_rs::NvimObjectEnum::WasmRef(*rc_wasmref)).into_ffi()
}

pub unsafe extern "C" fn rc_wasmref_clone(rc_wasmref: RcWasmRef) -> RcWasmRef {
    let rc = Rc::from_raw(rc_wasmref);
    Rc::into_raw(Rc::clone(&rc))
}

pub unsafe extern "C" fn rc_wasmref_drop(rc_wasmref: RcWasmRef) {
    let rc = Rc::from_raw(rc_wasmref);
    if Rc::strong_count(&rc) == 1 {
        wasmref_drop(*rc_wasmref);
    }
}
