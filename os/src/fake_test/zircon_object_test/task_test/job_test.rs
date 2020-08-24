use crate::zircon_object::task::*;
use crate::zircon_object::ZxError;
use crate::zircon_object::object::KernelObject;
use alloc::sync::Arc;
use crate::{print, println};

pub fn test_create_job() {
    let root_job = Job::root();
        let job: Arc<dyn KernelObject> =
            Job::create_child(&root_job).expect("failed to create job");

        assert!(Arc::ptr_eq(&root_job.get_child(job.id()).unwrap(), &job));
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

pub fn test_get_child_in_job() {
    let root_job = Job::root();
    let job = Job::create_child(&root_job).expect("failed to create job");
    let proc = Process::create(&root_job, "proc").expect("failed to create process");
    let root_job: Arc<dyn KernelObject> = root_job;
    assert_eq!(root_job.get_child(job.id()).unwrap().id(), job.id());
    assert_eq!(root_job.get_child(proc.id()).unwrap().id(), proc.id());
    assert_eq!(
        root_job.get_child(root_job.id()).err(),
        Some(ZxError::NOT_FOUND)
    );
    println!("test_get_child pass");
}