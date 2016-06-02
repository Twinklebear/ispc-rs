extern crate ispc;

fn main() {
    ispc::compile_library("rt", &["src/rt.ispc"]);
}

