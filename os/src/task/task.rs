//! Types related to task management

use super::TaskContext;

/// Import the maximum number of system calls
use crate::config::MAX_SYSCALL_NUM;

/// The task control block (TCB) of a task.
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,
    /// The task's initial start time
    pub task_first_start_time: usize,
    /// The number of system calls made by the task
    pub syscall_times: [u32; MAX_SYSCALL_NUM],

}

/// The status of a task
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    /// uninitialized
    UnInit,
    /// ready to run
    Ready,
    /// running
    Running,
    /// exited
    Exited,
}
