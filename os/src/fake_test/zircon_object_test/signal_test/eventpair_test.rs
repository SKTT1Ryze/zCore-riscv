use crate::zircon_object::signal::*;
use crate::zircon_object::object::*;
use alloc::sync::Arc;
pub fn test_allowed_signals_2() {
    let (event0, event1) = EventPair::create();
    assert!(Signal::verify_user_signal(
        event0.allowed_signals(),
        (Signal::USER_SIGNAL_5 | Signal::SIGNALED).bits().into()
    )
    .is_ok());
    assert_eq!(event0.allowed_signals(), event1.allowed_signals());

    event0.peer().unwrap();
    println!("test_allowed_signals_2 pass");
}

pub fn test_peer_closed() {
    let (event0, event1) = EventPair::create();
    assert!(Arc::ptr_eq(&event0.peer().unwrap(), &event1));
    assert_eq!(event0.related_koid(), event1.id());

    drop(event1);
    assert_eq!(event0.signal(), Signal::PEER_CLOSED);
    assert_eq!(event0.peer().err(), Some(ZxError::PEER_CLOSED));
    assert_eq!(event0.related_koid(), 0);
    println!("peer_closed pass");
}