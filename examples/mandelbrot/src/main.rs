#[macro_use]
extern crate ispc;
extern crate image;

ispc_module!(mandelbrot);

fn main() {
    let x = (-2.0, 1.0);
    let y = (-1.0, 1.0);
    let width = 1080;
    let height = 720;
    let max_iters = 255;
    let mut counts = vec![0; width * height];
    unsafe {
        mandelbrot::mandelbrot(x.0, x.1, y.0, y.1, width as i32, height as i32, max_iters, counts.as_mut_ptr());
        //mandelbrot::mandelbrot_tasks(x.0, x.1, y.0, y.1, width as i32, height as i32,
        //                             max_iters, counts.as_mut_ptr());
    }
    // Convert the image to grey scale u8 format for saving
    let img = counts.iter().map(|x| 255 - *x as u8).collect::<Vec<u8>>();
    match image::save_buffer("mandelbrot.png", &img[..], width as u32, height as u32, image::Gray(8)) {
        Ok(_) => println!("Mandelbrot image saved to mandelbrot.png"),
        Err(e) => panic!("Error saving Mandelbrot image: {}", e),
    };
}

