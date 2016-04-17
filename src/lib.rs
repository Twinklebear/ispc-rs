//! A small library meant to be used as a build dependency with Cargo for easily
//! integrating [ISPC](https://ispc.github.io/) code into Rust projects.
//!
//! # Documentation
//!
//! Rust doc can be found [here](http://www.willusher.io/ispc-rs/ispc)
//!
//! # Using ispc-rs
//!
//! You'll want to add a build script to your crate (`build.rs`), tell Cargo about it and add this crate
//! as a build dependency.
//!
//! ```toml
//! # Cargo.toml
//! [package]
//! # ...
//! build = "build.rs"
//!
//! [build-dependencies]
//! ispc = "0.0.1"
//! ```
//!
//! Now you can use `ispc` to compile your code into a static library:
//!
//! ```rust
//! extern crate ispc;
//!
//! fn main() {
//!     let ispc_files = vec!["src/simple.ispc"];
//!     // Optional: Only re-run the build script if the ISPC files have been changed
//!     for s in &ispc_files[..] {
//!         println!("cargo:rerun-if-changed={}", s);
//!     }
//! 	// Compile our ISPC library and make sure it went ok
//!     if !ispc::compile_library("simple", &ispc_files[..]) {
//!         panic!("Failed to compile ISPC library 'simple'");
//!     }
//! }
//! ```
//!
//! Running `cargo build` should now build your ISPC files into a library and link your Rust
//! application with it. For extra convenience the `ispc_module` macro is provided to import
//! bindings to the library generated with [rust-bindgen](https://github.com/crabtw/rust-bindgen)
//! into a module of the same name. Note that all the functions imported will be unsafe as they're
//! the raw C bindings to your lib.
//!
//! ```rust
//! #[macro_use]
//! extern crate ispc;
//!
//! // Functions exported from simple will be callable under simple::*
//! ispc_module!(simple);
//! ```
//!
//! Some more complete examples can be found in the [examples/](examples/) folder.
//!
//! # Compile-time Requirements
//!
//! Both the [ISPC compiler](https://ispc.github.io/) and [libclang](http://clang.llvm.org/)
//! (for [rust-bindgen](https://github.com/crabtw/rust-bindgen)) must be available in your path.
//!
//! ## Windows Users
//!
//! You'll need to use the MSVC ABI version of Rust since this is what ISPC and Clang link with
//! on windows. For bindgen to find libclang you'll need to copy `libclang.lib` to `clang.lib` and
//! place it in your path.
//!
//! I've also had issues with multiple definition link errors coming up when compiling multiple
//! ISPC files into a library on MSVC, I haven't figured out the cause yet. On Linux the repeated
//! symbols are defined in each object as well but the linker doesn't seem to mind.

#![allow(dead_code)]

extern crate bindgen;
extern crate gcc;

use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Write;
use std::process::{Command, ExitStatus};
use std::env;

/// Convenience macro for generating the module to hold the raw/unsafe ISPC bindings.
///
/// In addition to building the library with ISPC we use rust-bindgen to generate
/// a rust module containing bindings to the functions exported from ISPC. These
/// can be imported by passing the name of your library to the `ispc_module` macro.
///
/// # Example
///
/// ```rust
/// #[macro_use]
/// extern crate ispc;
///
/// // Functions exported from foo will be callable under foo::*
/// ispc_module!(foo);
/// ```
#[macro_export]
macro_rules! ispc_module {
    ($lib:ident) => (
        #[allow(dead_code, non_camel_case_types)]
        mod $lib {
            include!(concat!(env!("OUT_DIR"), "/", stringify!($lib), ".rs"));
        }
    )
}

