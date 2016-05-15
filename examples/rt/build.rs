extern crate ispc;

fn main() {
    let ispc_files = vec!["src/rt.ispc"];
    for s in &ispc_files[..] {
        println!("cargo:rerun-if-changed={}", s);
    }
    ispc::compile_library("rt", &ispc_files[..]);
}

