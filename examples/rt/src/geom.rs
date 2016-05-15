//! Provides the base geometry struct which we use to setup "virtual"
//! function calls on the ISPC side to intersect different geometries

use ::rt::Enum_GEOM_TYPE;

pub type Geometry = ::rt::Struct_Geometry;

pub enum GeomType {
    Sphere,
}

impl Geometry {
    pub fn new(geom_type: GeomType) -> Geometry {
        match geom_type {
            GeomType::Sphere => Geometry { geom_type: Enum_GEOM_TYPE::SPHERE },
        }
    }
}

