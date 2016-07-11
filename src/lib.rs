#![cfg_attr(feature = "unstable", feature(plugin))]
#![cfg_attr(feature = "unstable", plugin(clippy))]

//! A small library meant to be used as a build dependency with Cargo for easily
//! integrating [ISPC](https://ispc.github.io/) code into Rust projects.
//!
//! # Using ispc-rs
//!
//! You'll want to add a build script to your crate (`build.rs`), tell Cargo about it and add this crate
//! as a build dependency and optionally as a runtime dependency if you plan to use the `ispc_module` macro
//! or ISPC tasks.
//!
//! ```toml
//! # Cargo.toml
//! [package]
//! # ...
//! build = "build.rs"
//!
//! [dependencies]
//! ispc = "0.2.1"
//!
//! [build-dependencies]
//! ispc = "0.2.1"
//! ```
//!
//! Now you can use `ispc` to compile your code into a static library:
//!
//! ```ignore
//! extern crate ispc;
//!
//! fn main() {
//! 	// Compile our ISPC library, this call will exit with EXIT_FAILURE if
//! 	// compilation fails.
//!     ispc::compile_library("simple", &["src/simple.ispc"]);
//! }
//! ```
//!
//! Running `cargo build` should now build your ISPC files into a library and link your Rust
//! application with it. For extra convenience the `ispc_module` macro is provided to import
//! bindings to the library generated with [rust-bindgen](https://github.com/crabtw/rust-bindgen)
//! into a module of the same name. Note that all the functions imported will be unsafe as they're
//! the raw C bindings to your lib.
//!
//! ```ignore
//! #[macro_use]
//! extern crate ispc;
//!
//! // Functions exported from simple will be callable under simple::*
//! ispc_module!(simple);
//! // Alternatively if the module should be public:
//! // ispc_module!(pub simple);
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
//! *Multiple ISPC Files:* Unfortunately with multiple ISPC files when building with debug symbols
//! some of the debug symbols for each compiled object will conflict, resulting in link errors and
//! your program failing to compile. The workaround for this on Windows is to not build the ISPC
//! code with debugging info if you're using multiple ISPC files, see the
//! [multi file examples](https://github.com/Twinklebear/ispc-rs/tree/master/examples/multi_file).

#![allow(dead_code)]

extern crate bindgen;
extern crate gcc;
extern crate libc;
extern crate aligned_alloc;
extern crate num_cpus;

pub mod task;
pub mod exec;

use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{self, Write, BufRead, BufReader};
use std::process::{Command, ExitStatus};
use std::env;
use std::mem;
use std::sync::{Once, ONCE_INIT, Arc};
use std::fmt::Display;
use std::collections::BTreeSet;

use task::ISPCTaskFn;
use exec::{TaskSystem, Parallel};

/// Different math libraries that ISPC can use for computations.
pub enum MathLib {
    /// Use ispc's built-in math functions (the default).
    ISPCDefault,
    /// Use high-performance but lower-accuracy math functions.
    Fast,
    /// Use the Intel(r) SVML math libraries.
    SVML,
    /// Use the system's math library (**may be quite slow**).
    System,
}

impl ToString for MathLib {
    fn to_string(&self) -> String {
        match *self {
            MathLib::ISPCDefault => String::from("--math-lib=default"),
            MathLib::Fast => String::from("--math-lib=fast"),
            MathLib::SVML => String::from("--math-lib=svml"),
            MathLib::System => String::from("--math-lib=system"),
        }
    }
}

/// Select 32 or 64 bit addressing to be used by ISPC. Note: 32-bit
/// addressing calculations are done by default, even on 64 bit target
/// architectures.
pub enum Addressing {
    /// Select 32 bit addressing calculations.
    A32,
    /// Select 64 bit addressing calculations.
    A64,
}

impl ToString for Addressing {
    fn to_string(&self) -> String {
        match *self {
            Addressing::A32 => String::from("--addressing=32"),
            Addressing::A64 => String::from("--addressing=64"),
        }
    }
}

/// ISPC optimization options.
#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub enum OptimizationOpt {
    /// Remove assertion statements from final code.
    DisableAssertions,
    /// Disable 'fused multiply-add' instructions (on targets that support them).
    DisableFMA,
    /// Disable loop unrolling.
    DisableLoopUnroll,
    /// Faster masked vector loads on SSE (may go past end of array).
    FastMaskedVload,
    /// Perform non-IEEE-compliant optimizations of numeric expressions.
    FastMath,
    /// Always issue aligned vector load and store instructions.
    ForceAlignedMemory,
}

