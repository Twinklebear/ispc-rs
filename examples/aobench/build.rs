extern crate ispc;

fn main() {
    let ispc_files = vec!["src/ao.ispc"];
    for s in &ispc_files[..] {
        println!("cargo:rerun-if-changed={}", s);
    }
    if !ispc::compile_library("ao", &ispc_files[..]) {
        panic!("Failed to compile ISPC library 'ao'");
    }
}


