use crate::zircon_object::object::*;

pub fn test_verify_user_signal() {
    assert_eq!(
        Err(ZxError::INVALID_ARGS),
        Signal::verify_user_signal(Signal::USER_ALL, 1 << 0)
    );

    assert_eq!(
        Ok(Signal::USER_SIGNAL_0),
        Signal::verify_user_signal(Signal::USER_ALL, 1 << 24)
    );
    println!("test_verify_user_signal pass");
}