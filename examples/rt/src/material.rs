//! Defines the Rust-side interface to various ISPC materials

use std::ptr;

use rt;
use vec3f::Vec3f;

/// Type alias for the Geometry base struct in ISPC
pub type Material = ::rt::Material;

/// A simple Lambertian material
pub struct Lambertian {
    albedo: Vec3f,
    ispc_equiv: *const Material,
}

impl Lambertian {
    pub fn new(albedo: Vec3f) -> Lambertian {
        let mut mat: *const Material = ptr::null();
        unsafe {
            rt::make_lambertian(&mut mat as *mut *const Material, &albedo as *const Vec3f);
        }
        Lambertian {
            albedo: albedo,
            ispc_equiv: mat,
        }
    }
    pub fn ispc_equiv(&self) -> *const Material {
        self.ispc_equiv
    }
}

impl Drop for Lambertian {
    fn drop(&mut self) {
        unsafe {
            rt::drop_lambertian(self.ispc_equiv);
        }
    }
}
