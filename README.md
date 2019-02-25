# ispc-rs

A small library meant to be used as a build dependency with Cargo for easily
integrating [ISPC](https://ispc.github.io/) code into Rust projects.
ispc-rs is split into two crates: a compile time crate [ispc\_compile](https://crates.io/crates/ispc_compile)
and a runtime crate [ispc\_rt](https://crates.io/crates/ispc_rt). This split allows library authors to
avoid pushing unecessary dependencies onto end users of the library, who do not
plan to modify the ISPC code. The [ispc-rs](https://crates.io/crates/ispc) crate is also provided,
which bundles the compile time and runtime crates into a convenient single
crate, if this separation isn't needed.

[![Crates.io](https://img.shields.io/crates/v/ispc.svg)](https://crates.io/crates/ispc)
[![Build Status](https://travis-ci.org/Twinklebear/ispc-rs.svg?branch=master)](https://travis-ci.org/Twinklebear/ispc-rs)

# Documentation

Rust doc can be found [here](http://www.willusher.io/ispc-rs/ispc), ISPC documentation can
be found [here](https://ispc.github.io).

# Using ispc-rs

With ispc-rs you can compile your ISPC code from your build script to
generate a native library and a Rust module containing bindings to
the exported ISPC functions. ispc-rs will output commands to Cargo to link
the native library, and you can import the Rust bindings into your code using
a provided macro to call into the library. Using ispc-rs in this mode
requires that the ISPC compiler and clang are available when compiling your
crate.

When writing a crate or program which wants to package and use ISPC
code, but not necessarily require these dependencies on the end user's system,
ispc-rs is actually split into two crates: a compile time crate (`ispc_compile`)
and a runtime crate (`ispc_rt`). The `ispc_compile` crate is used to compile
the ISPC code in a build script, generating the native library and Rust bindings.
The `ispc_rt` crate contains lightweight code to include in the build script
which will find and link against the previously compiled native libraries,
and a macro to import the previously generated Rust bindings. The recommended
use case is to include `ispc_compile` as an optional dependency behind a feature
gate. When building with this feature gate the ISPC code will be built, otherwise
the runtime crate will find and use the existing libraries.

# Using ispc-rs as a Single Crate

To use ispc-rs as a single crate, you'll want to add a build script to your
crate (`build.rs`), tell Cargo about it, and add ispc-rs as a build time and
compile time dependency

```toml
# Cargo.toml
[package]
# ...
build = "build.rs"

[dependencies]
ispc = "1.0.5"

[build-dependencies]
ispc = "1.0.5"
```

Now you can use `ispc` to compile your code into a static library:

```rust
extern crate ispc;

fn main() {
    // Compile our ISPC library, this call will exit with EXIT_FAILURE if
    // compilation fails.
    ispc::compile_library("simple", &["src/simple.ispc"]);
}
```

Running `cargo build` should now build your ISPC files into a library and link your Rust
application with it. For extra convenience the `ispc_module` macro is provided to import
bindings to the library generated with [rust-bindgen](https://github.com/crabtw/rust-bindgen)
into a module of the same name. Note that all the functions imported will be unsafe as they're
the raw C bindings to your lib.

```rust
#[macro_use]
extern crate ispc;

// Functions exported from simple will be callable under simple::*
ispc_module!(simple);
```

## Requirements for Compiling ISPC Code

Both the [ISPC compiler](https://ispc.github.io/) and [libclang](http://clang.llvm.org/)
(for [rust-bindgen](https://github.com/crabtw/rust-bindgen)) must be available in your path
to compile the ISPC code and generate the bindings. These are not required if using `ispc_rt`
to link against a previously compiled library.

### Windows Users

You'll need Visual Studio and will have to use the MSVC ABI version of Rust since ISPC
and Clang link with MSVC on Windows. For bindgen to find libclang you'll need to copy
`libclang.lib` to `clang.lib` and place it in your path.


# Using the Separate Compile and Runtime Crates

The process of using the separate crates is similar to that of the single crate;
however, you'll use the individual `ispc_compile` and `ispc_rt` crates, with the
former marked as an optional dependency. This will allow end users to use the
crate and leverage its ISPC code, without needing to re-build the code on their
machine. For this reason, it's also recommended to build your ISPC code for multiple
vector ISAs, to allow for portability across CPU architectures. You'll also need
to package a compiled ISPC library for each host target triple. This can
be done by building your crate with the ispc feature enabled on each target
host system you want to support users of your library on. Note that users
of your crate on a system you haven't provided a binary for can still compile the ISPC
code themselves, by using your crate with the ispc feature enabled.

```toml
# Cargo.toml
[package]
# ...
build = "build.rs"

[dependencies]
ispc_rt = "1.0.2"

[build-dependencies]
ispc_rt = "1.0.2"
ispc_compile = { "1.0.5", optional = true }

[features]
ispc = ["ispc_compile"]
```

In the build script we can now use the `ispc` feature to optionally
compile the ispc code using `ispc_compile`, otherwise we'll link the
previously built code with `ispc_rt`. Here we'll also output the
compiled ISPC libraries and bindings into the src/ directory.

```rust
extern crate ispc_rt;
#[cfg(feature = "ispc")]
extern crate ispc_compile;

#[cfg(feature = "ispc")]
fn link_ispc() {
    use ispc_compile::TargetISA;
    ispc_compile::Config::new()
        .file("src/simple.ispc")
        .target_isas(vec![
            TargetISA::SSE2i32x4,
            TargetISA::SSE4i32x4,
            TargetISA::AVX1i32x8,
            TargetISA::AVX2i32x8,
            TargetISA::AVX512KNLi32x16,
            TargetISA::AVX512SKXi32x16])
        .out_dir("src/")
        .compile("simple");
}

#[cfg(not(feature = "ispc"))]
fn link_ispc() {
    ispc_rt::PackagedModule::new("simple")
        .lib_path("src/")
        .link();
}

fn main() {
    link_ispc();
}
```

Running `cargo build --features ispc` will now build your ISPC files into a library
and generate bindings for your exported ISPC functions. The compiled library and
generated bindings file will be saved under `src/`, to allow packaging with the rest
of the crate. When building with `cargo build`, the previously compiled library
for the host system will be linked against.

Whether building with or without the ispc feature, you can import the generated
bindings into your rust code with the `ispc_module!` macro as before:

```rust
#[macro_use]
extern crate ispc;

// Functions exported from simple will be callable under simple::*
ispc_module!(simple);
```

Some more complete examples can be found in the
[examples/](https://github.com/Twinklebear/ispc-rs/tree/master/examples) folder.
The separate crates example is [here](https://github.com/Twinklebear/ispc-rs/tree/master/examples/simple)

