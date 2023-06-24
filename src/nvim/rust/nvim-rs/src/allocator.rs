use std::alloc::{GlobalAlloc, Layout};

/// Allocator that implements Neovim's allocation behavior
///
/// This allocator is implemented with libc functions (`malloc`, `free`, etc.). It make Rust and
/// Neovim allocation behavior the same. We make a further (shady) assumption that the pointer
/// returned from `into_raw*` function of Rust's `Vec` and `CString` can be safely freed if both
/// used the libc allocator.
//
// TODO: Eliminate the need of this assumption by implementing Vec and String data structure
// ourselves.
struct LibcAllocator;

#[global_allocator]
static LIBC_ALLOCATOR: LibcAllocator = LibcAllocator;

unsafe impl GlobalAlloc for LibcAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.align() <= MAX_LIBC_GUARANTEED_ALIGN {
            nvim_sys::xmalloc(layout.size()) as _
        } else {
            aligned_alloc(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _: Layout) {
        nvim_sys::xfree(ptr as _)
    }

    unsafe fn realloc(&self, ptr: *mut u8, _: Layout, new_size: usize) -> *mut u8 {
        nvim_sys::xrealloc(ptr as _, new_size) as _
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if layout.align() <= MAX_LIBC_GUARANTEED_ALIGN {
            return nvim_sys::xcalloc(layout.size(), 1) as _;
        }
        let ptr = aligned_alloc(layout);
        if ptr.is_null() {
            return ptr;
        }
        std::ptr::write_bytes(ptr, 0, layout.size());
        ptr
    }
}

const MAX_LIBC_GUARANTEED_ALIGN: usize = std::mem::align_of::<libc::max_align_t>();

#[cfg(target_family = "unix")]
unsafe fn aligned_alloc(layout: Layout) -> *mut u8 {
    assert!(
        layout.align() >= std::mem::size_of::<usize>(),
        "posix_memalign requires that align is greater than sizeof(*void)"
    );
    let mut ptr = std::ptr::null_mut();
    let result = unsafe { libc::posix_memalign(&mut ptr, layout.align(), layout.size()) };
    if result == 0 {
        return ptr as _;
    }
    nvim_sys::try_to_free_memory();
    let result = libc::posix_memalign(&mut ptr, layout.align(), layout.size());
    if result != 0 {
        nvim_sys::preserve_exit(nvim_sys::e_outofmem.as_ptr());
    }
    ptr as _
}

#[cfg(not(target_family = "unix"))]
unsafe fn aligned_alloc(layout: Layout) -> *mut u8 {
    unimplemented!("aligned_alloc is only implemented for unix targets for now.");
}
