use vec3f::Vec3f;
use geom::{Geometry, GeomType};

/// A simple sphere with some radius located at `center`
pub struct Sphere {
    center: Vec3f,
    radius: f32,
    ispc_geom: Geometry,
}

impl Sphere {
    pub fn new(center: Vec3f, radius: f32) -> Sphere {
        // TODO: Want to do something like ospray where we build the ISPC side geometry
        // but would this mean the sphere member must be a *const Geometry? Or could it be
        // a &Geometry? But then what lifetime would we assign it?
        // Having a *const Geometry would make the interop with ISPC easy but a lot of the Sphere
        // operations become unsafe, though they kind of would be either way? We will also need
        // a custom Drop for the Sphere to call the corresponding deletion on the ISPC side,
        // again like what ospray does.
    }
}

