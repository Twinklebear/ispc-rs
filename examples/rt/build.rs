extern crate ispc;

fn main() {
    let mut cfg = ispc::Config::new();
    let ispc_files = [
        "src/rt.ispc",
        "src/geom.ispc",
        "src/material.ispc",
        "src/lights.ispc",
        "src/mc.ispc",
    ];
    for s in &ispc_files[..] {
        cfg.file(*s);
    }
    cfg.compile("rt");
}
