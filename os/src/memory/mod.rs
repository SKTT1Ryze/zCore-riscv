//pub mod heap;
pub mod config;
pub mod heap_x;
pub use {
    config::*,
    //heap::*,
};

use crate::{print, println};
pub fn init() {
    //heap::init();
    heap_x::init_frame_allocator(START_FRAME, END_FRAME);
    heap_x::init_heap();
    // 允许内核读写用户态内存
    unsafe { riscv::register::sstatus::set_sum() };
    println!("mod memory initialized");
}