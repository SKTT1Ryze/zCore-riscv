pub mod thread_test;
pub mod process_test;
pub mod suspend_token_test;
pub mod job_test;
pub mod job_policy_test;
pub mod exception_test;

use crate::{print, println};

use {
    thread_test::*,
    process_test::*,
    suspend_token_test::*,
    job_policy_test::*,
    job_test::*,
    exception_test::*,
};

pub fn test_all_in_task_test() {
    test_create_job();
    test_set_policy();
    //test_get_child_in_job();
    //test_create_process();
    //test_handle();
    //test_handle_duplicate();
    //test_get_child_in_process();
    //test_create_thread();
    println!("all test in task_test pass");
}