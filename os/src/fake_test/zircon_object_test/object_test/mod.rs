pub mod handle_test;
pub mod right_test;
pub mod signal_test;


use crate::zircon_object::object::*;
use alloc::{string::String, format};
use {
    handle_test::*,
    right_test::*,
    signal_test::*,
};
use crate::{print, println};

pub fn test_trait_with_dummy() {
    let dummy = DummyObject::new();
    assert_eq!(dummy.name(), String::from(""));
    dummy.set_name("test");
    assert_eq!(dummy.name(), String::from("test"));
    dummy.signal_set(Signal::WRITABLE);
    assert_eq!(dummy.signal(), Signal::WRITABLE);
    dummy.signal_change(Signal::WRITABLE, Signal::READABLE);
    assert_eq!(dummy.signal(), Signal::READABLE);

    assert_eq!(dummy.get_child(0).unwrap_err(), ZxError::WRONG_TYPE);
    assert_eq!(dummy.peer().unwrap_err(), ZxError::NOT_SUPPORTED);
    assert_eq!(dummy.related_koid(), 0);
    assert_eq!(dummy.allowed_signals(), Signal::USER_ALL);
    
    assert_eq!(
        format!("{:?}", dummy),
        format!("DummyObject({}, \"test\")", dummy.id())
    );
    println!("test_trait_with_dummy pass");
}

/* #[cfg(test)]
mod tests {
    use super::*;
    use async_std::sync::Barrier;
    use std::time::Duration;

    #[async_std::test]
    async fn wait() {
        let object = DummyObject::new();
        let barrier = Arc::new(Barrier::new(2));
        async_std::task::spawn({
            let object = object.clone();
            let barrier = barrier.clone();
            async move {
                async_std::task::sleep(Duration::from_millis(20)).await;

                // Assert an irrelevant signal to test the `false` branch of the callback for `READABLE`.
                object.signal_set(Signal::USER_SIGNAL_0);
                object.signal_clear(Signal::USER_SIGNAL_0);
                object.signal_set(Signal::READABLE);
                barrier.wait().await;

                object.signal_set(Signal::WRITABLE);
            }
        });
        let object: Arc<dyn KernelObject> = object;

        let signal = object.wait_signal(Signal::READABLE).await;
        assert_eq!(signal, Signal::READABLE);
        barrier.wait().await;

        let signal = object.wait_signal(Signal::WRITABLE).await;
        assert_eq!(signal, Signal::READABLE | Signal::WRITABLE);
    }

    #[async_std::test]
    async fn wait_many() {
        let objs = [DummyObject::new(), DummyObject::new()];
        let barrier = Arc::new(Barrier::new(2));
        async_std::task::spawn({
            let objs = objs.clone();
            let barrier = barrier.clone();
            async move {
                async_std::task::sleep(Duration::from_millis(20)).await;

                objs[0].signal_set(Signal::READABLE);
                barrier.wait().await;

                objs[1].signal_set(Signal::WRITABLE);
            }
        });
        let obj0: Arc<dyn KernelObject> = objs[0].clone();
        let obj1: Arc<dyn KernelObject> = objs[1].clone();

        let signals = wait_signal_many(&[
            (obj0.clone(), Signal::READABLE),
            (obj1.clone(), Signal::READABLE),
        ])
        .await;
        assert_eq!(signals, [Signal::READABLE, Signal::empty()]);
        barrier.wait().await;

        let signals = wait_signal_many(&[
            (obj0.clone(), Signal::WRITABLE),
            (obj1.clone(), Signal::WRITABLE),
        ])
        .await;
        assert_eq!(signals, [Signal::READABLE, Signal::WRITABLE]);
    }
} */

pub fn test_all_in_object_test() {
    test_trait_with_dummy();
    test_ojb_type_unknown();
    test_get_info();
    test_try_from();
    test_verify_user_signal();
    println!("all test in object_test pass");
}