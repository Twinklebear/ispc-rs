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

impl ToString for MathLib {
    fn to_string(&self) -> String {
        match *self {
            MathLib::ISPCDefault => String::from("--math-lib=default"),
            MathLib::Fast => String::from("--math-lib=fast"),
            MathLib::SVML => String::from("--math-lib=svml"),
            MathLib::System => String::from("--math-lib=system"),
        }
    }
}

/// Select the target CPU architecture
pub enum Architecture {
    Arm,
    Aarch64,
    X86,
    X64
}

impl ToString for Architecture {
    fn to_string(&self) -> String {
        match *self {
            Architecture::Arm => String::from("--arch=arm"),
            Architecture::Aarch64 => String::from("--arch=aarch64"),
            Architecture::X86 => String::from("--arch=x86"),
            Architecture::X64 => String::from("--arch=x86_64"),
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

impl ToString for Addressing {
    fn to_string(&self) -> String {
        match *self {
            Addressing::A32 => String::from("--addressing=32"),
            Addressing::A64 => String::from("--addressing=64"),
        }
    }
}

/// ISPC target CPU ISA options, if none is set ISPC will
/// target the machine being compile on.
#[derive(Eq, PartialEq)]
pub enum CPU {
    Generic,
    /// Synonym for Atom target
    Bonnell,
    Core2,
    Penryn,
    /// Synonym for corei7 target
    Nehalem,
    /// Synonym for btver2
    Ps4,
    /// Synonym for corei7-avx
    SandyBridge,
    /// Synonym for core-avx-i target
    IvyBridge,
    /// Synonym for core-avx2 target
    Haswell,
    Broadwell,
    Knl,
    Skx,
    Icl,
    /// Synonym for slm target
    Silvermont,
    CoretexA15,
    CoretexA9,
    CoretexA35,
    CoretexA53,
    CoretexA57,
}

impl ToString for CPU {
    fn to_string(&self) -> String {
        match *self {
            CPU::Generic => String::from("--cpu=generic"),
            CPU::Bonnell => String::from("--cpu=bonnell"),
            CPU::Core2 => String::from("--cpu=core2"),
            CPU::Penryn => String::from("--cpu=penryn"),
            CPU::Nehalem => String::from("--cpu=nehalem"),
            CPU::Ps4 => String::from("--cpu=ps4"),
            CPU::SandyBridge => String::from("--cpu=sandybridge"),
            CPU::IvyBridge => String::from("--cpu=ivybridge"),
            CPU::Haswell => String::from("--cpu=haswell"),
            CPU::Broadwell => String::from("--cpu=broadwell"),
            CPU::Knl => String::from("--cpu=knl"),
            CPU::Skx => String::from("--cpu=skx"),
            CPU::Icl => String::from("--cpu=icl"),
            CPU::Silvermont => String::from("--cpu=silvermont"),
            CPU::CoretexA15 => String::from("--cpu=cortex-a15"),
            CPU::CoretexA9 => String::from("--cpu=cortex-a9"),
            CPU::CoretexA35 => String::from("--cpu=cortex-a35"),
            CPU::CoretexA53 => String::from("--cpu=cortex-a53"),
            CPU::CoretexA57 => String::from("--cpu=cortex-a57"),
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
    /// Disable loop unrolling.
    DisableLoopUnroll,
    /// Faster masked vector loads on SSE (may go past end of array).
    FastMaskedVload,
    /// Perform non-IEEE-compliant optimizations of numeric expressions.
    FastMath,
    /// Always issue aligned vector load and store instructions.
    ForceAlignedMemory,
    /// Disable using zmm registers in favor of ymm on avx512skx-i32x16 (ISPC 1.13+)
    DisableZmm,
}

impl ToString for OptimizationOpt {
    fn to_string(&self) -> String {
        match *self {
            OptimizationOpt::DisableAssertions => String::from("--opt=disable-assertions"),
            OptimizationOpt::DisableFMA => String::from("--opt=disable-fma"),
            OptimizationOpt::DisableLoopUnroll => String::from("--opt=disable-loop-unroll"),
            OptimizationOpt::FastMaskedVload => String::from("--opt=fast-masked-vload"),
            OptimizationOpt::FastMath => String::from("--opt=fast-math"),
            OptimizationOpt::ForceAlignedMemory => String::from("--opt=force-aligned-memory"),
            OptimizationOpt::DisableZmm => String::from("--opt=disable-zmm"),
        }
    }
}

/// Target instruction sets and vector widths available to specialize for. The
/// default if none is set will be the host CPU's ISA and vector width.
pub enum TargetISA {
    Host,
    SSE2i32x4,
    SSE2i32x8,
    SSE4i32x4,
    SSE4i32x8,
    SSE4i16x8,
    SSE4i8x16,
    AVX1i32x4,
    AVX1i32x8,
    AVX1i32x16,
    AVX1i64x4,
    AVX2i32x8,
    AVX2i32x16,
    AVX2i64x4,
    AVX512KNLi32x16,
    AVX512SKXi32x16,
    AVX512SKXi32x8,
    Neoni8x16,
    Neoni16x8,
    Neoni32x4,
    Neoni32x8,
}

impl TargetISA {
    pub fn lib_suffix(&self) -> String {
        match *self {
            TargetISA::Host => String::from("host"),
            TargetISA::SSE2i32x4 => String::from("sse2"),
            TargetISA::SSE2i32x8 => String::from("sse2"),
            TargetISA::SSE4i32x4 => String::from("sse4"),
            TargetISA::SSE4i32x8 => String::from("sse4"),
            TargetISA::SSE4i16x8 => String::from("sse4"),
            TargetISA::SSE4i8x16 => String::from("sse4"),
            TargetISA::AVX1i32x4 => String::from("avx"),
            TargetISA::AVX1i32x8 => String::from("avx"),
            TargetISA::AVX1i32x16 => String::from("avx"),
            TargetISA::AVX1i64x4 => String::from("avx"),
            TargetISA::AVX2i32x8 => String::from("avx2"),
            TargetISA::AVX2i32x16 => String::from("avx2"),
            TargetISA::AVX2i64x4 => String::from("avx2"),
            TargetISA::AVX512KNLi32x16 => String::from("avx512knl"),
            TargetISA::AVX512SKXi32x16 => String::from("avx512skx"),
            TargetISA::AVX512SKXi32x8 => String::from("avx512skx"),
            TargetISA::Neoni8x16 => String::from("neon"),
            TargetISA::Neoni16x8 => String::from("neon"),
            TargetISA::Neoni32x4 => String::from("neon"),
            TargetISA::Neoni32x8 => String::from("neon"),
        }
    }
}

impl ToString for TargetISA {
    fn to_string(&self) -> String {
        match *self {
            TargetISA::Host => String::from("host"),
            TargetISA::SSE2i32x4 => String::from("sse2-i32x4"),
            TargetISA::SSE2i32x8 => String::from("sse2-i32x8"),
            TargetISA::SSE4i32x4 => String::from("sse4-i32x4"),
            TargetISA::SSE4i32x8 => String::from("sse4-i32x8"),
            TargetISA::SSE4i16x8 => String::from("sse4-i16x8"),
            TargetISA::SSE4i8x16 => String::from("sse4-i8x16"),
            TargetISA::AVX1i32x4 => String::from("avx1-i32x4"),
            TargetISA::AVX1i32x8 => String::from("avx1-i32x8"),
            TargetISA::AVX1i32x16 => String::from("avx1-i32x16"),
            TargetISA::AVX1i64x4 => String::from("avx1-i64x4"),
            TargetISA::AVX2i32x8 => String::from("avx2-i32x8"),
            TargetISA::AVX2i32x16 => String::from("avx2-i32x16"),
            TargetISA::AVX2i64x4 => String::from("avx2-i64x4"),
            TargetISA::AVX512KNLi32x16 => String::from("avx512knl-i32x16"),
            TargetISA::AVX512SKXi32x16 => String::from("avx512skx-i32x16"),
            TargetISA::AVX512SKXi32x8 => String::from("avx512skx-i32x8"),
            TargetISA::Neoni8x16 => String::from("neon-i8x16"),
            TargetISA::Neoni16x8 => String::from("neon-i16x8"),
            TargetISA::Neoni32x4 => String::from("neon-i32x4"),
            TargetISA::Neoni32x8 => String::from("neon-i32x8"),
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

impl ToString for TargetOS {
    fn to_string(&self) -> String {
        match *self {
            TargetOS::Windows => String::from("--target-os=windows"),
            TargetOS::Ps4 => String::from("--target-os=ps4"),
            TargetOS::Linux => String::from("--target-os=linux"),
            TargetOS::Macos => String::from("--target-os=macos"),
            TargetOS::Android => String::from("--target-os=android"),
            TargetOS::Ios => String::from("--target-os=ios"),
        }
    }
}

