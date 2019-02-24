//! A small library meant to be used as a build dependency with Cargo for easily
//! integrating [ISPC](https://ispc.github.io/) code into Rust projects. The
//! `ispc_compile` crate provides functionality to use the ISPC compiler
//! from your build script to build your ISPC code into a library,
//! and generate Rust bindings to this library. The `ispc_rt` crate is
//! still required at runtime to provide a macro to import the generated
//! bindings, along with the task system and performance instrumentation system.
//!
//! # Requirements for Compiling ISPC Code
//!
//! Both the [ISPC compiler](https://ispc.github.io/) and [libclang](http://clang.llvm.org/)
//! (for [rust-bindgen](https://github.com/crabtw/rust-bindgen)) must be available in your path
//! to compile the ISPC code and generate the bindings. These are not required if using `ispc_rt`
//! to link against a previously compiled library.
//!
//! ## Windows Users
//!
//! You'll need Visual Studio and will have to use the MSVC ABI version of Rust since ISPC
//! and Clang link with MSVC on Windows. For bindgen to find libclang you'll need to copy
//! `libclang.lib` to `clang.lib` and place it in your path.
//!

extern crate bindgen;
extern crate gcc;
extern crate libc;
extern crate regex;
extern crate semver;

pub mod opt;

use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{Write, BufRead, BufReader};
use std::process::{Command, ExitStatus};
use std::fmt::Display;
use std::env;
use std::collections::BTreeSet;

use regex::Regex;
use semver::Version;

pub use opt::{MathLib, Addressing, CPU, OptimizationOpt, TargetISA};

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
/// extern crate ispc_compile;
///
/// ispc_compile::compile_library("foo", &["src/foo.ispc", "src/bar.ispc"]);
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
        eprintln!($fmt);
        std::process::exit(libc::EXIT_FAILURE);
    }};
    ($fmt:expr, $($arg:tt)*) => {{
        eprintln!($fmt, $($arg)*);
        std::process::exit(libc::EXIT_FAILURE);
    }}
}

/// Extra configuration to be passed to ISPC
pub struct Config {
    ispc_version: Version,
    ispc_files: Vec<PathBuf>,
    objects: Vec<PathBuf>,
    headers: Vec<PathBuf>,
    include_paths: Vec<PathBuf>,
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
    addressing: Option<Addressing>,
    optimization_opts: BTreeSet<OptimizationOpt>,
    cpu_target: Option<CPU>,
    force_alignment: Option<u32>,
    no_omit_frame_ptr: bool,
    no_stdlib: bool,
    no_cpp: bool,
    quiet: bool,
    werror: bool,
    woff: bool,
    wno_perf: bool,
    instrument: bool,
    target_isa: Option<Vec<TargetISA>>,
}

