use ISPCHandle;
use empty_handle;
use ddvol;
use vec3::Vec3f;

/// The camera that the scene is being rendered from
pub struct Camera {
    ispc_handle: ISPCHandle,
}

impl Camera {
    pub fn new(pos: Vec3f, target: Vec3f, up: Vec3f, fovy: f32, width: i32, height: i32) -> Camera {
        let mut cam = empty_handle();
        unsafe {
            ddvol::make_camera(&mut cam as *mut ISPCHandle, &pos as *const Vec3f,
                               &target as *const Vec3f, &up as *const Vec3f,
                               fovy, width, height);
        }
        Camera { ispc_handle: cam }
    }
    pub fn ispc_equiv(&self) -> ISPCHandle {
        self.ispc_handle
    }
}

impl Drop for Camera {
    fn drop(&mut self) {
        if !self.ispc_handle.is_null() {
            unsafe { ddvol::drop_camera(self.ispc_handle); }
        }
    }
}

