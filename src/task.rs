//! Defines structs for operating on ISPC task groups and getting chunks
//! of a task to be scheduled on to threads

use libc;

use std::cmp;
use std::iter::Iterator;
use std::sync::{Mutex};
use std::sync::atomic::{AtomicBool, ATOMIC_BOOL_INIT};

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
    pub start: Mutex<i32>,
    /// Total number of tasks scheduled in this group
    pub total: (i32, i32, i32),
    /// Function to run for this task
    pub fcn: ISPCTaskFn,
    /// Data pointer to user params to pass to the function
    pub data: *mut libc::c_void,
    /// Whether all tasks have been completed or not, TODO: should become
    /// an atomic or semaphore
    /// I'm unsure whether an atomic or semaphore would be the better choice here
    /// The TASK_LIST would want to send an alert when new tasks are pushed so in
    /// Sync we could wait on the context to finish?
    pub finished: AtomicBool,
}

impl Group {
    /// Create a new task group for execution of the function
    pub fn new(total: (i32, i32, i32), data: *mut libc::c_void, fcn: ISPCTaskFn) -> Group {
        Group { start: Mutex::new(0), total: total, data: data, fcn: fcn, finished: ATOMIC_BOOL_INIT }
    }
    pub fn chunks(&self, chunk_size: i32) -> GroupChunks {
        GroupChunks { group: self, chunk_size: chunk_size }
    }
    /// Get a chunk of tasks from the group to run if there are any tasks left to run
    ///
    /// `desired_tasks` specifies the number of tasks we'd like the chunk to contain,
    /// though you may get fewer if there aren't that many tasks left. If the chunk
    /// you get is the last chunk to be executed (`chunk.end == total.0 * total.1 * total.2`)
    /// you must mark this group as finished upon completing execution of the chunk
    fn get_chunk(&self, desired_tasks: i32) -> Option<Chunk> {
        let end = self.total.0 * self.total.1 * self.total.2;
        let mut start = self.start.lock().unwrap();
        if *start < end {
            // Give the chunk 4 tasks or whatever remain
            let c = Some(Chunk::new(self, *start, cmp::min(*start + desired_tasks, end)));
            *start += desired_tasks;
            c
        } else {
            None
        }
    }
}

/// An iterator over chunks of tasks to be executed in a Group
pub struct GroupChunks<'a> {
    group: &'a Group,
    chunk_size: i32,
}

impl<'a> Iterator for GroupChunks<'a> {
    type Item = Chunk;

    /// Get the next chunk of tasks to be executed
    fn next(&mut self) -> Option<Chunk> {
        self.group.get_chunk(self.chunk_size)
    }
}

/// A chunk of tasks from a Group to be executed
///
/// Executes task in the range [start, end)
#[derive(Debug)]
pub struct Chunk {
    /// The next task to be executed in this chunk
    start: i32,
    /// The last task to be executed in this chunk
    end: i32,
    /// Total number of tasks scheduled in the group this chunk came from
    total: (i32, i32, i32),
    /// Function to run for this task
    fcn: ISPCTaskFn,
    /// Data pointer to user params to pass to the function
    data: *mut libc::c_void,
}

impl Chunk {
    /// Create a new chunk to execute tasks in the group from [start, end)
    pub fn new(group: &Group, start: i32, end: i32) -> Chunk {
        Chunk { start: start, end: end, total: group.total,
                fcn: group.fcn, data: group.data
        }
    }
    /// Execute all tasks in this chunk
    pub fn execute(&self, thread_id: i32, total_threads: i32) {
        let total_tasks = self.total.0 * self.total.1 * self.total.2;
        for t in self.start..self.end {
            let id = self.task_indices(t);
            (self.fcn)(self.data, thread_id as libc::c_int, total_threads as libc::c_int,
                       t as libc::c_int, total_tasks as libc::c_int,
                       id.0 as libc::c_int, id.1 as libc::c_int, id.2 as libc::c_int,
                       self.total.0 as libc::c_int, self.total.1 as libc::c_int,
                       self.total.2 as libc::c_int);
        }
    }
    /// Check if this is the last chunk in the group
    pub fn is_last(&self) -> bool {
        self.end == self.total.0 * self.total.1 * self.total.2
    }
    /// Get the global task id for the task index
    fn task_indices(&self, id: i32) -> (i32, i32, i32) {
        (id % self.total.0, (id / self.total.0) % self.total.1, id / (self.total.0 * self.total.1))
    }
}

