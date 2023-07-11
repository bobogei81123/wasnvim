use std::{
    mem,
    ops::Deref,
    ptr::{self, slice_from_raw_parts},
};

use super::{
    ffi_wrapper::{NvimFfiClone, NvimFfiType, NvimFfiWrapper},
    NvimObject,
};

/// Wraps a Neovim's Array. (see nvim/api/private/defs.h).
pub type NvimArray = NvimFfiWrapper<nvim_sys::Array>;

impl NvimArray {
    /// Create an new empty Neovim array.
    pub fn new() -> Self {
        unsafe {
            Self::from_ffi(nvim_sys::Array {
                items: ptr::null_mut(),
                size: 0,
                capacity: 0,
            })
        }
    }

    /// Creates a new array from a vector of [`NvimObject`](NvimObject)s.
    pub fn from_vec(mut vec: Vec<NvimObject>) -> Self {
        let size = vec.len();
        let capacity = vec.capacity();
        let items = vec.as_mut_ptr() as *mut nvim_sys::Object;
        mem::forget(vec);

        unsafe {
            Self::from_ffi(nvim_sys::Array {
                items,
                size,
                capacity,
            })
        }
    }

    /// Converts this dictionary into a vector of [`NvimObject`](NvimObject)s.
    pub fn into_vec(self) -> Vec<NvimObject> {
        let array = self.into_ffi();
        let size = array.size;
        let capacity = array.capacity;
        let items = array.items as *mut NvimObject;

        unsafe { Vec::from_raw_parts(items, size, capacity) }
    }

    /// Returns the length of the array.
    pub fn len(&self) -> usize {
        self.as_ffi_ref().size
    }

    /// Returns true if the array is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl NvimFfiType for nvim_sys::Array {
    fn ffi_drop(self) {
        unsafe {
            nvim_sys::api_free_array(self);
        }
    }
}

unsafe impl NvimFfiClone for nvim_sys::Array {
    /// Returns a deep copy of this array.
    fn ffi_clone(self) -> Self {
        unsafe { nvim_sys::copy_array(self, ptr::null_mut()) }
    }
}

impl Deref for NvimArray {
    type Target = [NvimObject];

    fn deref(&self) -> &Self::Target {
        unsafe {
            std::slice::from_raw_parts(self.as_ffi_ref().items as *const NvimObject, self.len())
        }
    }
}

impl FromIterator<NvimObject> for NvimArray {
    fn from_iter<T: IntoIterator<Item = NvimObject>>(iter: T) -> Self {
        Self::from_vec(iter.into_iter().collect())
    }
}

/// Creates a reference to a slice of `NvimObject`s from a reference to an FFI array.
pub fn slice_from_ffi_ref(arr: &nvim_sys::Array) -> &[NvimObject] {
    unsafe { &*slice_from_raw_parts(arr.items as *const NvimObject, arr.size) }
}
