/* 
#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{Job, Process, ThreadState};
    use core::time::Duration;

    #[async_std::test]
    async fn wait() {
        static VALUE: AtomicI32 = AtomicI32::new(1);
        let futex = Futex::new(&VALUE);

        let count = futex.wake(1);
        assert_eq!(count, 0);

        // inconsistent value should fail.
        assert_eq!(futex.wait(0).await, Err(ZxError::BAD_STATE));

        // spawn a new task to wake me up.
        {
            let futex = futex.clone();
            async_std::task::spawn(async move {
                async_std::task::sleep(Duration::from_millis(10)).await;
                VALUE.store(2, Ordering::SeqCst);
                let count = futex.wake(1);
                assert_eq!(count, 1);
            });
        }
        // wait for wake.
        futex.wait(1).await.unwrap();
        assert_eq!(VALUE.load(Ordering::SeqCst), 2);
        assert_eq!(futex.wake(1), 0);
    }

    #[async_std::test]
    async fn requeue() {
        static VALUE: AtomicI32 = AtomicI32::new(1);
        let futex = Futex::new(&VALUE);
        static REQUEUE_VALUE: AtomicI32 = AtomicI32::new(100);
        let requeue_futex = Futex::new(&REQUEUE_VALUE);

        let count = futex.wake(1);
        assert_eq!(count, 0);

        // inconsistent value should fail.
        assert_eq!(futex.wait(0).await, Err(ZxError::BAD_STATE));

        // spawn a new task to wait
        {
            let futex = futex.clone();
            async_std::task::spawn(async move {
                futex.wait(1).await.unwrap();
            });
        }
        // spawn a new task to requeue.
        {
            let futex = futex.clone();
            async_std::task::spawn(async move {
                async_std::task::sleep(Duration::from_millis(10)).await;
                VALUE.store(2, Ordering::SeqCst);

                let waiters = futex.inner.lock().waiter_queue.clone();
                assert_eq!(waiters.len(), 2);

                // inconsistent value should fail.
                assert_eq!(
                    futex.requeue(1, 1, 1, &requeue_futex, None),
                    Err(ZxError::BAD_STATE)
                );
                assert!(futex.requeue(2, 1, 1, &requeue_futex, None).is_ok());
                // 1 waiter waken, 1 waiter moved into `requeue_futex`.
                assert_eq!(futex.inner.lock().waiter_queue.len(), 0);
                assert_eq!(requeue_futex.inner.lock().waiter_queue.len(), 1);
                assert!(Arc::ptr_eq(
                    &requeue_futex.inner.lock().waiter_queue[0],
                    &waiters[1]
                ));
                // wake the requeued waiter.
                assert_eq!(requeue_futex.wake(1), 1);
            });
        }
        // wait for wake.
        futex.wait(1).await.unwrap();
        assert_eq!(VALUE.load(Ordering::SeqCst), 2);
    }

    #[async_std::test]
    async fn owner() {
        let root_job = Job::root();
        let proc = Process::create(&root_job, "proc", 0).expect("failed to create process");
        let thread = Thread::create(&proc, "thread", 0).expect("failed to create thread");

        static VALUE: AtomicI32 = AtomicI32::new(1);
        let futex = proc.get_futex(&VALUE);
        assert!(futex.owner().is_none());
        futex.inner.lock().set_owner(Some(thread.clone()));

        {
            let futex = futex.clone();
            let thread = thread.clone();
            async_std::task::spawn(async move {
                futex
                    .wait_with_owner(1, Some(thread.clone()), Some(thread))
                    .await
                    .unwrap();
            });
        }
        async_std::task::sleep(Duration::from_millis(10)).await;
        assert_eq!(
            futex
                .wait_with_owner(1, Some(thread.clone()), Some(thread.clone()))
                .await
                .unwrap_err(),
            ZxError::INVALID_ARGS
        );

        futex.inner.lock().set_owner(None);
        futex.wake_single_owner();
        assert!(Arc::ptr_eq(&futex.owner().unwrap(), &thread));
        assert_eq!(futex.wake(1), 0);
    }

    #[async_std::test]
    async fn time_out() {
        let root_job = Job::root();
        let proc = Process::create(&root_job, "proc", 0).expect("failed to create process");
        let thread = Thread::create(&proc, "thread", 0).expect("failed to create thread");

        static VALUE: AtomicI32 = AtomicI32::new(1);
        let futex = proc.get_futex(&VALUE);
        let future = futex.wait_with_owner(1, Some(thread.clone()), Some(thread.clone()));
        let result: ZxResult = thread
            .blocking_run(future, ThreadState::BlockedFutex, Duration::from_millis(1))
            .await;
        assert_eq!(result.unwrap_err(), ZxError::TIMED_OUT);
        assert_eq!(futex.wake(1), 0);
    }
}
 */