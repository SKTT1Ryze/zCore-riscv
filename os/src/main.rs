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
#![feature(linkage)]

#[macro_use]
mod console;
mod panic;
mod sbi;
mod zircon_object;
mod memory;
mod kernel_hal;
mod kernel_hal_bare;
mod fake_test;

extern crate alloc;

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

use zircon_object::object::{
    KernelObject,
    DummyObject,
};

use fake_test::{
    trapframe_test,
    kobject_test,
    alloc_test,
    fill_random_test,
    frame_test,
};

use alloc::sync::Arc;


//entry
global_asm!(include_str!("asm/entry.asm"));

// the first function to be called after _start
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    println!("Welcome to zCore on riscv64");
    memory::init();
    kobject_test();
    alloc_test();
    trapframe_test();
    fill_random_test();
    frame_test();
    panic!("Hi, panic here...")
}

