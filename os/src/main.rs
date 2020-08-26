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
#![feature(lang_items)]
#![feature(bool_to_option)]

/* #[macro_use]
mod console; */
mod panic;
mod sbi;
mod zircon_object;
mod memory;
mod kernel_hal;
mod kernel_hal_bare;
mod fake_test;
mod zircon_syscall;
mod zircon_loader;
mod linux_object;
mod linux_syscall;
mod linux_loader;
//mod lang;

#[macro_use]
mod logging;

#[macro_use]
extern crate alloc;

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

extern crate bit;


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

use crate::zircon_loader::{simple_run_userboot_zircon, Images};
use crate::linux_loader::{run};
//entry
global_asm!(include_str!("asm/entry.asm"));

// the first function to be called after _start
#[no_mangle]
pub extern "C" fn rust_main(ramfs_data: &'static mut [u8], cmdline: &str) -> ! {
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
    test_all_in_ipc_test();
    test_all_in_task_test();
    //test_all_in_vm_test();
    //run_with_zircon_loader(ramfs_data, cmdline);
    run_with_linux_loader(ramfs_data, cmdline);
    unreachable!();
}

fn run_with_zircon_loader(ramfs_data: &[u8], cmdline: &str) {
    let images = Images::<&[u8]> {
        userboot: include_bytes!("./hello"),
        vdso: include_bytes!("./hello_world"),
        zbi: ramfs_data,
    };
    //let _proc = just_run_userboot(&images, cmdline);
    let _proc = simple_run_userboot_zircon(&images, cmdline);
    run_loop();
}

fn run_with_linux_loader(ramfs_data: &'static mut [u8], _cmdline: &str) {
    use alloc::boxed::Box;
    use alloc::sync::Arc;
    use alloc::vec;
    use linux_object::fs::MemBuf;
    use linux_object::fs::STDIN;
    println!("run with linux loader");
    crate::kernel_hal_bare::serial_set_callback(Box::new({
        move || {
            let mut buffer = [0; 255];
            let len = kernel_hal_bare::serial_read(&mut buffer);
            for c in &buffer[..len] {
                STDIN.push((*c).into());
                kernel_hal_bare::serial_write(alloc::format!("{}", *c as char).as_str());
            }
            false
        }
    }));

    let args = vec!["/bin/busybox".into(), "sh".into()];
    let envs = vec!["PATH=/usr/sbin:/usr/bin:/sbin:/bin:/usr/x86_64-alpine-linux-musl/bin".into()];

    let device = Arc::new(MemBuf::new(ramfs_data));
    let rootfs = rcore_fs_sfs::SimpleFileSystem::open(device).unwrap();
    let _proc = linux_loader::run(args, envs, rootfs);
    run_loop();
}

fn run_loop() -> ! {
    let mut counter = 0;
    loop {
        counter += 1;
        if counter%1000 == 0 {
            //println!("fake timer tick");
        }
    }
}