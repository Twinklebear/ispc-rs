extern crate ispc;

fn main() {
    ispc::compile_library("simple_tasks", &["src/simple_tasks.ispc"]);
}


