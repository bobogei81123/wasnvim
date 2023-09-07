use std::fmt::Display;

use crate::{
    nvim_types,
    wasm_ref::{wasmref_new, WasmRefInner},
};
use nvim_rs::{
    IntoObject, NvimApiType, NvimArray, NvimBuffer, NvimDictionary, NvimObject, NvimObjectEnum,
    NvimObjectEnumRef, NvimString, NvimTabpage, NvimWindow, ObjectConversionError, TryFromObject,
    TryFromObjectRef,
};

type Result<T> = std::result::Result<T, TypeConversionError>;

/// Types that can be converted into WASM type `T`.
pub(crate) trait TryIntoWasmType<T> {
    /// Converts the host type into the WASM client type `T`.
    ///
    /// Returns an error if the object contains the wrong variant or the conversion is not possible
    /// because the current limitation that WASM component model does not support recursive types.
    fn try_into_wasm_type(self, context: &Context) -> Result<T>;
}

/// Types that can be converted from WASM type `T`.
pub(crate) trait FromWasmType<T> {
    /// Converts the WASM client type `T` into the host type.
    fn from_wasm_type(t: T, context: &Context) -> Self;
}

pub struct Context {
    pub current_instance_id: i32,
}

/// Errors when failing to convert a host type into a WASM type.
#[derive(Debug)]
pub(crate) enum TypeConversionError {
    /// Indicates that the conversion failed because of the limitation that WASM component model
    /// does not support recursive types.
    NonPrimitiveType(NvimApiType),
    /// Indicates that the conversion failed because the object is converted to an incorrect type.
    ObjectConversionError(ObjectConversionError),
    /// Indicates that the conversion failed because the object is converted to an incorrect type.
    InstanceIdMismatch { current: i32, wasmref: i32 },
}

impl std::error::Error for TypeConversionError {}

impl Display for TypeConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error when converting host API type to WASM counterpart: "
        )?;
        match self {
            Self::NonPrimitiveType(non_primitive_type) => {
                write!(
                    f,
                    "Expect the inner type of a recursive type to be a primitive type, got {}.\
                    This is currently forbidden because WASM component model does not support \
                    recursive types.",
                    non_primitive_type
                )
            }
            Self::ObjectConversionError(err) => {
                write!(f, "{}", err)
            }
            Self::InstanceIdMismatch { current, wasmref } => {
                write!(
                    f,
                    "Expect the current instance ID ({}) to match that in the given wasmref, \
                       got ID ({})",
                    current, wasmref
                )
            }
        }
    }
}

impl From<ObjectConversionError> for TypeConversionError {
    fn from(err: ObjectConversionError) -> Self {
        Self::ObjectConversionError(err)
    }
}

macro_rules! impl_identity_wasm_conversion_for_type {
    ($ty:ty) => {
        impl TryIntoWasmType<$ty> for $ty {
            fn try_into_wasm_type(self, _context: &Context) -> Result<$ty> {
                Ok(self)
            }
        }

        impl TryIntoWasmType<$ty> for &$ty {
            fn try_into_wasm_type(self, _context: &Context) -> Result<$ty> {
                Ok(*self)
            }
        }

        impl FromWasmType<$ty> for $ty {
            fn from_wasm_type(value: $ty, _context: &Context) -> Self {
                value
            }
        }
    };
}

impl_identity_wasm_conversion_for_type!(());
impl_identity_wasm_conversion_for_type!(bool);
impl_identity_wasm_conversion_for_type!(i64);
impl_identity_wasm_conversion_for_type!(f64);

impl TryIntoWasmType<String> for NvimString {
    fn try_into_wasm_type(self, _context: &Context) -> Result<String> {
        Ok(self.into_string())
    }
}

impl TryIntoWasmType<String> for &NvimString {
    fn try_into_wasm_type(self, _context: &Context) -> Result<String> {
        Ok(self.clone().into_string())
    }
}

impl FromWasmType<String> for NvimString {
    fn from_wasm_type(value: String, _context: &Context) -> Self {
        NvimString::new(value)
    }
}

macro_rules! impl_wasm_conversion_for_remote_type {
    ($ty:ty, $wasm_ty:ident) => {
        impl TryIntoWasmType<nvim_types::$wasm_ty> for $ty {
            fn try_into_wasm_type(self, _context: &Context) -> Result<nvim_types::$wasm_ty> {
                Ok(self.handle())
            }
        }

        impl TryIntoWasmType<nvim_types::$wasm_ty> for &$ty {
            fn try_into_wasm_type(self, _context: &Context) -> Result<nvim_types::$wasm_ty> {
                Ok(self.handle())
            }
        }

        impl FromWasmType<nvim_types::$wasm_ty> for $ty {
            fn from_wasm_type(value: nvim_types::$wasm_ty, _context: &Context) -> Self {
                Self::from_handle(value)
            }
        }
    };
}

