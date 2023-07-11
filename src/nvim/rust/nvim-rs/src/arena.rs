use std::ptr;

/// Wraps an `Arena`.
///
/// See `src/nvim/memory.h`.
pub struct NvimArena(nvim_sys::Arena);

impl NvimArena {
    /// Creates an empty arena.
    pub fn new() -> Self {
        NvimArena(nvim_sys::Arena {
            cur_blk: ptr::null_mut(),
            pos: 0,
            size: 0,
        })
    }

    /// Returns a mutable reference to the inner FFI struct.
    pub fn as_ffi_mut(&mut self) -> &mut nvim_sys::Arena {
        &mut self.0
    }
}

impl Default for NvimArena {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for NvimArena {
    fn drop(&mut self) {
        unsafe {
            nvim_sys::arena_mem_free(nvim_sys::arena_finish(&mut self.0 as *mut _));
        }
    }
}
