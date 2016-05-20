//! Provides the Rust-side interface to the Sphere geometry

use std::ptr;

use rt;
use vec3f::Vec3f;
use ::Geometry;

/// A simple sphere with some radius located at `center`
pub struct Sphere {
    center: Vec3f,
    radius: f32,
    ispc_geom: *const Geometry,
}

impl Sphere {
    pub fn new(center: Vec3f, radius: f32) -> Sphere {
        // Currently acting exactly like ospray, and I think this is probably the way to go
        // we maintain a separate ISPC and Rust side geometry potentially with the ISPC side
        // pointing back to data on the Rust side. Thus we're free to do C-style polymorphism
        // in ISPC and whatever we want in Rust however it does make writing new geometries or
        // whatever a bit awkward.
        let mut geom: *const Geometry = ptr::null();
        unsafe {
            rt::make_sphere(&mut geom as *mut *const Geometry, &center as *const Vec3f, radius);
        }
        Sphere { center: center, radius: radius, ispc_geom: geom }
    }
    pub fn ispc_equiv(&self) -> *const Geometry {
        self.ispc_geom
    }
}

impl Drop for Sphere {
    fn drop(&mut self) {
        unsafe { rt::drop_sphere(self.ispc_geom); }
    }
}

