extern crate ispc;

fn main() {
    // We need to use a custom config to explicitly not generate debug info
    // for ISPC code on windows otherwise we get repeated symbol declarations
    let mut cfg = ispc::Config::new();
    if cfg!(windows) {
        cfg.debug(false);
    }
    let ispc_files = vec!["src/vol.ispc"];
    for s in &ispc_files[..] {
        cfg.file(*s);
    }
    cfg.compile("ddvol");
}

