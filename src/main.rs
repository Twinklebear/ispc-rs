
#[allow(dead_code, non_camel_case_types)]
mod ispc {
    include!(concat!(env!("OUT_DIR"), "/say_hello.rs"));
}

fn main() {
    unsafe {
        ispc::say_hello();
        println!("Result from ispc: {}", ispc::add_nums(1, 2));
    }
    println!("Hello from Rust!");
}

