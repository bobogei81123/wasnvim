use std::{
    borrow::Borrow,
    ffi::{c_char, CStr, CString},
    fmt::Display,
    ops::{Deref, Index},
};

use crate::{Error, Result};

/// Represents the result of calling neovim functions.
///
/// It wraps nvim's `Error` (see nvim/api/private/defs.h) and resembles `Result<(), NvimError>` in
/// rust. Nvim's `Error` has a "none" state which makes it more like `Result` than an `Error`, in
/// Rust's point of view.
pub struct NvimResult(nvim_sys::Error);

impl NvimResult {
    /// Creates a new Ok `NvimResult`.
    pub fn new_ok() -> Self {
        Self(nvim_sys::Error {
            type_: nvim_sys::ErrorType_kErrorTypeNone,
            msg: std::ptr::null_mut(),
        })
    }

    pub fn as_ffi_error_mut(&mut self) -> &mut nvim_sys::Error {
        &mut self.0
    }

    pub fn into_result(self) -> std::result::Result<(), NvimError> {
        let NvimResult(nvim_sys::Error { type_, msg }) = self;
        match type_ {
            nvim_sys::ErrorType_kErrorTypeNone => Ok(()),
            nvim_sys::ErrorType_kErrorTypeException => Err(NvimError {
                kind: NvimErrorKind::Exception,
                msg: unsafe { cstring_from_raw_check_null(msg) },
            }),
            nvim_sys::ErrorType_kErrorTypeValidation => Err(NvimError {
                kind: NvimErrorKind::Validation,
                msg: unsafe { cstring_from_raw_check_null(msg) },
            }),
            _ => {
                panic!(
                    "Encounter unknown error value ({:?}) when converting nvim error",
                    type_
                );
            }
        }
    }
}

impl Default for NvimResult {
    fn default() -> Self {
        NvimResult::new_ok()
    }
}

impl From<NvimResult> for std::result::Result<(), NvimError> {
    fn from(value: NvimResult) -> Self {
        value.into_result()
    }
}

/// Wraps nvim's `ErrorType` (see nvim/api/private/defs.h).
#[derive(Debug)]
pub enum NvimErrorKind {
    Exception,
    Validation,
}

#[derive(Debug)]
pub struct NvimError {
    kind: NvimErrorKind,
    msg: CString,
}

impl Display for NvimError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match self.kind {
            NvimErrorKind::Exception => "Exception: ",
            NvimErrorKind::Validation => "Validation: ",
        };
        write!(f, "{kind}: {}", self.msg.to_string_lossy())
    }
}

impl std::error::Error for NvimError {}

/// Wraps owned Neovim's `String` (see nvim/api/private/defs.h).
pub struct NvimString(CString);

impl NvimString {
    pub fn new<T>(t: T) -> Result<Self>
    where
        T: Into<Vec<u8>>,
    {
        let cstring =
            CString::new(t).map_err(|_| Error::Other(crate::OtherError::StringConversionError))?;
        Ok(Self(cstring))
    }

    pub unsafe fn from_ffi(s: nvim_sys::String) -> Self {
        Self(unsafe { CString::from_raw(s.data) })
    }

    /// Borrows this `NvimString` and returns an FFI String.
    ///
    /// # Safety
    ///
    /// The returned string is a immutable borrow. It should not be dropped or modified.
    pub unsafe fn as_ffi_borrowed_string(&self) -> nvim_sys::String {
        nvim_sys::String {
            data: self.0.as_ptr() as *mut _,
            size: self.len(),
        }
    }

    pub fn as_str(&self) -> &NvimStr {
        &*self
    }

    pub fn try_into_string(self) -> Result<String> {
        self.0
            .into_string()
            .map_err(|_| Error::Other(crate::OtherError::StringConversionError))
    }

    pub fn into_string(self) -> String {
        self.0
            .into_string()
            .expect("Failed to convert NvimString into a Rust String")
    }

    pub fn into_ffi(self) -> nvim_sys::String {
        let size = self.len();
        nvim_sys::String {
            data: self.0.into_raw(),
            size,
        }
    }

    pub fn len(&self) -> usize {
        self.0.as_bytes().len()
    }

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
    pub unsafe fn from_ffi(s: &nvim_sys::String) -> &Self {
        Self::from_c_str(unsafe { CStr::from_ptr(s.data as *const _) })
    }

    pub fn from_c_str(s: &CStr) -> &Self {
        unsafe { &*(s as *const _ as *const Self) }
    }

    /// Convert this borrowed NvimStr into an borrowed FFI String.
    ///
    /// # Safety
    ///
    /// The returned string is a immutable borrow. It should not be dropped or modified.
    pub unsafe fn to_ffi(&self) -> nvim_sys::String {
        let size = self.0.to_bytes().len();

        nvim_sys::String {
            data: self.0.as_ptr() as *mut _,
            size,
        }
    }
}

/// An owned Neovim `Object` (see nvim/api/private/defs.h).
#[non_exhaustive]
pub enum NvimObject {
    Nil,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(NvimString),
    Array(NvimArray),
}

impl NvimObject {
    pub fn from_ffi(obj: nvim_sys::Object) -> Self {
        unsafe {
            match obj.type_ {
                nvim_sys::ObjectType_kObjectTypeNil => Self::Nil,
                nvim_sys::ObjectType_kObjectTypeBoolean => {
                    NvimObject::Boolean(*obj.data.boolean.as_ref())
                }
                nvim_sys::ObjectType_kObjectTypeInteger => {
                    NvimObject::Integer(*obj.data.integer.as_ref())
                }
                nvim_sys::ObjectType_kObjectTypeFloat => {
                    NvimObject::Float(*obj.data.floating.as_ref())
                }
                nvim_sys::ObjectType_kObjectTypeString => {
                    NvimObject::String(NvimString::from_ffi(*obj.data.string.as_ref()))
                }
                _ => {
                    todo!()
                }
            }
        }
    }

