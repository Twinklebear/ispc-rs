extern crate ispc;

fn main() {
    // This build script shows how to target a specific of vector ISAs
    // using the target_isas function. We can also compile for all ISAs,
    // in which case ISPC will internally dispatch the function calls to
    // the correct ISA for the host system
    ispc::Config::new().file("src/ao.ispc").compile("ao");
}
