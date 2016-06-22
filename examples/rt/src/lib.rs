//! This is a simple packetized ray tracer example which demonstrates
//! interopability with structs in Rust and ISPC.

#[macro_use]
extern crate ispc;
extern crate image;
extern crate rand;

use std::time::Instant;

use rand::Rng;

ispc_module!(rt);

pub use vec3f::Vec3f;
pub use camera::Camera;
pub use geom::{Sphere, Plane, Geometry};
pub use lights::PointLight;
pub use material::Lambertian;

pub mod vec3f;
pub mod camera;
pub mod geom;
pub mod lights;
pub mod material;

pub fn render() {
    let width = 512;
    let height = 512;
    let camera = Camera::new(Vec3f::new(0.0, 0.0, -3.0), Vec3f::new(0.0, 0.0, 0.0),
                             Vec3f::new(0.0, 1.0, 0.0), 60.0, width, height);
    let white_mat = Lambertian::new(Vec3f::new(0.9, 0.9, 0.9));
    let red_mat = Lambertian::new(Vec3f::new(0.8, 0.1, 0.1));
    let blue_mat = Lambertian::new(Vec3f::new(0.1, 0.2, 0.7));
    let sphere = Sphere::new(Vec3f::new(0.0, 0.0, 0.0), 0.5, red_mat);
    let floor = Plane::new(Vec3f::new(0.0, -0.5, 0.0), Vec3f::new(0.0, 1.0, 0.0), white_mat);
    let back_wall = Plane::new(Vec3f::new(0.0, 0.0, 2.0), Vec3f::new(0.0, -0.6, -1.0), blue_mat);
    let light = PointLight::new(Vec3f::new(0.75, 0.75, -2.0), Vec3f::broadcast(10.0));
    let mut framebuffer = vec![0.0; width * height * 3];
    let mut srgb_img_buf = vec![0u8; width * height * 3];
    let mut rng = rand::thread_rng();
    // We need a random seed for each scanline of the image
    let scanline_seeds: Vec<_> = rng.gen_iter::<i32>().take(height).collect();
    unsafe {
        let geom = vec![sphere.ispc_equiv(), floor.ispc_equiv(), back_wall.ispc_equiv()];
        let start = Instant::now();
        rt::render(&camera as *const Camera, geom.as_ptr(), geom.len() as i32, light.ispc_equiv(),
                   scanline_seeds.as_ptr(), width as i32, height as i32, framebuffer.as_mut_ptr());
        let elapsed = start.elapsed();
        println!("Rendering took {}s", elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9);
        rt::framebuffer_to_srgb(framebuffer.as_ptr(), srgb_img_buf.as_mut_ptr(), width as i32, height as i32);
    }
    match image::save_buffer("rt.png", &srgb_img_buf[..], width as u32, height as u32, image::RGB(8)) {
        Ok(_) => println!("Rendered image saved to rt.png"),
        Err(e) => panic!("Error saving image: {}", e),
    };
}

