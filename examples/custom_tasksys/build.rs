extern crate ispc;

fn main() {
    ispc::compile_library("custom_tasksys", &["src/custom_tasksys.ispc"]);
}
