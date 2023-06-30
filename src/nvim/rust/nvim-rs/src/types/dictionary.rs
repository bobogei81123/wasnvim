use std::{
    borrow::Borrow,
    mem::{self, ManuallyDrop},
    ops::Deref,
    ptr,
};

use super::{NvimObject, NvimString};

/// Wraps a Neovim's Dictionary. (see nvim/api/private/defs.h).
///
/// Neovim's API dictionary type is nothing more than a vector of key-value pairs.
/// The keys are strings and the values are objects.
#[derive(Default)]
pub struct NvimDictionary(nvim_sys::Dictionary);

impl NvimDictionary {
    /// Creates an empty Neovim dictionary.
    pub fn new() -> Self {
        Self(nvim_sys::Dictionary {
            items: ptr::null_mut(),
            size: 0,
            capacity: 0,
        })
    }

    /// Creates an `NvimDictionary` from an owned FFI dictionary.
    ///
    /// # Safety
    /// The caller must owned the dictionary and ensure that it remains valid throughout the
    /// lifetime of this object.
    pub unsafe fn from_ffi(dict: nvim_sys::Dictionary) -> Self {
        Self(dict)
    }

    /// Converts this dictionary into an owned FFI dictionary.
    ///
    /// The caller is then responsible for freeing the dictionary.
    pub fn into_ffi(self) -> nvim_sys::Dictionary {
        let me = ManuallyDrop::new(self);
        me.as_borrowed_ffi()
    }

    /// Converts this dictionary into an borrowed FFI dictionary.
    ///
    /// The returned FFI dictionary is only a borrow, so the caller is responsible to make sure
    /// that it remain intact throughout the time it is borrowed.
    pub fn as_borrowed_ffi(&self) -> nvim_sys::Dictionary {
        unsafe { ptr::read(&self.0) }
    }

    /// Creates a new dictionary from a vector of key-value pairs.
    pub fn from_vec(vec: Vec<(NvimString, NvimObject)>) -> Self {
        let mut vec: Vec<_> = vec
            .into_iter()
            .map(|(key, val)| nvim_sys::KeyValuePair {
                key: key.into_ffi(),
                value: val.into_ffi(),
            })
            .collect();
        let size = vec.len();
        let capacity = vec.capacity();
        let items = vec.as_mut_ptr() as *mut nvim_sys::KeyValuePair;
        mem::forget(vec);

        Self(nvim_sys::Dictionary {
            items,
            size,
            capacity,
        })
    }

    /// Converts this dictionary into a vector of key-value pairs.
    pub fn into_vec(self) -> Vec<(NvimString, NvimObject)> {
        let dict = self.into_ffi();
        let size = dict.size;
        let capacity = dict.capacity;
        let items = dict.items;
        let vec = unsafe { Vec::from_raw_parts(items, size, capacity) };
        unsafe {
            vec.into_iter()
                .map(|p| (NvimString::from_ffi(p.key), NvimObject::from_ffi(p.value)))
                .collect()
        }
    }
}

impl Drop for NvimDictionary {
    fn drop(&mut self) {
        unsafe {
            nvim_sys::api_free_dictionary(self.as_borrowed_ffi());
        }
    }
}

impl Clone for NvimDictionary {
    /// Returns a deep copy of this dictionary.
    fn clone(&self) -> Self {
        unsafe {
            NvimDictionary::from_ffi(nvim_sys::copy_dictionary(
                self.as_borrowed_ffi(),
                ptr::null_mut(),
            ))
        }
    }
}

impl Deref for NvimDictionary {
    type Target = NvimDictionaryRef;

    fn deref(&self) -> &Self::Target {
        unsafe {
            &*(ptr::slice_from_raw_parts(self.0.items, self.0.size) as *const NvimDictionaryRef)
        }
    }
}

/// Represents a view into an `NvimDictionary`.
#[repr(transparent)]
pub struct NvimDictionaryRef([nvim_sys::KeyValuePair]);

impl NvimDictionaryRef {
    /// Creates an `NvimDictionaryRef` from a borrowed FFI dictionary.
    ///
    /// # Safety
    /// The caller must ensure that the underlying dictionary remains valid throughout the lifetime
    /// of this object.
    pub unsafe fn from_ffi(obj: &nvim_sys::Dictionary) -> &Self {
        Self::from_slice(unsafe { std::slice::from_raw_parts(obj.items, obj.size) })
    }

    /// Converts this dictionary into a borrowed FFI dictionary.
    ///
    /// The returned FFI dictionary is only a borrow, so the caller is responsible to make sure
    /// that it remain intact throughout the time it is borrowed.
    pub fn as_borrowed_ffi(&self) -> nvim_sys::Dictionary {
        nvim_sys::Dictionary {
            size: self.0.len(),
            // It should be safe to just set the capacity to be the size if the string is only
            // immutably borrowed, see for example, `MAXSIZE_TEMP_DICT` in
            // `nvim/api/private/helper.h`.
            capacity: self.0.len(),
            items: self.0.as_ptr() as *mut _,
        }
    }

    /// Creates an `NvimDictionaryRef` from a borrowed slice of `KeyValuePair`.
    pub fn from_slice(s: &[nvim_sys::KeyValuePair]) -> &Self {
        unsafe { &*(s as *const _ as *const Self) }
    }

    /// Returns a iterator that iterates through all the entries.
    pub fn iter(&self) -> impl Iterator<Item = (&NvimString, &NvimObject)> {
        self.0.iter().map(|kv_pair| unsafe {
            (
                NvimString::from_borrowed_ffi(&kv_pair.key),
                NvimObject::from_ffi_ref(&kv_pair.value),
            )
        })
    }

    /// Returns a iterator that iterates through all the entries with mutable reference to values.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&NvimString, &mut NvimObject)> {
        self.0.iter_mut().map(|kv_pair| unsafe {
            (
                NvimString::from_borrowed_ffi(&kv_pair.key),
                NvimObject::from_ffi_mut(&mut kv_pair.value),
            )
        })
    }

    /// Returns a reference to the value associated with the target key.
    ///
    /// Returns None if the key is not found in the list.
    pub fn get<Q>(&self, target: &Q) -> Option<&NvimObject>
    where
        NvimString: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        for (key, value) in self.iter() {
            if key.borrow() == target {
                return Some(value);
            }
        }

        None
    }

    /// Returns a mutable reference to the value associated with the target key.
    ///
    /// Returns None if the key is not found in the list.
    pub fn get_mut<Q>(&mut self, target: &Q) -> Option<&mut NvimObject>
    where
        NvimString: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        for (key, value) in self.iter_mut() {
            if key.borrow() == target {
                return Some(value);
            }
        }

        None
    }
}
