use {
    super::exception::*,
    super::process::Process,
    super::*,
    super::object::*,
    alloc::{boxed::Box, sync::Arc},
    bitflags::bitflags,
    core::{
        any::Any,
        future::Future,
        pin::Pin,
        task::{Context, Poll, Waker},
        time::Duration,
    },
    futures::{channel::oneshot::*, future::FutureExt, select_biased},
    crate::kernel_hal::{sleep_until, GeneralRegs, UserContext},
    spin::Mutex,
};

use riscv::register::sstatus::{self, FS, SPP::*};
use bit::BitIndex;
use crate::println;

pub use self::thread_state::*;

mod thread_state;

/// Runnable / computation entity
///
/// ## SYNOPSIS
///
/// TODO
///
/// ## DESCRIPTION
///
/// The thread object is the construct that represents a time-shared CPU execution
/// context. Thread objects live associated to a particular
/// [Process Object](crate::task::Process) which provides the memory and the handles to other
/// objects necessary for I/O and computation.
///
/// ### Lifetime
/// Threads are created by calling [`Thread::create()`], but only start executing
/// when either [`Thread::start()`] or [`Process::start()`] are called. Both syscalls
/// take as an argument the entrypoint of the initial routine to execute.
///
/// The thread passed to [`Process::start()`] should be the first thread to start execution
/// on a process.
///
/// A thread terminates execution:
/// - by calling [`Thread::exit()`]
/// - when the parent process terminates
/// - by calling [`Task::kill()`]
/// - after generating an exception for which there is no handler or the handler
/// decides to terminate the thread.
///
/// Returning from the entrypoint routine does not terminate execution. The last
/// action of the entrypoint should be to call [`Thread::exit()`].
///
/// Closing the last handle to a thread does not terminate execution. In order to
/// forcefully kill a thread for which there is no available handle, use
/// `KernelObject::get_child()` to obtain a handle to the thread. This method is strongly
/// discouraged. Killing a thread that is executing might leave the process in a
/// corrupt state.
///
/// Fuchsia native threads are always *detached*. That is, there is no *join()* operation
/// needed to do a clean termination. However, some runtimes above the kernel, such as
/// C11 or POSIX might require threads to be joined.
///
/// ### Signals
/// Threads provide the following signals:
/// - [`THREAD_TERMINATED`]
/// - [`THREAD_SUSPENDED`]
/// - [`THREAD_RUNNING`]
///
/// When a thread is started [`THREAD_RUNNING`] is asserted. When it is suspended
/// [`THREAD_RUNNING`] is deasserted, and [`THREAD_SUSPENDED`] is asserted. When
/// the thread is resumed [`THREAD_SUSPENDED`] is deasserted and
/// [`THREAD_RUNNING`] is asserted. When a thread terminates both
/// [`THREAD_RUNNING`] and [`THREAD_SUSPENDED`] are deasserted and
/// [`THREAD_TERMINATED`] is asserted.
///
/// Note that signals are OR'd into the state maintained by the
/// `KernelObject::wait_signal()` family of functions thus
/// you may see any combination of requested signals when they return.
///
/// [`Thread::create()`]: Thread::create
/// [`Thread::exit()`]: Thread::exit
/// [`Process::exit()`]: crate::task::Process::exit
/// [`THREAD_TERMINATED`]: crate::object::Signal::THREAD_TERMINATED
/// [`THREAD_SUSPENDED`]: crate::object::Signal::THREAD_SUSPENDED
/// [`THREAD_RUNNING`]: crate::object::Signal::THREAD_RUNNING
pub struct Thread {
    base: KObjectBase,
    _counter: CountHelper,
    proc: Arc<Process>,
    ext: Box<dyn Any + Send + Sync>,
    inner: Mutex<ThreadInner>,
    exceptionate: Arc<Exceptionate>,
}

impl_kobject!(Thread
    fn related_koid(&self) -> KoID {
        self.proc.id()
    }
);
define_count_helper!(Thread);

#[derive(Default)]
struct ThreadInner {
    /// Thread context
    ///
    /// It will be taken away when running this thread.
    context: Option<Box<UserContext>>,

    /// The number of existing `SuspendToken`.
    suspend_count: usize,
    /// The waker of task when suspending.
    waker: Option<Waker>,
    /// A token used to kill blocking thread
    killer: Option<Sender<()>>,
    /// Thread state
    ///
    /// NOTE: This variable will never be `Suspended`. On suspended, the
    /// `suspend_count` is non-zero, and this represents the state before suspended.
    state: ThreadState,
    /// The currently processing exception
    exception: Option<Arc<Exception>>,
    /// The time this thread has run on cpu
    time: u128,
    flags: ThreadFlag,
}

