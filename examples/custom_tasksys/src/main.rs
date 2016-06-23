#[macro_use]
extern crate ispc;
extern crate libc;
extern crate aligned_alloc;

use std::mem;
use std::sync::Arc;

use ispc::task::ISPCTaskFn;
use ispc::exec::TaskSystem;

ispc_module!(custom_tasksys);

/// This task system implements a very simple serial execution of tasks
/// where we run them immediately on launch
#[derive(Default)]
struct CustomTaskSys;

impl TaskSystem for CustomTaskSys {
    unsafe fn alloc(&self, handle_ptr: *mut *mut libc::c_void, size: i64, align: i32) -> *mut libc::c_void {
        println!("CustomTaskSys::alloc: size = {}, align = {}", size, align);
        // If the handle ptr is null we need a new container to store the allocations made in
        // this execution context
        let mut ctx =
            if (*handle_ptr).is_null() {
                // Allocate a new vector we can store this context's allocations in
                Box::new(Vec::new())
            } else {
                // Get the vector containing the context's memory allocations and add a new allocation
                mem::transmute(*handle_ptr)
            };
        let buf = aligned_alloc::aligned_alloc(size as usize, align as usize) as *mut libc::c_void;
        ctx.push(buf);
        // Set the handle ptr to our list of allocations that we need to free in ISPCSync so sync
        // will be called and we can find the ptrs to free. We also will release the box at that
        // point by going through `from_raw`.
        *handle_ptr = Box::into_raw(ctx) as *mut libc::c_void;
        buf
    }
    unsafe fn launch(&self, handle_ptr: *mut *mut libc::c_void, f: ISPCTaskFn, data: *mut libc::c_void,
                     count0: i32, count1: i32, count2: i32) {
        println!("CustomTaskSys::launch: counts = [{}, {}, {}]", count0, count1, count2);
        // This task system simply executes the tasks serially in a nested loop
        let total_tasks = count0 * count1 * count2;
        for z in 0..count2 {
            for y in 0..count1 {
                for x in 0..count0 {
                    let task_id = x + y * count0 + z * count0 * count1;
                    // Our thread id is 0 and there's only 1 thread running here
                    (f)(data, 0, 1, task_id, total_tasks, x, y, z, count0, count1, count2);
                }
            }
        }
    }
    unsafe fn sync(&self, handle: *mut libc::c_void) {
        println!("CustomTaskSys::sync");
        let ctx: Box<Vec<*mut libc::c_void>> = Box::from_raw(handle as *mut Vec<*mut libc::c_void>);
        // Free each buffer allocated in `alloc` for this context
        for buf in *ctx {
            aligned_alloc::aligned_free(buf as *mut ());
        }
        // We're done with the context so it can be dropped and free'd automatically for us now
    }
}

fn main() {
    // Tell ispc-rs to use our custom task system **before** calling any ISPC functions which
    // launch tasks
    ispc::set_task_system(|| {
        let t: CustomTaskSys = Default::default();
        Arc::new(t)
    });
    unsafe {
        custom_tasksys::custom_tasksys(4);
    }
}

