//! Defines structs for operating on ISPC task groups and getting chunks
//! of a task to be scheduled on to threads

use libc;

/// The ISPC task function pointer is:
/// ```c
/// void (*TaskFuncPtr)(void *data, int threadIndex, int threadCount,
///                     int taskIndex, int taskCount,
///                     int taskIndex0, int taskIndex1, int taskIndex2,
///                     int taskCount0, int taskCount1, int taskCount2);
/// ```
pub type ISPCTaskFn = extern "C" fn(data: *mut libc::c_void, thread_idx: libc::c_int, thread_cnt: libc::c_int,
                                    task_idx: libc::c_int, task_cnt: libc::c_int, task_idx0: libc::c_int,
                                    task_idx1: libc::c_int, task_idx2: libc::c_int, task_cnt0: libc::c_int,
                                    task_cnt1: libc::c_int, task_cnt2: libc::c_int);

/// A group of tasks spawned by a call to `launch` in ISPC
#[derive(Debug)]
pub struct TaskGroup {
    /// Current starting index to execute the remaining tasks in this group
    pub start: (isize, isize, isize),
    /// Total number of tasks scheduled in this group
    pub total: (isize, isize, isize),
    /// Function to run for this task
    pub fcn: ISPCTaskFn,
    /// Data pointer to user params to pass to the function
    pub data: *mut libc::c_void,
    /// Whether all tasks have been completed or not, TODO: should become
    /// an atomic or semaphore
    pub finished: bool,
}

impl TaskGroup {
    /// Create a new task group for execution of the function
    pub fn new(total: (isize, isize, isize), data: *mut libc::c_void, fcn: ISPCTaskFn) -> TaskGroup {
        TaskGroup { start: (0, 0, 0), total: total, data: data, fcn: fcn, finished: false }
    }
}

/// A list of all task groups spawned by a function
#[derive(Debug)]
pub struct Tasks {
    /// Task groups launched by this function
    pub tasks: Vec<TaskGroup>,
    /// The memory allocated for the various task group's parameters
    pub mem: Vec<*mut libc::c_void>,
    pub id: usize,
}

impl Tasks {
    /// Create a new list of tasks for some function with id `id`
    pub fn new(id: usize) -> Tasks {
        Tasks { tasks: Vec::new(), mem: Vec::new(), id: id }
    }
}

