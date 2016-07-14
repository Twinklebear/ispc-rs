extern crate ispc;

fn main() {
    let mut cfg = ispc::Config::new();
    cfg.file("src/mandelbrot.ispc");
    cfg.instrument();
    cfg.compile("mandelbrot");
    //ispc::compile_library("mandelbrot", &["src/mandelbrot.ispc"]);
}

