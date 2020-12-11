extern crate ispc;

fn main() {
    let mut cfg = ispc::Config::new();
    // Only re-run the build script if the ISPC files have been changed
    let ispc_files = vec!["src/file_a.ispc", "src/file_b.ispc"];
    for s in &ispc_files[..] {
        cfg.file(*s);
    }
    cfg.compile("multi_file")
}