/// ISPC target CPU type options, if none is set ISPC will
/// target the machine being compile on.
#[derive(Eq, PartialEq)]
pub enum CPU {
    Generic,
    /// Synonym for Atom target
    Bonnell,
    Core2,
    Penryn,
    /// Synonym for corei7 target
    Nehalem,
    /// Synonym for corei7-avx
    SandyBridge,
    /// Synonym for core-avx-i target
    IvyBridge,
    /// Synonym for core-avx2 target
    Haswell,
    Broadwell,
    Knl,
    /// Synonym for slm target
    Silvermont,
}

impl ToString for CPU {
    fn to_string(&self) -> String {
        match *self {
            CPU::Generic => String::from("--cpu=generic"),
            CPU::Bonnell => String::from("--cpu=bonnell"),
            CPU::Core2 => String::from("--cpu=core2"),
            CPU::Penryn => String::from("--cpu=penryn"),
            CPU::Nehalem => String::from("--cpu=nehalem"),
            CPU::SandyBridge => String::from("--cpu=sandybridge"),
            CPU::IvyBridge => String::from("--cpu=ivybridge"),
            CPU::Haswell => String::from("--cpu=haswell"),
            CPU::Broadwell => String::from("--cpu=broadwell"),
            CPU::Knl => String::from("--cpu=knl"),
            CPU::Silvermont => String::from("--cpu=silvermont"),
        }
    }
}

impl ToString for OptimizationOpt {
    fn to_string(&self) -> String {
        match *self {
            OptimizationOpt::DisableAssertions => String::from("--opt=disable-assertions"),
            OptimizationOpt::DisableFMA => String::from("--opt=disable-fma"),
            OptimizationOpt::DisableLoopUnroll => String::from("--opt=disable-loop-unroll"),
            OptimizationOpt::FastMaskedVload => String::from("--opt=fast-masked-vload"),
            OptimizationOpt::FastMath => String::from("--opt=fast-math"),
            OptimizationOpt::ForceAlignedMemory => String::from("--opt=force-aligned-memory"),
        }
    }
}

/// Convenience macro for generating the module to hold the raw/unsafe ISPC bindings.
///
/// In addition to building the library with ISPC we use rust-bindgen to generate
/// a rust module containing bindings to the functions exported from ISPC. These
/// can be imported by passing the name of your library to the `ispc_module` macro.
///
/// # Example
///
/// ```ignore
/// #[macro_use]
/// extern crate ispc;
///
/// // Functions exported from foo will be callable under foo::*
/// ispc_module!(foo);
/// // Alternatively if the module should be public:
/// // ispc_module!(pub simple);
/// ```
#[macro_export]
macro_rules! ispc_module {
    ($lib:ident) => (
        #[allow(dead_code, non_camel_case_types)]
        mod $lib {
            include!(concat!(env!("OUT_DIR"), "/", stringify!($lib), ".rs"));
        }
    );
    (pub $lib:ident) => (
        #[allow(dead_code, non_camel_case_types)]
        pub mod $lib {
            include!(concat!(env!("OUT_DIR"), "/", stringify!($lib), ".rs"));
        }
    )
}

/// Compile the list of ISPC files into a static library and generate bindings
/// using bindgen. The library name should not contain a lib prefix or a lib
/// extension like '.a' or '.lib', the appropriate prefix and suffix will be
/// added based on the compilation target.
///
/// This function will exit the process with `EXIT_FAILURE` if any stage of
/// compilation or linking fails.
///
/// # Example
/// ```no_run
/// ispc::compile_library("foo", &["src/foo.ispc", "src/bar.ispc"]);
/// ```
pub fn compile_library(lib: &str, files: &[&str]) {
    let mut cfg = Config::new();
    for f in &files[..] {
        cfg.file(*f);
    }
    cfg.compile(lib)
}