impl_wasm_conversion_for_remote_type!(NvimBuffer, Buffer);
impl_wasm_conversion_for_remote_type!(NvimWindow, Window);
impl_wasm_conversion_for_remote_type!(NvimTabpage, Tabpage);

impl TryIntoWasmType<()> for NvimObject {
    fn try_into_wasm_type(self, _context: &Context) -> Result<()> {
        Ok(self.try_into_unit()?)
    }
}

impl TryIntoWasmType<()> for &NvimObject {
    fn try_into_wasm_type(self, _context: &Context) -> Result<()> {
        Ok(self.try_into_unit()?)
    }
}

impl FromWasmType<()> for NvimObject {
    fn from_wasm_type(_: (), _context: &Context) -> Self {
        Self::nil()
    }
}

macro_rules! impl_wasm_conversion_for_obj_variant {
    ($ty:ty, $middle_ty:ty) => {
        impl TryIntoWasmType<$ty> for NvimObject {
            fn try_into_wasm_type(self, context: &Context) -> Result<$ty> {
                <$middle_ty>::try_from_object(self)?.try_into_wasm_type(context)
            }
        }

        impl TryIntoWasmType<$ty> for &NvimObject {
            fn try_into_wasm_type(self, context: &Context) -> Result<$ty> {
                <$middle_ty>::try_from_object_ref(self)?.try_into_wasm_type(context)
            }
        }

        impl FromWasmType<$ty> for NvimObject {
            fn from_wasm_type(value: $ty, context: &Context) -> Self {
                <$middle_ty>::from_wasm_type(value, context).into_object()
            }
        }
    };
}

impl_wasm_conversion_for_obj_variant!(bool, bool);
impl_wasm_conversion_for_obj_variant!(i64, i64);
impl_wasm_conversion_for_obj_variant!(f64, f64);
impl_wasm_conversion_for_obj_variant!(String, NvimString);

impl<T> TryIntoWasmType<Vec<T>> for NvimObject
where
    NvimArray: TryIntoWasmType<Vec<T>>,
{
    fn try_into_wasm_type(self, context: &Context) -> Result<Vec<T>> {
        NvimArray::try_from_object(self)?.try_into_wasm_type(context)
    }
}

impl<T> TryIntoWasmType<Vec<T>> for &NvimObject
where
    for<'a> &'a NvimArray: TryIntoWasmType<Vec<T>>,
{
    fn try_into_wasm_type(self, context: &Context) -> Result<Vec<T>> {
        NvimArray::try_from_object_ref(self)?.try_into_wasm_type(context)
    }
}

impl<T> TryIntoWasmType<Vec<(String, T)>> for NvimObject
where
    NvimDictionary: TryIntoWasmType<Vec<(String, T)>>,
{
    fn try_into_wasm_type(self, context: &Context) -> Result<Vec<(String, T)>> {
        NvimDictionary::try_from_object(self)?.try_into_wasm_type(context)
    }
}

impl<T> TryIntoWasmType<Vec<(String, T)>> for &NvimObject
where
    for<'a> &'a NvimDictionary: TryIntoWasmType<Vec<(String, T)>>,
{
    fn try_into_wasm_type(self, context: &Context) -> Result<Vec<(String, T)>> {
        NvimDictionary::try_from_object_ref(self)?.try_into_wasm_type(context)
    }
}

impl<T> TryIntoWasmType<Vec<T>> for NvimArray
where
    NvimObject: TryIntoWasmType<T>,
{
    fn try_into_wasm_type(self, context: &Context) -> Result<Vec<T>> {
        self.into_vec()
            .into_iter()
            .map(|x| x.try_into_wasm_type(context))
            .collect()
    }
}

impl<T> TryIntoWasmType<Vec<T>> for &NvimArray
where
    for<'a> &'a NvimObject: TryIntoWasmType<T>,
{
    fn try_into_wasm_type(self, context: &Context) -> Result<Vec<T>> {
        self.iter().map(|x| x.try_into_wasm_type(context)).collect()
    }
}

impl<T> FromWasmType<Vec<T>> for NvimArray
where
    NvimObject: FromWasmType<T>,
{
    fn from_wasm_type(t: Vec<T>, context: &Context) -> Self {
        NvimArray::from_vec(
            t.into_iter()
                .map(|x| NvimObject::from_wasm_type(x, context))
                .collect(),
        )
    }
}

impl<T> TryIntoWasmType<Vec<(String, T)>> for NvimDictionary
where
    NvimObject: TryIntoWasmType<T>,
{
    fn try_into_wasm_type(self, context: &Context) -> Result<Vec<(String, T)>> {
        self.into_vec()
            .into_iter()
            .map(|(key, val)| Ok((key.into_string(), val.try_into_wasm_type(context)?)))
            .collect()
    }
}

