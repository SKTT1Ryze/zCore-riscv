use crate::zircon_object::task::*;
use crate::zircon_object::ZxError;
use crate::zircon_object::object::KernelObject;
use crate::zircon_object::object::Signal;
use alloc::sync::Arc;
use crate::{print, println};

pub fn test_create_job() {
    let root_job = Job::root();
    let job: Arc<dyn KernelObject> =
        Job::create_child(&root_job).expect("failed to create job");

    //assert!(Arc::ptr_eq(&root_job.get_child(job.id()).unwrap(), &job));
    assert_eq!(job.related_koid(), root_job.id());
    assert_eq!(root_job.related_koid(), 0);

    root_job.kill();
    assert_eq!(root_job.create_child().err(), Some(ZxError::BAD_STATE));
    println!("test_create pass");
}

pub fn test_set_policy() {
    let root_job = Job::root();

        // default policy
        assert_eq!(
            root_job.policy().get_action(PolicyCondition::BadHandle),
            None
        );

        // set policy for root job
        let policy = &[BasicPolicy {
            condition: PolicyCondition::BadHandle,
            action: PolicyAction::Deny,
        }];
        root_job
            .set_policy_basic(SetPolicyOptions::Relative, policy)
            .expect("failed to set policy");
        assert_eq!(
            root_job.policy().get_action(PolicyCondition::BadHandle),
            Some(PolicyAction::Deny)
        );

        // override policy should success
        let policy = &[BasicPolicy {
            condition: PolicyCondition::BadHandle,
            action: PolicyAction::Allow,
        }];
        root_job
            .set_policy_basic(SetPolicyOptions::Relative, policy)
            .expect("failed to set policy");
        assert_eq!(
            root_job.policy().get_action(PolicyCondition::BadHandle),
            Some(PolicyAction::Allow)
        );

        // create a child job
        let job = Job::create_child(&root_job).expect("failed to create job");

        // should inherit parent's policy.
        assert_eq!(
            job.policy().get_action(PolicyCondition::BadHandle),
            Some(PolicyAction::Allow)
        );

        // setting policy for a non-empty job should fail.
        assert_eq!(
            root_job.set_policy_basic(SetPolicyOptions::Relative, &[]),
            Err(ZxError::BAD_STATE)
        );

        // set new policy should success.
        let policy = &[BasicPolicy {
            condition: PolicyCondition::WrongObject,
            action: PolicyAction::Allow,
        }];
        job.set_policy_basic(SetPolicyOptions::Relative, policy)
            .expect("failed to set policy");
        assert_eq!(
            job.policy().get_action(PolicyCondition::WrongObject),
            Some(PolicyAction::Allow)
        );

        // relatively setting existing policy should be ignored.
        let policy = &[BasicPolicy {
            condition: PolicyCondition::BadHandle,
            action: PolicyAction::Deny,
        }];
        job.set_policy_basic(SetPolicyOptions::Relative, policy)
            .expect("failed to set policy");
        assert_eq!(
            job.policy().get_action(PolicyCondition::BadHandle),
            Some(PolicyAction::Allow)
        );

        // absolutely setting existing policy should fail.
        assert_eq!(
            job.set_policy_basic(SetPolicyOptions::Absolute, policy),
            Err(ZxError::ALREADY_EXISTS)
        );
    println!("test_set_policy pass");
}

pub fn test_parent_child_job() {
    let root_job = Job::root();
    let job = Job::create_child(&root_job).expect("failed to create job");
    let proc = Process::create(&root_job, "proc").expect("failed to create process");

    assert_eq!(root_job.get_child(job.id()).unwrap().id(), job.id());
    assert_eq!(root_job.get_child(proc.id()).unwrap().id(), proc.id());
    assert_eq!(
        root_job.get_child(root_job.id()).err(),
        Some(ZxError::NOT_FOUND)
    );
    assert!(Arc::ptr_eq(&job.parent().unwrap(), &root_job));

    let job1 = root_job.create_child().expect("failed to create job");
    let proc1 = Process::create(&root_job, "proc1").expect("failed to create process");
    assert_eq!(root_job.children_ids(), vec![job.id(), job1.id()]);
    assert_eq!(root_job.process_ids(), vec![proc.id(), proc1.id()]);

    root_job.kill();
    assert_eq!(root_job.create_child().err(), Some(ZxError::BAD_STATE));
    println!("test_parent_child pass");
}

