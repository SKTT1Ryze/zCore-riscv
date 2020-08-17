use super::*;
use super::vdso::VdsoConstants;
use acpi::Acpi;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::future::Future;
use core::ops::FnOnce;
use core::pin::Pin;
use core::time::Duration;
use crate::{print, println};
use crate::memory::heap_x::*;

use crate::kernel_hal_bare::{
    PageTableImpl,
};

type ThreadId = usize;

#[repr(C)]
pub struct Thread {
    id: ThreadId,
}

impl Thread {
    /// Spawn a new thread.
    #[linkage = "weak"]
    #[export_name = "hal_thread_spawn_unimplemented"]
    pub fn spawn(
        _future: Pin<Box<dyn Future<Output = ()> + Send + 'static>>,
        _vmtoken: usize,
    ) -> Self {
        Self {
            id: crate::kernel_hal_bare::Thread::spawn(_future, _vmtoken).get_thread()
        }
        /* println!("unimplemented in src/kernel_hal/dummy.rs impl Thread");
        unimplemented!() */
    }

    /// Set tid and pid of current task.
    #[linkage = "weak"]
    #[export_name = "hal_thread_set_tid_unimplemented"]
    pub fn set_tid(_tid: u64, _pid: u64) {
        println!("unimplemented in src/kernel_hal/dummy.rs impl Thread");
        unimplemented!()
    }

    /// Get tid and pid of current task.
    #[linkage = "weak"]
    #[export_name = "hal_thread_get_tid_unimplemented"]
    pub fn get_tid() -> (u64, u64) {
        println!("unimplemented in src/kernel_hal/dummy.rs impl Thread");
        unimplemented!()
    }
}

#[linkage = "weak"]
#[export_name = "hal_context_run_unimplemented"]
pub fn context_run(_context: &mut UserContext) {
    println!("unimplemented in src/kernel_hal/dummy.rs context_run");
    unimplemented!()
}

pub trait PageTableTrait: Sync + Send {
    /// Map the page of `vaddr` to the frame of `paddr` with `flags`.
    fn map(&mut self, _vaddr: VirtAddr, _paddr: PhysAddr, _flags: MMUFlags) -> Result<(), ()>;

    /// Unmap the page of `vaddr`.
    fn unmap(&mut self, _vaddr: VirtAddr) -> Result<(), ()>;

    /// Change the `flags` of the page of `vaddr`.
    fn protect(&mut self, _vaddr: VirtAddr, _flags: MMUFlags) -> Result<(), ()>;

    /// Query the physical address which the page of `vaddr` maps to.
    fn query(&mut self, _vaddr: VirtAddr) -> Result<PhysAddr, ()>;

    /// Get the physical address of root page table.
    fn table_phys(&self) -> PhysAddr;

    fn map_many(
        &mut self,
        mut vaddr: VirtAddr,
        paddrs: &[PhysAddr],
        flags: MMUFlags,
    ) -> Result<(), ()> {
        for &paddr in paddrs {
            self.map(vaddr, paddr, flags)?;
            vaddr += PAGE_SIZE;
        }
        Ok(())
    }

    fn map_cont(
        &mut self,
        mut vaddr: VirtAddr,
        paddr: PhysAddr,
        pages: usize,
        flags: MMUFlags,
    ) -> Result<(), ()> {
        for i in 0..pages {
            let paddr = paddr + i * PAGE_SIZE;
            self.map(vaddr, paddr, flags)?;
            vaddr += PAGE_SIZE;
        }
        Ok(())
    }

    fn unmap_cont(&mut self, vaddr: VirtAddr, pages: usize) -> Result<(), ()> {
        for i in 0..pages {
            self.unmap(vaddr + i * PAGE_SIZE)?;
        }
        Ok(())
    }
}

/// Page Table
#[repr(C)]
pub struct PageTable {
    table_phys: PhysAddr,
}

impl PageTable {
    /// Get current page table
    #[linkage = "weak"]
    #[export_name = "hal_pt_current_unimplemented"]
    pub fn current() -> Self {
        println!("unimplemented in src/kernel_hal/dummy.rs impl PageTable current()");
        unimplemented!()
    }

    /// Create a new `PageTable`.
    #[allow(clippy::new_without_default)]
    #[linkage = "weak"]
    #[export_name = "hal_pt_new_unimplemented"]
    pub fn new() -> Self {
        /* println!("unimplemented in src/kernel_hal/dummy.rs impl PageTable new()");
        unimplemented!() */
        Self {
            table_phys: PageTableImpl::fake_new()
        }
    }
}