/// Compile the list of ISPC files into a static library and generate bindings
/// using bindgen.
///
/// Returns true if compilation and binding generation
/// succeeded, will panic or return false depending on what operations failed.
/// If compilation fails your build script would likely want to panic to show the
/// compilation errors.
///
/// The library name should not contain a lib prefix or a lib extension like
/// '.a' or '.lib', the appropriate prefix and suffix wil be added based on
/// the compilation target.
///
/// # Example
/// ```no_run
/// ispc::compile_library("foo", &["src/foo.ispc", "src/bar.ispc"]);
/// ```
pub fn compile_library(lib: &str, files: &[&str]) -> bool {
    let mut cfg = Config::new();
    for f in &files[..] {
        cfg.file(*f);
    }
    cfg.compile(lib)
}

/// Extra configuration to be passed to ISPC
pub struct Config {
    ispc_files: Vec<PathBuf>,
    objects: Vec<PathBuf>,
    headers: Vec<PathBuf>,
    include_directories: Vec<PathBuf>,
    // We need to generate a single header so we have one header to give bindgen
    bindgen_header: PathBuf,
    // These options are set from the environment if not set by the user
    out_dir: Option<PathBuf>,
    debug: Option<bool>,
    opt_level: Option<u32>,
    target: Option<String>,
    cargo_metadata: bool,
}

