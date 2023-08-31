/// Bindings for safely interact with Neovim ffi types and functions.
///
/// Using this crate will set the global allocator to match neovim's allocator.
mod allocator;
pub mod arena;
pub mod types;

use std::ffi::CString;

use nvim_sys::emsg_multiline;
pub use types::*;

pub fn emsg(s: &str) {
    let s = CString::new(s).unwrap();
    unsafe {
        emsg_multiline(s.as_ptr(), true);
    }
}