impl PageTableTrait for PageTable {
    /// Map the page of `vaddr` to the frame of `paddr` with `flags`.
    #[linkage = "weak"]
    #[export_name = "hal_pt_map_unimplemented"]
    fn map(&mut self, _vaddr: VirtAddr, _paddr: PhysAddr, _flags: MMUFlags) -> Result<(), ()> {
        unimplemented!()
    }
    /// Unmap the page of `vaddr`.
    #[linkage = "weak"]
    #[export_name = "hal_pt_unmap_unimplemented"]
    fn unmap(&mut self, _vaddr: VirtAddr) -> Result<(), ()> {
        unimplemented!()
    }
    /// Change the `flags` of the page of `vaddr`.
    #[linkage = "weak"]
    #[export_name = "hal_pt_protect_unimplemented"]
    fn protect(&mut self, _vaddr: VirtAddr, _flags: MMUFlags) -> Result<(), ()> {
        unimplemented!()
    }
    /// Query the physical address which the page of `vaddr` maps to.
    #[linkage = "weak"]
    #[export_name = "hal_pt_query_unimplemented"]
    fn query(&mut self, _vaddr: VirtAddr) -> Result<PhysAddr, ()> {
        unimplemented!()
    }
    /// Get the physical address of root page table.
    #[linkage = "weak"]
    #[export_name = "hal_pt_table_phys_unimplemented"]
    fn table_phys(&self) -> PhysAddr {
        self.table_phys
    }
    #[linkage = "weak"]
    #[export_name = "hal_pt_unmap_cont_unimplemented"]
    fn unmap_cont(&mut self, vaddr: VirtAddr, pages: usize) -> Result<(), ()> {
        for i in 0..pages {
            self.unmap(vaddr + i * PAGE_SIZE)?;
        }
        Ok(())
    }
}

#[repr(C)]
pub struct PhysFrame {
    paddr: PhysAddr,
}

impl PhysFrame {
    #[linkage = "weak"]
    #[export_name = "hal_frame_alloc_unimplemented"]
    pub extern "C" fn alloc() -> Option<Self> {
        /* println!("unimplemented in src/kernel_hal/dummy.rs impl PhysFrame");
        unimplemented!() */
        match hal_frame_alloc() {
            None => {
                panic!("src/kernel_hal/dummy.rs impl PhyFrame: hal_frame_alloc() return None")
            },
            Some(paddr) => {
                Some(
                    Self {
                        paddr
                    }
                )
            },
        }
    }

    #[linkage = "weak"]
    #[export_name = "hal_frame_alloc_contiguous_unimplemented"]
    pub extern "C" fn alloc_contiguous_base(_size: usize, _align_log2: usize) -> Option<PhysAddr> {
        println!("unimplemented in src/kernel_hal/dummy.rs impl PhysFrame");
        unimplemented!()
    }

    pub fn alloc_contiguous(size: usize, align_log2: usize) -> Vec<Self> {
        PhysFrame::alloc_contiguous_base(size, align_log2).map_or(Vec::new(), |base| {
            (0..size)
                .map(|i| PhysFrame {
                    paddr: base + i * PAGE_SIZE,
                })
                .collect()
        })
    }

    pub fn addr(&self) -> PhysAddr {
        self.paddr
    }

    #[linkage = "weak"]
    #[export_name = "hal_zero_frame_paddr_unimplemented"]
    pub fn zero_frame_addr() -> PhysAddr {
        crate::kernel_hal_bare::Frame::zero_frame_addr()
        /* println!("unimplemented in src/kernel_hal/dummy.rs impl PhysFrame");
        unimplemented!() */
    }
}

impl Drop for PhysFrame {
    #[linkage = "weak"]
    #[export_name = "hal_frame_dealloc_unimplemented"]
    fn drop(&mut self) {
        println!("unimplemented in src/kernel_hal/dummy.rs impl Drop for PhysFrame");
        unimplemented!()
    }
}

/// Read physical memory from `paddr` to `buf`.
#[linkage = "weak"]
#[export_name = "hal_pmem_read_unimplemented"]
pub fn pmem_read(_paddr: PhysAddr, _buf: &mut [u8]) {
    println!("unimplemented in src/kernel_hal/dummy.rs pmem_read");
    unimplemented!()
}

/// Write physical memory to `paddr` from `buf`.
#[linkage = "weak"]
#[export_name = "hal_pmem_write_unimplemented"]
pub fn pmem_write(_paddr: PhysAddr, _buf: &[u8]) {
    println!("unimplemented in src/kernel_hal/dummy.rs pmem_write");
    unimplemented!()
}

