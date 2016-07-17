use ISPCHandle;
use empty_handle;
use ddvol;
use vec3::Vec3f;

/// A transfer function used to map values of the volume to colors
pub struct TransferFunction {
    ispc_handle: ISPCHandle,
}

impl TransferFunction {
    /// Create a default grayscale transferfunction which maps values from black/transparent
    /// to white/opaque.
    pub fn grayscale() -> TransferFunction {
        let colors = [Vec3f::broadcast(0.0), Vec3f::broadcast(1.0)];
        let opacities = [0.0, 1.0];
        TransferFunction::new(&colors[..], &opacities[..])
    }
    pub fn new(colors: &[Vec3f], opacities: &[f32]) -> TransferFunction {
        let mut tfn = empty_handle();
        unsafe {
            ddvol::make_transfer_function(&mut tfn as *mut ISPCHandle, colors.as_ptr(),
                                          colors.len() as i32, opacities.as_ptr(), opacities.len() as i32);
        }
        TransferFunction { ispc_handle: tfn }
    }
    pub fn ispc_equiv(&self) -> ISPCHandle {
        self.ispc_handle
    }
}

impl Drop for TransferFunction {
    fn drop(&mut self) {
        if !self.ispc_handle.is_null() {
            unsafe { ddvol::drop_transfer_function(self.ispc_handle); }
        }
    }
}


