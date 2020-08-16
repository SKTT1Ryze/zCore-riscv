use crate::zircon_object::object::*;
use alloc::sync::Arc;
use crate::{print, println};

pub fn test_ojb_type_unknown() {
    let obj: Arc<dyn KernelObject> = DummyObject::new();
    assert_eq!(33, obj_type(&obj));
    println!("test_ojb_type_unknown pass");
}

pub fn test_get_info() {
    let obj = crate::zircon_object::task::Job::root();
    let handle1 = Handle::new(obj.clone(), Rights::DEFAULT_JOB);
    let info1 = handle1.get_info();
    assert_eq!(info1.obj_type_test(), 17);
    assert_eq!(info1.props_test(), 1);

    let handle_info = handle1.get_handle_info();
    assert_eq!(handle_info.obj_type_test(), 17);

    let handle2 = Handle::new(obj, Rights::READ);
    let info2 = handle2.get_info();
    assert_eq!(info2.props_test(), 0);

    // Let struct lines counted covered.
    // See https://github.com/mozilla/grcov/issues/450
    let _ = HandleBasicInfo::default();
    println!("test_get_info pass");
}