/// Copy content of `src` frame to `target` frame.
#[linkage = "weak"]
#[export_name = "hal_frame_copy_unimplemented"]
pub fn frame_copy(_src: PhysAddr, _target: PhysAddr) {
    println!("unimplemented in src/kernel_hal/dummy.rs frame_copy");
    unimplemented!()
}

/// Zero `target` frame.
#[linkage = "weak"]
#[export_name = "hal_frame_zero_unimplemented"]
pub fn frame_zero_in_range(_target: PhysAddr, _start: usize, _end: usize) {
    /* println!("unimplemented in src/kernel_hal/dummy.rs frame_zero_in_range");
    unimplemented!() */
    crate::kernel_hal_bare::frame_zero_in_range(_target, _start, _end);
}

/// Flush the physical frame.
#[linkage = "weak"]
#[export_name = "hal_frame_flush_unimplemented"]
pub fn frame_flush(_target: PhysAddr) {
    println!("unimplemented in src/kernel_hal/dummy.rs frame_flush");
    unimplemented!()
}

/// Register a callback of serial readable event.
#[linkage = "weak"]
#[export_name = "hal_serial_set_callback_unimplemented"]
pub fn serial_set_callback(_callback: Box<dyn FnOnce() + Send + Sync>) {
    println!("unimplemented in src/kernel_hal/dummy.rs serial_set_callback");
    unimplemented!()
}

/// Read a string from console.
#[linkage = "weak"]
#[export_name = "hal_serial_read_unimplemented"]
pub fn serial_read(_buf: &mut [u8]) -> usize {
    println!("unimplemented in src/kernel_hal/dummy.rs serial_read");
    unimplemented!()
}

/// Output a string to console.
#[linkage = "weak"]
#[export_name = "hal_serial_write_unimplemented"]
pub fn serial_write(_s: &str) {
    println!("unimplemented in src/kernel_hal/dummy.rs serial_write");
    unimplemented!()
}

/// Get current time.
#[linkage = "weak"]
#[export_name = "hal_timer_now_unimplemented"]
pub fn timer_now() -> Duration {
    println!("unimplemented in src/kernel_hal/dummy.rs timer_now");
    unimplemented!()
}

/// Set a new timer. After `deadline`, the `callback` will be called.
#[linkage = "weak"]
#[export_name = "hal_timer_set_unimplemented"]
pub fn timer_set(_deadline: Duration, _callback: Box<dyn FnOnce(Duration) + Send + Sync>) {
    println!("unimplemented in src/kernel_hal/dummy.rs timer_set");
    unimplemented!()
}

/// Check timers, call when timer interrupt happened.
#[linkage = "weak"]
#[export_name = "hal_timer_tick_unimplemented"]
pub fn timer_tick() {
    println!("unimplemented in src/kernel_hal/dummy.rs timer_tick");
    unimplemented!()
}

pub struct InterruptManager {}
impl InterruptManager {
    /// Handle IRQ.
    #[linkage = "weak"]
    #[export_name = "hal_irq_handle_unimplemented"]
    pub fn handle(_irq: u8) {
        println!("unimplemented in src/kernel_hal/dummy.rs impl InterruptManager");
        unimplemented!()
    }
    ///
    #[linkage = "weak"]
    #[export_name = "hal_ioapic_set_handle_unimplemented"]
    pub fn set_ioapic_handle(_global_irq: u32, _handle: Box<dyn Fn() + Send + Sync>) -> Option<u8> {
        println!("unimplemented in src/kernel_hal/dummy.rs impl InterruptManager");
        unimplemented!()
    }
    /// Add an interrupt handle to an irq
    #[linkage = "weak"]
    #[export_name = "hal_irq_add_handle_unimplemented"]
    pub fn add_handle(_global_irq: u8, _handle: Box<dyn Fn() + Send + Sync>) -> Option<u8> {
        println!("unimplemented in src/kernel_hal/dummy.rs impl InterruptManager");
        unimplemented!()
    }
    ///
    #[linkage = "weak"]
    #[export_name = "hal_ioapic_reset_handle_unimplemented"]
    pub fn reset_ioapic_handle(_global_irq: u32) -> bool {
        println!("unimplemented in src/kernel_hal/dummy.rs impl InterruptManager");
        unimplemented!()
    }
    /// Remove the interrupt handle of an irq
    #[linkage = "weak"]
    #[export_name = "hal_irq_remove_handle_unimplemented"]
    pub fn remove_handle(_irq: u8) -> bool {
        println!("unimplemented in src/kernel_hal/dummy.rs impl InterruptManager");
        unimplemented!()
    }
    /// Allocate contiguous positions for irq
    #[linkage = "weak"]
    #[export_name = "hal_irq_allocate_block_unimplemented"]
    pub fn allocate_block(_irq_num: u32) -> Option<(usize, usize)> {
        println!("unimplemented in src/kernel_hal/dummy.rs impl InterruptManager");
        unimplemented!()
    }
    #[linkage = "weak"]
    #[export_name = "hal_irq_free_block_unimplemented"]
    pub fn free_block(_irq_start: u32, _irq_num: u32) {
        println!("unimplemented in src/kernel_hal/dummy.rs impl InterruptManager");
        unimplemented!()
    }
    #[linkage = "weak"]
    #[export_name = "hal_irq_overwrite_handler_unimplemented"]
    pub fn overwrite_handler(_msi_id: u32, _handle: Box<dyn Fn() + Send + Sync>) -> bool {
        println!("unimplemented in src/kernel_hal/dummy.rs impl InterruptManager");
        unimplemented!()
    }

