use crate::zircon_object::object::*;

pub fn test_try_from() {
    assert_eq!(Err(ZxError::INVALID_ARGS), Rights::try_from_test(0xffff_ffff));
    assert_eq!(Ok(Rights::SAME_RIGHTS), Rights::try_from_test(1 << 31));
    println!("test_try_from pass");
}