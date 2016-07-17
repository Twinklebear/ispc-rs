#[macro_use]
extern crate ispc;
extern crate image;
extern crate rand;
extern crate num;

use std::ptr;
use std::time::Instant;
use std::path::Path;

use rand::Rng;

use camera::Camera;
use vec3::{Vec3f, Vec3i};

mod raw;
mod vol;
mod vec3;
mod camera;
mod tfn;

ispc_module!(ddvol);

type RenderParams = ddvol::RenderParams;
pub type ISPCHandle = *mut ::std::os::raw::c_void;
/// Create a new null ISPCHandle
pub fn empty_handle() -> ISPCHandle {
    ptr::null::<*mut ::std::os::raw::c_void>() as ISPCHandle
}

fn main() {
    let width = 512;
    let height = 512;
    let camera = Camera::new(Vec3f::new(-0.5, 0.5, 1.5), Vec3f::new(0.5, 0.5, 0.5),
                             Vec3f::new(0.0, 1.0, 0.0), 60.0, width as i32, height as i32);
    let path = Path::new("./csafe-heptane-302-volume/csafe-heptane-302-volume.raw");
    let params = RenderParams { background: Vec3f::new(0.431, 0.384, 0.349), n_samples: 4 };
    let volume = raw::import::<u8>(path, Vec3i::broadcast(302));
    let mut framebuffer = vec![0.0; width * height * 3];
    let mut srgb_img_buf = vec![0u8; width * height * 3];
    let mut rng = rand::thread_rng();
    // We need a random seed for each scanline of the image
    let scanline_seeds: Vec<_> = rng.gen_iter::<i32>().take(height).collect();
    unsafe {
        let start = Instant::now();
        ddvol::render(camera.ispc_equiv(), volume.ispc_equiv(), &params as *const RenderParams,
                      scanline_seeds.as_ptr(), width as i32, height as i32, framebuffer.as_mut_ptr());
        let elapsed = start.elapsed();
        println!("Rendering took {}s", elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9);
        ddvol::framebuffer_to_srgb(framebuffer.as_ptr(), srgb_img_buf.as_mut_ptr(), width as i32, height as i32);
    }
    match image::save_buffer("ddvol.png", &srgb_img_buf[..], width as u32, height as u32, image::RGB(8)) {
        Ok(_) => println!("Rendered image saved to ddvol.png"),
        Err(e) => panic!("Error saving image: {}", e),
    };
}

