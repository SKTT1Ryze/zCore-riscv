use crate::zircon_object::task::*;
use crate::{print, println};

pub fn test_create_thread() {
    let root_job = Job::root();
    let proc = Process::create(&root_job, "proc").expect("failed to create process");
    let _thread = Thread::create(&proc, "thread").expect("failed to create thread");
    println!("test_create_thread pass");
}

/* 
#[cfg(test)]
mod tests {
    use super::job::Job;
    use super::*;
    use std::sync::atomic::*;
    use std::vec;

    #[test]
    #[ignore]
    fn start() {
        let root_job = Job::root();
        let proc = Process::create(&root_job, "proc", 0).expect("failed to create process");
        let thread = Thread::create(&proc, "thread", 0).expect("failed to create thread");
        let thread1 = Thread::create(&proc, "thread1", 0).expect("failed to create thread");

        // allocate stack for new thread
        let mut stack = vec![0u8; 0x1000];
        let stack_top = stack.as_mut_ptr() as usize + 0x1000;

        // global variable for validation
        static ARG1: AtomicUsize = AtomicUsize::new(0);
        static ARG2: AtomicUsize = AtomicUsize::new(0);

        // function for new thread
        #[allow(unsafe_code)]
        unsafe extern "C" fn entry(arg1: usize, arg2: usize) -> ! {
            ARG1.store(arg1, Ordering::SeqCst);
            ARG2.store(arg2, Ordering::SeqCst);
            kernel_hal_unix::syscall_entry();
            unreachable!();
        }
        let entry = entry as usize;

        fn spawn(_thread: Arc<Thread>) {
            unimplemented!()
        }

        // start a new thread
        let thread_ref_count = Arc::strong_count(&thread);
        let handle = Handle::new(proc.clone(), Rights::DEFAULT_PROCESS);
        proc.start(&thread, entry, stack_top, Some(handle.clone()), 2, spawn)
            .expect("failed to start thread");

        // wait 100ms for the new thread to exit
        std::thread::sleep(core::time::Duration::from_millis(100));

        // validate the thread have started and received correct arguments
        assert_eq!(ARG1.load(Ordering::SeqCst), 0);
        assert_eq!(ARG2.load(Ordering::SeqCst), 2);

        // no other references to `Thread`
        assert_eq!(Arc::strong_count(&thread), thread_ref_count);

        // start again should fail
        assert_eq!(
            proc.start(&thread, entry, stack_top, Some(handle.clone()), 2, spawn),
            Err(ZxError::BAD_STATE)
        );

        // start another thread should fail
        assert_eq!(
            proc.start(&thread1, entry, stack_top, Some(handle.clone()), 2, spawn),
            Err(ZxError::BAD_STATE)
        );
    }
} */

