use crate::zircon_object::object::*;
use alloc::sync::Arc;

pub fn kobject_test() {
    use alloc::format;
    let dummy = DummyObject::new();
    let object: Arc<dyn KernelObject> = dummy;
    assert_eq!(object.type_name(), "DummyObject");
    assert_eq!(object.name(), "");
    object.set_name("dummy");
    assert_eq!(object.name(), "dummy");
    assert_eq!(object.cookie(), "");
    object.set_cookie("test");
    assert_eq!(object.cookie(), "test");
    assert_eq!(
        format!("{:?}",object),
        format!("DummyObject({}, \"dummy\", \"{}\")", object.id(), object.cookie())
    );
    let _result: Arc<DummyObject> = object.downcast_arc::<DummyObject>().unwrap();
    println!("test {} pass", "impl_kobject");
}