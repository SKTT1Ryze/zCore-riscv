use {
    super::*,
    core::time::Duration,
    crate::zircon_object::{signal::*, task::PolicyCondition},
};

impl Syscall<'_> {
    pub fn sys_timer_create(
        &self,
        options: u32,
        clock_id: u32,
        mut out: UserOutPtr<HandleValue>,
    ) -> ZxResult {
        info!(
            "timer.create: options={:#x}, clock_id={:#x}",
            options, clock_id
        );
        if clock_id != 0 {
            return Err(ZxError::INVALID_ARGS);
        }
        let proc = self.thread.proc();
        proc.check_policy(PolicyCondition::NewTimer)?;
        let slack = match options {
            0 => Slack::Center,
            1 => Slack::Early,
            2 => Slack::Late,
            _ => return Err(ZxError::INVALID_ARGS),
        };
        let handle = Handle::new(Timer::with_slack(slack), Rights::DEFAULT_TIMER);
        out.write(proc.add_handle(handle))?;
        Ok(())
    }

    pub fn sys_event_create(&self, options: u32, mut out: UserOutPtr<HandleValue>) -> ZxResult {
        info!("event.create: options={:#x}", options);
        if options != 0 {
            return Err(ZxError::INVALID_ARGS);
        }
        let proc = self.thread.proc();
        proc.check_policy(PolicyCondition::NewEvent)?;
        let handle = Handle::new(Event::new(), Rights::DEFAULT_EVENT);
        out.write(proc.add_handle(handle))?;
        Ok(())
    }

    pub fn sys_eventpair_create(
        &self,
        options: u32,
        mut out0: UserOutPtr<HandleValue>,
        mut out1: UserOutPtr<HandleValue>,
    ) -> ZxResult {
        info!("eventpair.create: options={:#x}", options);
        if options != 0 {
            return Err(ZxError::NOT_SUPPORTED);
        }
        let proc = self.thread.proc();
        proc.check_policy(PolicyCondition::NewEvent)?;
        let (event0, event1) = EventPair::create();
        let handle0 = Handle::new(event0, Rights::DEFAULT_EVENTPAIR);
        let handle1 = Handle::new(event1, Rights::DEFAULT_EVENTPAIR);
        out0.write(proc.add_handle(handle0))?;
        out1.write(proc.add_handle(handle1))?;
        Ok(())
    }

    pub fn sys_timer_set(&self, handle: HandleValue, deadline: Deadline, slack: i64) -> ZxResult {
        info!(
            "timer.set: handle={:#x}, deadline={:#x?}, slack={:#x}",
            handle, deadline, slack
        );
        if slack.is_negative() {
            return Err(ZxError::OUT_OF_RANGE);
        }
        let proc = self.thread.proc();
        let timer = proc.get_object_with_rights::<Timer>(handle, Rights::WRITE)?;
        timer.set(Duration::from(deadline), Duration::from_nanos(slack as u64));
        Ok(())
    }

    pub fn sys_timer_cancel(&self, handle: HandleValue) -> ZxResult {
        info!("timer.cancel: handle={:#x}", handle);
        let proc = self.thread.proc();
        let timer = proc.get_object_with_rights::<Timer>(handle, Rights::WRITE)?;
        timer.cancel();
        Ok(())
    }
}
