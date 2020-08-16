use crate::zircon_object::signal::*;
use crate::zircon_object::object::*;
use crate::{print, println};

pub fn test_allowed_signals_1() {
    let event = Event::new();
    assert!(Signal::verify_user_signal(
        event.allowed_signals(),
        (Signal::USER_SIGNAL_5 | Signal::SIGNALED).bits().into()
    )
    .is_ok());
    println!("test_allowed_signals_1 pass");
}