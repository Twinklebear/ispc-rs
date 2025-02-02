//! This module has various option flags and configs we can pass to ISPC,
//! located here for convience and clutter reduction.

/// Different math libraries that ISPC can use for computations.
pub enum MathLib {
    /// Use ispc's built-in math functions (the default).
    ISPCDefault,
    /// Use high-performance but lower-accuracy math functions.
    Fast,
    /// Use the Intel(r) SVML math libraries.
    SVML,
    /// Use the system's math library (**may be quite slow**).
    System,
}

impl std::fmt::Display for MathLib {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            MathLib::ISPCDefault => write!(f, "--math-lib=default"),
            MathLib::Fast => write!(f, "--math-lib=fast"),
            MathLib::SVML => write!(f, "--math-lib=svml"),
            MathLib::System => write!(f, "--math-lib=system"),
        }
    }
}

/// Select the target CPU architecture
pub enum Architecture {
    Arm,
    Aarch64,
    X86,
    X64,
    Xe64,
}

impl std::fmt::Display for Architecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Architecture::Arm => write!(f, "--arch=arm"),
            Architecture::Aarch64 => write!(f, "--arch=aarch64"),
            Architecture::X86 => write!(f, "--arch=x86"),
            Architecture::X64 => write!(f, "--arch=x86_64"),
            Architecture::Xe64 => write!(f, "--arch=xe64"),
        }
    }
}

/// Select 32 or 64 bit addressing to be used by ISPC. Note: 32-bit
/// addressing calculations are done by default, even on 64 bit target
/// architectures.
pub enum Addressing {
    /// Select 32 bit addressing calculations.
    A32,
    /// Select 64 bit addressing calculations.
    A64,
}

impl std::fmt::Display for Addressing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Addressing::A32 => write!(f, "--addressing=32"),
            Addressing::A64 => write!(f, "--addressing=64"),
        }
    }
}

/// ISPC target CPU ISA options. If none is set, ISPC will target the machine being compiled on.
#[derive(Eq, PartialEq)]
pub enum CPU {
    Generic,
    X8664,
    /// Synonym for Atom target
    Bonnell,
    Core2,
    Penryn,
    /// Synonym for corei7 target
    Nehalem,
    /// Synonym for btver2 target
    Ps4,
    /// Synonym for corei7-avx target
    SandyBridge,
    /// Synonym for core-avx-i target
    IvyBridge,
    /// Synonym for core-avx2 target
    Haswell,
    Broadwell,
    Skylake,
    Knl,
    Skx,
    /// icelake-client
    Icl,
    /// Synonym for slm target
    Silvermont,
    /// icelake-server
    Icx,
    /// tigerlake
    Tgl,
    /// alderlake
    Adl,
    /// meteorlake
    Mtl,
    /// sapphirerapids
    Spr,
    /// graniterapids
    Gnr,
    /// arrowlake
    Arl,
    /// lunarlake
    Lnl,
    Znver1,
    /// Synonym: znver2 (also ps5)
    Znver2,
    Znver3,
    CortexA9,
    CortexA15,
    CortexA35,
    CortexA53,
    CortexA57,
    AppleA7,
    AppleA10,
    AppleA11,
    AppleA12,
    AppleA13,
    AppleA14,
}

impl std::fmt::Display for CPU {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            CPU::Generic => write!(f, "--cpu=generic"),
            CPU::X8664 => write!(f, "--cpu=x86-64"),
            CPU::Bonnell => write!(f, "--cpu=bonnell"),
            CPU::Core2 => write!(f, "--cpu=core2"),
            CPU::Penryn => write!(f, "--cpu=penryn"),
            CPU::Nehalem => write!(f, "--cpu=nehalem"),
            CPU::Ps4 => write!(f, "--cpu=ps4"),
            CPU::SandyBridge => write!(f, "--cpu=sandybridge"),
            CPU::IvyBridge => write!(f, "--cpu=ivybridge"),
            CPU::Haswell => write!(f, "--cpu=haswell"),
            CPU::Broadwell => write!(f, "--cpu=broadwell"),
            CPU::Skylake => write!(f, "--cpu=skylake"),
            CPU::Knl => write!(f, "--cpu=knl"),
            CPU::Skx => write!(f, "--cpu=skx"),
            CPU::Icl => write!(f, "--cpu=icl"),
            CPU::Silvermont => write!(f, "--cpu=silvermont"),
            CPU::Icx => write!(f, "--cpu=icx"),
            CPU::Tgl => write!(f, "--cpu=tgl"),
            CPU::Adl => write!(f, "--cpu=adl"),
            CPU::Mtl => write!(f, "--cpu=mtl"),
            CPU::Spr => write!(f, "--cpu=spr"),
            CPU::Gnr => write!(f, "--cpu=gnr"),
            CPU::Arl => write!(f, "--cpu=arl"),
            CPU::Lnl => write!(f, "--cpu=lnl"),
            CPU::Znver1 => write!(f, "--cpu=znver1"),
            CPU::Znver2 => write!(f, "--cpu=znver2"),
            CPU::Znver3 => write!(f, "--cpu=znver3"),
            CPU::CortexA9 => write!(f, "--cpu=cortex-a9"),
            CPU::CortexA15 => write!(f, "--cpu=cortex-a15"),
            CPU::CortexA35 => write!(f, "--cpu=cortex-a35"),
            CPU::CortexA53 => write!(f, "--cpu=cortex-a53"),
            CPU::CortexA57 => write!(f, "--cpu=cortex-a57"),
            CPU::AppleA7 => write!(f, "--cpu=apple-a7"),
            CPU::AppleA10 => write!(f, "--cpu=apple-a10"),
            CPU::AppleA11 => write!(f, "--cpu=apple-a11"),
            CPU::AppleA12 => write!(f, "--cpu=apple-a12"),
            CPU::AppleA13 => write!(f, "--cpu=apple-a13"),
            CPU::AppleA14 => write!(f, "--cpu=apple-a14"),
        }
    }
}

