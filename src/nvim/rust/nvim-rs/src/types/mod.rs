pub use array::{slice_from_ffi_ref, NvimArray};
pub use dictionary::{NvimDictionary, NvimDictionaryRef};
pub use object::{
    IntoObject, NvimApiType, NvimObject, NvimObjectEnum, NvimObjectEnumRef, ObjectConversionError,
    TryFromObject, TryFromObjectRef,
};
pub use result::{NvimError, NvimErrorKind, NvimResult};
pub use string::{bytes_from_ffi_ref, str_from_ffi_ref, NvimString};

mod array;
mod dictionary;
mod ffi_wrapper;
mod object;
mod result;
mod string;

/// Represents a handle to an Neovim buffer.
#[repr(transparent)]
pub struct NvimBuffer(i64);

/// Represents a handle to an Neovim Window.
#[repr(transparent)]
pub struct NvimWindow(i64);

/// Represents a handle to an Neovim Tabpage.
#[repr(transparent)]
pub struct NvimTabpage(i64);

impl NvimBuffer {
    pub fn from_handle(handle: i64) -> Self {
        Self(handle)
    }
    pub fn from_handle_ref(handle: &i64) -> &Self {
        unsafe { &*(handle as *const i64 as *const Self) }
    }
    pub fn handle(&self) -> i64 {
        self.0
    }
}

impl NvimWindow {
    pub fn from_handle(handle: i64) -> Self {
        Self(handle)
    }
    pub fn from_handle_ref(handle: &i64) -> &Self {
        unsafe { &*(handle as *const i64 as *const Self) }
    }
    pub fn handle(&self) -> i64 {
        self.0
    }
}

impl NvimTabpage {
    pub fn from_handle(handle: i64) -> Self {
        Self(handle)
    }
    pub fn from_handle_ref(handle: &i64) -> &Self {
        unsafe { &*(handle as *const i64 as *const Self) }
    }
    pub fn handle(&self) -> i64 {
        self.0
    }
}
