use core::slice;
use std::{mem::ManuallyDrop, ops::Deref, ptr};

/// Wraps an owned Neovim `String` (see nvim/api/private/defs.h).
///
/// Neovim API String is assumed to be UTF-8 (because it is also used in Lua) and can contains
/// NULL bytes.
#[repr(transparent)]
pub struct NvimString(nvim_sys::String);

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
        Self(nvim_sys::String { data, size })
    }

    /// Constructs an `NvimString` from a Rust string.
    pub fn from_string(s: String) -> Self {
        Self::new(s)
    }

    /// Converts a `NvimString` into a Rust String.
    pub fn into_string(self) -> String {
        let nvim_sys::String { data, size } = self.0;
        // Neovim always allocate the C string with an additional null byte at the end.
        let slice = unsafe { Box::from_raw(slice::from_raw_parts_mut(data as *mut u8, size + 1)) };
        let vec = slice.into_vec();

        String::from_utf8(vec).expect(
            "The Neovim String is not UTF-8 and cannot be convert to Rust String.\
            Rust strings must be UTF-8",
        )
    }

    /// Creates a `NvimString` from an owned Neovim native string.
    ///
    /// # Safety
    /// The caller must own the Neovim string. That is, the string buffer should remain valid
    /// throughout the lifetime of this object.
    pub unsafe fn from_ffi(s: nvim_sys::String) -> Self {
        Self(s)
    }

    /// Converts a `NvimString` into a Neovim native string.
    ///
    /// The caller is then responsible for freeing this string.
    pub fn into_ffi(self) -> nvim_sys::String {
        let me = ManuallyDrop::new(self);
        me.as_borrowed_ffi()
    }

    /// Creates a borrowed NvimString from a borrowed FFI string.
    ///
    /// # Safety
    /// The caller is responsible to make sure that the string remain valid throughout the
    /// lifetime of this object.
    pub unsafe fn from_borrowed_ffi(s: &nvim_sys::String) -> &NvimString {
        &*(s as *const _ as *const NvimString)
    }

    /// Converts this string into an borrowed FFI string.
    ///
    /// The returned FFI string is only a borrow, so the caller is responsible to make sure
    /// that it remain intact throughout the time it is borrowed.
    pub fn as_borrowed_ffi(&self) -> nvim_sys::String {
        unsafe { ptr::read(&self.0) }
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
        self.0.size
    }

    /// Returns true if the string is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Deref for NvimString {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.0.data as *mut u8, self.0.size) }
    }
}

pub fn bytes_from_borrowed_ffi(s: &nvim_sys::String) -> &[u8] {
    unsafe { NvimString::from_borrowed_ffi(s) }
}

pub fn str_from_borrowed_ffi(s: &nvim_sys::String) -> &str {
    unsafe { NvimString::from_borrowed_ffi(s).as_str() }
}
