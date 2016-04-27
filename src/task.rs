//! Defines structs for operating on ISPC task groups and getting chunks
//! of a task to be scheduled on to threads

use libc;

use std::cmp;

/// A pointer to an ISPC task function.
///
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

/// A list of all task groups spawned by a function in some launch context
/// These will be sync'd at an explicit `sync` call or function exit
#[derive(Debug)]
pub struct Context {
    /// Task groups launched by this function
    /// TODO: Must be protected by a Reader-Writer lock, though I don't think we'd want to
    /// protect each Group, it'd be an RwLock<Vec<Group>>
    pub tasks: Vec<Group>,
    /// The memory allocated for the various task group's parameters
    /// TODO: Must be protected by a Reader-Writer lock
    pub mem: Vec<*mut libc::c_void>,
    pub id: usize,
    /// TODO: Semaphore or atomic? There's some trickiness here actually since we
    /// can't really say a context is done until we've called sync on it, until
    /// that point new tasks could be launched for it on any thread. Maybe finished
    /// could be a semaphore and we'd have some atomic `syncing` which would be set
    /// when sync is called. Then if Group's are still running they would set the finished
    /// semaphore once the last group has been finished on a context or if all groups
    /// are already done we'd just retire the context immediately.
    pub finished: bool,
}

impl Context {
    /// Create a new list of tasks for some function with id `id`
    pub fn new(id: usize) -> Context {
        Context { tasks: Vec::new(), mem: Vec::new(), id: id, finished: false }
    }
}

/// A group of tasks spawned by a call to `launch` in ISPC
#[derive(Debug)]
pub struct Group {
    /// Current starting index to execute the remaining tasks in this group
    /// TODO: Protect start by a mutex since it will be modified by `get_chunk`
    /// which would get a chunk of tasks to be executed along with a copy of the
    /// total, fcn ptr and data. This would be wrapped in to a struct, `Chunk` which
    /// would expose next() and behave like an iterator to go through the chunk of tasks
    /// and run them. Right now we just schedule tasks like in a nested for loop,
    /// would some tiled scheduling be better?
    pub start: isize,
    /// Total number of tasks scheduled in this group
    pub total: (isize, isize, isize),
    /// Function to run for this task
    pub fcn: ISPCTaskFn,
    /// Data pointer to user params to pass to the function
    pub data: *mut libc::c_void,
    /// Whether all tasks have been completed or not, TODO: should become
    /// an atomic or semaphore
    /// I'm unsure whether an atomic or semaphore would be the better choice here
    /// The TASK_LIST would want to send an alert when new tasks are pushed so in
    /// Sync we could wait on the context to finish?
    pub finished: bool,
}

impl Group {
    /// Create a new task group for execution of the function
    pub fn new(total: (isize, isize, isize), data: *mut libc::c_void, fcn: ISPCTaskFn) -> Group {
        Group { start: 0, total: total, data: data, fcn: fcn, finished: false }
    }
    /// Get a chunk of tasks from the group to run if there are any tasks left to run
    ///
    /// `desired_tasks` specifies the number of tasks we'd like the chunk to contain,
    /// though you may get fewer if there aren't that many tasks left
    pub fn get_chunk(&mut self, desired_tasks: isize) -> Option<Chunk> {
        let end = self.total.0 * self.total.1 * self.total.2;
        if self.start <  end {
            // Give the chunk 4 tasks or whatever remain
            let c = Some(Chunk::new(self, cmp::min(self.start + desired_tasks, end)));
            self.start += desired_tasks;
            c
        } else {
            None
        }
    }
}

/// A chunk of tasks from a Group to be executed
///
/// Executes task in the range [start, end)
#[derive(Debug)]
pub struct Chunk {
    /// The next task to be executed in this chunk
    pub start: isize,
    /// The last task to be executed in this chunk
    pub end: isize,
    /// Total number of tasks scheduled in the group this chunk came from
    pub total: (isize, isize, isize),
    /// Function to run for this task
    pub fcn: ISPCTaskFn,
    /// Data pointer to user params to pass to the function
    pub data: *mut libc::c_void,
}

impl Chunk {
    /// Create a new chunk to execute tasks in the group from [start, end)
    pub fn new(group: &Group, end: isize) -> Chunk {
        Chunk { start: group.start, end: end, total: group.total,
                fcn: group.fcn, data: group.data
        }
    }
    /// Get the global task id for the task index
    pub fn task_indices(&self, id: isize) -> (isize, isize, isize) {
        (id % self.total.0, (id / self.total.0) % self.total.1, id / (self.total.0 * self.total.1))
    }
}

