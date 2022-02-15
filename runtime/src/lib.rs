//! A small library meant to be used as a build dependency with Cargo for easily
//! integrating [ISPC](https://ispc.github.io/) code into Rust projects. The
//! `ispc_rt` crate is specifically targetted at linking with a previously
//! compiled ISPC library and generated bindings (built with `ispc_compile`),
//! to allow end users to link ISPC code without needing the ISPC compiler or clang.
//!
//! This crate also includes the various runtime components for the ISPC
//! language, including the parallel task system and performance instrumentation.
//!

#![allow(dead_code)]

extern crate libc;
extern crate aligned_alloc;
extern crate num_cpus;

pub mod task;
pub mod exec;
pub mod instrument;

use std::mem;
use std::env;
use std::sync::{Once, Arc};
use std::ffi::CStr;
use std::path::{Path, PathBuf};

pub use task::ISPCTaskFn;
pub use exec::{TaskSystem, Parallel};
pub use instrument::{Instrument, SimpleInstrument};

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
/// extern crate ispc_rt;
///
/// // Functions exported from foo will be callable under foo::*
/// ispc_module!(foo);
/// ```
#[macro_export]
macro_rules! ispc_module {
    ($lib:ident) => (
        include!(concat!(env!("ISPC_OUT_DIR"), "/", stringify!($lib), ".rs"));
    )
}

/// A `PackagedModule` refers to an ISPC module which was previously
/// built using `ispc_compile`, and is now distributed with
/// the crate.
pub struct PackagedModule {
    path: Option<PathBuf>,
    lib: String,
}

impl PackagedModule {
    /// Create a new `PackagedModule` to link against the previously compiled
    /// library named `lib`. As in `ispc_compile`, the library name should not
    /// have any prefix or suffix. For example, instead of `libexample.a` or
    /// `example.lib`, simple pass `example`
    pub fn new(lib: &str) -> PackagedModule {
        PackagedModule {path: None, lib: lib.to_owned()}
    }
    /// Specify the path to search for the packaged ISPC libraries and bindings
    pub fn lib_path<P: AsRef<Path>>(&mut self, path: P) -> &mut PackagedModule {
        self.path = Some(path.as_ref().to_path_buf());
        self
    }
    /// Link with a previously built ISPC library packaged with the crate
    pub fn link(&self) {
        let path = self.get_lib_path();
        let libfile = self.lib.clone() + &env::var("TARGET").unwrap();
        let bindgen_file = self.lib.clone() + ".rs";
        
        println!("cargo:rustc-link-lib=static={}", libfile);
        println!("cargo:rerun-if-changed={}", path.join(get_lib_filename(&libfile)).display());
        println!("cargo:rerun-if-changed={}", path.join(bindgen_file).display());
        println!("cargo:rustc-link-search=native={}", path.display());
        println!("cargo:rustc-env=ISPC_OUT_DIR={}", path.display());
    }
    /// Returns the user-set output directory if they've set one, otherwise
    /// returns env("OUT_DIR")
    fn get_lib_path(&self) -> PathBuf {
        let p = self.path.clone().unwrap_or_else(|| {
            env::var_os("OUT_DIR").map(PathBuf::from).unwrap()
        });
        if p.is_relative() {
            env::current_dir().unwrap().join(p)
        } else {
            p
        }
    }
}

fn get_lib_filename(libfile: &str) -> String {
    if libfile.contains("windows") {
        format!("{}.lib", libfile)
    } else {
        format!("lib{}.a", libfile)
    }
}

static mut TASK_SYSTEM: Option<&'static dyn TaskSystem> = None;
static TASK_INIT: Once = Once::new();

static mut INSTRUMENT: Option<&'static dyn Instrument> = None;
static INSTRUMENT_INIT: Once = Once::new();

/// If you have implemented your own task system you can provide it for use instead
/// of the default threaded one. This must be done prior to calling ISPC code which
/// spawns tasks otherwise the task system will have already been initialized to
/// `Parallel`, which you can also see as an example for implementing a task system.
///
/// Use the function to do any extra initialization for your task system. Note that
/// the task system will be leaked and not destroyed until the program exits and the
/// memory space is cleaned up.
pub fn set_task_system<F: FnOnce() -> Arc<dyn TaskSystem>>(f: F) {
    TASK_INIT.call_once(|| {
        let task_sys = f();
        unsafe {
            let s = &*task_sys as *const (dyn TaskSystem + 'static);
            mem::forget(task_sys);
            TASK_SYSTEM = Some(&*s);
        }
    });
}

fn get_task_system() -> &'static dyn TaskSystem {
    // TODO: This is a bit nasty, but I'm not sure on a nicer solution. Maybe something that
    // would let the user register the desired (or default) task system? But if
    // mutable statics can't have destructors we still couldn't have an Arc or Box to something?
    TASK_INIT.call_once(|| {
        unsafe {
            let task_sys = Parallel::new() as Arc<dyn TaskSystem>;
            let s = &*task_sys as *const (dyn TaskSystem + 'static);
            mem::forget(task_sys);
            TASK_SYSTEM = Some(&*s);
        }
    });
    unsafe { TASK_SYSTEM.unwrap() }
}

/// If you have implemented your own instrument for logging ISPC performance
/// data you can use this function to provide it for use instead of the
/// default one. This function **must** be called before calling into ISPC code,
/// otherwise the instrumenter will already be set to the default.
pub fn set_instrument<F: FnOnce() -> Arc<dyn Instrument>>(f: F) {
    INSTRUMENT_INIT.call_once(|| {
        let instrument = f();
        unsafe {
            let s = &*instrument as *const (dyn Instrument + 'static);
            mem::forget(instrument);
            INSTRUMENT = Some(&*s);
        }
    });
}

/// Print out a summary of performace data gathered from instrumenting ISPC.
/// Must enable instrumenting to have this record and print data, see
/// `Config::instrument`.
pub fn print_instrumenting_summary() {
    get_instrument().print_summary();
}

fn get_instrument() -> &'static dyn Instrument {
    // TODO: This is a bit nasty, like above
    INSTRUMENT_INIT.call_once(|| {
        unsafe {
            let instrument = Arc::new(SimpleInstrument) as Arc<dyn Instrument>;
            let s = &*instrument as *const (dyn Instrument + 'static);
            mem::forget(instrument);
            INSTRUMENT = Some(&*s);
        }
    });
    unsafe { INSTRUMENT.unwrap() }
}

#[allow(non_snake_case)]
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn ISPCAlloc(handle_ptr: *mut *mut libc::c_void, size: i64,
                                   align: i32) -> *mut libc::c_void {
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

#[allow(non_snake_case)]
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn ISPCInstrument(cfile: *const libc::c_char, cnote: *const libc::c_char,
                                        line: libc::c_int, mask: u64) {

    let file_name = CStr::from_ptr(cfile);
    let note = CStr::from_ptr(cnote);
    let active_count = (mask as u64).count_ones();
    get_instrument().instrument(file_name, note, line as i32, mask as u64, active_count);
}