/// ISPC optimization options.
#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub enum OptimizationOpt {
    /// Remove assertion statements from final code.
    DisableAssertions,
    /// Disable 'fused multiply-add' instructions (on targets that support them).
    DisableFMA,
    /// Disable generation of gather instructions on targets that support them.
    DisableGathers,
    /// Disable loop unrolling.
    DisableLoopUnroll,
    /// Disable generation of scatter instructions on targets that support them.
    DisableScatters,
    /// Disable using ZMM registers in favor of YMM on AVX512 targets.
    DisableZmm,
    /// Enable faster masked vector loads on SSE (may access beyond array end).
    FastMaskedVload,
    /// Perform non-IEEE-compliant optimizations of numeric expressions.
    FastMath,
    /// Always issue aligned vector load and store instructions.
    ForceAlignedMemory,
    /// Reset FTZ (Flush-to-Zero) and DAZ (Denormals-Are-Zero) flags on ISPC extern function entrance and restore them on return.
    ResetFTZDaz,
}

impl std::fmt::Display for OptimizationOpt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let option_str = match self {
            OptimizationOpt::DisableAssertions => "disable-assertions",
            OptimizationOpt::DisableFMA => "disable-fma",
            OptimizationOpt::DisableGathers => "disable-gathers",
            OptimizationOpt::DisableLoopUnroll => "disable-loop-unroll",
            OptimizationOpt::DisableScatters => "disable-scatters",
            OptimizationOpt::DisableZmm => "disable-zmm",
            OptimizationOpt::FastMaskedVload => "fast-masked-vload",
            OptimizationOpt::FastMath => "fast-math",
            OptimizationOpt::ForceAlignedMemory => "force-aligned-memory",
            OptimizationOpt::ResetFTZDaz => "reset-ftz-daz",
        };
        write!(f, "--opt={}", option_str)
    }
}

/// Target instruction sets and vector widths available to specialize for. The
/// default if none is set will be the host CPU's ISA and vector width.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TargetISA {
    Host,
    // x86: SSE2
    SSE2i32x4,
    SSE2i32x8,
    // x86: SSE4.1
    SSE41i8x16,
    SSE41i16x8,
    SSE41i32x4,
    SSE41i32x8,
    // x86: SSE4.2
    SSE42i8x16,
    SSE42i16x8,
    SSE42i32x4,
    SSE42i32x8,
    // x86: AVX1
    AVX1i32x4,
    AVX1i32x8,
    AVX1i32x16,
    AVX1i64x4,
    // x86: AVX2
    AVX2i32x8,
    AVX2i32x16,
    AVX2i64x4,
    AVX2i8x32,
    AVX2i16x16,
    AVX2i32x4,
    // x86: AVX2 VNNI
    AVX2VNNIi32x4,
    AVX2VNNIi32x8,
    AVX2VNNIi32x16,
    // x86: AVX512 variants
    AVX512KNLx16,
    // AVX512SKX variants
    AVX512SKXx4,
    AVX512SKXx8,
    AVX512SKXx16,
    AVX512SKXx32,
    AVX512SKXx64,
    // AVX512ICL variants
    AVX512ICLx4,
    AVX512ICLx8,
    AVX512ICLx16,
    AVX512ICLx32,
    AVX512ICLx64,
    // AVX512SPR variants
    AVX512SPRx4,
    AVX512SPRx8,
    AVX512SPRx16,
    AVX512SPRx32,
    AVX512SPRx64,
    // Neon targets
    Neoni8x16,
    Neoni16x8,
    Neoni32x4,
    Neoni32x8,
    // Xe targets
    GEN9x8,
    GEN9x16,
    XELPx8,
    XELPx16,
    XEHPGx8,
    XEHPGx16,
    XEHPCx16,
    XEHPCx32,
}

