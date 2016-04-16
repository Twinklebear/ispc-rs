extern crate ispc;

fn main() {
    // Only re-run the build script if the ISPC files have been changed 
    let ispc_files = vec!["src/say_hello.ispc", "src/add_nums.ispc"];
    for s in &ispc_files[..] {
        println!("cargo:rerun-if-changed={}", s);
    }
    if !ispc::compile_library("say_hello", &ispc_files[..]) {
        panic!("Failed to compile ISPC library say hello");
    }
}

