#![allow(dead_code)]

extern crate bindgen;
extern crate gcc;

use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Write;
use std::process::{Command, ExitStatus};
use std::env;

// Convenience macro for generating the module to hold the raw/unsafe ISPC bindings
// Pulls in the generated bindings for the ispc file 
#[macro_export]
macro_rules! ispc_module {
    ($lib:ident) => (
        #[allow(dead_code, non_camel_case_types)]
        mod $lib {
            include!(concat!(env!("OUT_DIR"), "/", stringify!($lib), ".rs"));
        }
    )
}

/// Use the ISPC compiler to compile all ISPC files into object files on Linux
/// Returns true if all ISPC files compiled, or false immediately after one fails to compile
/// Object files will be written to $OUT_DIR/<file_name>.o and headers to
/// $OUT_DIR/<file_name>.h
pub fn compile_ispc(srcs: &Vec<PathBuf>) -> bool {
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
    let mut headers = Vec::with_capacity(srcs.len());
    for s in srcs {
        let fname = s.file_stem().expect("ISPC source files must be files")
            .to_str().expect("ISPC source file names must be valid UTF-8");

        let status = Command::new("ispc").args(&ispc_args[..])
            .arg(s)
            .args(&["-o", &format!("{}/{}.o", out_dir, fname)])
            .args(&["-h", &format!("{}/{}.h", out_dir, fname)])
            .status().unwrap();
        headers.push(fname);
        if !status.success() {
            return false;
        }
    }
    // Generate a single header that includes everything that we can pass to bindgen
    // TODO: This global header name should be based on the libname to avoid conflicts
    let mut include_file = File::create(out_dir + "/ispc_header.h").unwrap();
    for h in headers {
        write!(include_file, "#include \"{}.h\"\n", h).unwrap();
    }
    true
}
/// Link the ISPC code into a static library on Unix using `ar`
#[cfg(unix)]
pub fn link_ispc(lib_name: &str, objs: &Vec<String>) -> ExitStatus {
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
pub fn link_ispc(lib_name: &str, objs: &Vec<String>) -> ExitStatus {
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
pub fn generate_bindings(lib_name: &str, srcs: &Vec<PathBuf>) -> bool {
    let out_dir = env::var("OUT_DIR").unwrap();
    let mut bindings = bindgen::builder();
    bindings.forbid_unknown_types()
        .header(format!("{}/ispc_header.h", out_dir))
        .link_static(lib_name);
    match bindings.generate() {
        Ok(b) => b.write_to_file(Path::new(&format!("{}/{}.rs", out_dir, lib_name))).unwrap(),
        Err(_) => return false,
    };
    true
}