impl ThreadInner {
    pub fn get_state(&self) -> ThreadState {
        if self.suspend_count == 0 {
            self.state
        } else {
            ThreadState::Suspended
        }
    }
}

bitflags! {
    #[derive(Default)]
    pub struct ThreadFlag: usize {
        const VCPU = 1 << 3;
    }
}

impl Thread {
    /// Create a new thread.
    pub fn create(proc: &Arc<Process>, name: &str, _options: u32) -> ZxResult<Arc<Self>> {
        Self::create_with_ext(proc, name, ())
    }

    /// Create a new thread with extension info.
    pub fn create_with_ext(
        proc: &Arc<Process>,
        name: &str,
        ext: impl Any + Send + Sync,
    ) -> ZxResult<Arc<Self>> {
        // TODO: options
        let thread = Arc::new(Thread {
            base: KObjectBase::with_name(name),
            _counter: CountHelper::new(),
            proc: proc.clone(),
            ext: Box::new(ext),
            exceptionate: Exceptionate::new(ExceptionChannelType::Thread),
            inner: Mutex::new(ThreadInner {
                context: Some(Box::new(UserContext::default())),
                ..Default::default()
            }),
        });
        proc.add_thread(thread.clone())?;
        Ok(thread)
    }

    /// Get the process.
    pub fn proc(&self) -> &Arc<Process> {
        &self.proc
    }

    /// Get the extension.
    pub fn ext(&self) -> &Box<dyn Any + Send + Sync> {
        &self.ext
    }

    /// Start execution on the thread.
    pub fn start(
        self: &Arc<Self>,
        entry: usize,
        stack: usize,
        arg1: usize,
        arg2: usize,
        spawn_fn: fn(thread: Arc<Thread>),
    ) -> ZxResult {
        {
            let mut inner = self.inner.lock();
            let context = inner.context.as_mut().ok_or(ZxError::BAD_STATE)?;
            #[cfg(target_arch = "x86_64")]
            {
                context.general.rip = entry;
                context.general.rsp = stack;
                context.general.rdi = arg1;
                context.general.rsi = arg2;
                context.general.rflags |= 0x3202;
            }
            #[cfg(target_arch = "aarch64")]
            {
                context.elr = entry;
                context.sp = stack;
                context.general.x0 = arg1;
                context.general.x1 = arg2;
            }
            #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
            {
                context.sepc = entry;
                context.set_sp(stack);
                context.sstatus.set_bit(0, sstatus::read().uie());
                context.sstatus.set_bit(1, sstatus::read().sie());
                context.sstatus.set_bit(4, sstatus::read().upie());
                context.sstatus.set_bit(5, sstatus::read().spie());
                match sstatus::read().spp() {
                    Supervisor => {
                        context.sstatus.set_bit(8, true);
                    },
                    User => {
                        context.sstatus.set_bit(8, false);
                    }
                }
                match sstatus::read().fs() {
                    FS::Off => {
                        context.sstatus.set_bit(13, false);
                        context.sstatus.set_bit(14, false);
                        context.sstatus.set_bit(15, false);
                    },
                    FS::Initial => {
                        context.sstatus.set_bit(13, true);
                        context.sstatus.set_bit(14, false);
                        context.sstatus.set_bit(15, false);
                    },
                    FS::Clean => {
                        context.sstatus.set_bit(13, false);
                        context.sstatus.set_bit(14, true);
                        context.sstatus.set_bit(15, false);
                    },
                    FS::Dirty => {
                        context.sstatus.set_bit(13, true);
                        context.sstatus.set_bit(14, true);
                        context.sstatus.set_bit(15, false);
                    },
                    _ => unreachable!(),
                }
                match sstatus::read().xs() {
                    FS::Off => {
                        context.sstatus.set_bit(15, false);
                        context.sstatus.set_bit(16, false);
                        context.sstatus.set_bit(17, false);
                    },
                    FS::Initial => {
                        context.sstatus.set_bit(13, true);
                        context.sstatus.set_bit(14, false);
                        context.sstatus.set_bit(15, false);
                    },
                    FS::Clean => {
                        context.sstatus.set_bit(13, false);
                        context.sstatus.set_bit(14, true);
                        context.sstatus.set_bit(15, false);
                    },
                    FS::Dirty => {
                        context.sstatus.set_bit(13, true);
                        context.sstatus.set_bit(14, true);
                        context.sstatus.set_bit(15, false);
                    },
                    _ => unreachable!(),
                }
                context.sstatus.set_bit(18, sstatus::read().sum());
                context.sstatus.set_bit(19, sstatus::read().mxr());
                context.sstatus.set_bit(8, true);
                context.sstatus.set_bit(5, true);
            }
            inner.state = ThreadState::Running;
            self.base.signal_set(Signal::THREAD_RUNNING);
        }
        spawn_fn(self.clone());
        Ok(())
    }

