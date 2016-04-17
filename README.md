# ispc-rs

A small library meant to be used as a build dependency with Cargo for easily
integrating [ISPC](https://ispc.github.io/) code into Rust projects.

## Documentation

Rust doc can be found [here](www.willusher.io/ispc-rs/ispc)

## Compile-time Requirements

Both the [ISPC compiler](https://ispc.github.io/) and [libclang](http://clang.llvm.org/)
(for [rust-bindgen](https://github.com/crabtw/rust-bindgen)) must be available in your path.

*Windows note:* For bindgen to find libclang you'll need to copy `libclang.lib` to `clang.lib` and
place it in your path.

