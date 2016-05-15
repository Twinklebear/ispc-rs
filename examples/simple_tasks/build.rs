extern crate ispc;

fn main() {
    // Only re-run the build script if the ISPC files have been changed
    let ispc_files = vec!["src/simple_tasks.ispc"];
    for s in &ispc_files[..] {
        println!("cargo:rerun-if-changed={}", s);
    }
    ispc::compile_library("simple_tasks", &ispc_files[..]);
}