    fn into_ffi(self) -> nvim_sys::Object {
        let mut obj = nvim_sys::Object::default();
        unsafe {
            use NvimObject::*;
            match self {
                Nil => {
                    obj.type_ = nvim_sys::ObjectType_kObjectTypeNil;
                }
                Boolean(b) => {
                    obj.type_ = nvim_sys::ObjectType_kObjectTypeBoolean;
                    *obj.data.boolean.as_mut() = b;
                }
                Integer(i) => {
                    obj.type_ = nvim_sys::ObjectType_kObjectTypeInteger;
                    *obj.data.integer.as_mut() = i;
                }
                Float(f) => {
                    obj.type_ = nvim_sys::ObjectType_kObjectTypeFloat;
                    *obj.data.floating.as_mut() = f;
                }
                String(s) => {
                    obj.type_ = nvim_sys::ObjectType_kObjectTypeString;
                    *obj.data.string.as_mut() = s.into_ffi();
                }
                Array(a) => todo!(),
            }
        }
        obj
    }
}

#[derive(Copy, Clone)]
#[non_exhaustive]
pub enum NvimObjectRef<'a> {
    Nil,
    Boolean(&'a bool),
    Integer(&'a i64),
    Float(&'a f64),
    String(&'a NvimStr),
    Array(&'a NvimArraySlice),
}

impl<'a> NvimObjectRef<'a> {
    fn from_ffi(obj: &'a nvim_sys::Object) -> Self {
        unsafe {
            match obj.type_ {
                nvim_sys::ObjectType_kObjectTypeNil => NvimObjectRef::Nil,
                nvim_sys::ObjectType_kObjectTypeBoolean => {
                    NvimObjectRef::Boolean(obj.data.boolean.as_ref())
                }
                nvim_sys::ObjectType_kObjectTypeInteger => {
                    NvimObjectRef::Integer(obj.data.integer.as_ref())
                }
                nvim_sys::ObjectType_kObjectTypeFloat => {
                    NvimObjectRef::Float(obj.data.floating.as_ref())
                }
                nvim_sys::ObjectType_kObjectTypeString => {
                    NvimObjectRef::String(NvimStr::from_ffi(obj.data.string.as_ref()))
                }
                nvim_sys::ObjectType_kObjectTypeArray => {
                    NvimObjectRef::Array(NvimArraySlice::from_ffi(obj.data.array.as_ref()))
                }
                _ => todo!(),
            }
        }
    }

    unsafe fn to_ffi(self) -> nvim_sys::Object {
        let mut obj = nvim_sys::Object::default();
        unsafe {
            use NvimObjectRef::*;
            match self {
                Nil => {
                    obj.type_ = nvim_sys::ObjectType_kObjectTypeNil;
                }
                Boolean(&b) => {
                    obj.type_ = nvim_sys::ObjectType_kObjectTypeBoolean;
                    *obj.data.boolean.as_mut() = b;
                }
                Integer(&i) => {
                    obj.type_ = nvim_sys::ObjectType_kObjectTypeInteger;
                    *obj.data.integer.as_mut() = i;
                }
                Float(&f) => {
                    obj.type_ = nvim_sys::ObjectType_kObjectTypeFloat;
                    *obj.data.floating.as_mut() = f;
                }
                String(s) => {
                    obj.type_ = nvim_sys::ObjectType_kObjectTypeString;
                    *obj.data.string.as_mut() = s.to_ffi();
                }
                _ => todo!(),
            }
        }
        obj
    }
}

pub struct NvimArray(Vec<nvim_sys::Object>);

impl NvimArray {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub unsafe fn from_ffi(obj: nvim_sys::Array) -> Self {
        let vec = unsafe { Vec::from_raw_parts(obj.items, obj.size, obj.capacity) };
        NvimArray(vec)
    }

    pub fn into_ffi(self) -> nvim_sys::Array {
        let mut vec = self.0;
        let size = vec.len();
        let capacity = vec.capacity();
        let items = vec.as_mut_ptr();
        std::mem::forget(vec);

        nvim_sys::Array {
            items,
            size,
            capacity,
        }
    }

    pub fn push(&mut self, obj: NvimObject) {
        self.0.push(obj.into_ffi());
    }

    pub fn get(&self, index: usize) -> NvimObjectRef {
        NvimObjectRef::from_ffi(&self.0[index])
    }
}

impl Deref for NvimArray {
    type Target = NvimArraySlice;

    fn deref(&self) -> &Self::Target {
        NvimArraySlice::from_slice(&*self.0)
    }
}

#[repr(transparent)]
pub struct NvimArraySlice([nvim_sys::Object]);

impl NvimArraySlice {
    pub unsafe fn from_ffi(obj: &nvim_sys::Array) -> &Self {
        Self::from_slice(unsafe { std::slice::from_raw_parts(obj.items, obj.size) })
    }

    pub fn from_slice(s: &[nvim_sys::Object]) -> &Self {
        unsafe { &*(s as *const _ as *const Self) }
    }
}

unsafe fn cstring_from_raw_check_null(msg: *mut c_char) -> CString {
    if msg.is_null() {
        panic!("Try to covert a null pointer to a CString");
    }
    unsafe { CString::from_raw(msg) }
}
