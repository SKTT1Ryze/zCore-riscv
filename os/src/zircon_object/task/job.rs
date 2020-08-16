use {
    super::exception::*, super::job_policy::*, super::process::Process, super::object::*,
    super::task::Task, alloc::sync::Arc, alloc::vec::Vec, spin::Mutex,
};

/// Control a group of processes
///
/// ## SYNOPSIS
///
/// A job is a group of processes and possibly other (child) jobs. Jobs are used to
/// track privileges to perform kernel operations (i.e., make various syscalls,
/// with various options), and track and limit basic resource (e.g., memory, CPU)
/// consumption. Every process belongs to a single job. Jobs can also be nested,
/// and every job except the root job also belongs to a single (parent) job.
///
/// ## DESCRIPTION
///
/// A job is an object consisting of the following:
/// - a reference to a parent job
/// - a set of child jobs (each of whom has this job as parent)
/// - a set of member [processes](crate::task::Process)
/// - a set of policies
///
/// Jobs control "applications" that are composed of more than one process to be
/// controlled as a single entity.
#[allow(dead_code)]
pub struct Job {
    base: KObjectBase,
    _counter: CountHelper,
    parent: Option<Arc<Job>>,
    parent_policy: JobPolicy,
    exceptionate: Arc<Exceptionate>,
    debug_exceptionate: Arc<Exceptionate>,
    inner: Mutex<JobInner>,
}

impl_kobject!(Job
    fn get_child(&self, id: KoID) -> ZxResult<Arc<dyn KernelObject>> {
        let inner = self.inner.lock();
        if let Some(job) = inner.children.iter().find(|o| o.id() == id) {
            return Ok(job.clone());
        }
        if let Some(proc) = inner.processes.iter().find(|o| o.id() == id) {
            return Ok(proc.clone());
        }
        Err(ZxError::NOT_FOUND)
    }
    fn related_koid(&self) -> KoID {
        self.parent.as_ref().map(|p| p.id()).unwrap_or(0)
    }
);
define_count_helper!(Job);

#[derive(Default)]
struct JobInner {
    policy: JobPolicy,
    children: Vec<Arc<Job>>,
    processes: Vec<Arc<Process>>,
    // if the job is killed, no more child creation should works
    killed: bool,
    timer_policy: TimerSlack,
}

impl Job {
    /// Create the root job.
    pub fn root() -> Arc<Self> {
        Arc::new(Job {
            base: KObjectBase::new(),
            _counter: CountHelper::new(),
            parent: None,
            parent_policy: JobPolicy::default(),
            exceptionate: Exceptionate::new(ExceptionChannelType::Job),
            debug_exceptionate: Exceptionate::new(ExceptionChannelType::JobDebugger),
            inner: Mutex::new(JobInner::default()),
        })
    }

    /// Create a new child job object.
    pub fn create_child(self: &Arc<Self>, _options: u32) -> ZxResult<Arc<Self>> {
        // TODO: options
        let mut inner = self.inner.lock();
        if inner.killed {
            return Err(ZxError::BAD_STATE);
        }
        let child = Arc::new(Job {
            base: KObjectBase::new(),
            _counter: CountHelper::new(),
            parent: Some(self.clone()),
            parent_policy: inner.policy.merge(&self.parent_policy),
            exceptionate: Exceptionate::new(ExceptionChannelType::Job),
            debug_exceptionate: Exceptionate::new(ExceptionChannelType::JobDebugger),
            inner: Mutex::new(JobInner::default()),
        });
        inner.children.push(child.clone());
        Ok(child)
    }

    /// Get the policy of the job.
    pub fn policy(&self) -> JobPolicy {
        self.inner.lock().policy.merge(&self.parent_policy)
    }

    /// Get the parent job.
    pub fn parent(&self) -> Option<Arc<Self>> {
        self.parent.clone()
    }

    /// Sets one or more security and/or resource policies to an empty job.
    ///
    /// The job's effective policies is the combination of the parent's
    /// effective policies and the policies specified in policy.
    ///
    /// After this call succeeds any new child process or child job will have
    /// the new effective policy applied to it.
    pub fn set_policy_basic(&self, options: SetPolicyOptions, policys: &[BasicPolicy]) -> ZxResult {
        let mut inner = self.inner.lock();
        if !inner.is_empty() {
            return Err(ZxError::BAD_STATE);
        }
        for policy in policys {
            if self.parent_policy.get_action(policy.condition).is_some() {
                match options {
                    SetPolicyOptions::Absolute => return Err(ZxError::ALREADY_EXISTS),
                    SetPolicyOptions::Relative => {}
                }
            } else {
                inner.policy.apply(*policy);
            }
        }
        Ok(())
    }

    pub fn set_policy_timer_slack(&self, policy: TimerSlackPolicy) -> ZxResult {
        let mut inner = self.inner.lock();
        if !inner.is_empty() {
            return Err(ZxError::BAD_STATE);
        }
        check_timer_policy(&policy)?;
        inner.timer_policy = inner.timer_policy.generate_new(policy);
        Ok(())
    }

    /// Add a process to the job.
    pub(super) fn add_process(&self, process: Arc<Process>) -> ZxResult {
        let mut inner = self.inner.lock();
        if inner.killed {
            return Err(ZxError::BAD_STATE);
        }
        inner.processes.push(process);
        Ok(())
    }

    pub(super) fn remove_process(&self, id: KoID) {
        let mut inner = self.inner.lock();
        inner.processes.retain(|proc| proc.id() != id);
    }

    pub fn get_info(&self) -> JobInfo {
        JobInfo::default()
    }

    pub fn check_root_job(&self) -> ZxResult {
        if self.parent.is_some() {
            Err(ZxError::ACCESS_DENIED)
        } else {
            Ok(())
        }
    }

    pub fn get_exceptionate(&self) -> Arc<Exceptionate> {
        self.exceptionate.clone()
    }

    pub fn get_debug_exceptionate(&self) -> Arc<Exceptionate> {
        self.debug_exceptionate.clone()
    }

    /// Get KoIDs of Processes.
    pub fn process_ids(&self) -> Vec<KoID> {
        self.inner.lock().processes.iter().map(|p| p.id()).collect()
    }

    /// Get KoIDs of children Jobs.
    pub fn children_ids(&self) -> Vec<KoID> {
        self.inner.lock().children.iter().map(|j| j.id()).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.lock().is_empty()
    }

    pub fn kill(&self) {
        let (children, processes) = {
            let mut inner = self.inner.lock();
            if inner.killed {
                return;
            }
            inner.killed = true;
            (
                core::mem::take(&mut inner.children),
                core::mem::take(&mut inner.processes),
            )
        };
        for child in children {
            child.kill();
        }
        for proc in processes {
            proc.kill();
        }
        self.base.signal_set(Signal::JOB_TERMINATED);
    }
}

impl JobInner {
    fn is_empty(&self) -> bool {
        self.processes.is_empty() && self.children.is_empty()
    }
}

#[repr(C)]
#[derive(Default)]
pub struct JobInfo {
    return_code: i64,
    exited: bool,
    kill_on_oom: bool,
    debugger_attached: bool,
    padding: [u8; 5],
}