pub fn test_check_job() {
    let root_job = Job::root();
    assert!(root_job.is_empty());
    let job = root_job.create_child().expect("failed to create job");
    assert_eq!(root_job.check_root_job(), Ok(()));
    assert_eq!(job.check_root_job(), Err(ZxError::ACCESS_DENIED));

    assert!(!root_job.is_empty());
    assert!(job.is_empty());

    let _proc = Process::create(&job, "proc").expect("failed to create process");
    assert!(!job.is_empty());
    println!("test_check_job pass");
}

/* pub fn test_kill_job() {
    let root_job = Job::root();
    let job = Job::create_child(&root_job).expect("failed to create job");
    let proc = Process::create(&root_job, "proc").expect("failed to create process");
    let thread = Thread::create(&proc, "thread").expect("failed to create thread");
    let current_thread = CurrentThread(thread.clone());

    root_job.kill();
    assert!(root_job.inner.lock().killed);
    assert!(job.inner.lock().killed);
    assert_eq!(proc.status(), Status::Exited(TASK_RETCODE_SYSCALL_KILL));
    assert_eq!(thread.state(), ThreadState::Dying);
    // killed but not terminated, since `CurrentThread` not dropped.
    assert!(!root_job.signal().contains(Signal::JOB_TERMINATED));
    assert!(job.signal().contains(Signal::JOB_TERMINATED)); // but the lonely job is terminated
    assert!(!proc.signal().contains(Signal::PROCESS_TERMINATED));
    assert!(!thread.signal().contains(Signal::THREAD_TERMINATED));

    std::mem::drop(current_thread);
    assert!(root_job.inner.lock().killed);
    assert!(job.inner.lock().killed);
    assert_eq!(proc.status(), Status::Exited(TASK_RETCODE_SYSCALL_KILL));
    assert_eq!(thread.state(), ThreadState::Dead);
    // all terminated now
    assert!(root_job.signal().contains(Signal::JOB_TERMINATED));
    assert!(job.signal().contains(Signal::JOB_TERMINATED));
    assert!(proc.signal().contains(Signal::PROCESS_TERMINATED));
    assert!(thread.signal().contains(Signal::THREAD_TERMINATED));

    // The job has no children.
    let root_job = Job::root();
    root_job.kill();
    assert!(root_job.inner.lock().killed);
    assert!(root_job.signal().contains(Signal::JOB_TERMINATED));

    // The job's process have no threads.
    let root_job = Job::root();
    let job = Job::create_child(&root_job).expect("failed to create job");
    let proc = Process::create(&root_job, "proc").expect("failed to create process");
    root_job.kill();
    assert!(root_job.inner.lock().killed);
    assert!(job.inner.lock().killed);
    assert_eq!(proc.status(), Status::Exited(TASK_RETCODE_SYSCALL_KILL));
    assert!(root_job.signal().contains(Signal::JOB_TERMINATED));
    assert!(job.signal().contains(Signal::JOB_TERMINATED));
    assert!(proc.signal().contains(Signal::PROCESS_TERMINATED));
} */

pub fn test_critical_process() {
    let root_job = Job::root();
    let job = root_job.create_child().unwrap();
    let job1 = root_job.create_child().unwrap();

    let proc = Process::create(&job, "proc").expect("failed to create process");

    assert_eq!(
        proc.set_critical_at_job(&job1, true).err(),
        Some(ZxError::INVALID_ARGS)
    );
    proc.set_critical_at_job(&root_job, true).unwrap();
    assert_eq!(
        proc.set_critical_at_job(&job, true).err(),
        Some(ZxError::ALREADY_BOUND)
    );

    proc.exit(666);
    assert!(root_job.killed_test());
    assert!(root_job.signal().contains(Signal::JOB_TERMINATED));
    println!("test_critical_process pass");
}