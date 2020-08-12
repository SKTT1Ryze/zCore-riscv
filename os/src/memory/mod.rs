pub mod heap;
pub mod config;
use riscv;
pub use {
    config::*,
    heap::*,
};
pub fn init() {
    heap::init();
    // 允许内核读写用户态内存
    unsafe { riscv::register::sstatus::set_sum() };
    println!("mod memory initialized");
}