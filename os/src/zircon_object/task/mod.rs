//! Objects for Task Management.

use super::*;

mod exception;
mod job;
mod job_policy;
mod process;
mod suspend_token;
mod thread;

pub use {
    self::exception::*, self::job::*, self::job_policy::*, self::process::*,
    self::suspend_token::*, self::thread::*,
    alloc::sync::Arc,
};

/// Task (Thread, Process, or Job)
pub trait Task: Sync + Send {
    /// Kill the task.
    fn kill(&self);

    /// Suspend the task. Currently only thread or process handles may be suspended.
    fn suspend(&self);

    /// Resume the task
    fn resume(&self);

    /// Get the exceptionate.
    fn exceptionate(&self) -> Arc<Exceptionate>;

    /// Get the debug exceptionate.
    fn debug_exceptionate(&self) -> Arc<Exceptionate>;
}

pub const TASK_RETCODE_SYSCALL_KILL: i64 = -1024;
