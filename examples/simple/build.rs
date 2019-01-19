extern crate ispc_rt;
#[cfg(feature = "ispc")]
extern crate ispc_compile;

#[cfg(feature = "ispc")]
fn link_ispc() {
    use ispc_compile::TargetISA;
    // For a portable program we can explicitly compile for each target ISA
    // we want. Then ISPC will pick the correct ISA at runtime to call
    // for the target CPU.
    ispc_compile::Config::new()
        .file("src/simple.ispc")
        .target_isas(vec![
                     TargetISA::SSE2i32x4,
                     TargetISA::SSE4i32x4,
                     TargetISA::AVX1i32x8,
                     TargetISA::AVX2i32x8,
                     TargetISA::AVX512KNLi32x16])
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

