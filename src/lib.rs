//! A small library meant to be used as a build dependency with Cargo for easily
//! integrating [ISPC](https://ispc.github.io/) code into Rust projects.
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
//! Some more complete examples can be found in the
//! [examples/](https://github.com/Twinklebear/ispc-rs/tree/master/examples) folder.
//!
//! # Compile-time Requirements
//!
//! Both the [ISPC compiler](https://ispc.github.io/) and [libclang](http://clang.llvm.org/)
//! (for [rust-bindgen](https://github.com/crabtw/rust-bindgen)) must be available in your path.
//!
//! ## Windows Users
//!
//! You'll need Visual Studio and will have to use the MSVC ABI version of Rust since ISPC
//! and Clang link with MSVC on Windows. For bindgen to find libclang you'll need to copy
//! `libclang.lib` to `clang.lib` and place it in your path.
//!
//! I've also had issues with multiple definition link errors coming up when compiling multiple
//! ISPC files into a library on MSVC, I haven't figured out the cause yet. On Linux the repeated
//! symbols are defined in each object as well but the linker doesn't seem to mind.

#![allow(dead_code)]

extern crate bindgen;
extern crate gcc;
extern crate libc;
extern crate aligned_alloc;

mod task;

use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Write;
use std::process::{Command, ExitStatus};
use std::env;
use std::mem;

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

static mut TASK_LIST: Option<&'static mut Vec<Box<task::Context>>> = None;
static mut NEXT_TASK_ID: usize = 0;

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn ISPCAlloc(handle_ptr: *mut *mut libc::c_void, size: libc::int64_t,
                                   align: libc::int32_t) -> *mut libc::c_void {
    // TODO: This is a bit nasty, but I'm not sure on a nicer solution. Maybe something that
    // would let the user register the desired (or default) task system? But if
    // mutable statics can't have destructors we still couldn't have an Arc or Box to something?
    // TODO: This init should be done with a `Once`
    if TASK_LIST.is_none() {
        let mut list = Box::new(Vec::new());
        let l: *mut Vec<Box<task::Context>> = &mut *list;
        mem::forget(list);
        TASK_LIST = Some(&mut *l);
    }
    println!("ISPCAlloc, size: {}, align: {}", size, align);
    // If the handle is null this is the first time this function has spawned tasks
    // and we should create a new Context structure in the TASK_LIST for it, otherwise
    // it's the pointer to where we should append the new Group
    let mut tasks = if (*handle_ptr).is_null() {
        println!("handle ptr is null");
        // This is a bit hairy. We allocate the new task context in a box, then
        // unbox it into a raw ptr to get a ptr we can pass back to ISPC through
        // the handle_ptr and then re-box it into our TASK_LIST so it will
        // be free'd properly when we erase it from the vector in ISPCSync
        let t = Box::new(task::Context::new(NEXT_TASK_ID));
        let h = Box::into_raw(t);
        *handle_ptr = mem::transmute(h);
        NEXT_TASK_ID += 1;
        TASK_LIST.as_mut().map(|list| {
            list.push(Box::from_raw(h));
            list.last_mut().unwrap()
        }).unwrap()
    } else {
        println!("handle ptr is not null");
        let handle_task: *mut task::Context = mem::transmute(*handle_ptr);
        TASK_LIST.as_mut().map(|list| {
            list.iter_mut().find(|t| (*handle_task).id == t.id).unwrap()
        }).unwrap()
    };
    // TODO: The README for this lib mentions it may be slow. Maybe use some other allocator?
    tasks.mem.push(aligned_alloc::aligned_alloc(size as usize, align as usize) as *mut libc::c_void);
    println!("tasks.id = {}", tasks.id);
    tasks.mem[tasks.mem.len() - 1]
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn ISPCLaunch(handle_ptr: *mut *mut libc::c_void, f: *mut libc::c_void,
                                    data: *mut libc::c_void, count0: libc::c_int,
                                    count1: libc::c_int, count2: libc::c_int) {
    // Push the tasks being launched on to the list of task groups for this function
    let mut tasks: &mut task::Context = mem::transmute(*handle_ptr);
    // TODO: Launching tasks in parallel
    println!("ISPCLaunch, tasks.id = {}, counts: [{}, {}, {}]", tasks.id, count0, count1, count2);
    let task_fn: task::ISPCTaskFn = mem::transmute(f);
    tasks.tasks.push(task::Group::new((count0 as isize, count1 as isize, count2 as isize), data, task_fn));
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn ISPCSync(handle: *mut libc::c_void){
    // TODO: Sync tasks
    let tasks: &mut task::Context = mem::transmute(handle);
    // Make sure all tasks are done, and execute them if not for this simple
    // serial version. TODO: In the future we'd wait on each Group's semaphore or atomic bool
    // Maybe the waiting thread could help execute tasks as well, otherwise it might be
    // possible to deadlock, where all threads are waiting for some enqueue'd tasks but no
    // threads are available to run them. Just running tasks in our context is not sufficient
    // to prevent deadlock actually, because those tasks could in turn launch & sync and get stuck
    // so if our tasks aren't done and there's none left to run in our context we should start
    // running tasks from other contexts to help out
    println!("ISPCSync, tasks.id = {}", tasks.id);
    for tg in tasks.tasks.iter_mut() {
        loop {
            let chunk = match tg.get_chunk(4) {
                Some(c) => c,
                None => break,
            };
            let total_tasks = chunk.total.0 * chunk.total.1 * chunk.total.2;
            println!("Running chunk {:?}", chunk);
            for t in chunk.start..chunk.end {
                let indices = chunk.task_indices(t);
                (chunk.fcn)(chunk.data, 0, 1, t as libc::c_int, total_tasks as libc::c_int,
                            indices.0 as libc::c_int, indices.1 as libc::c_int, indices.2 as libc::c_int,
                            chunk.total.0 as libc::c_int, chunk.total.1 as libc::c_int,
                            chunk.total.2 as libc::c_int);
            }
        }
        // TODO: Note the free of this must wait until all tasks in the group have been finished
        aligned_alloc::aligned_free(tg.data as *mut ());
    }
    // Now erase this task list from our vector
    TASK_LIST.as_mut().map(|list| {
        let pos = list.iter().position(|t| tasks.id == t.id).unwrap();
        list.remove(pos);
    }).unwrap();
}

