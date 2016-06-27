use std::ptr;

use ddvol;

type ISPCHandle = *mut ::std::os::raw::c_void;

/// A volume dataset being rendered with its ISPC handle
pub struct Volume {
    ispc_handle: ISPCHandle,
}

impl Drop for Volume {
    fn drop(&mut self) {
        if !self.ispc_handle.is_null() {
            unsafe { ddvol::drop_volume(self.ispc_handle); }
        }
    }
}

