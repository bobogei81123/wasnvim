use std::{fmt::Display, mem::ManuallyDrop, ptr};

use super::{NvimArray, NvimDictionary, NvimString};

/// Wraps an owned Neovim `Object` (see nvim/api/private/defs.h).
#[repr(transparent)]
pub struct NvimObject(nvim_sys::Object);

/// Wraps a Neovim's Object. (see nvim/api/private/defs.h).
#[non_exhaustive]
#[derive(derive_more::TryInto)]
#[try_into(owned, ref, ref_mut)]
pub enum NvimObjectEnum {
    Nil,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(NvimString),
    Array(NvimArray),
    Dictionary(NvimDictionary),
}

/// Represents the type of a Neovim object.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NvimApiType {
    Nil,
    Boolean,
    Integer,
    Float,
    String,
    Array,
    Dictionary,
}

impl NvimObject {
    /// Returns a nil object.
    pub fn nil() -> Self {
        Self::from_enum(NvimObjectEnum::Nil)
    }

    /// Creates an `NvimObject` from an owned FFI object.
    ///
    /// # Safety
    /// The caller must owned the dictionary and ensure that it remains valid throughout the
    /// lifetime of this object.
    pub unsafe fn from_ffi(obj: nvim_sys::Object) -> Self {
        if obj.type_ > nvim_sys::ObjectType_kObjectTypeDictionary {
            // Tabpage is the last object type in the enum
            if obj.type_ > nvim_sys::ObjectType_kObjectTypeTabpage {
                panic!("Unknown object type ({}).", obj.type_);
            } else {
                panic!(
                    "Unsupported object type ({}). This is one of LuaRef, Buffer, Window, Tabpage.",
                    obj.type_
                );
            }
        }
        Self(obj)
    }

    /// Creates a reference to the `NvimObject` from an borrowed FFI object.
    ///
    /// # Safety
    /// The caller must ensure that the Neovim object remains valid throughout the lifetime of this
    /// object.
    pub unsafe fn from_ffi_ref(obj: &nvim_sys::Object) -> &Self {
        unsafe { &*(obj as *const _ as *const Self) }
    }

    /// Creates a mutable reference to the `NvimObject` from an uniquely borrowed FFI object.
    ///
    /// # Safety
    /// The caller must ensure that the Neovim object remains valid throughout the lifetime of this
    /// object.
    pub unsafe fn from_ffi_mut(obj: &mut nvim_sys::Object) -> &mut Self {
        unsafe { &mut *(obj as *mut _ as *mut Self) }
    }

    /// Converts this object into an owned FFI object.
    ///
    /// The caller is then responsible for freeing the object.
    pub fn into_ffi(self) -> nvim_sys::Object {
        let me = ManuallyDrop::new(self);
        me.as_borrowed_ffi()
    }

    /// Converts this object into an borrowed FFI object.
    ///
    /// The returned FFI object is only a borrow, so the caller is responsible to make sure
    /// that it remain intact throughout the time it is borrowed.
    pub fn as_borrowed_ffi(&self) -> nvim_sys::Object {
        unsafe { ptr::read(&self.0) }
    }

    /// Constructs an `NvimObject from an `NvimObjectEnum`.
    pub fn from_enum(obj: NvimObjectEnum) -> Self {
        let mut result = nvim_sys::Object::default();
        match obj {
            NvimObjectEnum::Nil => {
                result.type_ = nvim_sys::ObjectType_kObjectTypeNil;
            }
            NvimObjectEnum::Boolean(value) => {
                result.type_ = nvim_sys::ObjectType_kObjectTypeBoolean;
                unsafe {
                    *result.data.boolean.as_mut() = value;
                }
            }
            NvimObjectEnum::Integer(value) => {
                result.type_ = nvim_sys::ObjectType_kObjectTypeInteger;
                unsafe {
                    *result.data.integer.as_mut() = value;
                }
            }
            NvimObjectEnum::Float(value) => {
                result.type_ = nvim_sys::ObjectType_kObjectTypeFloat;
                unsafe {
                    *result.data.floating.as_mut() = value;
                }
            }
            NvimObjectEnum::String(value) => {
                result.type_ = nvim_sys::ObjectType_kObjectTypeString;
                unsafe {
                    *result.data.string.as_mut() = value.into_ffi();
                }
            }
            NvimObjectEnum::Array(value) => {
                result.type_ = nvim_sys::ObjectType_kObjectTypeArray;
                unsafe {
                    *result.data.array.as_mut() = value.into_ffi();
                }
            }
            NvimObjectEnum::Dictionary(value) => {
                result.type_ = nvim_sys::ObjectType_kObjectTypeDictionary;
                unsafe {
                    *result.data.dictionary.as_mut() = value.into_ffi();
                }
            }
        }
        Self(result)
    }

