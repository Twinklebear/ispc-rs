extern crate ispc;

fn main() {
    ispc::compile_library("mandelbrot", &["src/mandelbrot.ispc"]);
}
