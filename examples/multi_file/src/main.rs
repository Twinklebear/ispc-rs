#[macro_use]
extern crate ispc;

ispc_module!(multi_file);

fn main() {
    println!("Hello, world!");
    unsafe {
        multi_file::fcn_a();
        multi_file::fcn_b();
    }
}