    /// Start execution with given registers.
    pub fn start_with_regs(
        self: &Arc<Self>,
        regs: GeneralRegs,
        spawn_fn: fn(thread: Arc<Thread>),
    ) -> ZxResult {
        {
            let mut inner = self.inner.lock();
            let context = inner.context.as_mut().ok_or(ZxError::BAD_STATE)?;
            context.general = regs;
            #[cfg(target_arch = "x86_64")]
            {
                context.general.rflags |= 0x3202;
            }
            inner.state = ThreadState::Running;
            self.base.signal_set(Signal::THREAD_RUNNING);
        }
        spawn_fn(self.clone());
        Ok(())
    }

    /// Terminate the current running thread.
    /// TODO: move to CurrentThread
    pub fn exit(&self) {
        self.proc().remove_thread(self.base.id);
        self.internal_exit();
    }

    pub fn internal_exit(&self) {
        self.base.signal_set(Signal::THREAD_TERMINATED);
        self.inner.lock().state = ThreadState::Dead;
    }

    /// Read one aspect of thread state.
    pub fn read_state(&self, kind: ThreadStateKind, buf: &mut [u8]) -> ZxResult<usize> {
        let inner = self.inner.lock();
        let context = inner.context.as_ref().ok_or(ZxError::BAD_STATE)?;
        context.read_state(kind, buf)
    }

    /// Write one aspect of thread state.
    pub fn write_state(&self, kind: ThreadStateKind, buf: &[u8]) -> ZxResult {
        let mut inner = self.inner.lock();
        let context = inner.context.as_mut().ok_or(ZxError::BAD_STATE)?;
        context.write_state(kind, buf)
    }

    pub fn wait_for_run(self: &Arc<Thread>) -> impl Future<Output = Box<UserContext>> {
        #[must_use = "wait_for_run does nothing unless polled/`await`-ed"]
        struct RunnableChecker {
            thread: Arc<Thread>,
        }
        impl Future for RunnableChecker {
            type Output = Box<UserContext>;

            fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
                let mut inner = self.thread.inner.lock();
                if inner.suspend_count == 0 {
                    // resume:  return the context token from thread object
                    Poll::Ready(inner.context.take().unwrap())
                } else {
                    // suspend: put waker into the thread object
                    inner.waker = Some(cx.waker().clone());
                    Poll::Pending
                }
            }
        }
        RunnableChecker {
            thread: self.clone(),
        }
    }

    pub fn end_running(&self, context: Box<UserContext>) {
        self.inner.lock().context = Some(context);
    }

    pub fn get_thread_info(&self) -> ThreadInfo {
        let inner = self.inner.lock();
        ThreadInfo {
            state: inner.get_state() as u32,
            wait_exception_channel_type: inner
                .exception
                .as_ref()
                .map_or(0, |exception| exception.get_current_channel_type() as u32),
            cpu_affnity_mask: [0u64; 8],
        }
    }

    pub fn get_thread_exception_info(&self) -> ZxResult<ExceptionReport> {
        let inner = self.inner.lock();
        if inner.get_state() != ThreadState::BlockedException {
            return Err(ZxError::BAD_STATE);
        }
        inner
            .exception
            .as_ref()
            .ok_or(ZxError::BAD_STATE)
            .map(|exception| exception.get_report())
    }

    /// Run async future and change state while blocking.
    pub async fn blocking_run<F, T, FT>(
        &self,
        future: F,
        state: ThreadState,
        deadline: Duration,
    ) -> ZxResult<T>
    where
        F: Future<Output = FT> + Unpin,
        FT: IntoResult<T>,
    {
        let (old_state, killed) = {
            let mut inner = self.inner.lock();
            if inner.get_state() == ThreadState::Dying {
                return Err(ZxError::STOP);
            }
            let (sender, receiver) = channel::<()>();
            inner.killer = Some(sender);
            (core::mem::replace(&mut inner.state, state), receiver)
        };
        let ret = select_biased! {
            ret = future.fuse() => ret.into_result(),
            _ = killed.fuse() => Err(ZxError::STOP),
            _ = sleep_until(deadline).fuse() => Err(ZxError::TIMED_OUT),
        };
        let mut inner = self.inner.lock();
        inner.killer = None;
        if inner.state == ThreadState::Dying {
            return ret;
        }
        assert_eq!(inner.state, state);
        inner.state = old_state;
        ret
    }

    /// Run a cancelable async future and change state while blocking.
    pub async fn cancelable_blocking_run<F, T, FT>(
        &self,
        future: F,
        state: ThreadState,
        deadline: Duration,
        cancel_token: Receiver<()>,
    ) -> ZxResult<T>
    where
        F: Future<Output = FT> + Unpin,
        FT: IntoResult<T>,
    {
        let (old_state, killed) = {
            let mut inner = self.inner.lock();
            if inner.get_state() == ThreadState::Dying {
                return Err(ZxError::STOP);
            }
            let (sender, receiver) = channel::<()>();
            inner.killer = Some(sender);
            (core::mem::replace(&mut inner.state, state), receiver)
        };
        let ret = select_biased! {
            ret = future.fuse() => ret.into_result(),
            _ = killed.fuse() => Err(ZxError::STOP),
            _ = sleep_until(deadline).fuse() => Err(ZxError::TIMED_OUT),
            _ = cancel_token.fuse() => Err(ZxError::CANCELED),
        };
        let mut inner = self.inner.lock();
        inner.killer = None;
        if inner.state == ThreadState::Dying {
            return ret;
        }
        assert_eq!(inner.state, state);
        inner.state = old_state;
        ret
    }

    pub fn state(&self) -> ThreadState {
        self.inner.lock().get_state()
    }

    pub fn get_exceptionate(&self) -> Arc<Exceptionate> {
        self.exceptionate.clone()
    }

    pub fn time_add(&self, time: u128) {
        self.inner.lock().time += time;
    }

    pub fn get_time(&self) -> u64 {
        self.inner.lock().time as u64
    }
    pub fn set_exception(&self, exception: Option<Arc<Exception>>) {
        self.inner.lock().exception = exception;
    }

    pub fn get_flags(&self) -> ThreadFlag {
        self.inner.lock().flags
    }

    pub fn update_flags<F>(&self, f: F)
    where
        F: FnOnce(&mut ThreadFlag),
    {
        f(&mut self.inner.lock().flags)
    }
}

