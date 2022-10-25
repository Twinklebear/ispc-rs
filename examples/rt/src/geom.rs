//! Defines the Rust-side geometry types for setting up a scene in Rust
//! to render with ISPC.

use std::ptr;

use crate::material::Lambertian;
use crate::rt;
use crate::vec3f::Vec3f;

/// Type alias for the Geometry base struct in ISPC
pub type Geometry = crate::rt::Geometry;

pub trait ISPCGeometry {
    fn ispc_equiv(&self) -> *const Geometry;
}

/// A simple sphere with some radius located at `center`

#[allow(dead_code)]
pub struct Sphere {
    center: Vec3f,
    radius: f32,
    material: Lambertian,
    ispc_geom: *const Geometry,
}

impl Sphere {
    pub fn new(center: Vec3f, radius: f32, mat: Lambertian) -> Sphere {
        // Currently acting exactly like ospray, and I think this is probably the way to go
        // we maintain a separate ISPC and Rust side geometry potentially with the ISPC side
        // pointing back to data on the Rust side. Thus we're free to do C-style polymorphism
        // in ISPC and whatever we want in Rust however it does make writing new geometries or
        // whatever a bit awkward.
        let mut geom: *const Geometry = ptr::null();
        unsafe {
            rt::make_sphere(
                &mut geom as *mut *const Geometry,
                &center as *const Vec3f,
                radius,
                mat.ispc_equiv(),
            );
        }
        Sphere {
            center,
            radius,
            material: mat,
            ispc_geom: geom,
        }
    }
}

impl ISPCGeometry for Sphere {
    fn ispc_equiv(&self) -> *const Geometry {
        self.ispc_geom
    }
}

impl Drop for Sphere {
    fn drop(&mut self) {
        unsafe {
            rt::drop_sphere(self.ispc_geom);
        }
    }
}

/// A simple infinite plane "centered" at center
#[allow(dead_code)]
pub struct Plane {
    center: Vec3f,
    normal: Vec3f,
    material: Lambertian,
    ispc_geom: *const Geometry,
}

impl Plane {
    pub fn new(center: Vec3f, normal: Vec3f, mat: Lambertian) -> Plane {
        let mut geom: *const Geometry = ptr::null();
        let n = normal.normalized();
        unsafe {
            rt::make_plane(
                &mut geom as *mut *const Geometry,
                &center as *const Vec3f,
                &n as *const Vec3f,
                mat.ispc_equiv(),
            );
        }
        Plane {
            center,
            normal: n,
            material: mat,
            ispc_geom: geom,
        }
    }
}

impl ISPCGeometry for Plane {
    fn ispc_equiv(&self) -> *const Geometry {
        self.ispc_geom
    }
}

impl Drop for Plane {
    fn drop(&mut self) {
        unsafe {
            rt::drop_plane(self.ispc_geom);
        }
    }
}