impl TargetISA {
    /// Returns the library-suffix associated with the target. Adjust these
    /// strings to match your naming conventions.
    pub fn lib_suffix(&self) -> String {
        match *self {
            TargetISA::Host => String::from("host"),
            // SSE2
            TargetISA::SSE2i32x4 | TargetISA::SSE2i32x8 => String::from("sse2"),
            // SSE4.1
            TargetISA::SSE41i8x16
            | TargetISA::SSE41i16x8
            | TargetISA::SSE41i32x4
            | TargetISA::SSE41i32x8 => String::from("sse4.1"),
            // SSE4.2
            TargetISA::SSE42i8x16
            | TargetISA::SSE42i16x8
            | TargetISA::SSE42i32x4
            | TargetISA::SSE42i32x8 => String::from("sse4.2"),
            // AVX1
            TargetISA::AVX1i32x4
            | TargetISA::AVX1i32x8
            | TargetISA::AVX1i32x16
            | TargetISA::AVX1i64x4 => String::from("avx1"),
            // AVX2
            TargetISA::AVX2i32x8
            | TargetISA::AVX2i32x16
            | TargetISA::AVX2i64x4
            | TargetISA::AVX2i8x32
            | TargetISA::AVX2i16x16
            | TargetISA::AVX2i32x4 => String::from("avx2"),
            // AVX2 VNNI
            TargetISA::AVX2VNNIi32x4
            | TargetISA::AVX2VNNIi32x8
            | TargetISA::AVX2VNNIi32x16 => String::from("avx2vnni"),
            // AVX512 variants:
            TargetISA::AVX512KNLx16 => String::from("avx512knl"),
            TargetISA::AVX512SKXx4
            | TargetISA::AVX512SKXx8
            | TargetISA::AVX512SKXx16
            | TargetISA::AVX512SKXx32
            | TargetISA::AVX512SKXx64 => String::from("avx512skx"),
            TargetISA::AVX512ICLx4
            | TargetISA::AVX512ICLx8
            | TargetISA::AVX512ICLx16
            | TargetISA::AVX512ICLx32
            | TargetISA::AVX512ICLx64 => String::from("avx512icl"),
            TargetISA::AVX512SPRx4
            | TargetISA::AVX512SPRx8
            | TargetISA::AVX512SPRx16
            | TargetISA::AVX512SPRx32
            | TargetISA::AVX512SPRx64 => String::from("avx512spr"),
            // Neon targets
            TargetISA::Neoni8x16
            | TargetISA::Neoni16x8
            | TargetISA::Neoni32x4
            | TargetISA::Neoni32x8 => String::from("neon"),
            // Xe targets
            TargetISA::GEN9x8 | TargetISA::GEN9x16 => String::from("gen9"),
            TargetISA::XELPx8 | TargetISA::XELPx16 => String::from("xelp"),
            TargetISA::XEHPGx8 | TargetISA::XEHPGx16 => String::from("xehpg"),
            TargetISA::XEHPCx16 | TargetISA::XEHPCx32 => String::from("xehpc"),
        }
    }
}