/// Handy wrapper around calling exit that will log the message passed first
/// then exit with a failure exit code.
macro_rules! exit_failure {
    ($fmt:expr) => {{
        write!(io::stderr(), $fmt).unwrap();
        std::process::exit(libc::EXIT_FAILURE);
    }};
    ($fmt:expr, $($arg:tt)*) => {{
        write!(io::stderr(), $fmt, $($arg)*).unwrap();
        std::process::exit(libc::EXIT_FAILURE);
    }}
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
    // Additional ISPC compiler options that the user can set
    defines: Vec<(String, Option<String>)>,
    math_lib: MathLib,
    werror: bool,
    addressing: Option<Addressing>,
    optimization_opts: BTreeSet<OptimizationOpt>,
    cpu_target: Option<CPU>,
    force_alignment: Option<u32>,
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
            defines: Vec::new(),
            math_lib: MathLib::ISPCDefault,
            werror: false,
            addressing: None,
            optimization_opts: BTreeSet::new(),
            cpu_target: None,
            force_alignment: None,
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
    /// Add a define to be passed to the ISPC compiler, e.g. `-DFOO`
    /// or `-DBAR=FOO` if a value should also be set.
    pub fn add_define(&mut self, define: &str, value: Option<&str>)  -> &mut Config {
        self.defines.push((define.to_string(), value.map(|s| s.to_string())));
        self
    }
    /// Select the 32 or 64 bit addressing calculations for addressing calculations in ISPC.
    pub fn addressing(&mut self, addressing: Addressing) -> &mut Config {
        self.addressing = Some(addressing);
        self
    }
    /// Set the math library used by ISPC code, defaults to the ISPC math library.
    pub fn math_lib(&mut self, math_lib: MathLib) -> &mut Config {
        self.math_lib = math_lib;
        self
    }
    /// Toggle warnings as errors on/off, defaults to off.
    pub fn werror(&mut self, on: bool) -> &mut Config {
        self.werror = on;
        self
    }
    /// Set an optimization option.
    pub fn optimization_opt(&mut self, opt: OptimizationOpt) -> &mut Config {
        self.optimization_opts.insert(opt);
        self
    }
    /// Set the cpu target. This overrides the default choice of ISPC which
    /// is to target the CPU of the machine we're compiling on.
    pub fn cpu(&mut self, cpu: CPU) -> &mut Config {
        self.cpu_target = Some(cpu);
        self
    }
    /// Force ISPC memory allocations to be aligned to `alignment`.
    pub fn force_alignment(&mut self, alignment: u32) -> &mut Config {
        self.force_alignment = Some(alignment);
        self
    }
    /// Run the compiler, producing the library `lib`. If compilation fails
    /// the process will exit with EXIT_FAILURE to log build errors to the console.
    ///
    /// The library name should not have any prefix or suffix, e.g. instead of
    /// `libexample.a` or `example.lib` simply pass `example`
    pub fn compile(&mut self, lib: &str) {
        let dst = self.get_out_dir();
        let default_args = self.default_args();
        for s in &self.ispc_files[..] {
            let fname = s.file_stem().expect("ISPC source files must be files")
                .to_str().expect("ISPC source file names must be valid UTF-8");
            self.print(&format!("cargo:rerun-if-changed={}", s.display()));

            let ispc_fname = String::from(fname) + "_ispc";
            let object = dst.join(ispc_fname.clone()).with_extension("o");
            let header = dst.join(ispc_fname.clone()).with_extension("h");
            let deps = dst.join(ispc_fname).with_extension("idep");
            let status = Command::new("ispc").args(&default_args[..])
                .arg(s).arg("-o").arg(&object).arg("-h").arg(&header)
                .arg("-MMM").arg(&deps).status().unwrap();

            if !status.success() {
                exit_failure!("Failed to compile ISPC source file {}", s.display());
            }
            self.objects.push(object);
            self.headers.push(header);

            // Go this files dependencies and add them to Cargo's watch list
            let deps_list = File::open(deps)
                .expect(&format!("Failed to open dependencies list for {}", s.display())[..]);
            let reader = BufReader::new(deps_list);
            for d in reader.lines() {
                self.print(&format!("cargo:rerun-if-changed={}", d.unwrap()));
            }
        }
        if !self.assemble(lib).success() {
            exit_failure!("Failed to assemble ISPC objects into library {}", lib);
        }
        // Now generate a header we can give to bindgen and generate bindings
        self.generate_bindgen_header(lib);
        let mut bindings = bindgen::builder();
        bindings.forbid_unknown_types()
            .header(self.bindgen_header.to_str().unwrap())
            .link(lib, bindgen::LinkType::Static);
        let bindgen_file = dst.join(lib).with_extension("rs");
        match bindings.generate() {
            Ok(b) => b.write_to_file(bindgen_file).unwrap(),
            Err(_) => exit_failure!("Failed to generating Rust bindings to {}", lib),
        };
        // Tell cargo where to find the library we just built if we're running
        // in a build script
        self.print(&format!("cargo:rustc-link-search=native={}", dst.display()));
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
        if let Some(ref c) = self.cpu_target {
            ispc_args.push(c.to_string());
            // The ispc compiler crashes if we give -O0 and --cpu=generic,
            // see https://github.com/ispc/ispc/issues/1223
            if *c != CPU::Generic || (*c == CPU::Generic && opt_level != 0) {
                ispc_args.push(String::from("-O") + &opt_level.to_string());
            } else {
                println!("cargo:warning=ispc-rs: Omitting -O0 on CPU::Generic target, ispc bug 1223");
            }
        } else {
            ispc_args.push(String::from("-O") + &opt_level.to_string());
        }

        // If we're on Unix we need position independent code
        if cfg!(unix) {
            ispc_args.push(String::from("--pic"));
        }
        let target = self.get_target();
        if target.starts_with("i686") {
            ispc_args.push(String::from("--arch=x86"));
        } else if target.starts_with("x86_64") {
            ispc_args.push(String::from("--arch=x86-64"));
        }
        for d in &self.defines {
            match d.1 {
                Some(ref v) => ispc_args.push(format!("-D{}={}", d.0, v)),
                None => ispc_args.push(format!("-D{}", d.0)),
            }
        }
        ispc_args.push(self.math_lib.to_string());
        if self.werror {
            ispc_args.push(String::from("--werror"));
        }
        if let Some(ref s) = self.addressing {
            ispc_args.push(s.to_string());
        }
        if let Some(ref f) = self.force_alignment {
            ispc_args.push(String::from("--force-alignment=") + &f.to_string());
        }
        for o in &self.optimization_opts {
            ispc_args.push(o.to_string());
        }
        ispc_args
    }
    /// Returns the user-set output directory if they've set one, otherwise
    /// returns env("OUT_DIR")
    fn get_out_dir(&self) -> PathBuf {
        self.out_dir.clone().unwrap_or_else(|| {
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
    fn print<T: Display>(&self, s: &T) {
        if self.cargo_metadata {
            println!("{}", s);
        }
    }
}

impl Default for Config {
    fn default() -> Config { Config::new() }
}

static mut TASK_SYSTEM: Option<&'static TaskSystem> = None;
static TASK_INIT: Once = ONCE_INIT;

/// If you have implemented your own task system you can provide it for use instead
/// of the default threaded one. This must be done prior to calling ISPC code which
/// spawns tasks otherwise the task system will have already been initialized to
/// `Parallel`, which you can also see as an example for implementing a task system.
///
///
/// Use the function to do any extra initialization for your task system. Note that
/// the task system will be leaked and not destroyed until the program exits and the
/// memory space is cleaned up.
pub fn set_task_system<F: FnOnce() -> Arc<TaskSystem>>(f: F) {
    TASK_INIT.call_once(|| {
        let task_sys = f();
        unsafe {
            let s: *const TaskSystem = mem::transmute(&*task_sys);
            mem::forget(task_sys);
            TASK_SYSTEM = Some(&*s);
        }
    });
}

fn get_task_system() -> &'static TaskSystem {
    // TODO: This is a bit nasty, but I'm not sure on a nicer solution. Maybe something that
    // would let the user register the desired (or default) task system? But if
    // mutable statics can't have destructors we still couldn't have an Arc or Box to something?
    TASK_INIT.call_once(|| {
        unsafe {
            let task_sys = Parallel::new() as Arc<TaskSystem>;
            let s: *const TaskSystem = mem::transmute(&*task_sys);
            mem::forget(task_sys);
            TASK_SYSTEM = Some(&*s);
        }
    });
    unsafe { TASK_SYSTEM.unwrap() }
}

#[allow(non_snake_case)]
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn ISPCAlloc(handle_ptr: *mut *mut libc::c_void, size: libc::int64_t,
                                   align: libc::int32_t) -> *mut libc::c_void {
    get_task_system().alloc(handle_ptr, size, align)
}

#[allow(non_snake_case)]
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn ISPCLaunch(handle_ptr: *mut *mut libc::c_void, f: *mut libc::c_void,
                                    data: *mut libc::c_void, count0: libc::c_int,
                                    count1: libc::c_int, count2: libc::c_int) {
    let task_fn: ISPCTaskFn = mem::transmute(f);
    get_task_system().launch(handle_ptr, task_fn, data, count0 as i32, count1 as i32, count2 as i32);
}

#[allow(non_snake_case)]
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn ISPCSync(handle: *mut libc::c_void){
    get_task_system().sync(handle);
}

