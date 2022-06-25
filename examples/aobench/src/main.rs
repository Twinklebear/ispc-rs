#[macro_use]
extern crate ispc;
extern crate image;
extern crate rand;

use rand::distributions::Standard;
use rand::{thread_rng, Rng};

ispc_module!(ao);

fn main() {
    let width = 256;
    let height = 256;
    let n_samples = 16;
    let mut fimg = vec![0.0; width * height];
    // We need a random seed for each scanline of the image
    let scanline_seeds: Vec<i32> = thread_rng().sample_iter(&Standard).take(height).collect();
    unsafe {
        ao::aobench_parallel(
            width as i32,
            height as i32,
            n_samples,
            scanline_seeds.as_ptr(),
            fimg.as_mut_ptr(),
        );
    }
    // Convert the image to grey scale u8 to save
    let img = fimg
        .iter()
        .map(|x| {
            if *x >= 1.0 {
                255
            } else if *x <= 0.0 {
                0
            } else {
                (*x * 255.0) as u8
            }
        })
        .collect::<Vec<u8>>();
    match image::save_buffer(
        "ao.png",
        &img[..],
        width as u32,
        height as u32,
        image::ColorType::L8,
    ) {
        Ok(_) => println!("AO Bench image saved to ao.png"),
        Err(e) => panic!("Error saving AO Bench image: {}", e),
    };
}