impl Config {
    pub fn new() -> Config {
        // Query the ISPC compiler version. This also acts as a check that we can
        // find the ISPC compiler when we need it later.
        let cmd_output = Command::new("ispc").arg("--version").output()
            .expect("Failed to find ISPC compiler in PATH");
        if !cmd_output.status.success() {
            exit_failure!("Failed to get ISPC version, is it in your PATH?");
        }
        let ver_string = String::from_utf8_lossy(&cmd_output.stdout);
        let re = Regex::new(r"Intel\(r\) SPMD Program Compiler \(ispc\), (\d+\.\d+\.\d+)").unwrap();
        let ispc_ver = Version::parse(re.captures_iter(&ver_string).next()
                                      .expect("Failed to parse ISPC version").get(1)
                                      .expect("Failed to parse ISPC version").as_str())
                                      .expect("Failed to parse ISPC version");

        Config {
            ispc_version: ispc_ver,
            ispc_files: Vec::new(),
            objects: Vec::new(),
            headers: Vec::new(),
            include_paths: Vec::new(),
            bindgen_header: PathBuf::new(),
            out_dir: None,
            debug: None,
            opt_level: None,
            target: None,
            cargo_metadata: true,
            defines: Vec::new(),
            math_lib: MathLib::ISPCDefault,
            addressing: None,
            optimization_opts: BTreeSet::new(),
            cpu_target: None,
            force_alignment: None,
            no_omit_frame_ptr: false,
            no_stdlib: false,
            no_cpp: false,
            quiet: false,
            werror: false,
            woff: false,
            wno_perf: false,
            instrument: false,
            target_isa: None,
        }
    }
    /// Add an ISPC file to be compiled
    pub fn file<P: AsRef<Path>>(&mut self, file: P) -> &mut Config {
        self.ispc_files.push(file.as_ref().to_path_buf());
        self
    }
    /// Set the output directory to override the default of `env!("OUT_DIR")`
    pub fn out_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Config {
        self.out_dir = Some(dir.as_ref().to_path_buf());
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
    /// Set an optimization option.
    pub fn optimization_opt(&mut self, opt: OptimizationOpt) -> &mut Config {
        self.optimization_opts.insert(opt);
        self
    }
    /// Set the cpu target. This overrides the default choice of ISPC which
    /// is to target the host CPU.
    pub fn cpu(&mut self, cpu: CPU) -> &mut Config {
        self.cpu_target = Some(cpu);
        self
    }
    /// Force ISPC memory allocations to be aligned to `alignment`.
    pub fn force_alignment(&mut self, alignment: u32) -> &mut Config {
        self.force_alignment = Some(alignment);
        self
    }
    /// Add an extra include path for the ispc compiler to search for files.
    pub fn include_path<P: AsRef<Path>>(&mut self, path: P) -> &mut Config {
        self.include_paths.push(path.as_ref().to_path_buf());
        self
    }
    /// Disable frame pointer omission. It may be useful for profiling to
    /// disable omission.
    pub fn no_omit_frame_pointer(&mut self) -> &mut Config {
        self.no_omit_frame_ptr = true;
        self
    }
    /// Don't make the ispc standard library available.
    pub fn no_stdlib(&mut self) -> &mut Config {
        self.no_stdlib = true;
        self
    }
    /// Don't run the C preprocessor
    pub fn no_cpp(&mut self) -> &mut Config {
        self.no_cpp = true;
        self
    }
    /// Enable suppression of all ispc compiler output.
    pub fn quiet(&mut self) -> &mut Config {
        self.quiet = true;
        self
    }
    /// Enable treating warnings as errors.
    pub fn werror(&mut self) -> &mut Config {
        self.werror = true;
        self
    }
    /// Disable all warnings.
    pub fn woff(&mut self) -> &mut Config {
        self.woff = true;
        self
    }
    /// Don't issue warnings related to performance issues
    pub fn wno_perf(&mut self) -> &mut Config {
        self.wno_perf = true;
        self
    }
    /// Emit instrumentation code for ISPC to gather performance data such
    /// as vector utilization.
    pub fn instrument(&mut self) -> &mut Config {
        let min_ver = Version { major: 1, minor: 9, patch: 1, pre: vec![], build: vec![] };
        if self.ispc_version < min_ver {
            exit_failure!("Error: instrumentation is not supported on ISPC versions \
                          older than 1.9.1 as it generates a non-C compatible header");
        }
        self.instrument = true;
        self
    }
    /// Select the target ISA and vector width. If none is specified ispc will
    /// choose the host CPU ISA and vector width.
    pub fn target_isa(&mut self, target: TargetISA) -> &mut Config {
        self.target_isa = Some(vec![target]);
        self
    }
    /// Select multiple target ISAs and vector widths. If none is specified ispc will
    /// choose the host CPU ISA and vector width.
    /// Note that certain options are not compatible with this use case,
    /// e.g. AVX1.1 will replace AVX1, Host should not be passed (just use the default)
    pub fn target_isas(&mut self, targets: Vec<TargetISA>) -> &mut Config {
        self.target_isa = Some(targets);
        self
    }
    /// Set whether Cargo metadata should be emitted to link to the compiled library
    pub fn cargo_metadata(&mut self, metadata: bool) -> &mut Config {
        self.cargo_metadata = metadata;
        self
    }
    /// The library name should not have any prefix or suffix, e.g. instead of
    /// `libexample.a` or `example.lib` simply pass `example`
    pub fn compile(&mut self, lib: &str) {
        let dst = self.get_out_dir();
        let build_dir = self.get_build_dir();
        let default_args = self.default_args();
        for s in &self.ispc_files[..] {
            let fname = s.file_stem().expect("ISPC source files must be files")
                .to_str().expect("ISPC source file names must be valid UTF-8");
            self.print(&format!("cargo:rerun-if-changed={}", s.display()));

            let ispc_fname = String::from(fname) + "_ispc";
            let object = build_dir.join(ispc_fname.clone()).with_extension("o");
            let header = build_dir.join(ispc_fname.clone()).with_extension("h");
            let deps = build_dir.join(ispc_fname.clone()).with_extension("idep");
            let output = Command::new("ispc").args(&default_args[..])
                .arg(s).arg("-o").arg(&object).arg("-h").arg(&header)
                .arg("-MMM").arg(&deps).output().unwrap();

            if !output.stderr.is_empty() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                for l in stderr.lines() {
                    self.print(&format!("cargo:warning={}", l));
                }
            }
            if !output.status.success() {
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

            // Push on the additional ISA-specific object files if any were generated
            if let Some(ref t) = self.target_isa {
                if t.len() > 1 {
                    for isa in t.iter() {
                        let isa_fname = ispc_fname.clone() + "_" + &isa.lib_suffix();
                        let isa_obj = build_dir.join(isa_fname).with_extension("o");
                        self.objects.push(isa_obj);
                    }
                }
            }
        }
        let libfile = lib.to_owned() + &env::var("HOST").unwrap();
        if !self.assemble(&libfile).success() {
            exit_failure!("Failed to assemble ISPC objects into library {}", lib);
        }
        self.print(&format!("cargo:rustc-link-lib=static={}", libfile));

        // Now generate a header we can give to bindgen and generate bindings
        self.generate_bindgen_header(lib);
        let bindings = bindgen::Builder::default()
            .header(self.bindgen_header.to_str().unwrap());

        let bindgen_file = dst.join(lib).with_extension("rs");

        let generated_bindings = match bindings.generate() {
            Ok(b) => b.to_string(),
            Err(_) => exit_failure!("Failed to generating Rust bindings to {}", lib),
        };
        let mut file = match File::create(bindgen_file) {
            Ok(f) => f,
            Err(e) => exit_failure!("Failed to open bindgen mod file for writing: {}", e),
        };
        file.write_all("#[allow(non_camel_case_types,dead_code,non_upper_case_globals,non_snake_case,improper_ctypes)]\n"
                       .as_bytes()).unwrap();
        file.write_all(format!("pub mod {} {{\n", lib).as_bytes()).unwrap();
        file.write_all(generated_bindings.as_bytes()).unwrap();
        file.write_all(b"}").unwrap();

        self.print(&format!("cargo:rustc-link-search=native={}", dst.display()));
        self.print(&format!("cargo:rustc-env=ISPC_OUT_DIR={}", dst.display()));
    }
    /// Get the ISPC compiler version.
    pub fn ispc_version(&self) -> &Version {
        &self.ispc_version
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
        self.bindgen_header = self.get_build_dir().join(format!("_{}_ispc_bindgen_header.h", lib));
        let mut include_file = File::create(&self.bindgen_header).unwrap();
       
        write!(include_file, "#include <stdint.h>\n").unwrap();
        write!(include_file, "#include <stdbool.h>\n").unwrap();

        for h in &self.headers[..] {
            write!(include_file, "#include \"{}\"\n", h.display()).unwrap();
        }
    }
    /// Build up list of basic args for each target, debug, opt level, etc.
    fn default_args(&self) -> Vec<String> {
        let mut ispc_args = Vec::new();
        let opt_level = self.get_opt_level();
        if self.get_debug() && opt_level == 0 {
            ispc_args.push(String::from("-g"));
        }
        if let Some(ref c) = self.cpu_target {
            ispc_args.push(c.to_string());
            // The ispc compiler crashes if we give -O0 and --cpu=generic,
            // see https://github.com/ispc/ispc/issues/1223
            if *c != CPU::Generic || (*c == CPU::Generic && opt_level != 0) {
                ispc_args.push(String::from("-O") + &opt_level.to_string());
            } else {
                self.print(&format!("cargo:warning=ispc-rs: Omitting -O0 on CPU::Generic target, ispc bug 1223"));
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
        if let Some(ref s) = self.addressing {
            ispc_args.push(s.to_string());
        }
        if let Some(ref f) = self.force_alignment {
            ispc_args.push(String::from("--force-alignment=") + &f.to_string());
        }
        for o in &self.optimization_opts {
            ispc_args.push(o.to_string());
        }
        for p in &self.include_paths {
            ispc_args.push(format!("-I{}", p.display()));
        }
        if self.no_omit_frame_ptr {
            ispc_args.push(String::from("--no-omit-frame-pointer"));
        }
        if self.no_stdlib {
            ispc_args.push(String::from("--nostdlib"));
        }
        if self.no_cpp {
            ispc_args.push(String::from("--nocpp"));
        }
        if self.quiet {
            ispc_args.push(String::from("--quiet"));
        }
        if self.werror {
            ispc_args.push(String::from("--werror"));
        }
        if self.woff {
            ispc_args.push(String::from("--woff"));
        }
        if self.wno_perf {
            ispc_args.push(String::from("--wno-perf"));
        }
        if self.instrument {
            ispc_args.push(String::from("--instrument"));
        }
        if let Some(ref t) = self.target_isa {
            let mut isa_str = String::from("--target=");
            isa_str.push_str(&t[0].to_string());
            for isa in t.iter().skip(1) {
                isa_str.push_str(&format!(",{}", isa.to_string()));
            }
            ispc_args.push(isa_str);
        }
        println!("cargo:warning=args={:?}", ispc_args);
        ispc_args
    }
    /// Returns the user-set output directory if they've set one, otherwise
    /// returns env("OUT_DIR")
    fn get_out_dir(&self) -> PathBuf {
        let p = self.out_dir.clone().unwrap_or_else(|| {
            env::var_os("OUT_DIR").map(PathBuf::from).unwrap()
        });
        if p.is_relative() {
            env::current_dir().unwrap().join(p)
        } else {
            p
        }
    }
    /// Returns the default cargo output dir for build scripts (env("OUT_DIR"))
    fn get_build_dir(&self) -> PathBuf {
        env::var_os("OUT_DIR").map(PathBuf::from).unwrap()
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

