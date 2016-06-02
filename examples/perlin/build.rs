extern crate ispc;

fn main() {
    ispc::compile_library("perlin", &["src/perlin.ispc"]);
}


