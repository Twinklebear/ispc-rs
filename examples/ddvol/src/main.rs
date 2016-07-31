#[macro_use]
extern crate ispc;
extern crate image;
extern crate rand;
extern crate num;
extern crate serde_json;
extern crate docopt;
extern crate rustc_serialize;

use std::ptr;
use std::time::Instant;

use rand::Rng;
use docopt::Docopt;

use scene::{RenderParams, Scene};
use fb::Framebuffer;

mod raw;
mod vol;
mod vec3;
mod camera;
mod tfn;
mod scene;
mod fb;

ispc_module!(ddvol);

pub type ISPCHandle = *mut ::std::os::raw::c_void;
/// Create a new null ISPCHandle
pub fn empty_handle() -> ISPCHandle {
    ptr::null::<*mut ::std::os::raw::c_void>() as ISPCHandle
}

const USAGE: &'static str = "
Usage:
  ddvol <scene> [options]
  ddvol (-h | --help)

Options:
  -o OUT                Specify a file to writing the render to, defaults to 'ddvol.png'.
  -i, --isovalue VAL    Set an isovalue to render an implicit isosurface with the volume.
  -h, --help            Show this message.
";

#[derive(RustcDecodable)]
pub struct Args {
    arg_scene: String,
    flag_o: Option<String>,
    flag_i: Option<f32>,
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());
    let mut scene = Scene::load(&args.arg_scene[..]);
    if let Some(val) = args.flag_i {
        scene.volume.set_isovalue(val);
    }
    let mut framebuffer = Framebuffer::new(scene.width, scene.height);
    let mut rng = rand::thread_rng();
    // We need a random seed for each scanline of the image
    let scanline_seeds: Vec<_> = rng.gen_iter::<i32>().take(scene.height).collect();
    unsafe {
        let start = Instant::now();
        ddvol::render(scene.camera.ispc_equiv(), scene.volume.ispc_equiv(), &scene.params as *const RenderParams,
                      scanline_seeds.as_ptr(), scene.width as u32, scene.height as u32,
                      framebuffer.data.as_mut_ptr());
        let elapsed = start.elapsed();
        println!("Rendering took {}s", elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9);
    }
    let out_file = match args.flag_o {
        Some(s) => s.clone(),
        None => String::from("ddvol.png"),
    };
    let srgb_img = framebuffer.srgb8();
    match image::save_buffer(&out_file[..], &srgb_img[..], scene.width as u32, scene.height as u32,
                             image::RGB(8)) {
        Ok(_) => println!("Rendered image saved to {}", out_file),
        Err(e) => panic!("Error saving image: {}", e),
    };
}

