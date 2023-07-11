use core::slice;
use std::{ops::Deref, ptr};

use super::ffi_wrapper::{NvimFfiClone, NvimFfiType, NvimFfiWrapper};

/// Wraps an owned Neovim `String` (see nvim/api/private/defs.h).
///
/// Neovim API String is assumed to be UTF-8 (because it is also used in Lua) and can contains
/// NULL bytes.
pub type NvimString = NvimFfiWrapper<nvim_sys::String>;

impl NvimString {
    /// Constructs an `NvimString` from bytes.
    pub fn new<T>(t: T) -> Self
    where
        T: Into<Vec<u8>>,
    {
        let mut data: Vec<u8> = t.into();
        let size = data.len();
        // Push the null byte at the end.
        data.push(b'\0');
        let data = Box::into_raw(data.into_boxed_slice()) as *mut i8;
        // Neovim C and rust code use the same allocator, so it is safe to have C free the pointer.
        unsafe { Self::from_ffi(nvim_sys::String { data, size }) }
    }

    /// Constructs an `NvimString` from a Rust string.
    pub fn from_string(s: String) -> Self {
        Self::new(s)
    }

    /// Converts a `NvimString` into a Rust String.
    pub fn into_string(self) -> String {
        let nvim_sys::String { data, size } = self.into_ffi();
        // Neovim always allocate the C string with an additional null byte at the end.
        let slice = unsafe { Box::from_raw(slice::from_raw_parts_mut(data as *mut u8, size + 1)) };
        let mut vec = slice.into_vec();
        // Pop the null byte at the end.
        vec.pop();

        String::from_utf8(vec).expect(
            "The Neovim String is not UTF-8 and cannot be convert to Rust String.\
            Rust strings must be UTF-8",
        )
    }

    /// Converts a `NvimString` into bytes.
    pub fn as_bytes(&self) -> &[u8] {
        self
    }

    /// Converts a `NvimString` into a Rust str.
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(self).expect(
            "The Neovim String is not UTF-8 and cannot be convert to Rust String.\
            Rust strings must be UTF-8",
        )
    }

    /// Returns the byte length of the string.
    pub fn len(&self) -> usize {
        self.as_ffi_ref().size
    }

    /// Returns true if the string is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl NvimFfiType for nvim_sys::String {
    fn ffi_drop(self) {
        unsafe { nvim_sys::api_free_string(self) }
    }
}

unsafe impl NvimFfiClone for nvim_sys::String {
    /// Creates a deep copy of the string.
    fn ffi_clone(self) -> Self {
        unsafe { nvim_sys::copy_string(self, ptr::null_mut()) }
    }
}

impl Deref for NvimString {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.as_ffi_ref().data as *const u8, self.len()) }
    }
}

/// Returns the bytes slice of the string.
pub fn bytes_from_ffi_ref(s: &nvim_sys::String) -> &[u8] {
    unsafe { NvimString::from_ffi_ref(s) }
}

/// Returns a `str` of the string.
pub fn str_from_ffi_ref(s: &nvim_sys::String) -> &str {
    unsafe { NvimString::from_ffi_ref(s).as_str() }
}
