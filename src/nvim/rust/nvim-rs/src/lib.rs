use std::ffi::CString;

use types::{NvimArray, NvimError, NvimObject, NvimResult, NvimString};

pub mod types;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Nvim error: {0}")]
    Nvim(#[from] NvimError),
    #[error("Other: {0}")]
    Other(#[from] OtherError),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum OtherError {
    #[error(
        "Failed to convert a Rust String to a CString or the other way around. \
            This happens when trying to convert a non-UTF-8 CString to a Rust String, \
            or when trying to convert a Rust String with Null bytes to a CString."
    )]
    StringConversionError,
}

pub fn nvim_command(cmd: String) -> Result<()> {
    let len = cmd.len();
    let cmd = CString::new(cmd).map_err(|_| Error::Other(OtherError::StringConversionError))?;
    let cmd_nvim_str = nvim_sys::String {
        data: cmd.as_ptr() as *mut _,
        size: len,
    };
    let mut result = NvimResult::default();
    unsafe {
        nvim_sys::nvim_command(cmd_nvim_str, result.as_ffi_error_mut());
    }
    Ok(result.into_result()?)
}

pub fn nvim_call_function(name: String, args: NvimArray) -> Result<NvimObject> {
    let name = NvimString::new(name)?;
    let mut result = NvimResult::default();
    let obj = unsafe {
        nvim_sys::nvim_call_function(
            name.as_ffi_borrowed_string(),
            args.into_ffi(),
            result.as_ffi_error_mut(),
        )
    };
    Ok(result.into_result().map(|()| NvimObject::from_ffi(obj))?)
}

pub fn emsg(s: &str) {
    unsafe {
        let s = CString::new(s).unwrap();
        nvim_sys::emsg_multiline(s.as_ptr(), true);
    }
}

pub fn msg(s: &str) {
    unsafe {
        let s = CString::new(s).unwrap();
        nvim_sys::msg(s.as_ptr());
    }
}
