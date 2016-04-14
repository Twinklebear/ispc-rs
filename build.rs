extern crate bindgen;

use std::path::Path;
use std::process::Command;
use std::env;

fn main() {
    println!("cargo:rerun-if-changed=src/say_hello.ispc");

    let out_dir = env::var("OUT_DIR").unwrap();
    let debug = env::var("DEBUG").unwrap();
    let opt_level = env::var("OPT_LEVEL").unwrap();

    let mut ispc_args = vec!["src/say_hello.ispc", "--pic"];
    if debug == "true" {
        ispc_args.push("-g");
    }
    if opt_level == "0" {
        ispc_args.push("-O0");
    } else if opt_level == "1" {
        ispc_args.push("-O1");
    } else if opt_level == "2" {
        ispc_args.push("-O2");
    } else if opt_level == "3" {
        ispc_args.push("-O3");
    }

    // Invoke ISPC to compile our code
    let ispc_status = Command::new("ispc").args(&ispc_args[..])
        .args(&["-o", &format!("{}/say_hello.o", out_dir)])
        .args(&["-h", &format!("{}/say_hello.h", out_dir)])
        .status().unwrap();
    if !ispc_status.success() {
        panic!("ISPC compilation failed");
    }

    // Place our code into a static library we can link against
    let link_status = Command::new("ar").args(&["crus", "libsay_hello.a", "say_hello.o"])
        .current_dir(&Path::new(&out_dir))
        .status().unwrap();
    if !link_status.success() {
        panic!("Linking ISPC code into archive failed");
    }
    println!("cargo:rustc-flags=-L native={}", out_dir);

    // Generate Rust bindings for the header
    let mut bindings = bindgen::builder();
    bindings.forbid_unknown_types()
        .header(format!("{}/say_hello.h", out_dir))
        .link_static("say_hello");
    match bindings.generate() {
        Ok(b) => b.write_to_file(Path::new(&format!("{}/say_hello.rs", out_dir))).unwrap(),
        Err(_) => panic!("Binding generation failed!"),
    };
}

