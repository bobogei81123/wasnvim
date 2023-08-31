use super::ffi_wrapper::{NvimFfiClone, NvimFfiType, NvimFfiWrapper};

pub type NvimWasmRef = NvimFfiWrapper<nvim_sys::WasmRef>;

impl NvimFfiType for nvim_sys::WasmRef {
    fn ffi_drop(self) {}
}

unsafe impl NvimFfiClone for nvim_sys::WasmRef {
    fn ffi_clone(self) -> Self {
        self
    }
}

impl NvimWasmRef {
    pub fn new(instance_id: i32, ref_: u32) -> Self {
        unsafe { Self::from_ffi(nvim_sys::WasmRef { instance_id, ref_ }) }
    }
}
