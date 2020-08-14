//! Zircon HAL implementation for bare metal environment on riscv64.
//!
//! This crate implements the following interfaces:
//! - `hal_pt_new`
//! - `hal_pt_map`
//! - `hal_pt_unmap`
//! - `hal_pt_protect`
//! - `hal_pt_query`
//! - `hal_pmem_read`
//! - `hal_pmem_write`
//!
//! And you have to implement these interfaces in addition:
//! - `hal_pt_map_kernel`
//! - `hal_pmem_base`

#![no_std]
#![feature(asm)]
#![feature(linkage)]
//#![deny(warnings)]

use alloc::boxed::Box;
use core::time::Duration;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use naive_timer::Timer;
use spin::Mutex;
use crate::kernel_hal::{
    defs::*,
    vdso,
    UserContext,
};
pub mod arch;
pub use arch::*;

#[allow(improper_ctypes)]
extern "C" {
    fn hal_pt_map_kernel(pt: *mut u8, current: *const u8);
    fn hal_frame_alloc() -> Option<PhysAddr>;
    fn hal_frame_dealloc(paddr: &PhysAddr);
    #[link_name = "hal_pmem_base"]
    static PMEM_BASE: usize;
}


#[repr(C)]
pub struct Thread {
    thread: usize,
}

impl Thread {
    #[export_name = "hal_thread_spawn"]
    pub fn spawn(
        future: Pin<Box<dyn Future<Output = ()> + Send + 'static>>,
        vmtoken: usize,
    ) -> Self {
        struct PageTableSwitchWrapper {
            inner: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
            vmtoken: usize,
        }
        impl Future for PageTableSwitchWrapper {
            type Output = ();
            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                unsafe {
                    arch::set_page_table(self.vmtoken);
                }
                self.inner.lock().as_mut().poll(cx)
            }
        }

        executor::spawn(PageTableSwitchWrapper {
            inner: Mutex::new(future),
            vmtoken,
        });
        Thread { thread: 0 }
    }

    #[export_name = "hal_thread_set_tid"]
    pub fn set_tid(_tid: u64, _pid: u64) {}

    #[export_name = "hal_thread_get_tid"]
    pub fn get_tid() -> (u64, u64) {
        (0, 0)
    }
}

#[export_name = "hal_context_run"]
pub fn context_run(context: &mut UserContext) {
    context.run();
}


/// Map kernel for the new page table.
///
/// `pt` is a page-aligned pointer to the root page table.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn map_kernel(pt: *mut u8, current: *const u8) {
    unsafe {
        hal_pt_map_kernel(pt, current);
    }
}

#[repr(C)]
pub struct Frame {
    paddr: PhysAddr,
}

impl Frame {
    pub fn alloc() -> Option<Self> {
        unsafe { hal_frame_alloc().map(|paddr| Frame { paddr }) }
    }

    pub fn dealloc(&mut self) {
        unsafe {
            hal_frame_dealloc(&self.paddr);
        }
    }

    #[export_name = "hal_zero_frame_paddr"]
    pub fn zero_frame_addr() -> PhysAddr {
        #[repr(align(0x1000))]
        struct Page([u8; PAGE_SIZE]);
        static ZERO_PAGE: Page = Page([0u8; PAGE_SIZE]);
        unsafe { ZERO_PAGE.0.as_ptr() as usize - PMEM_BASE }
    }
}

fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
    unsafe { PMEM_BASE + paddr }
}

/// Read physical memory from `paddr` to `buf`.
#[export_name = "hal_pmem_read"]
pub fn pmem_read(paddr: PhysAddr, buf: &mut [u8]) {
    trace!("pmem_read: addr={:#x}, len={:#x}", paddr, buf.len());
    unsafe {
        (phys_to_virt(paddr) as *const u8).copy_to_nonoverlapping(buf.as_mut_ptr(), buf.len());
    }
}

/// Write physical memory to `paddr` from `buf`.
#[export_name = "hal_pmem_write"]
pub fn pmem_write(paddr: PhysAddr, buf: &[u8]) {
    trace!("pmem_write: addr={:#x}, len={:#x}", paddr, buf.len());
    unsafe {
        buf.as_ptr()
            .copy_to_nonoverlapping(phys_to_virt(paddr) as _, buf.len());
    }
}

/// Zero `target` frame.
#[export_name = "hal_frame_zero"]
pub fn frame_zero_in_range(target: PhysAddr, start: usize, end: usize) {
    assert!(start < PAGE_SIZE && end <= PAGE_SIZE);
    trace!("frame_zero: {:#x}", target);
    unsafe {
        core::ptr::write_bytes(phys_to_virt(target + start) as *mut u8, 0, end - start);
    }
}

lazy_static! {
    pub static ref NAIVE_TIMER: Mutex<Timer> = Mutex::new(Timer::default());
}

#[export_name = "hal_timer_set"]
pub fn timer_set(deadline: Duration, callback: Box<dyn FnOnce(Duration) + Send + Sync>) {
    NAIVE_TIMER.lock().add(deadline, callback);
}

/*
#[export_name = "hal_timer_tick"]
pub fn timer_tick() {
    let now = arch::timer_now();
    NAIVE_TIMER.lock().expire(now);
}
*/

/*
/// Initialize the HAL.
pub fn init(config: Config) {
    unsafe {
        trapframe::init();
    }
    arch::init(config);
}
*/