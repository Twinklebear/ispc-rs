# ispc-rs

A small library meant to be used as a build dependency with Cargo for easily
integrating [ISPC](https://ispc.github.io/) code into Rust projects.

[![Crates.io](https://img.shields.io/crates/v/ispc.svg)](https://crates.io/crates/ispc)
[![Build Status](https://travis-ci.org/Twinklebear/ispc-rs.svg?branch=master)](https://travis-ci.org/Twinklebear/ispc-rs)

## Documentation

Rust doc can be found [here](http://www.willusher.io/ispc-rs/ispc), ISPC documentation can
be found [here](https://ispc.github.io).

## Using ispc-rs

You'll want to add a build script to your crate (`build.rs`), tell Cargo about it and add this crate
as a build dependency and optionally as a runtime dependency if you plan to use the `ispc_module` macro
or ISPC tasks.

```toml
# Cargo.toml
[package]
# ...
build = "build.rs"

[dependencies]
ispc = "0.3.0"

[build-dependencies]
ispc = "0.3.0"
```

Now you can use `ispc` to compile your code into a static library:

```rust
extern crate ispc;

fn main() {
	// Compile our ISPC library, this call will exit with EXIT_FAILURE if
	// compilation fails
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

Some more complete examples can be found in the [examples/](examples/) folder.

## Compile-time Requirements

Both the [ISPC compiler](https://ispc.github.io/) and [libclang](http://clang.llvm.org/)
(for [rust-bindgen](https://github.com/crabtw/rust-bindgen)) must be available in your path.

### Windows Users

You'll need Visual Studio and will have to use the MSVC ABI version of Rust since ISPC
and Clang link with MSVC on Windows. For bindgen to find libclang you'll need to copy
`libclang.lib` to `clang.lib` and place it in your path.

*Multiple ISPC Files:* Unfortunately with multiple ISPC files when building with debug symbols
some of the debug symbols for each compiled object will conflict, resulting in link errors and
your program failing to compile. The workaround for this on Windows is to not build the ISPC
code with debugging info if you're using multiple ISPC files, see the [multi file example](examples/multi_file).