    /// Converts this `NvimObject into an `NvimObjectEnum`.
    pub fn into_enum(self) -> NvimObjectEnum {
        let me = ManuallyDrop::new(self);
        unsafe {
            match me.type_() {
                NvimApiType::Nil => NvimObjectEnum::Nil,
                NvimApiType::Boolean => NvimObjectEnum::Boolean(*me.0.data.boolean.as_ref()),
                NvimApiType::Integer => NvimObjectEnum::Integer(*me.0.data.integer.as_ref()),
                NvimApiType::Float => NvimObjectEnum::Float(*me.0.data.floating.as_ref()),
                NvimApiType::String => NvimObjectEnum::String(NvimString::from_ffi(
                    std::ptr::read(me.0.data.string.as_ref()),
                )),
                NvimApiType::Array => NvimObjectEnum::Array(NvimArray::from_ffi(std::ptr::read(
                    me.0.data.array.as_ref(),
                ))),
                NvimApiType::Dictionary => NvimObjectEnum::Dictionary(NvimDictionary::from_ffi(
                    std::ptr::read(me.0.data.dictionary.as_ref()),
                )),
            }
        }
    }

    /// Returns the type of this object.
    pub fn type_(&self) -> NvimApiType {
        NvimApiType::from_ffi_enum(self.0.type_)
    }
}

impl Drop for NvimObject {
    fn drop(&mut self) {
        unsafe { nvim_sys::api_free_object(self.as_borrowed_ffi()) };
    }
}

impl Clone for NvimObject {
    /// Returns a deep copy of this object.
    fn clone(&self) -> Self {
        unsafe {
            NvimObject::from_ffi(nvim_sys::copy_object(
                self.as_borrowed_ffi(),
                ptr::null_mut(),
            ))
        }
    }
}

impl From<bool> for NvimObject {
    fn from(value: bool) -> Self {
        Self::from_enum(NvimObjectEnum::Boolean(value))
    }
}

impl From<i64> for NvimObject {
    fn from(value: i64) -> Self {
        Self::from_enum(NvimObjectEnum::Integer(value))
    }
}

impl From<f64> for NvimObject {
    fn from(value: f64) -> Self {
        Self::from_enum(NvimObjectEnum::Float(value))
    }
}

impl From<NvimString> for NvimObject {
    fn from(value: NvimString) -> Self {
        Self::from_enum(NvimObjectEnum::String(value))
    }
}

impl From<NvimArray> for NvimObject {
    fn from(value: NvimArray) -> Self {
        Self::from_enum(NvimObjectEnum::Array(value))
    }
}

impl From<NvimDictionary> for NvimObject {
    fn from(value: NvimDictionary) -> Self {
        Self::from_enum(NvimObjectEnum::Dictionary(value))
    }
}

impl NvimObjectEnum {
    /// Returns the type of this object.
    pub fn type_(&self) -> NvimApiType {
        use NvimObjectEnum::*;

        match self {
            Nil => NvimApiType::Nil,
            Boolean(_) => NvimApiType::Boolean,
            Integer(_) => NvimApiType::Integer,
            Float(_) => NvimApiType::Float,
            String(_) => NvimApiType::String,
            Array(_) => NvimApiType::Array,
            Dictionary(_) => NvimApiType::Dictionary,
        }
    }
}

impl NvimApiType {
    fn from_ffi_enum(value: u32) -> Self {
        match value {
            nvim_sys::ObjectType_kObjectTypeNil => Self::Nil,
            nvim_sys::ObjectType_kObjectTypeBoolean => Self::Boolean,
            nvim_sys::ObjectType_kObjectTypeInteger => Self::Integer,
            nvim_sys::ObjectType_kObjectTypeFloat => Self::Float,
            nvim_sys::ObjectType_kObjectTypeString => Self::String,
            nvim_sys::ObjectType_kObjectTypeDictionary => Self::Dictionary,
            _ => panic!("Unsupported or Unknown type ({}).", value),
        }
    }

    /// Returns the typename
    // See `nvim/api/private/helpers.c` for reference.
    pub fn name(self) -> &'static str {
        match self {
            NvimApiType::Nil => "nil",
            NvimApiType::Boolean => "Boolean",
            NvimApiType::Integer => "Integer",
            NvimApiType::Float => "Float",
            NvimApiType::String => "String",
            NvimApiType::Array => "Array",
            NvimApiType::Dictionary => "Dict",
        }
    }
}

impl Display for NvimApiType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
