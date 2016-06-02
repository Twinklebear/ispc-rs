extern crate ispc;

fn main() {
    ispc::compile_library("ao", &["src/ao.ispc"]);
}


