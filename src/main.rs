mod ispc {
    mod raw {
        #[link(name="say_hello", kind="static")]
        extern {
            pub fn say_hello();
        }
    }

    pub fn say_hello() {
        unsafe { raw::say_hello(); };
    }
}

fn main() {
    ispc::say_hello();
    println!("Hello from Rust!");
}

