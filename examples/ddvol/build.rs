extern crate ispc;

fn main() {
    let mut cfg = ispc::Config::new();
    let ispc_files = vec![
        "src/ddvol.ispc",
        "src/vol.ispc",
        "src/camera.ispc",
        "src/tfn.ispc",
        "src/fb.ispc",
    ];
    for s in &ispc_files[..] {
        cfg.file(*s);
    }
    cfg.compile("ddvol");
}
