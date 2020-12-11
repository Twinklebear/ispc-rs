#[macro_use]
extern crate ispc;

ispc_module!(simple_tasks);

fn main() {
    unsafe {
        simple_tasks::run_tasks();
    }
}
