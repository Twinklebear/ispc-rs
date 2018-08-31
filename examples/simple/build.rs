extern crate ispc;

fn main() {
    let mut cfg = ispc::Config::new();
    let ispc_files = vec!["src/simple.ispc"];
    for s in &ispc_files[..] {
        cfg.file(*s);
    }
    // For a portable program we can explicitly compile for each target ISA
    // we want. Then ISPC will pick the correct ISA at runtime to call
    // for the target CPU.
    cfg.target_isas(vec![
                    ispc::opt::TargetISA::SSE2i32x4,
                    ispc::opt::TargetISA::SSE4i32x4,
                    ispc::opt::TargetISA::AVX1i32x8,
                    ispc::opt::TargetISA::AVX2i32x8,
                    ispc::opt::TargetISA::AVX512KNLi32x16]);
    cfg.compile("simple");
}

