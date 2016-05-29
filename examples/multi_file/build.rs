extern crate ispc;

fn main() {
    // We need to use a custom config to explicitly not generate debug info
    // for ISPC code on windows otherwise we get repeated symbol declarations
    let mut cfg = ispc::Config::new();
    if cfg!(windows) {
        cfg.debug(false);
    }
    // Only re-run the build script if the ISPC files have been changed
    let ispc_files = vec!["src/file_a.ispc", "src/file_b.ispc"];
    for s in &ispc_files[..] {
        println!("cargo:rerun-if-changed={}", s);
        cfg.file(*s);
    }
    cfg.compile("multi_file")
}


