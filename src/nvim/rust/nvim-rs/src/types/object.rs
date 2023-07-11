use std::{fmt::Display, mem::ManuallyDrop, ptr};

use crate::{NvimBuffer, NvimTabpage, NvimWindow};

use super::{
    ffi_wrapper::{NvimFfiClone, NvimFfiType, NvimFfiWrapper},
    NvimArray, NvimDictionary, NvimString,
};

/// Wraps an owned Neovim `Object` (see nvim/api/private/defs.h).
pub type NvimObject = NvimFfiWrapper<nvim_sys::Object>;

/// Wraps a Neovim's Object. (see nvim/api/private/defs.h).
#[non_exhaustive]
// #[derive(derive_more::TryInto)]
// #[try_into(owned, ref, ref_mut)]
pub enum NvimObjectEnum {
    Nil,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(NvimString),
    Array(NvimArray),
    Dictionary(NvimDictionary),
    Buffer(NvimBuffer),
    Window(NvimWindow),
    Tabpage(NvimTabpage),
}

/// Wraps a Neovim's Object. (see nvim/api/private/defs.h).
#[non_exhaustive]
// #[derive(derive_more::TryInto)]
// #[try_into(owned, ref, ref_mut)]
pub enum NvimObjectEnumRef<'a> {
    Nil,
    Boolean(&'a bool),
    Integer(&'a i64),
    Float(&'a f64),
    String(&'a NvimString),
    Array(&'a NvimArray),
    Dictionary(&'a NvimDictionary),
    Buffer(&'a NvimBuffer),
    Window(&'a NvimWindow),
    Tabpage(&'a NvimTabpage),
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
    Buffer,
    Window,
    Tabpage,
}

/// Rust types that can be convert to an [`NvimObject`].
pub trait IntoObject: Sized {
    /// Converts `self` into an [`NvimObject`].
    fn into_object(self) -> NvimObject;
}

/// Rust types that can possibly be created from an [`NvimObject`].
pub trait TryFromObject: Sized {
    /// Tries to convert an [`NvimObject`] into `Self` type.
    fn try_from_object(obj: NvimObject) -> Result<Self, ObjectConversionError>;
}

/// Rust types that its reference can be created from a reference to [`NvimObject`].
pub trait TryFromObjectRef: Sized {
    /// Tries to convert an [`&NvimObject`](NvimObject) into `&Self`.
    fn try_from_object_ref(obj: &NvimObject) -> Result<&Self, ObjectConversionError>;
}

/// Error when trying to convert an [`NvimObject`] holding a variant into an incorrect type.
///
/// This happens, for example, when converting an object holding a string into an integer.
#[derive(Debug)]
pub struct ObjectConversionError {
    object_type: NvimApiType,
    expected_type: NvimApiType,
}

impl std::error::Error for ObjectConversionError {}

impl Display for ObjectConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to convert an NvimObject with type {:?} into {:?}",
            self.object_type, self.expected_type
        )
    }
}

impl NvimObject {
    /// Returns a nil object.
    pub fn nil() -> Self {
        Self::from_enum(NvimObjectEnum::Nil)
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
            NvimObjectEnum::Buffer(buffer) => {
                result.type_ = nvim_sys::ObjectType_kObjectTypeBuffer;
                unsafe {
                    *result.data.integer.as_mut() = buffer.handle();
                }
            }
            NvimObjectEnum::Window(window) => {
                result.type_ = nvim_sys::ObjectType_kObjectTypeWindow;
                unsafe {
                    *result.data.integer.as_mut() = window.handle();
                }
            }
            NvimObjectEnum::Tabpage(tabpage) => {
                result.type_ = nvim_sys::ObjectType_kObjectTypeTabpage;
                unsafe {
                    *result.data.integer.as_mut() = tabpage.handle();
                }
            }
        }

        unsafe { Self::from_ffi(result) }
    }

    /// Converts this `NvimObject` into an `NvimObjectEnum`.
    pub fn into_enum(self) -> NvimObjectEnum {
        let me = ManuallyDrop::new(self);
        unsafe {
            match me.type_() {
                NvimApiType::Nil => NvimObjectEnum::Nil,
                NvimApiType::Boolean => {
                    NvimObjectEnum::Boolean(*me.as_ffi_ref().data.boolean.as_ref())
                }
                NvimApiType::Integer => {
                    NvimObjectEnum::Integer(*me.as_ffi_ref().data.integer.as_ref())
                }
                NvimApiType::Float => {
                    NvimObjectEnum::Float(*me.as_ffi_ref().data.floating.as_ref())
                }
                NvimApiType::String => NvimObjectEnum::String(NvimString::from_ffi(
                    std::ptr::read(me.as_ffi_ref().data.string.as_ref()),
                )),
                NvimApiType::Array => NvimObjectEnum::Array(NvimArray::from_ffi(std::ptr::read(
                    me.as_ffi_ref().data.array.as_ref(),
                ))),
                NvimApiType::Dictionary => NvimObjectEnum::Dictionary(NvimDictionary::from_ffi(
                    std::ptr::read(me.as_ffi_ref().data.dictionary.as_ref()),
                )),
                NvimApiType::Buffer => NvimObjectEnum::Buffer(NvimBuffer::from_handle(
                    *me.as_ffi_ref().data.integer.as_ref(),
                )),
                NvimApiType::Window => NvimObjectEnum::Window(NvimWindow::from_handle(
                    *me.as_ffi_ref().data.integer.as_ref(),
                )),
                NvimApiType::Tabpage => NvimObjectEnum::Tabpage(NvimTabpage::from_handle(
                    *me.as_ffi_ref().data.integer.as_ref(),
                )),
            }
        }
    }

    /// Converts a reference to this `NvimObject` into an `NvimObjectEnumRef`.
    pub fn as_enum_ref(&self) -> NvimObjectEnumRef {
        let me = ManuallyDrop::new(self);
        unsafe {
            match me.type_() {
                NvimApiType::Nil => NvimObjectEnumRef::Nil,
                NvimApiType::Boolean => {
                    NvimObjectEnumRef::Boolean(me.as_ffi_ref().data.boolean.as_ref())
                }
                NvimApiType::Integer => {
                    NvimObjectEnumRef::Integer(me.as_ffi_ref().data.integer.as_ref())
                }
                NvimApiType::Float => {
                    NvimObjectEnumRef::Float(me.as_ffi_ref().data.floating.as_ref())
                }
                NvimApiType::String => NvimObjectEnumRef::String(NvimString::from_ffi_ref(
                    me.as_ffi_ref().data.string.as_ref(),
                )),
                NvimApiType::Array => NvimObjectEnumRef::Array(NvimArray::from_ffi_ref(
                    me.as_ffi_ref().data.array.as_ref(),
                )),
                NvimApiType::Dictionary => NvimObjectEnumRef::Dictionary(
                    NvimDictionary::from_ffi_ref(me.as_ffi_ref().data.dictionary.as_ref()),
                ),
                NvimApiType::Buffer => NvimObjectEnumRef::Buffer(NvimBuffer::from_handle_ref(
                    me.as_ffi_ref().data.integer.as_ref(),
                )),
                NvimApiType::Window => NvimObjectEnumRef::Window(NvimWindow::from_handle_ref(
                    me.as_ffi_ref().data.integer.as_ref(),
                )),
                NvimApiType::Tabpage => NvimObjectEnumRef::Tabpage(NvimTabpage::from_handle_ref(
                    me.as_ffi_ref().data.integer.as_ref(),
                )),
            }
        }
    }

    /// Returns the type of this object.
    pub fn type_(&self) -> NvimApiType {
        NvimApiType::from_ffi_enum(self.as_ffi_ref().type_)
    }

    /// Returns true if the object is nil.
    pub fn is_nil(&self) -> bool {
        self.type_() == NvimApiType::Nil
    }

    /// Returns `()` if the object is nil, and an `ObjectConverError` otherwise.
    pub fn try_into_unit(&self) -> Result<(), ObjectConversionError> {
        if !self.is_nil() {
            return Err(ObjectConversionError {
                object_type: self.type_(),
                expected_type: NvimApiType::Nil,
            });
        }

        Ok(())
    }
}

impl NvimFfiType for nvim_sys::Object {
    fn ffi_drop(self) {
        unsafe {
            nvim_sys::api_free_object(self);
        }
    }
}

unsafe impl NvimFfiClone for nvim_sys::Object {
    /// Creates a deep copy of the object.
    fn ffi_clone(self) -> Self {
        unsafe { nvim_sys::copy_object(self, ptr::null_mut()) }
    }
}

macro_rules! impl_object_conversion_for_variant {
    ($ty:ty, $api_type:ident) => {
        impl IntoObject for $ty {
            fn into_object(self) -> NvimObject {
                NvimObject::from_enum(NvimObjectEnum::$api_type(self))
            }
        }

        impl TryFromObject for $ty {
            fn try_from_object(value: NvimObject) -> Result<Self, ObjectConversionError> {
                match value.into_enum() {
                    NvimObjectEnum::$api_type(value) => Ok(value),
                    v => Err(ObjectConversionError {
                        object_type: v.type_(),
                        expected_type: NvimApiType::$api_type,
                    }),
                }
            }
        }

        impl<'a> TryFromObjectRef for $ty {
            fn try_from_object_ref(value: &NvimObject) -> Result<&Self, ObjectConversionError> {
                match value.as_enum_ref() {
                    NvimObjectEnumRef::$api_type(value) => Ok(value),
                    v => Err(ObjectConversionError {
                        object_type: v.type_(),
                        expected_type: NvimApiType::$api_type,
                    }),
                }
            }
        }
    };
}

impl_object_conversion_for_variant!(bool, Boolean);
impl_object_conversion_for_variant!(i64, Integer);
impl_object_conversion_for_variant!(f64, Float);
impl_object_conversion_for_variant!(NvimString, String);
impl_object_conversion_for_variant!(NvimArray, Array);
impl_object_conversion_for_variant!(NvimDictionary, Dictionary);
impl_object_conversion_for_variant!(NvimBuffer, Buffer);
impl_object_conversion_for_variant!(NvimWindow, Window);
impl_object_conversion_for_variant!(NvimTabpage, Tabpage);

impl IntoObject for () {
    fn into_object(self) -> NvimObject {
        NvimObject::from_enum(NvimObjectEnum::Nil)
    }
}

impl TryFromObject for () {
    fn try_from_object(value: NvimObject) -> Result<Self, ObjectConversionError> {
        match value.into_enum() {
            NvimObjectEnum::Nil => Ok(()),
            v => Err(ObjectConversionError {
                object_type: v.type_(),
                expected_type: NvimApiType::Nil,
            }),
        }
    }
}

impl IntoObject for NvimObject {
    fn into_object(self) -> NvimObject {
        self
    }
}

impl TryFromObject for NvimObject {
    fn try_from_object(obj: NvimObject) -> Result<Self, ObjectConversionError> {
        Ok(obj)
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
            Buffer(_) => NvimApiType::Buffer,
            Window(_) => NvimApiType::Window,
            Tabpage(_) => NvimApiType::Tabpage,
        }
    }
}

impl<'a> NvimObjectEnumRef<'a> {
    /// Returns the type of this object.
    pub fn type_(&self) -> NvimApiType {
        use NvimObjectEnumRef::*;

        match self {
            Nil => NvimApiType::Nil,
            Boolean(_) => NvimApiType::Boolean,
            Integer(_) => NvimApiType::Integer,
            Float(_) => NvimApiType::Float,
            String(_) => NvimApiType::String,
            Array(_) => NvimApiType::Array,
            Dictionary(_) => NvimApiType::Dictionary,
            Buffer(_) => NvimApiType::Buffer,
            Window(_) => NvimApiType::Window,
            Tabpage(_) => NvimApiType::Tabpage,
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
    // See `src/nvim/api/private/helpers.c` for reference.
    pub fn name(self) -> &'static str {
        match self {
            NvimApiType::Nil => "nil",
            NvimApiType::Boolean => "Boolean",
            NvimApiType::Integer => "Integer",
            NvimApiType::Float => "Float",
            NvimApiType::String => "String",
            NvimApiType::Array => "Array",
            NvimApiType::Dictionary => "Dict",
            NvimApiType::Buffer => "Buffer",
            NvimApiType::Window => "Window",
            NvimApiType::Tabpage => "Tabpage",
        }
    }
}

impl Display for NvimApiType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
