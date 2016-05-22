//! Provides the Rust-side interface to the Plane geometry

use std::ptr;

use rt;
use vec3f::Vec3f;
use ::Geometry;

/// A simple infinite plane "centered" at center
pub struct Plane {
    center: Vec3f,
    normal: Vec3f,
    ispc_geom: *const Geometry,
}

impl Plane {
    pub fn new(center: Vec3f, normal: Vec3f) -> Plane {
        let mut geom: *const Geometry = ptr::null();
        let n = normal.normalized();
        unsafe {
            rt::make_plane(&mut geom as *mut *const Geometry, &center as *const Vec3f,
                          &n as *const Vec3f);
        }
        Plane { center: center, normal: n, ispc_geom: geom }
    }
    pub fn ispc_equiv(&self) -> *const Geometry {
        self.ispc_geom
    }
}

impl Drop for Plane {
    fn drop(&mut self) {
        unsafe { rt::drop_plane(self.ispc_geom); }
    }
}

