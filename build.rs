extern crate bindgen;
extern crate gcc;

use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::env;

/// Use the ISPC compiler to compile all ISPC files into object files on Linux
/// Returns true if all ISPC files compiled, or false immediately after one fails to compile
/// Object files will be written to $OUT_DIR/<file_name>.o and headers to
/// $OUT_DIR/<file_name>.h
fn compile_ispc(srcs: &Vec<PathBuf>) -> bool {
    // TODO: again should push into a struct that keeps this state
    let out_dir = env::var("OUT_DIR").unwrap();
    let debug = env::var("DEBUG").unwrap();
    let opt_level = env::var("OPT_LEVEL").unwrap();

    let mut ispc_args = Vec::new();
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
    // If we're on Unix we need position independent code
    if cfg!(unix) {
        ispc_args.push("--pic");
    }

    for s in srcs {
        let fname = s.file_name().expect("ISPC source files must be files")
            .to_str().expect("ISPC source file names must be valid UTF-8");

        let status = Command::new("ispc").args(&ispc_args[..])
            .arg(s)
            .args(&["-o", &format!("{}/{}.o", out_dir, fname)])
            .args(&["-h", &format!("{}/{}.h", out_dir, fname)])
            .status().unwrap();
        if !status.success() {
            return false;
        }
    }
    true
}
/// Link the ISPC code into a static library on Unix using `ar`
#[cfg(unix)]
fn link_ispc(lib_name: &str, objs: &Vec<String>) -> ExitStatus {
    // TODO: should push into a struct
    let out_dir = env::var("OUT_DIR").unwrap();
    let mut args = Vec::with_capacity(2 + objs.len());
    args.push(String::from("crus"));
    args.push(String::from("lib") + lib_name + ".a");
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
    let target = env::var("TARGET").unwrap();
    let lib_cmd = gcc::windows_registry::find_tool(&target[..], "lib.exe")
        .expect("Failed to find lib.exe for MSVC toolchain, aborting");
    let mut args = Vec::with_capacity(2 + objs.len());
    println!("Linker command = {:?}", lib_cmd.path());
    args.push(String::from("/OUT:") + lib_name + ".lib");
    for o in objs {
        args.push(o.clone());
    }
    lib_cmd.to_command().args(&args[..])
        .current_dir(&Path::new(&out_dir))
        .status().unwrap()
}
// Generate Rust bindings for each ISPC file we compiled into the library
fn generate_bindings(lib_name: &str, srcs: &Vec<PathBuf>) -> bool {
    let out_dir = env::var("OUT_DIR").unwrap();
    for s in srcs {
        let fname = s.file_name().expect("ISPC source files must be files")
            .to_str().expect("ISPC source file names must be valid UTF-8");
        let mut bindings = bindgen::builder();
        bindings.forbid_unknown_types()
            .header(format!("{}/{}.h", out_dir, fname))
            .link_static(lib_name);
        match bindings.generate() {
            Ok(b) => b.write_to_file(Path::new(&format!("{}/{}.rs", out_dir, fname))).unwrap(),
            Err(_) => return false,
        };
    }
    true
}

fn main() {
    println!("cargo:rerun-if-changed=src/say_hello.ispc");

    let out_dir = env::var("OUT_DIR").unwrap();

    // Invoke ISPC to compile our code
    let ispc_files = vec![PathBuf::from("src/say_hello.ispc")];
    let ispc_status = compile_ispc(&ispc_files);
    if !ispc_status {
        panic!("ISPC compilation failed");
    }

    // Place our code into a static library we can link against
    if !link_ispc("say_hello", &vec![String::from("say_hello.o")]).success() {
        panic!("Linking ISPC code into archive failed");
    }
    println!("cargo:rustc-flags=-L native={}", out_dir);

    // Generate Rust bindings for the header
    generate_bindings("say_hello", &ispc_files);
}

