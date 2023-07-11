use std::{mem::ManuallyDrop, ptr};

use crate::arena::NvimArena;

pub(crate) use self::hidden::NvimFfiType;

#[repr(transparent)]
pub struct NvimFfiWrapper<T: NvimFfiType>(T);

pub(crate) mod hidden {
    /// Neovim API FFI types.
    pub trait NvimFfiType {
        /// Frees the FFI struct and the resource it used.
        fn ffi_drop(self);
    }
}

/// Neovim API FFI types that can be cloned.
///
/// # Safety
/// `ffi_clone` must be implemented correctly so that it indeed create a **deep copy** of the
/// original object that can be freed seperately.
pub unsafe trait NvimFfiClone {
    /// Returns a **deep** copy of the given FFI struct. This means that the returned struct should
    /// remain valid even if the argument struct is freed elsewhere.
    fn ffi_clone(self) -> Self;
}

impl<T: NvimFfiType> NvimFfiWrapper<T> {
    /// Creates an rust FFI wrapper from an owned FFI struct.
    ///
    /// # Safety
    /// The caller must owned the struct and ensure that it remains valid throughout the
    /// lifetime of this wrapper.
    pub unsafe fn from_ffi(obj: T) -> Self {
        NvimFfiWrapper(obj)
    }

    /// Creates a reference to the rust FFI wrapper from a borrowed FFI struct.
    ///
    /// # Safety
    /// The caller must ensure that the FFI struct remains valid throughout the lifetime of this
    /// reference.
    pub unsafe fn from_ffi_ref(obj: &T) -> &Self {
        unsafe { &*(obj as *const _ as *const Self) }
    }

    /// Creates an exclusive reference to the rust FFI wrapper from a borrowed FFI struct.
    ///
    /// # Safety
    /// The caller must ensure that the FFI struct remains valid throughout the lifetime of this
    /// exclusive reference.
    pub unsafe fn from_ffi_mut(obj: &mut nvim_sys::Object) -> &mut Self {
        unsafe { &mut *(obj as *mut _ as *mut Self) }
    }

    /// Creates an reference from a FFI struct allocated in the given arena.
    ///
    /// # Safety
    /// The caller must ensure that the FFI struct is allocated in the arena.
    pub unsafe fn from_ffi_ref_with_arena<'a>(obj: &'a T, _arena: &'a NvimArena) -> &'a Self {
        unsafe { &*(obj as *const _ as *const Self) }
    }

    /// Converts this wrapper into an owned FFI struct.
    ///
    /// The caller is then responsible for freeing the struct.
    pub fn into_ffi(self) -> T {
        let me = ManuallyDrop::new(self);
        me.as_borrowed_ffi()
    }

    /// Returns a reference to the inner FFI struct.
    pub fn as_ffi_ref(&self) -> &T {
        &self.0
    }

    /// Returns a mutable reference to the inner FFI struct.
    pub fn as_ffi_mut(&mut self) -> &mut T {
        &mut self.0
    }

    /// Converts this wrapper into a borrowed FFI struct.
    ///
    /// The returned FFI struct is only a borrow, so the caller is responsible to make sure
    /// that it remain intact throughout the time it is borrowed.
    pub fn as_borrowed_ffi(&self) -> T {
        unsafe { ptr::read(&self.0) }
    }

    /// Converts this wrapper into a exclusively borrowed FFI struct.
    ///
    /// The returned FFI struct is only a borrow, so the caller is responsible to make sure
    /// that it remain intact throughout the time it is borrowed.
    pub fn as_mut_borrowed_ffi(&mut self) -> T {
        unsafe { ptr::read(&self.0) }
    }
}

impl<T: NvimFfiType> Drop for NvimFfiWrapper<T> {
    fn drop(&mut self) {
        self.as_mut_borrowed_ffi().ffi_drop();
    }
}

impl<T: NvimFfiType + NvimFfiClone> Clone for NvimFfiWrapper<T> {
    fn clone(&self) -> Self {
        unsafe { Self::from_ffi(self.as_borrowed_ffi().ffi_clone()) }
    }
}