impl<T> TryIntoWasmType<Vec<(String, T)>> for &NvimDictionary
where
    for<'a> &'a NvimObject: TryIntoWasmType<T>,
{
    fn try_into_wasm_type(self, context: &Context) -> Result<Vec<(String, T)>> {
        self.iter()
            .map(|(key, val)| Ok((key.clone().into_string(), val.try_into_wasm_type(context)?)))
            .collect()
    }
}

impl<T> FromWasmType<Vec<(String, T)>> for NvimDictionary
where
    NvimObject: FromWasmType<T>,
{
    fn from_wasm_type(t: Vec<(String, T)>, context: &Context) -> Self {
        NvimDictionary::from_vec(
            t.into_iter()
                .map(|(key, val)| {
                    (
                        NvimString::new(key),
                        NvimObject::from_wasm_type(val, context),
                    )
                })
                .collect(),
        )
    }
}

impl TryIntoWasmType<nvim_types::Object> for NvimObject {
    fn try_into_wasm_type(self, context: &Context) -> Result<nvim_types::Object> {
        use nvim_types::Object::*;

        Ok(match self.into_enum() {
            NvimObjectEnum::Nil => Nil,
            NvimObjectEnum::Boolean(b) => Boolean(b),
            NvimObjectEnum::Integer(i) => Integer(i),
            NvimObjectEnum::Float(f) => Float(f),
            NvimObjectEnum::String(s) => String(s.try_into_wasm_type(context).unwrap()),
            NvimObjectEnum::Array(arr) => Array(arr.try_into_wasm_type(context)?),
            NvimObjectEnum::Dictionary(dict) => Dictionary(dict.try_into_wasm_type(context)?),
            NvimObjectEnum::Buffer(buffer) => Buffer(buffer.try_into_wasm_type(context).unwrap()),
            NvimObjectEnum::Window(window) => Window(window.try_into_wasm_type(context).unwrap()),
            NvimObjectEnum::Tabpage(tabpage) => {
                Tabpage(tabpage.try_into_wasm_type(context).unwrap())
            }
            NvimObjectEnum::WasmRef(wasmref) => {
                let wasmref = unsafe { (wasmref as *const WasmRefInner).as_ref() };
                Wasmref(wasmref.map_or(0, |WasmRefInner { instance_id, ref_ }| {
                    if *instance_id != context.current_instance_id {
                        0
                    } else {
                        *ref_
                    }
                }))
            }
            _ => unimplemented!("Encounter a new Neovim type"),
        })
    }
}

impl TryIntoWasmType<nvim_types::Object> for &NvimObject {
    fn try_into_wasm_type(self, context: &Context) -> Result<nvim_types::Object> {
        use nvim_types::Object::*;

        Ok(match self.as_enum_ref() {
            NvimObjectEnumRef::Nil => Nil,
            NvimObjectEnumRef::Boolean(&b) => Boolean(b),
            NvimObjectEnumRef::Integer(&i) => Integer(i),
            NvimObjectEnumRef::Float(&f) => Float(f),
            NvimObjectEnumRef::String(s) => String(s.try_into_wasm_type(context).unwrap()),
            NvimObjectEnumRef::Array(arr) => Array(arr.try_into_wasm_type(context)?),
            NvimObjectEnumRef::Dictionary(dict) => Dictionary(dict.try_into_wasm_type(context)?),
            NvimObjectEnumRef::Buffer(buffer) => {
                Buffer(buffer.try_into_wasm_type(context).unwrap())
            }
            NvimObjectEnumRef::Window(window) => {
                Window(window.try_into_wasm_type(context).unwrap())
            }
            NvimObjectEnumRef::Tabpage(tabpage) => {
                Tabpage(tabpage.try_into_wasm_type(context).unwrap())
            }
            NvimObjectEnumRef::WasmRef(&wasmref) => {
                let wasmref = unsafe { (wasmref as *const WasmRefInner).as_ref() };
                Wasmref(wasmref.map_or(0, |WasmRefInner { instance_id, ref_ }| {
                    if *instance_id != context.current_instance_id {
                        0
                    } else {
                        *ref_
                    }
                }))
            }
            _ => unimplemented!("Encounter a new Neovim type"),
        })
    }
}

