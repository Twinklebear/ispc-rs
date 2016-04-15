extern crate bindgen;
extern crate gcc;

use std::path::Path;
use std::process::{Command, ExitStatus};
use std::env;

/// Link the ISPC code into a static library on Unix using `ar`
#[cfg(unix)]
fn link_ispc(lib_name: &str, objs: &Vec<String>) -> ExitStatus {
    // TODO: should push into a struct
    let out_dir = env::var("OUT_DIR").unwrap();
    let mut args = Vec::with_capacity(2 + objs.len());
    args.push(String::from("crus"));
    args.push(lib_name.to_string());
    for o in objs {
        args.push(o.clone());
    }
    Command::new("ar").args(&args[..])
        .current_dir(&Path::new(&out_dir))
        .status().unwrap()
}
/// Link the ISPC code into a static library on Windows using `lib.exe`
#[cfg(windows)]
fn link_ispc(lib_name: &str, objs: &Vec<String>) -> ExitStatus {
    let out_dir = env::var("OUT_DIR").unwrap();
    let mut lib_cmd = gcc::windows_registry::find_tool(lib_name, "lib.exe")
        .expect("Failed to find link.exe for MSVC toolchain, aborting");
    let mut args = Vec::with_capacity(1 + objs.len());
    args.push(String::from("/OUT:").push_str(lib_name));
    for o in objs {
        args.push(o);
    }
    lib_cmd.args(&args[..])
        .current_dir(&Path::new(&out_dir))
        .status().unwrap()
}

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
    if !link_ispc("libsay_hello.a", &vec![String::from("say_hello.o")]).success() {
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

