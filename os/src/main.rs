//! main.rs of zCore-riscv
#![no_std]
#![no_main]
//#![warn(missing_docs)]
//insert assemble file
#![feature(asm)]
#![feature(llvm_asm)]
#![feature(global_asm)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

#[macro_use]
mod console;
mod panic;
mod sbi;
mod object;
mod memory;
extern crate alloc;

use object::{
    KernelObject,
    DummyObject,
};
use alloc::sync::Arc;
//entry
global_asm!(include_str!("asm/entry.asm"));

// the first function to be called after _start
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    println!("Welcome to zCore on riscv64");
    memory::init();
    impl_kobject();
    alloc_test();
    panic!("Hi, panic here...")
}

fn impl_kobject() {
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

fn alloc_test() {
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    let v = Box::new(5);
    assert_eq!(*v, 5);
    core::mem::drop(v);
    let mut vec = Vec::new();
    for i in 0..10000 {
        vec.push(i);
    }
    assert_eq!(vec.len(), 10000);
    for (i, value) in vec.into_iter().enumerate() {
        assert_eq!(value, i);
    }
    println!("test {} pass", "alloc_test");
}