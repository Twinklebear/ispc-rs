//! Defines structs for operating on ISPC task groups and getting chunks
//! of a task to be scheduled on to threads

use libc;
use aligned_alloc;

use std::cmp;
use std::iter::Iterator;
use std::sync::{Mutex, RwLock, Arc};
use std::sync::atomic::{self, AtomicUsize, AtomicPtr};

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

/// A list of all task groups spawned by a function in some launch context which
/// will be sync'd at an explicit `sync` call or function exit.
///
/// **Note:** A Context is done if and only if `ISPCSync` has been called with
/// its handle and all of its tasks are finished. Until `ISPCSync` is called on the
/// Context's handle more tasks could be launched.
///
/// Additionally, because we're not really able to associate a call to `ISPCAlloc`
/// with a specific Group care must be taken that the Context is not dropped
/// until `ISPCSync` has been called on its handle and all Groups within have
/// completed execution.
#[derive(Debug)]
pub struct Context {
    /// Task groups launched by this function
    /// TODO: Must be protected by a Reader-Writer lock, though I don't think we'd want to
    /// protect each Group, it'd be an RwLock<Vec<Group>>
    /// PROBLEM: If we're accessing this from multiple threads and have other threads
    /// working on the group when we want to push a new group on we'll get stuck until those
    /// tasks finish because they'll have a read lock on the vec to access the Group safely.
    /// I guess an easy fix would be to push groups behind Arcs? But then how would the
    /// Chunk get the Arc?
    tasks: RwLock<Vec<Arc<Group>>>,
    /// The memory allocated for the various task group's parameters
    mem: Mutex<Vec<AtomicPtr<libc::c_void>>>,
    /// A unique identifier for this context
    pub id: usize,
}

impl Context {
    /// Create a new list of tasks for some function with id `id`
    pub fn new(id: usize) -> Context {
        Context { tasks: RwLock::new(Vec::new()), mem: Mutex::new(Vec::new()), id: id }
    }
    /// Add a task group for execution that was launched in this context
    pub fn launch(&self, total: (i32, i32, i32), data: *mut libc::c_void, fcn: ISPCTaskFn) {
        self.tasks.write().unwrap().push(Arc::new(Group::new(total, AtomicPtr::new(data), fcn)));
    }
    /// Check if all tasks currently in the task list are completed
    ///
    /// **Note:** A Context is done if and only if ISPCSync has been called with
    /// its handle and all of its tasks are finished. Until ISPCSync is called on the
    /// Context's handle more tasks could be launched.
    /// TODO: With this design we're essentially requiring the thread waiting on the context
    /// to busy wait since we provide no condition variable to block on.
    pub fn current_tasks_done(&self) -> bool {
        self.tasks.read().unwrap().iter().fold(true, |done, t| done && t.is_finished())
    }
    /// Allocate some memory for this Context's task groups, returns a pointer to the allocated memory.
    pub unsafe fn alloc(&self, size: usize, align: usize) -> *mut libc::c_void {
        // TODO: The README for this lib mentions it may be slow. Maybe use some other allocator?
        let ptr = aligned_alloc::aligned_alloc(size as usize, align as usize) as *mut libc::c_void;
        let mut mem = self.mem.lock().unwrap();
        mem.push(AtomicPtr::new(ptr));
        ptr
    }
    /// An iterator over the **current** groups in the context which have remaining tasks to
    /// run on a thread. If more task groups are added before this iterator has returned
    /// None those will appear as well.
    pub fn iter(&self) -> ContextIter {
        ContextIter { context: self }
    }
    /// Get a Group with tasks remaining to be executed, returns None if there
    /// are no groups left to run in this context.
    ///
    /// Note that you can't assume that the Group you get back is guaranteed
    /// to have tasks remaining since between the time of checking that the
    /// group has outstanding tasks and getting the group back to call `chunks`
    /// those remaining tasks may have been taken by another threaad.
    fn get_active_group(&self) -> Option<Arc<Group>> {
        let tasks = self.tasks.read().unwrap();
        for group in tasks.iter() {
            if group.has_tasks() {
                return Some(group.clone());
            }
        }
        None
    }
}

impl Drop for Context {
    /// Release memory for all the tasks in this context
    ///
    /// **Note:** that because we're not really able to associate a call to ISPCAlloc
    /// with a specific Group care must be taken that the Context is not dropped
    /// until ISPCSync has been called on its handle and all Groups within have
    /// completed execution.
    fn drop(&mut self) {
        let mut mem = self.mem.lock().unwrap();
        for ptr in mem.drain(0..) {
            let m = ptr.load(atomic::Ordering::SeqCst);
            unsafe { aligned_alloc::aligned_free(m as *mut ()); }
        }
    }
}

/// An iterator over the **current** groups in the context which have remaining tasks to
/// run on a thread. If more task groups are added before this iterator has returned
/// None those will appear as well.
pub struct ContextIter<'a> {
    context: &'a Context,
}

impl<'a> Iterator for ContextIter<'a> {
    type Item = Arc<Group>;

