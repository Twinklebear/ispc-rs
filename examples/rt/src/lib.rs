//! This is a simple packetized ray tracer example which demonstrates
//! interopability with structs in Rust and ISPC.

#[macro_use]
extern crate ispc;
extern crate image;
extern crate rand;

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
    let blue_mat = Lambertian::new(Vec3f::new(0.0, 0.0, 1.0));
    let red_mat = Lambertian::new(Vec3f::new(1.0, 0.0, 0.0));
    let sphere = Sphere::new(Vec3f::new(0.0, 0.0, 0.0), 0.5, red_mat);
    let plane = Plane::new(Vec3f::new(0.0, -0.5, 0.0), Vec3f::new(0.0, 1.0, 0.0), blue_mat);
    let light = PointLight::new(Vec3f::new(0.75, 0.75, -2.0), Vec3f::broadcast(10.0));
    let mut img_buf = vec![0u8; width * height * 3];
    let mut rng = rand::thread_rng();
    unsafe {
        let geom = vec![sphere.ispc_equiv(), plane.ispc_equiv()];
        rt::render(&camera as *const Camera, geom.as_ptr(), geom.len() as i32, light.ispc_equiv(),
                   rng.gen::<i32>(), width as i32, height as i32, img_buf.as_mut_ptr());
    }
    match image::save_buffer("rt.png", &img_buf[..], width as u32, height as u32, image::RGB(8)) {
        Ok(_) => println!("Rendered image saved to rt.png"),
        Err(e) => panic!("Error saving image: {}", e),
    };
}

