//! Defines the trait that must be implemented by ISPC instrumentation callbacks structs
//! and provides a default one.

use std::ffi::CStr;

/// Trait to be implemented to provide ISPC instrumentation functionality.
///
/// The [runtime required function](http://ispc.github.io/perfguide.html#instrumenting-ispc-programs-to-understand-runtime-behavior)
/// is wrapped and forwarded to your struct.
pub trait Instrument {
    /// instrument is called when ISPC calls the `ISPCInstrument` callback. The file
    /// and note strings are converted `CStr` and the number of active programs is
    /// computed from the mask.
    fn instrument(&self, file: &CStr, note: &CStr, line: i32, mask: u64, active_count: u32);
    /// Called through `ispc::print_instrumenting_summary`, optionally log out a summary
    /// of performance information gathered through the `instrument` callback.
    fn print_summary(&self) {}
}

/// A simple ISPC instrumenter which will print the information passed to it out.
pub struct SimpleInstrument;

impl Instrument for SimpleInstrument {
    fn instrument(&self, file: &CStr, note: &CStr, line: i32, mask: u64, active_count: u32) {
        println!(
            "SimpleInstrument:\n\tFile: {}\n\tNote: {}\
                 \n\tLine: {}\n\tActive: {}\nt\tMask: 0x{:x}",
            file.to_str().unwrap(),
            note.to_str().unwrap(),
            line,
            active_count,
            mask
        );
    }
}
