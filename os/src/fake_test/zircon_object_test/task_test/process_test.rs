use crate::zircon_object::task::*;
use crate::zircon_object::object::*;
use alloc::sync::Arc;
use crate::{print, println};

pub fn test_create_process() {
    let root_job = Job::root();
    let proc = Process::create(&root_job, "proc").expect("failed to create process");
    assert_eq!(proc.related_koid(), root_job.id());
    assert!(Arc::ptr_eq(&root_job, &proc.job()));
    println!("test_create_process pass");
}

pub fn test_handle() {
    let root_job = Job::root();
    let proc = Process::create(&root_job, "proc").expect("failed to create process");
    let handle = Handle::new(proc.clone(), Rights::DEFAULT_PROCESS);

    let handle_value = proc.add_handle(handle);
    let _info = proc.get_handle_info(handle_value).unwrap();

    // getting object should success
    let object: Arc<Process> = proc
        .get_object_with_rights(handle_value, Rights::DEFAULT_PROCESS)
        .expect("failed to get object");
    assert!(Arc::ptr_eq(&object, &proc));

    let (object, rights) = proc
        .get_object_and_rights::<Process>(handle_value)
        .expect("failed to get object");
    assert!(Arc::ptr_eq(&object, &proc));
    assert_eq!(rights, Rights::DEFAULT_PROCESS);

    // getting object with an extra rights should fail.
    assert_eq!(
        proc.get_object_with_rights::<Process>(handle_value, Rights::MANAGE_JOB)
            .err(),
        Some(ZxError::ACCESS_DENIED)
    );

    // getting object with invalid type should fail.
    assert_eq!(
        proc.get_object_with_rights::<Job>(handle_value, Rights::DEFAULT_PROCESS)
            .err(),
        Some(ZxError::WRONG_TYPE)
    );

    proc.remove_handle(handle_value).unwrap();

    // getting object with invalid handle should fail.
    assert_eq!(
        proc.get_object_with_rights::<Process>(handle_value, Rights::DEFAULT_PROCESS)
            .err(),
        Some(ZxError::BAD_HANDLE)
    );

    let handle1 = Handle::new(proc.clone(), Rights::DEFAULT_PROCESS);
    let handle2 = Handle::new(proc.clone(), Rights::DEFAULT_PROCESS);

    let handle_values = proc.add_handles(vec![handle1, handle2]);
    let object1: Arc<Process> = proc
        .get_object_with_rights(handle_values[0], Rights::DEFAULT_PROCESS)
        .expect("failed to get object");
    assert!(Arc::ptr_eq(&object1, &proc));

    proc.remove_handles(&handle_values).unwrap();
    assert_eq!(
        proc.get_object_with_rights::<Process>(handle_values[0], Rights::DEFAULT_PROCESS)
            .err(),
        Some(ZxError::BAD_HANDLE)
    );
    println!("test_handle pass");
}

pub fn test_handle_duplicate() {
    let root_job = Job::root();
    let proc = Process::create(&root_job, "proc").expect("failed to create process");

    // duplicate non-exist handle should fail.
    assert_eq!(
        proc.dup_handle_operating_rights(0, |_| Ok(Rights::empty())),
        Err(ZxError::BAD_HANDLE)
    );

    // duplicate handle with the same rights.
    let rights = Rights::DUPLICATE;
    let handle_value = proc.add_handle(Handle::new(proc.clone(), rights));
    let new_handle_value = proc
        .dup_handle_operating_rights(handle_value, |old_rights| Ok(old_rights))
        .unwrap();
    assert_eq!(proc.get_handle_test(new_handle_value).unwrap().rights, rights);

    // duplicate handle with subset rights.
    let new_handle_value = proc
        .dup_handle_operating_rights(handle_value, |_| Ok(Rights::empty()))
        .unwrap();
    assert_eq!(
        proc.get_handle_test(new_handle_value).unwrap().rights,
        Rights::empty()
    );

    // duplicate handle which does not have `Rights::DUPLICATE` should fail.
    let handle_value = proc.add_handle(Handle::new(proc.clone(), Rights::empty()));
    assert_eq!(
        proc.dup_handle_operating_rights(handle_value, |handle_rights| {
            if !handle_rights.contains(Rights::DUPLICATE) {
                return Err(ZxError::ACCESS_DENIED);
            }
            Ok(handle_rights)
        }),
        Err(ZxError::ACCESS_DENIED)
    );
    println!("test_handle_duplicate pass");
}

pub fn test_get_child_in_process() {
    let root_job = Job::root();
    let proc = Process::create(&root_job, "proc").expect("failed to create process");
    let thread = Thread::create(&proc, "thread").expect("failed to create thread");

    assert_eq!(proc.get_child(thread.id()).unwrap().id(), thread.id());
    assert_eq!(proc.get_child(proc.id()).err(), Some(ZxError::NOT_FOUND));

    let thread1 = Thread::create(&proc, "thread1").expect("failed to create thread");
    assert_eq!(proc.thread_ids(), vec![thread.id(), thread1.id()]);
    println!("test_get_child_in_process pass");
}

pub fn test_properties() {
    let root_job = Job::root();
    let proc = Process::create(&root_job, "proc").expect("failed to create process");

    proc.set_debug_addr(123);
    assert_eq!(proc.get_debug_addr(), 123);

    proc.set_dyn_break_on_load(2);
    assert_eq!(proc.get_dyn_break_on_load(), 2);
    println!("test_properties pass");
}

pub fn test_process_exit() {
    let root_job = Job::root();
    let proc = Process::create(&root_job, "proc").expect("failed to create process");
    let thread = Thread::create(&proc, "thread").expect("failed to create thread");

    let info = proc.get_info();
    assert!(!info.has_exited && !info.started && info.return_code == 0);

    proc.exit(666);
    let info = proc.get_info();
    assert!(info.has_exited && info.started && info.return_code == 666);
    assert_eq!(thread.state(), ThreadState::Dying);
    // TODO: when is the thread dead?

    assert_eq!(
        Thread::create(&proc, "thread1").err(),
        Some(ZxError::BAD_STATE)
    );
    println!("test_process_exit pass")
}

pub fn test_check_policy_process() {
    let root_job = Job::root();
    let policy1 = BasicPolicy {
        condition: PolicyCondition::BadHandle,
        action: PolicyAction::Allow,
    };
    let policy2 = BasicPolicy {
        condition: PolicyCondition::NewChannel,
        action: PolicyAction::Deny,
    };

    assert!(root_job
        .set_policy_basic(SetPolicyOptions::Absolute, &[policy1, policy2])
        .is_ok());
    let proc = Process::create(&root_job, "proc").expect("failed to create process");

    assert!(proc.check_policy(PolicyCondition::BadHandle).is_ok());
    assert!(proc.check_policy(PolicyCondition::NewProcess).is_ok());
    assert_eq!(
        proc.check_policy(PolicyCondition::NewChannel).err(),
        Some(ZxError::ACCESS_DENIED)
    );

    let _job = root_job.create_child().unwrap();
    assert_eq!(
        root_job
            .set_policy_basic(SetPolicyOptions::Absolute, &[policy1, policy2])
            .err(),
        Some(ZxError::BAD_STATE)
    );
    println!("test_check_policy_process pass");
}