use ISPCHandle;
use empty_handle;
use ddvol;
use vec3::Vec3i;

/// A volume dataset being rendered with its ISPC handle
pub struct Volume {
    ispc_handle: ISPCHandle,
}

impl Volume {
    /// Create a new volume with the desired dimensions. Enough room will be allocated to
    /// store `dimensions.x * dimensions.y * dimensions.z` voxels.
    pub fn new(dimensions: Vec3i) -> Volume {
        let mut vol = empty_handle();
        unsafe {
            ddvol::make_volume(&mut vol as *mut ISPCHandle, &dimensions as *const Vec3i);
        }
        Volume { ispc_handle: vol }
    }
    /// Set a region of voxel data for the volume.
    pub fn set_region(&self, region: &[f32], start: Vec3i, size: Vec3i) {
        assert_eq!(region.len(), (size.x * size.y * size.z) as usize);
        unsafe {
            ddvol::set_region(self.ispc_handle, region.as_ptr(),
                              &start as *const Vec3i, &size as *const Vec3i);
        }
    }
    pub fn ispc_equiv(&self) -> ISPCHandle {
        self.ispc_handle
    }
}

impl Drop for Volume {
    fn drop(&mut self) {
        if !self.ispc_handle.is_null() {
            unsafe { ddvol::drop_volume(self.ispc_handle); }
        }
    }
}