impl std::fmt::Display for TargetISA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            TargetISA::Host => write!(f, "host"),
            // SSE2
            TargetISA::SSE2i32x4 => write!(f, "sse2-i32x4"),
            TargetISA::SSE2i32x8 => write!(f, "sse2-i32x8"),
            // SSE4.1
            TargetISA::SSE41i8x16 => write!(f, "sse4.1-i8x16"),
            TargetISA::SSE41i16x8 => write!(f, "sse4.1-i16x8"),
            TargetISA::SSE41i32x4 => write!(f, "sse4.1-i32x4"),
            TargetISA::SSE41i32x8 => write!(f, "sse4.1-i32x8"),
            // SSE4.2
            TargetISA::SSE42i8x16 => write!(f, "sse4.2-i8x16"),
            TargetISA::SSE42i16x8 => write!(f, "sse4.2-i16x8"),
            TargetISA::SSE42i32x4 => write!(f, "sse4.2-i32x4"),
            TargetISA::SSE42i32x8 => write!(f, "sse4.2-i32x8"),
            // AVX1
            TargetISA::AVX1i32x4 => write!(f, "avx1-i32x4"),
            TargetISA::AVX1i32x8 => write!(f, "avx1-i32x8"),
            TargetISA::AVX1i32x16 => write!(f, "avx1-i32x16"),
            TargetISA::AVX1i64x4 => write!(f, "avx1-i64x4"),
            // AVX2
            TargetISA::AVX2i32x8 => write!(f, "avx2-i32x8"),
            TargetISA::AVX2i32x16 => write!(f, "avx2-i32x16"),
            TargetISA::AVX2i64x4 => write!(f, "avx2-i64x4"),
            TargetISA::AVX2i8x32 => write!(f, "avx2-i8x32"),
            TargetISA::AVX2i16x16 => write!(f, "avx2-i16x16"),
            TargetISA::AVX2i32x4 => write!(f, "avx2-i32x4"),
            // AVX2 VNNI
            TargetISA::AVX2VNNIi32x4 => write!(f, "avx2vnni-i32x4"),
            TargetISA::AVX2VNNIi32x8 => write!(f, "avx2vnni-i32x8"),
            TargetISA::AVX2VNNIi32x16 => write!(f, "avx2vnni-i32x16"),
            // AVX512 variants
            TargetISA::AVX512KNLx16 => write!(f, "avx512knl-x16"),
            TargetISA::AVX512SKXx4 => write!(f, "avx512skx-x4"),
            TargetISA::AVX512SKXx8 => write!(f, "avx512skx-x8"),
            TargetISA::AVX512SKXx16 => write!(f, "avx512skx-x16"),
            TargetISA::AVX512SKXx32 => write!(f, "avx512skx-x32"),
            TargetISA::AVX512SKXx64 => write!(f, "avx512skx-x64"),
            TargetISA::AVX512ICLx4 => write!(f, "avx512icl-x4"),
            TargetISA::AVX512ICLx8 => write!(f, "avx512icl-x8"),
            TargetISA::AVX512ICLx16 => write!(f, "avx512icl-x16"),
            TargetISA::AVX512ICLx32 => write!(f, "avx512icl-x32"),
            TargetISA::AVX512ICLx64 => write!(f, "avx512icl-x64"),
            TargetISA::AVX512SPRx4 => write!(f, "avx512spr-x4"),
            TargetISA::AVX512SPRx8 => write!(f, "avx512spr-x8"),
            TargetISA::AVX512SPRx16 => write!(f, "avx512spr-x16"),
            TargetISA::AVX512SPRx32 => write!(f, "avx512spr-x32"),
            TargetISA::AVX512SPRx64 => write!(f, "avx512spr-x64"),
            // Neon targets
            TargetISA::Neoni8x16 => write!(f, "neon-i8x16"),
            TargetISA::Neoni16x8 => write!(f, "neon-i16x8"),
            TargetISA::Neoni32x4 => write!(f, "neon-i32x4"),
            TargetISA::Neoni32x8 => write!(f, "neon-i32x8"),
            // Xe targets
            TargetISA::GEN9x8 => write!(f, "gen9-x8"),
            TargetISA::GEN9x16 => write!(f, "gen9-x16"),
            TargetISA::XELPx8 => write!(f, "xelp-x8"),
            TargetISA::XELPx16 => write!(f, "xelp-x16"),
            TargetISA::XEHPGx8 => write!(f, "xehpg-x8"),
            TargetISA::XEHPGx16 => write!(f, "xehpg-x16"),
            TargetISA::XEHPCx16 => write!(f, "xehpc-x16"),
            TargetISA::XEHPCx32 => write!(f, "xehpc-x32"),
        }
    }
}

/// Target instruction sets and vector widths available to specialize for. The
/// default if none is set will be the host CPU's ISA and vector width.
pub enum TargetOS {
    Windows,
    Ps4,
    Linux,
    Macos,
    Android,
    Ios,
}

impl TargetOS {
    pub fn lib_suffix(&self) -> String {
        match *self {
            TargetOS::Windows => String::from("windows"),
            TargetOS::Ps4 => String::from("ps4"),
            TargetOS::Linux => String::from("linux"),
            TargetOS::Macos => String::from("macos"),
            TargetOS::Android => String::from("android"),
            TargetOS::Ios => String::from("ios"),
        }
    }
}

impl std::fmt::Display for TargetOS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            TargetOS::Windows => write!(f, "--target-os=windows"),
            TargetOS::Ps4 => write!(f, "--target-os=ps4"),
            TargetOS::Linux => write!(f, "--target-os=linux"),
            TargetOS::Macos => write!(f, "--target-os=macos"),
            TargetOS::Android => write!(f, "--target-os=android"),
            TargetOS::Ios => write!(f, "--target-os=ios"),
        }
    }
}
