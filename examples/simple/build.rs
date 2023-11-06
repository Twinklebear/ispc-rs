#[cfg(feature = "ispc")]
extern crate ispc_compile;
extern crate ispc_rt;

#[cfg(feature = "ispc")]
fn link_ispc() {
    use ispc_compile::TargetISA;

    #[cfg(target_arch = "x86_64")]
    let target_isas = [
        TargetISA::SSE2i32x4,
        TargetISA::SSE4i32x4,
        TargetISA::AVX1i32x8,
        TargetISA::AVX2i32x8,
        TargetISA::AVX512KNLi32x16,
        TargetISA::AVX512SKXi32x8,
    ];

    #[cfg(target_arch = "aarch64")]
    let target_isas = vec![TargetISA::Neoni32x4];

    let bindgen_builder = ispc_compile::bindgen::builder().allowlist_function("add_lists");

    // For a portable program we can explicitly compile for each target ISA
    // we want. Then ISPC will pick the correct ISA at runtime to call
    // for the target CPU.
    ispc_compile::Config::new()
        .file("src/simple.ispc")
        .target_isas(target_isas)
        .bindgen_builder(bindgen_builder)
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
