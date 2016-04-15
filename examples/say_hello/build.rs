extern crate ispc;

use std::path::PathBuf;
use std::env;

fn main() {
    println!("cargo:rerun-if-changed=src/say_hello.ispc");

    let out_dir = env::var("OUT_DIR").unwrap();

    // Invoke ISPC to compile our code
    let ispc_files = vec![PathBuf::from("src/say_hello.ispc")];
    let ispc_status = ispc::compile_ispc(&ispc_files);
    if !ispc_status {
        panic!("ISPC compilation failed");
    }

    // Place our code into a static library we can link against
    // TODO: This state should be tracked internally, user shouldn't know or care
    // where we put the objs
    let objs = vec![format!("{}/say_hello.o", out_dir).to_string()];
    if !ispc::link_ispc("say_hello", &objs).success() {
        panic!("Linking ISPC code into archive failed");
    }
    println!("cargo:rustc-flags=-L native={}", out_dir);

    // Generate Rust bindings for the header
    ispc::generate_bindings("say_hello", &ispc_files);
}

