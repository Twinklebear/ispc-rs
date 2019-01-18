extern crate ispc;

fn main() {
    // This build script shows how to target a specific of vector ISAs
    // using the target_isas function. We can also compile for all ISAs,
    // in which case ISPC will internally dispatch the function calls to
    // the correct ISA for the host system
    ispc::Config::new()
        .file("src/ao.ispc")
        .target_isas(vec![ispc::TargetISA::SSE2i32x4,
                     ispc::TargetISA::SSE4i32x4,
                     ispc::TargetISA::AVX11i32x8,
                     ispc::TargetISA::AVX2i32x8,
                     ispc::TargetISA::AVX512KNLi32x16,
                     ispc::TargetISA::AVX512SKXi32x16])
        .opt_level(3)
        .optimization_opt(ispc::OptimizationOpt::FastMath)
        .compile("ao");
}


