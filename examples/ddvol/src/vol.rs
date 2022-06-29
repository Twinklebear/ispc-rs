use crate::ddvol;
use crate::empty_handle;
use crate::tfn::TransferFunction;
use crate::vec3::Vec3i;
use crate::ISPCHandle;

/// A volume dataset being rendered with its ISPC handle
pub struct Volume {
    ispc_handle: ISPCHandle,
    tfn: TransferFunction,
}

impl Volume {
    /// Create a new volume with the desired dimensions. Enough room will be allocated to
    /// store `dimensions.x * dimensions.y * dimensions.z` voxels.
    pub fn new(dimensions: Vec3i) -> Volume {
        let mut vol = empty_handle();
        let tfn = TransferFunction::cool_warm();
        unsafe {
            ddvol::make_volume(
                &mut vol as *mut ISPCHandle,
                &dimensions as *const Vec3i,
                tfn.ispc_equiv(),
            );
        }
        Volume {
            ispc_handle: vol,
            tfn: tfn,
        }
    }
    /// Set the transfer function used by the volume, overriding the default cool/warm.
    pub fn set_transfer_function(&mut self, tfn: TransferFunction) {
        self.tfn = tfn;
        unsafe {
            ddvol::volume_set_transfer_function(self.ispc_handle, self.tfn.ispc_equiv());
        }
    }
    /// Change the isovalue being rendered. Setting to a value less than 0 will turn off
    /// the isosurface.
    pub fn set_isovalue(&mut self, isovalue: f32) {
        unsafe {
            ddvol::volume_set_isovalue(self.ispc_handle, isovalue);
        }
    }
    /// Set a region of voxel data for the volume.
    pub fn set_region(&mut self, region: &[f32], start: Vec3i, size: Vec3i) {
        assert_eq!(region.len(), (size.x * size.y * size.z) as usize);
        unsafe {
            ddvol::set_region(
                self.ispc_handle,
                region.as_ptr(),
                &start as *const Vec3i,
                &size as *const Vec3i,
            );
        }
    }
    pub fn ispc_equiv(&self) -> ISPCHandle {
        self.ispc_handle
    }
}

impl Drop for Volume {
    fn drop(&mut self) {
        if !self.ispc_handle.is_null() {
            unsafe {
                ddvol::drop_volume(self.ispc_handle);
            }
        }
    }
}
