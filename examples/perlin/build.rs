extern crate ispc;

fn main() {
    let ispc_files = vec!["src/perlin.ispc"];
    for s in &ispc_files[..] {
        println!("cargo:rerun-if-changed={}", s);
    }
    ispc::compile_library("perlin", &ispc_files[..]);
}


