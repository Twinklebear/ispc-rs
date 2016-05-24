use std::f32;
use vec3f::Vec3f;

/// Set a nicer type alias for the exported ISPC struct
pub type Camera = ::rt::Struct_Camera;

impl Camera {
    /// Create a new camera at some orientation in the world
    pub fn new(pos: Vec3f, target: Vec3f, up: Vec3f, fovy: f32, width: usize, height: usize) -> Camera {
        let dir = target - pos;
        let dz = dir.normalized();
        let dx = -dz.cross(&up).normalized();
        let dy = dx.cross(&dz).normalized();
        let dim_y = 2.0 * f32::tan((fovy / 2.0) * f32::consts::PI / 180.0);
        let aspect_ratio = width as f32 / height as f32;
        let dim_x = dim_y * aspect_ratio;
        let screen_du = dx * dim_x;
        let screen_dv = dy * dim_y;
        let dir_top_left = dz - 0.5 * screen_du - 0.5 * screen_dv;
        Camera { pos: pos, dir: dir.normalized(), up: up.normalized(),
                 dir_top_left: dir_top_left, screen_du: screen_du,
                 screen_dv: screen_dv, width: width as i32, height: height as i32 }
    }
}

