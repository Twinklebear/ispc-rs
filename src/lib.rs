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
//! ispc = "0.3.8"
//!
//! [build-dependencies]
//! ispc = "0.3.8"
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

#![allow(dead_code)]

extern crate ispc_compile;
extern crate ispc_rt;

pub use ispc_compile::*;
pub use ispc_rt::*;

#[macro_export]
macro_rules! ispc_module {
    ($lib:ident) => (
        include!(concat!(env!("ISPC_OUT_DIR"), "/", stringify!($lib), ".rs"));
    )
}