impl Task for Thread {
    fn kill(&self) {
        let mut inner = self.inner.lock();
        if inner.state == ThreadState::Dying || inner.state == ThreadState::Dead {
            return;
        }
        inner.state = ThreadState::Dying;
        // For suspended thread, wake it and clear suspend count
        inner.suspend_count = 0;
        if let Some(waker) = inner.waker.take() {
            waker.wake();
        }
        // For blocking thread, use the killer
        if let Some(killer) = inner.killer.take() {
            // It's ok to ignore the error since the other end could be closed
            killer.send(()).ok();
        }
    }

    fn suspend(&self) {
        let mut inner = self.inner.lock();
        inner.suspend_count += 1;
        self.base.signal_set(Signal::THREAD_SUSPENDED);
        info!(
            "thread {:?} suspend: count={}",
            self.base.name(),
            inner.suspend_count
        );
    }

    fn resume(&self) {
        let mut inner = self.inner.lock();
        // assert_ne!(inner.suspend_count, 0);
        if inner.suspend_count == 0 {
            return;
        }
        inner.suspend_count -= 1;
        if inner.suspend_count == 0 {
            self.base.signal_set(Signal::THREAD_RUNNING);
            if let Some(waker) = inner.waker.take() {
                waker.wake();
            }
        }
    }
}

pub trait IntoResult<T> {
    fn into_result(self) -> ZxResult<T>;
}

impl<T> IntoResult<T> for T {
    fn into_result(self) -> ZxResult<T> {
        Ok(self)
    }
}

impl<T> IntoResult<T> for ZxResult<T> {
    fn into_result(self) -> ZxResult<T> {
        self
    }
}

/// The thread state.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ThreadState {
    /// The thread has been created but it has not started running yet.
    New = 0,
    /// The thread is running user code normally.
    Running = 1,
    /// Stopped due to `zx_task_suspend()`.
    Suspended = 2,
    /// In a syscall or handling an exception.
    Blocked = 3,
    /// The thread is in the process of being terminated, but it has not been stopped yet.
    Dying = 4,
    /// The thread has stopped running.
    Dead = 5,
    /// The thread is stopped in an exception.
    BlockedException = 0x103,
    /// The thread is stopped in `zx_nanosleep()`.
    BlockedSleeping = 0x203,
    /// The thread is stopped in `zx_futex_wait()`.
    BlockedFutex = 0x303,
    /// The thread is stopped in `zx_port_wait()`.
    BlockedPort = 0x403,
    /// The thread is stopped in `zx_channel_call()`.
    BlockedChannel = 0x503,
    /// The thread is stopped in `zx_object_wait_one()`.
    BlockedWaitOne = 0x603,
    /// The thread is stopped in `zx_object_wait_many()`.
    BlockedWaitMany = 0x703,
    /// The thread is stopped in `zx_interrupt_wait()`.
    BlockedInterrupt = 0x803,
    BlockedPager = 0x903,
}

impl Default for ThreadState {
    fn default() -> Self {
        ThreadState::New
    }
}

#[repr(C)]
pub struct ThreadInfo {
    state: u32,
    wait_exception_channel_type: u32,
    cpu_affnity_mask: [u64; 8],
}
