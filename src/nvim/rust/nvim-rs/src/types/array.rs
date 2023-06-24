use std::{
    mem::{self, ManuallyDrop},
    ops::{Deref, DerefMut},
    ptr::{self, slice_from_raw_parts},
};

use super::NvimObject;

/// Wraps a Neovim's Array. (see nvim/api/private/defs.h).
#[derive(Default)]
#[repr(transparent)]
pub struct NvimArray(nvim_sys::Array);

impl NvimArray {
    /// Create an new empty Neovim array.
    pub fn new() -> Self {
        Self(nvim_sys::Array {
            items: ptr::null_mut(),
            size: 0,
            capacity: 0,
        })
    }

    /// Creates an `NvimArray` from an owned FFI array.
    ///
    /// # Safety
    /// The caller must owned the dictionary and ensure that it remains valid throughout the
    /// lifetime of this object.
    pub unsafe fn from_ffi(obj: nvim_sys::Array) -> Self {
        Self(obj)
    }

    /// Converts this array into an owned FFI array.
    ///
    /// The caller is then responsible for freeing the array.
    pub fn into_ffi(self) -> nvim_sys::Array {
        let me = ManuallyDrop::new(self);
        me.as_borrowed_ffi()
    }

    /// Converts this array into an borrowed FFI array.
    ///
    /// The returned FFI array is only a borrow, so the caller is responsible to make sure
    /// that it remain intact throughout the time it is borrowed.
    pub fn as_borrowed_ffi(&self) -> nvim_sys::Array {
        unsafe { ptr::read(&self.0) }
    }

    /// Creates a new array from a vector of [`NvimObject`](NvimObject)s.
    pub fn from_vec(mut vec: Vec<NvimObject>) -> Self {
        let size = vec.len();
        let capacity = vec.capacity();
        let items = vec.as_mut_ptr() as *mut nvim_sys::Object;
        mem::forget(vec);

        Self(nvim_sys::Array {
            items,
            size,
            capacity,
        })
    }

    /// Converts this dictionary into a vector of [`NvimObject`](NvimObject)s.
    pub fn into_vec(self) -> Vec<NvimObject> {
        let array = self.into_ffi();
        let size = array.size;
        let capacity = array.capacity;
        let items = array.items as *mut NvimObject;

        unsafe { Vec::from_raw_parts(items, size, capacity) }
    }
}

impl Drop for NvimArray {
    fn drop(&mut self) {
        unsafe {
            nvim_sys::api_free_array(self.as_borrowed_ffi());
        }
    }
}

impl Clone for NvimArray {
    /// Returns a deep copy of this array.
    fn clone(&self) -> Self {
        unsafe {
            NvimArray::from_ffi(nvim_sys::copy_array(
                self.as_borrowed_ffi(),
                ptr::null_mut(),
            ))
        }
    }
}

impl Deref for NvimArray {
    type Target = Vec<NvimObject>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(&self.0 as *const _ as *const Vec<NvimObject>) }
    }
}

impl DerefMut for NvimArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(&mut self.0 as *mut _ as *mut Vec<NvimObject>) }
    }
}

impl FromIterator<NvimObject> for NvimArray {
    fn from_iter<T: IntoIterator<Item = NvimObject>>(iter: T) -> Self {
        Self::from_vec(iter.into_iter().collect())
    }
}

/// Creates a reference to a slice of `NvimObject`s from a reference to an FFI array.
pub fn slice_from_borrowed_ffi(arr: &nvim_sys::Array) -> &[NvimObject] {
    unsafe { &*slice_from_raw_parts(arr.items as *const NvimObject, arr.size) }
}
