#[macro_use]
extern crate ispc;
extern crate image;

ispc_module!(perlin);

fn main() {
    let width = 4.0;
    let height = 4.0;
    let img_width = 512;
    let img_height = 512;
    let mut fimg = vec![0.0; img_width * img_height];
    unsafe {
        perlin::perlin(width, height, img_width as i32, img_width as i32, fimg.as_mut_ptr());
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
    match image::save_buffer("perlin.png", &img[..], img_width as u32, img_height as u32, image::Gray(8)) {
        Ok(_) => println!("Perlin noise saved to perlin.png"),
        Err(e) => panic!("Error saving Perlin noise image: {}", e),
    };
}

