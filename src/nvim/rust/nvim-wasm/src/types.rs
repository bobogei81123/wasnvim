use std::fmt::Display;

use crate::nvim_api;
use nvim_rs::{NvimApiType, NvimArray, NvimDictionary, NvimObject, NvimObjectEnum, NvimString};

type Result<T> = std::result::Result<T, TypeConversionError>;

/// WASM client types that can be used by plugins.
pub(crate) trait WasmType: Sized {
    /// The corresponding host type.
    type HostType;

    /// Converts the client type into the host type.
    fn into_host(self) -> Self::HostType;

    /// Converts the client type into the host type.
    ///
    /// Returns an error if the conversion is not possible because the current limitation that WASM
    /// component model does not support recursive types.
    fn try_from_host(host: Self::HostType) -> Result<Self>;
}

#[derive(Debug)]
pub(crate) struct TypeConversionError {
    non_primitive_type: NvimApiType,
}

impl std::error::Error for TypeConversionError {}

impl Display for TypeConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "type conversion error: {} is not a primitive type. \
            This is currently forbidden because WASM component model does not \
            support recursive types.",
            self.non_primitive_type
        )
    }
}

impl WasmType for String {
    type HostType = NvimString;

    fn into_host(self) -> NvimString {
        NvimString::new(self)
    }

    fn try_from_host(host: NvimString) -> Result<Self> {
        Ok(host.into_string())
    }
}

impl WasmType for nvim_api::Array {
    type HostType = NvimArray;

    fn into_host(self) -> NvimArray {
        NvimArray::from_vec(self.into_iter().map(|p| p.into_host()).collect())
    }

    fn try_from_host(host: NvimArray) -> Result<Self> {
        host.into_vec()
            .into_iter()
            .map(nvim_api::Primitive::try_from_host)
            .collect::<Result<Vec<_>>>()
    }
}

impl WasmType for Vec<nvim_api::Object> {
    type HostType = NvimArray;

    fn into_host(self) -> NvimArray {
        NvimArray::from_vec(self.into_iter().map(|p| p.into_host()).collect())
    }

    fn try_from_host(host: NvimArray) -> Result<Self> {
        host.into_vec()
            .into_iter()
            .map(nvim_api::Object::try_from_host)
            .collect::<Result<Vec<_>>>()
    }
}

impl WasmType for nvim_api::Dictionary {
    type HostType = NvimDictionary;

    fn into_host(self) -> NvimDictionary {
        NvimDictionary::from_vec(
            self.into_iter()
                .map(|(key, value)| (NvimString::new(key), value.into_host()))
                .collect(),
        )
    }

    fn try_from_host(host: NvimDictionary) -> Result<Self> {
        host.into_vec()
            .into_iter()
            .map(|(key, value)| -> Result<_> {
                Ok((
                    key.into_string(),
                    nvim_api::Primitive::try_from_host(value)?,
                ))
            })
            .collect::<Result<Vec<_>>>()
    }
}

impl WasmType for nvim_api::Object {
    type HostType = NvimObject;

    fn into_host(self) -> NvimObject {
        use nvim_api::Object::*;

        match self {
            Nil => NvimObject::nil(),
            Boolean(b) => b.into(),
            Integer(i) => i.into(),
            Float(f) => f.into(),
            String(s) => NvimString::new(s).into(),
            Array(arr) => arr.into_host().into(),
            Dictionary(arr) => arr.into_host().into(),
        }
    }

    fn try_from_host(host: NvimObject) -> Result<Self> {
        use nvim_api::Object::*;

        Ok(match host.into_enum() {
            NvimObjectEnum::Nil => Nil,
            NvimObjectEnum::Boolean(b) => Boolean(b),
            NvimObjectEnum::Integer(i) => Integer(i),
            NvimObjectEnum::Float(f) => Float(f),
            NvimObjectEnum::String(s) => String(s.into_string()),
            NvimObjectEnum::Array(arr) => Array(nvim_api::Array::try_from_host(arr)?),
            NvimObjectEnum::Dictionary(arr) => {
                Dictionary(nvim_api::Dictionary::try_from_host(arr)?)
            }
            _ => unimplemented!("Encounter a new Neovim type"),
        })
    }
}

impl WasmType for nvim_api::Primitive {
    type HostType = NvimObject;

    fn into_host(self) -> NvimObject {
        use nvim_api::Primitive::*;

        match self {
            Nil => NvimObject::nil(),
            Boolean(b) => b.into(),
            Integer(i) => i.into(),
            Float(f) => f.into(),
            String(s) => NvimString::new(s).into(),
        }
    }

    fn try_from_host(host: NvimObject) -> Result<Self> {
        use nvim_api::Primitive::*;

        Ok(match host.into_enum() {
            NvimObjectEnum::Nil => Nil,
            NvimObjectEnum::Boolean(b) => Boolean(b),
            NvimObjectEnum::Integer(i) => Integer(i),
            NvimObjectEnum::Float(f) => Float(f),
            NvimObjectEnum::String(s) => nvim_api::Primitive::String(s.into_string()),
            recursive_type => Err(TypeConversionError {
                non_primitive_type: recursive_type.type_(),
            })?,
        })
    }
}
