use vec3f::Vec3f;

/// Set a nicer type alias for the exported ISPC struct
pub type Sphere = ::rt::Struct_Sphere;

impl Sphere {
    pub fn new(center: Vec3f, radius: f32) -> Sphere {
        Sphere { center: center, radius: radius }
    }
}

