use crate::zircon_object::task::*;
use crate::zircon_object::object::*;
use alloc::sync::Arc;
use crate::{print, println};

pub fn test_create_process() {
    let root_job = Job::root();
    let _proc = Process::create(&root_job, "proc", 0).expect("failed to create process");
    println!("test_create_process pass");
}

pub fn test_handle() {
    let root_job = Job::root();
    let proc = Process::create(&root_job, "proc", 0).expect("failed to create process");
    let handle = Handle::new(proc.clone(), Rights::DEFAULT_PROCESS);

    let handle_value = proc.add_handle(handle);

    // getting object should success
    let object: Arc<Process> = proc
        .get_object_with_rights(handle_value, Rights::DEFAULT_PROCESS)
        .expect("failed to get object");
    assert!(Arc::ptr_eq(&object, &proc));

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
    println!("test_handle pass");
}

pub fn test_handle_duplicate() {
    let root_job = Job::root();
    let proc = Process::create(&root_job, "proc", 0).expect("failed to create process");

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
    let proc = Process::create(&root_job, "proc", 0).expect("failed to create process");
    let thread = Thread::create(&proc, "thread", 0).expect("failed to create thread");

    let proc: Arc<dyn KernelObject> = proc;
    assert_eq!(proc.get_child(thread.id()).unwrap().id(), thread.id());
    assert_eq!(proc.get_child(proc.id()).err(), Some(ZxError::NOT_FOUND));
    println!("test_get_child_in_process pass");
}