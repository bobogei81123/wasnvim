/// Bindings for safely interact with Neovim ffi types and functions.
///
/// Using this crate will set the global allocator to match neovim's allocator.
mod allocator;
pub mod types;
pub mod arena;

pub use types::*;
