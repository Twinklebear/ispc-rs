#[macro_use]
extern crate ispc;

ispc_module!(say_hello);

fn main() {
    unsafe {
        say_hello::say_hello();
        println!("Result from ispc: {}", say_hello::add_nums(1, 2));
    }
    println!("Hello from Rust!");
}

