//! Provides the Rust-side interface to the lights available in the scene. Currently
//! just a point light though I may add directional lights later

use std::ptr;

use rt;
use vec3f::Vec3f;

/// Type alias for the Light base struct in ISPC
pub type Light = ::rt::Struct_Light;

/// A simple point light with some emissive color
pub struct PointLight {
    position: Vec3f,
    emission: Vec3f,
    ispc_equiv: *const Light,
}

impl PointLight {
    pub fn new(position: Vec3f, emission: Vec3f) -> PointLight {
        let mut light: *const Light = ptr::null();
        unsafe {
            rt::make_point_light(&mut light as *mut *const Light, &position as *const Vec3f,
                                 &emission as *const Vec3f);
        }
        PointLight { position: position, emission: emission, ispc_equiv: light }
    }
    pub fn ispc_equiv(&self) -> *const Light {
        self.ispc_equiv
    }
}

impl Drop for PointLight {
    fn drop(&mut self) {
        unsafe { rt::drop_point_light(self.ispc_equiv); }
    }
}