    /// Get a Group with tasks remaining to be executed, returns None if there
    /// are no groups left to run in this context.
    ///
    /// Note that you can't assume that the Group you get back is guaranteed
    /// to have tasks remaining since between the time of checking that the
    /// group has outstanding tasks and getting the group back to call `chunks`
    /// those remaining tasks may have been taken by another threaad.
    fn next(&mut self) -> Option<Arc<Group>> {
        self.context.get_active_group()
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
    start: AtomicUsize,
    end: usize,
    /// Total number of tasks scheduled in this group
    pub total: (i32, i32, i32),
    /// Function to run for this task
    pub fcn: ISPCTaskFn,
    /// Data pointer to user params to pass to the function
    pub data: AtomicPtr<libc::c_void>,
    /// Tracks how many chunks we've given out so far to threads
    chunks_launched: AtomicUsize,
    /// Tracks how many of the chunks we gave out are completed. A group is finished
    /// only when all chunks are done and start >= total tasks, call `is_finished` to check.
    ///
    /// I'm unsure whether or semaphore/condvar would be the better choice here
    /// The TASK_LIST would want to send an alert when new tasks are pushed so in
    /// Sync we could wait on the context to finish?
    /// TODO: We can't just have the last chunk executed mark the group as done
    /// because earlier chunks might still be running! We need to mark ourselves
    chunks_finished: AtomicUsize,
}

impl Group {
    /// Create a new task group for execution of the function
    pub fn new(total: (i32, i32, i32), data: AtomicPtr<libc::c_void>, fcn: ISPCTaskFn) -> Group {
        Group { start: AtomicUsize::new(0), end: (total.0 * total.1 * total.2) as usize,
                total: total, data: data, fcn: fcn,
                chunks_launched: AtomicUsize::new(0), chunks_finished: AtomicUsize::new(0) }
    }
    /// Get an iterator over `chunk_size` chunks of tasks to be executed for this group
    pub fn chunks(&self, chunk_size: usize) -> GroupChunks {
        GroupChunks { group: self, chunk_size: chunk_size }
    }
    /// Check if all tasks for this group have been completed
    pub fn is_finished(&self) -> bool {
        let finished = self.chunks_finished.load(atomic::Ordering::SeqCst);
        let launched = self.chunks_launched.load(atomic::Ordering::SeqCst);
        let start = self.start.load(atomic::Ordering::SeqCst);
        // This shouldn't happen, if it does some bad threading voodoo is afoot
        assert!(finished <= launched);
        finished == launched && start >= self.end
    }
    /// Check if this group has tasks left to execute
    fn has_tasks(&self) -> bool {
        let start = self.start.load(atomic::Ordering::SeqCst);
        start < self.end
    }
    /// Get a chunk of tasks from the group to run if there are any tasks left to run
    ///
    /// `desired_tasks` specifies the number of tasks we'd like the chunk to contain,
    /// though you may get fewer if there aren't that many tasks left. If the chunk
    /// you get is the last chunk to be executed (`chunk.end == total.0 * total.1 * total.2`)
    /// you must mark this group as finished upon completing execution of the chunk
    fn get_chunk(&self, desired_tasks: usize) -> Option<Chunk> {
        let start = self.start.fetch_add(desired_tasks, atomic::Ordering::SeqCst);
        if start < self.end {
            // Give the chunk 4 tasks or whatever remain
            let c = Some(Chunk::new(self, start, cmp::min(start + desired_tasks, self.end)));
            self.chunks_launched.fetch_add(1, atomic::Ordering::SeqCst);
            c
        } else {
            None
        }
    }
}

/// An iterator over chunks of tasks to be executed in a Group
pub struct GroupChunks<'a> {
    group: &'a Group,
    chunk_size: usize,
}

impl<'a> Iterator for GroupChunks<'a> {
    type Item = Chunk<'a>;

    /// Get the next chunk of tasks to be executed
    fn next(&mut self) -> Option<Chunk<'a>> {
        self.group.get_chunk(self.chunk_size)
    }
}

/// A chunk of tasks from a Group to be executed
///
/// Executes task in the range [start, end)
#[derive(Debug)]
pub struct Chunk<'a> {
    /// The next task to be executed in this chunk
    start: i32,
    /// The last task to be executed in this chunk
    end: i32,
    /// Total number of tasks scheduled in the group this chunk came from
    total: (i32, i32, i32),
    /// Function to run for this task
    fcn: ISPCTaskFn,
    /// Data pointer to user params to pass to the function
    data: AtomicPtr<libc::c_void>,
    /// The group this chunk is running tasks from
    group: &'a Group,
}

impl<'a> Chunk<'a> {
    /// Create a new chunk to execute tasks in the group from [start, end)
    pub fn new(group: &'a Group, start: usize, end: usize) -> Chunk {
        let d = AtomicPtr::new(group.data.load(atomic::Ordering::SeqCst));
        Chunk { start: start as i32, end: end as i32, total: group.total,
                fcn: group.fcn, data: d, group: group
        }
    }
    /// Execute all tasks in this chunk
    pub fn execute(&self, thread_id: i32, total_threads: i32) {
        let total_tasks = self.total.0 * self.total.1 * self.total.2;
        let data = self.data.load(atomic::Ordering::SeqCst);
        for t in self.start..self.end {
            let id = self.task_indices(t);
            (self.fcn)(data, thread_id as libc::c_int, total_threads as libc::c_int,
                       t as libc::c_int, total_tasks as libc::c_int,
                       id.0 as libc::c_int, id.1 as libc::c_int, id.2 as libc::c_int,
                       self.total.0 as libc::c_int, self.total.1 as libc::c_int,
                       self.total.2 as libc::c_int);
        }
        // Tell the group this chunk is done
        self.group.chunks_finished.fetch_add(1, atomic::Ordering::SeqCst);
    }
    /// Get the global task id for the task index
    fn task_indices(&self, id: i32) -> (i32, i32, i32) {
        (id % self.total.0, (id / self.total.0) % self.total.1, id / (self.total.0 * self.total.1))
    }
}

