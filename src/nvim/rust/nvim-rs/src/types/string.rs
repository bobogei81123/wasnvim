use std::{ffi::{CString, CStr}, ops::Deref};

/// Wraps an owned Neovim `String` (see nvim/api/private/defs.h).
///
/// Neovim String is assumed to be UTF-8 (because it is also used in Lua) and contains no NULL bytes
/// (because it is a CString).
pub struct NvimString(CString);

impl NvimString {
    /// Constructs an `NvimString` from bytes.
    ///
    /// The bytes must be UTF-8 and contains no NULL bytes.
    pub fn new<T>(t: T) -> Self
    where
        T: Into<Vec<u8>>,
    {
        let cstring = CString::new(t)
            .expect("Unable to construct CString (possibly because it contains NULL bytes)");
        Self(cstring)
    }

    /// Creates a `NvimString` from an owned Neovim native string.
    ///
    /// # Safety
    /// The caller must own the Neovim string. That is, the string buffer should remain valid
    /// throughout the lifetime of this object.
    pub unsafe fn from_ffi(s: nvim_sys::String) -> Self {
        Self(unsafe { CString::from_raw(s.data) })
    }

    /// Converts a `NvimString` into a borrowed string.
    pub fn as_str(&self) -> &NvimStr {
        self
    }

    /// Converts a `NvimString` into a Rust String.
    pub fn into_string(self) -> String {
        self.0
            .into_string()
            .expect("Failed to convert NvimString into a Rust String")
    }

    /// Converts a `NvimString` into a Neovim native string.
    ///
    /// The caller is then responsible for freeing this object.
    /// Although this crate makes Neovim and Rust use the same libc allocator, Rust does not
    /// guarantee that `CString` can be freed safely from C side.
    pub fn into_ffi(self) -> nvim_sys::String {
        let size = self.len();
        nvim_sys::String {
            data: self.0.into_raw(),
            size,
        }
    }

    /// Returns the byte length of the string.
    pub fn len(&self) -> usize {
        self.0.as_bytes().len()
    }

    /// Returns true if the string is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Deref for NvimString {
    type Target = NvimStr;

    fn deref(&self) -> &Self::Target {
        NvimStr::from_c_str(&self.0)
    }
}

/// Wraps a borrowed Neovim `String` (see nvim/api/private/defs.h).
///
/// NvimString is to &NvimStr as String is to &str.
#[repr(transparent)]
pub struct NvimStr(CStr);

impl NvimStr {
    /// Creates a `NvimStr` from a borrowed Neovim native string.
    ///
    /// # Safety
    /// The caller must ensure that the string buffer remain valid throughout the lifetime of this
    /// object.
    pub unsafe fn from_ffi(s: &nvim_sys::String) -> &Self {
        Self::from_c_str(unsafe { CStr::from_ptr(s.data as *const _) })
    }

    /// Creates a `NvimStr` from a borrowed `CStr`.
    pub fn from_c_str(s: &CStr) -> &Self {
        unsafe { &*(s as *const _ as *const Self) }
    }

    /// Converts this borrowed NvimStr into an borrowed FFI String.
    ///
    /// The returned string is a immutable borrow. It should not be dropped or modified.
    pub fn as_borrowed_ffi(&self) -> nvim_sys::String {
        let size = self.0.to_bytes().len();

        nvim_sys::String {
            data: self.0.as_ptr() as *mut _,
            size,
        }
    }
}