    /// Enable IRQ.
    #[linkage = "weak"]
    #[export_name = "hal_irq_enable_unimplemented"]
    pub fn enable(_global_irq: u32) {
        println!("unimplemented in src/kernel_hal/dummy.rs impl InterruptManager");
        unimplemented!()
    }

    /// Disable IRQ.
    #[linkage = "weak"]
    #[export_name = "hal_irq_disable_unimplemented"]
    pub fn disable(_global_irq: u32) {
        println!("unimplemented in src/kernel_hal/dummy.rs impl InterruptManager");
        unimplemented!()
    }
    /// Get IO APIC maxinstr
    #[linkage = "weak"]
    #[export_name = "hal_irq_maxinstr_unimplemented"]
    pub fn maxinstr(_irq: u32) -> Option<u8> {
        println!("unimplemented in src/kernel_hal/dummy.rs impl InterruptManager");
        unimplemented!()
    }
    #[linkage = "weak"]
    #[export_name = "hal_irq_configure_unimplemented"]
    pub fn configure(
        _irq: u32,
        _vector: u8,
        _dest: u8,
        _level_trig: bool,
        _active_high: bool,
    ) -> bool {
        println!("unimplemented in src/kernel_hal/dummy.rs impl InterruptManager");
        unimplemented!()
    }
    #[linkage = "weak"]
    #[export_name = "hal_irq_isvalid_unimplemented"]
    pub fn is_valid(_irq: u32) -> bool {
        println!("unimplemented in src/kernel_hal/dummy.rs impl InterruptManager");
        unimplemented!()
    }
}

/// Get platform specific information.
#[linkage = "weak"]
#[export_name = "hal_vdso_constants_unimplemented"]
pub fn vdso_constants() -> VdsoConstants {
    println!("unimplemented in src/kernel_hal/dummy.rs vdso_constants");
    unimplemented!()
}

/// Get fault address of the last page fault.
#[linkage = "weak"]
#[export_name = "fetch_fault_vaddr_unimplemented"]
pub fn fetch_fault_vaddr() -> VirtAddr {
    unimplemented!()
}

/// Get physical address of `acpi_rsdp` and `smbios` on x86_64.
#[linkage = "weak"]
#[export_name = "hal_pc_firmware_tables_unimplemented"]
pub fn pc_firmware_tables() -> (u64, u64) {
    unimplemented!()
}

/// Get ACPI Table
#[linkage = "weak"]
#[export_name = "hal_acpi_table_unimplemented"]
pub fn get_acpi_table() -> Option<Acpi> {
    unimplemented!()
}

/// IO Ports access on x86 platform
#[linkage = "weak"]
#[export_name = "hal_outpd_unimplemented"]
pub fn outpd(_port: u16, _value: u32) {
    unimplemented!()
}

#[linkage = "weak"]
#[export_name = "hal_inpd_unimplemented"]
pub fn inpd(_port: u16) -> u32 {
    unimplemented!()
}

/// Get local APIC ID
#[linkage = "weak"]
#[export_name = "hal_apic_local_id_unimplemented"]
pub fn apic_local_id() -> u8 {
    unimplemented!()
}

/// Fill random bytes to the buffer
#[cfg(target_arch = "x86_64")]
pub fn fill_random(buf: &mut [u8]) {
    // TODO: optimize
    for x in buf.iter_mut() {
        let mut r = 0;
        unsafe {
            core::arch::x86_64::_rdrand16_step(&mut r);
        }
        *x = r as _;
    }
}

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
pub fn fill_random(_buf: &mut [u8]) {
    // TODO
    println!("fill_random on riscv");
}

#[cfg(target_arch = "aarch64")]
pub fn fill_random(_buf: &mut [u8]) {
    // TODO
}