impl Config {
    pub fn new() -> Config {
        Config {
            ispc_files: Vec::new(),
            objects: Vec::new(),
            headers: Vec::new(),
            include_directories: Vec::new(),
            bindgen_header: PathBuf::new(),
            out_dir: None,
            debug: None,
            opt_level: None,
            target: None,
            cargo_metadata: true,
        }
    }
    /// Add an ISPC file to be compiled
    pub fn file<P: AsRef<Path>>(&mut self, p: P) -> &mut Config {
        self.ispc_files.push(p.as_ref().to_path_buf());
        self
    }
    /// Set the output directory to override the default of `env!("OUT_DIR")`
    pub fn out_dir<P: AsRef<Path>>(&mut self, p: P) -> &mut Config {
        self.out_dir = Some(p.as_ref().to_path_buf());
        self
    }
    /// Set whether debug symbols should be generated, symbols are generated by
    /// default if `env!("DEBUG") == "true"`
    pub fn debug(&mut self, debug: bool) -> &mut Config {
        self.debug = Some(debug);
        self
    }
    /// Set the optimization level to override the default of `env!("OPT_LEVEL")`
    pub fn opt_level(&mut self, opt_level: u32) -> &mut Config {
        self.opt_level = Some(opt_level);
        self
    }
    /// Set the target triple to compile for, overriding the default of `env!("TARGET")`
    pub fn target(&mut self, target: &str) -> &mut Config {
        self.target = Some(target.to_string());
        self
    }
    /// Set whether Cargo metadata should be emitted to link to the compiled library
    pub fn cargo_metadata(&mut self, metadata: bool) -> &mut Config {
        self.cargo_metadata = metadata;
        self
    }
    /// Run the compiler, producing the library `lib`. Returns false
    /// if compilation fails, in a build script to see ISPC compilation
    /// errors the caller should panic in this case as they'll be logged to stderr
    ///
    /// The library name should not have any prefix or suffix, e.g. instead of
    /// `libexample.a` or `example.lib` simply pass `example`
    pub fn compile(&mut self, lib: &str) -> bool {
        let dst = self.get_out_dir();
        println!("dst = {}", dst.display());
        let default_args = self.default_args();
        for s in &self.ispc_files[..] {
            let fname = s.file_stem().expect("ISPC source files must be files")
                .to_str().expect("ISPC source file names must be valid UTF-8");

            let ispc_fname = String::from(fname) + "_ispc";
            let object = dst.join(ispc_fname.clone()).with_extension("o");
            let header = dst.join(ispc_fname).with_extension("h");
            let status = Command::new("ispc").args(&default_args[..])
                .arg(s).arg("-o").arg(&object).arg("-h").arg(&header)
                .status().unwrap();

            if !status.success() {
                return false;
            }
            self.objects.push(object);
            self.headers.push(header);
        }
        if !self.assemble(lib).success() {
            return false;
        }
        // Now generate a header we can give to bindgen and generate bindings
        self.generate_bindgen_header(lib);
        let mut bindings = bindgen::builder();
        bindings.forbid_unknown_types()
            .header(self.bindgen_header.to_str().unwrap())
            .link_static(lib);
        let bindgen_file = dst.join(lib).with_extension("rs");
        match bindings.generate() {
            Ok(b) => b.write_to_file(bindgen_file).unwrap(),
            Err(_) => return false,
        };
        // Tell cargo where to find the library we just built if we're running
        // in a build script
        self.print(&format!("cargo:rustc-link-search=native={}", dst.display()));
        true
    }
    /// Link the ISPC code into a static library on Unix using `ar`
    #[cfg(unix)]
    fn assemble(&self, lib: &str) -> ExitStatus {
        Command::new("ar").arg("crus")
            .arg(format!("lib{}.a", lib))
            .args(&self.objects[..])
            .current_dir(&self.get_out_dir())
            .status().unwrap()
    }
    /// Link the ISPC code into a static library on Windows using `lib.exe`
    #[cfg(windows)]
    fn assemble(&self, lib: &str) -> ExitStatus {
        let target = self.get_target();
        let mut lib_cmd = gcc::windows_registry::find_tool(&target[..], "lib.exe")
            .expect("Failed to find lib.exe for MSVC toolchain, aborting")
            .to_command();
        lib_cmd.arg(format!("/OUT:{}.lib", lib))
            .args(&self.objects[..])
            .current_dir(&self.get_out_dir())
            .status().unwrap()
    }
    /// Generate a single header that includes all of our ISPC headers which we can
    /// pass to bindgen
    fn generate_bindgen_header(&mut self, lib: &str) {
        self.bindgen_header = self.get_out_dir().join(format!("_{}_ispc_bindgen_header.h", lib));
        let mut include_file = File::create(&self.bindgen_header).unwrap();
        for h in &self.headers[..] {
            write!(include_file, "#include \"{}\"\n", h.display()).unwrap();
        }
    }
    /// Build up list of basic args for each target, debug, opt level, etc.
    fn default_args(&self) -> Vec<String> {
        let mut ispc_args = Vec::new();
        if self.get_debug() {
            ispc_args.push(String::from("-g"));
        }
        let opt_level = self.get_opt_level();
        if opt_level == 0 {
            ispc_args.push(String::from("-O0"));
        } else if opt_level == 1 {
            ispc_args.push(String::from("-O1"));
        } else if opt_level == 2 {
            ispc_args.push(String::from("-O2"));
        } else if opt_level == 3 {
            ispc_args.push(String::from("-O3"));
        }
        // If we're on Unix we need position independent code
        if cfg!(unix) {
            ispc_args.push(String::from("--pic"));
        }
        ispc_args
    }
    /// Returns the user-set output directory if they've set one, otherwise
    /// returns env("OUT_DIR")
    fn get_out_dir(&self) -> PathBuf {
        self.out_dir.clone().unwrap_or_else(|| {
            // TODO: The */out part is incorrectly interpreted as the file name so append a /
            env::var_os("OUT_DIR").map(PathBuf::from).unwrap()
        })
    }
    /// Returns the user-set debug flag if they've set one, otherwise returns
    /// env("DEBUG")
    fn get_debug(&self) -> bool {
        self.debug.unwrap_or_else(|| {
            env::var("DEBUG").map(|x| x == "true").unwrap()
        })
    }
    /// Returns the user-set optimization level if they've set one, otherwise
    /// returns env("OPT_LEVEL")
    fn get_opt_level(&self) -> u32 {
        self.opt_level.unwrap_or_else(|| {
            let opt = env::var("OPT_LEVEL").unwrap();
            opt.parse::<u32>().unwrap()
        })
    }
    /// Returns the user-set target triple if they're set one, otherwise
    /// returns env("TARGET")
    fn get_target(&self) -> String {
        self.target.clone().unwrap_or_else(|| {
            env::var("TARGET").unwrap()
        })
    }
    /// Print out cargo metadata if enabled
    fn print(&self, s: &str) {
        if self.cargo_metadata {
            println!("{}", s);
        }
    }
}

