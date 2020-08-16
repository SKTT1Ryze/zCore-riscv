use crate::zircon_object::vm::*;

pub fn test_read_write_physical() {
    let vmo = VmObject::new_physical(0x1000, 2);
    let vmphy = vmo.test_inner();
    assert_eq!(vmphy.cache_policy(), CachePolicy::Uncached);
    super::read_write(&vmo);
    println!("test_read_write_physical pass");
}