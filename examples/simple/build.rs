extern crate ispc;

fn main() {
    ispc::compile_library("simple", &["src/simple.ispc"]);
}

