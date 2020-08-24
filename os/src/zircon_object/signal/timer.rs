use super::*;
use super::object::*;
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::time::Duration;
use spin::Mutex;

/// An object that may be signaled at some point in the future
///
/// ## SYNOPSIS
///
/// A timer is used to wait until a specified point in time has occurred
/// or the timer has been canceled.
pub struct Timer {
    base: KObjectBase,
    _counter: CountHelper,
    #[allow(dead_code)]
    slack: Slack,
    inner: Mutex<TimerInner>,
}

impl_kobject!(Timer);
define_count_helper!(Timer);

#[derive(Default)]
struct TimerInner {
    deadline: Option<Duration>,
}

/// Slack specifies how much a timer or event is allowed to deviate from its deadline.
///
/// **Not supported: Now slack has no effect on the timer.**
#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum Slack {
    /// slack is centered around deadline
    Center = 0,
    /// slack interval is (deadline - slack, deadline]
    Early = 1,
    /// slack interval is [deadline, deadline + slack)
    Late = 2,
}

impl Timer {
    /// Create a new `Timer`.
    pub fn new() -> Arc<Self> {
        Self::with_slack(Slack::Center)
    }

    /// Create a new `Timer` with slack.
    pub fn with_slack(slack: Slack) -> Arc<Self> {
        Arc::new(Timer {
            base: KObjectBase::default(),
            _counter: CountHelper::new(),
            slack,
            inner: Mutex::default(),
        })
    }

    /// Create a one-shot timer.
    pub fn one_shot(deadline: Duration) -> Arc<Self> {
        let timer = Timer::new();
        timer.set(deadline, Duration::default());
        timer
    }

    /// Starts a one-shot timer that will fire when `deadline` passes.
    ///
    /// If a previous call to `set` was pending, the previous timer is canceled
    /// and `Signal::SIGNALED` is de-asserted as needed.
    pub fn set(self: &Arc<Self>, deadline: Duration, _slack: Duration) {
        let mut inner = self.inner.lock();
        inner.deadline = Some(deadline);
        self.base.signal_clear(Signal::SIGNALED);
        let me = Arc::downgrade(self);
        crate::kernel_hal::timer_set(
            deadline,
            Box::new(move |now| me.upgrade().map(|timer| timer.touch(now)).unwrap_or(())),
        );
    }

    /// Cancel the pending timer started by `set`.
    pub fn cancel(&self) {
        let mut inner = self.inner.lock();
        inner.deadline = None;
    }

    /// Called by HAL timer.
    fn touch(&self, now: Duration) {
        let mut inner = self.inner.lock();
        if let Some(deadline) = inner.deadline {
            if now >= deadline {
                self.base.signal_set(Signal::SIGNALED);
                inner.deadline = None;
            }
        }
    }
}
