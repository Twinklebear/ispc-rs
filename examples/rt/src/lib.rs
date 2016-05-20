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
pub use sphere::Sphere;
pub use lights::PointLight;

/// Type alias for the Geometry base struct in ISPC
pub type Geometry = ::rt::Struct_Geometry;

pub mod vec3f;
pub mod camera;
pub mod sphere;
pub mod lights;

pub fn render() {
    let width = 512;
    let height = 512;
    // TODO: X is flipped
    let camera = Camera::new(Vec3f::new(0.0, 0.0, 1.0), Vec3f::new(0.0, 0.0, -1.0),
                             Vec3f::new(0.0, 1.0, 0.0), 65.0, width, height);
    let sphere = Sphere::new(Vec3f::new(0.0, 0.0, 0.0), 0.5);
    let light = PointLight::new(Vec3f::new(1.0, 0.0, 1.0), Vec3f::broadcast(2.0));
    let mut img_buf = vec![0.0; width * height * 3];
    let mut rng = rand::thread_rng();
    unsafe {
        rt::render(&camera as *const Camera, sphere.ispc_equiv(), light.ispc_equiv(),
                   rng.gen::<i32>(), width as i32, height as i32, img_buf.as_mut_ptr());
    }
    // Convert the image to RGB u8 to save
    let img = img_buf.iter().map(|x| {
        if *x >= 1.0 {
            255
        } else if *x <= 0.0 {
            0
        } else {
            (*x * 255.0) as u8
        }
    }).collect::<Vec<u8>>();
    match image::save_buffer("rt.png", &img[..], width as u32, height as u32, image::RGB(8)) {
        Ok(_) => println!("Rendered image saved to rt.png"),
        Err(e) => panic!("Error saving image: {}", e),
    };
}

