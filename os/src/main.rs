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
#![feature(drain_filter)]
#![feature(get_mut_unchecked)]
#![feature(naked_functions)]
#![feature(ptr_offset_from)]
#![feature(range_is_empty)]
#![feature(new_uninit)]
#![feature(const_in_array_repeat_expressions)]

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

use fake_test::{
    trapframe_test,
    alloc_test,
    fill_random_test,
    frame_test,
    pmem_test,
    page_table_test,
    zircon_object_test::object_test::test_all_in_object_test,
    zircon_object_test::signal_test::test_all_in_signal_test,
    zircon_object_test::task_test::test_all_in_task_test,
    zircon_object_test::ipc_test::test_all_in_ipc_test,
    zircon_object_test::vm_test::test_all_in_vm_test,
};



//entry
global_asm!(include_str!("asm/entry.asm"));

// the first function to be called after _start
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    println!("Welcome to zCore on riscv64");
    memory::init();
    alloc_test();
    trapframe_test();
    fill_random_test();
    frame_test();
    //pmem_test();
    //page_table_test();
    test_all_in_object_test();
    test_all_in_signal_test();
    test_all_in_task_test();
    test_all_in_ipc_test();
    //test_all_in_vm_test();
    panic!("Panic at the end...")
}

