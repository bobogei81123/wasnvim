#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_snake_case)]
#![allow(clippy::missing_safety_doc)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub unsafe fn xfree_clear<T>(ptr: &mut *mut T) {
    unsafe { xfree(*ptr as _); }
    *ptr = std::ptr::null_mut();
}