impl FromWasmType<nvim_types::Object> for NvimObject {
    fn from_wasm_type(t: nvim_types::Object, context: &Context) -> Self {
        use nvim_types::Object::*;

        NvimObject::from_enum(match t {
            Nil => NvimObjectEnum::Nil,
            Boolean(b) => NvimObjectEnum::Boolean(<_>::from_wasm_type(b, context)),
            Integer(i) => NvimObjectEnum::Integer(<_>::from_wasm_type(i, context)),
            Float(f) => NvimObjectEnum::Float(<_>::from_wasm_type(f, context)),
            String(s) => NvimObjectEnum::String(<_>::from_wasm_type(s, context)),
            Array(arr) => NvimObjectEnum::Array(<_>::from_wasm_type(arr, context)),
            Dictionary(dict) => NvimObjectEnum::Dictionary(<_>::from_wasm_type(dict, context)),
            Buffer(buffer) => NvimObjectEnum::Buffer(<_>::from_wasm_type(buffer, context)),
            Window(window) => NvimObjectEnum::Window(<_>::from_wasm_type(window, context)),
            Tabpage(tabpage) => NvimObjectEnum::Tabpage(<_>::from_wasm_type(tabpage, context)),
            Wasmref(wasmref) => {
                NvimObjectEnum::WasmRef(wasmref_new(context.current_instance_id, wasmref))
            }
        })
    }
}

impl TryIntoWasmType<nvim_types::Primitive> for NvimObject {
    fn try_into_wasm_type(self, context: &Context) -> Result<nvim_types::Primitive> {
        use nvim_types::Primitive::*;

        Ok(match self.into_enum() {
            NvimObjectEnum::Nil => Nil,
            NvimObjectEnum::Boolean(b) => Boolean(b),
            NvimObjectEnum::Integer(i) => Integer(i),
            NvimObjectEnum::Float(f) => Float(f),
            NvimObjectEnum::String(s) => String(s.try_into_wasm_type(context).unwrap()),
            NvimObjectEnum::Array(_) => {
                return Err(TypeConversionError::NonPrimitiveType(NvimApiType::Array))
            }
            NvimObjectEnum::Dictionary(_) => {
                return Err(TypeConversionError::NonPrimitiveType(
                    NvimApiType::Dictionary,
                ))
            }
            NvimObjectEnum::Buffer(buffer) => Buffer(buffer.try_into_wasm_type(context).unwrap()),
            NvimObjectEnum::Window(window) => Window(window.try_into_wasm_type(context).unwrap()),
            NvimObjectEnum::Tabpage(tabpage) => {
                Tabpage(tabpage.try_into_wasm_type(context).unwrap())
            }
            _ => unimplemented!("Encounter a new Neovim type"),
        })
    }
}

impl TryIntoWasmType<nvim_types::Primitive> for &NvimObject {
    fn try_into_wasm_type(self, context: &Context) -> Result<nvim_types::Primitive> {
        use nvim_types::Primitive::*;

        Ok(match self.as_enum_ref() {
            NvimObjectEnumRef::Nil => Nil,
            NvimObjectEnumRef::Boolean(&b) => Boolean(b),
            NvimObjectEnumRef::Integer(&i) => Integer(i),
            NvimObjectEnumRef::Float(&f) => Float(f),
            NvimObjectEnumRef::String(s) => String(s.try_into_wasm_type(context).unwrap()),
            NvimObjectEnumRef::Array(_) => {
                return Err(TypeConversionError::NonPrimitiveType(NvimApiType::Array))
            }
            NvimObjectEnumRef::Dictionary(_) => {
                return Err(TypeConversionError::NonPrimitiveType(
                    NvimApiType::Dictionary,
                ))
            }
            NvimObjectEnumRef::Buffer(buffer) => {
                Buffer(buffer.try_into_wasm_type(context).unwrap())
            }
            NvimObjectEnumRef::Window(window) => {
                Window(window.try_into_wasm_type(context).unwrap())
            }
            NvimObjectEnumRef::Tabpage(tabpage) => {
                Tabpage(tabpage.try_into_wasm_type(context).unwrap())
            }
            _ => unimplemented!("Encounter a new Neovim type"),
        })
    }
}

impl FromWasmType<nvim_types::Primitive> for NvimObject {
    fn from_wasm_type(t: nvim_types::Primitive, context: &Context) -> Self {
        use nvim_types::Primitive::*;

        NvimObject::from_enum(match t {
            Nil => NvimObjectEnum::Nil,
            Boolean(b) => NvimObjectEnum::Boolean(<_>::from_wasm_type(b, context)),
            Integer(i) => NvimObjectEnum::Integer(<_>::from_wasm_type(i, context)),
            Float(f) => NvimObjectEnum::Float(<_>::from_wasm_type(f, context)),
            String(s) => NvimObjectEnum::String(<_>::from_wasm_type(s, context)),
            Buffer(buffer) => NvimObjectEnum::Buffer(<_>::from_wasm_type(buffer, context)),
            Window(window) => NvimObjectEnum::Window(<_>::from_wasm_type(window, context)),
            Tabpage(tabpage) => NvimObjectEnum::Tabpage(<_>::from_wasm_type(tabpage, context)),
            Wasmref(wasmref) => {
                NvimObjectEnum::WasmRef(wasmref_new(context.current_instance_id, wasmref))
            }
        })
    }
}
