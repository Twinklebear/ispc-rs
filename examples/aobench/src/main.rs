#[macro_use]
extern crate ispc;
extern crate image;

ispc_module!(ao);

fn main() {
    let width = 256;
    let height = 256;
    let n_samples = 4;
    let mut fimg = vec![0.0; width * height];
    unsafe {
        ao::aobench(width as i32, height as i32, n_samples, fimg.as_mut_ptr());
    }
    // Convert the image to grey scale u8 to save
    let img = fimg.iter().map(|x| {
        if *x >= 1.0 {
            255
        } else if *x <= 0.0 {
            0
        } else {
            (*x * 255.0) as u8
        }
    }).collect::<Vec<u8>>();
    match image::save_buffer("ao.png", &img[..], width as u32, height as u32, image::Gray(8)) {
        Ok(_) => println!("AO Bench image saved to ao.png"),
        Err(e) => panic!("Error saving AO Bench image: {}", e),
    };
}

