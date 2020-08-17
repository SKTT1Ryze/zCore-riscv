use {
    bitmap_allocator::BitAlloc,
    buddy_system_allocator::LockedHeap,
    spin::Mutex,
    riscv::paging::{PageTable,PageTableFlags as EF},
};

use crate::println;
use super::config::*;

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
type FrameAlloc = bitmap_allocator::BitAlloc16M;

static FRAME_ALLOCATOR: Mutex<FrameAlloc> = Mutex::new(FrameAlloc::DEFAULT);

#[used]
#[export_name = "hal_pmem_base"]
static PMEM_BASE: usize = PHYSICAL_MEMORY_OFFSET;


/// Global heap allocator
///
/// Available after `memory::init_heap()`.
#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::new();

pub fn init_frame_allocator(start_frame: usize, end_frame: usize) {
    let mut ba = FRAME_ALLOCATOR.lock();
    ba.insert(start_frame..end_frame);
    info!("Frame allocator init end");
}


pub fn init_heap() {
    const MACHINE_ALIGN: usize = core::mem::size_of::<usize>();
    const HEAP_BLOCK: usize = KERNEL_HEAP_SIZE / MACHINE_ALIGN;
    static mut HEAP: [usize; HEAP_BLOCK] = [0; HEAP_BLOCK];
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP.as_ptr() as usize, HEAP_BLOCK * MACHINE_ALIGN);
    }
    info!("heap init end");
}

#[no_mangle]
pub extern "C" fn hal_frame_alloc() -> Option<usize> {
    // get the real address of the alloc frame
    let ret = FRAME_ALLOCATOR
        .lock()
        .alloc()
        .map(|id| id * PAGE_SIZE + MEMORY_OFFSET);
    trace!("Allocate frame: {:x?}", ret);
    ret
}

#[no_mangle]
pub extern "C" fn hal_frame_alloc_contiguous(page_num: usize, align_log2: usize) -> Option<usize> {
    let ret = FRAME_ALLOCATOR
        .lock()
        .alloc_contiguous(page_num, align_log2)
        .map(|id| id * PAGE_SIZE + MEMORY_OFFSET);
    trace!(
        "Allocate contiguous frames: {:x?} ~ {:x?}",
        ret,
        ret.map(|x| x + page_num)
    );
    ret
}

#[no_mangle]
pub extern "C" fn hal_frame_dealloc(target: &usize) {
    trace!("Deallocate frame: {:x}", *target);
    FRAME_ALLOCATOR
        .lock()
        .dealloc((*target - MEMORY_OFFSET) / PAGE_SIZE);
}

#[no_mangle]
pub extern "C" fn hal_pt_map_kernel(pt: &mut PageTable, current: &PageTable) {
    /* let ekernel = current[KERNEL_PM4].clone();
    let ephysical = current[PHYSICAL_MEMORY_PM4].clone();
    pt[KERNEL_PM4].set_addr(ekernel.addr(), ekernel.flags() | EF::GLOBAL);
    pt[PHYSICAL_MEMORY_PM4].set_addr(ephysical.addr(), ephysical.flags() | EF::GLOBAL);
     */
    println!("unimplemented in src/memory/head.rs hal_pt_map_kernel");
    //unimplemented!()
}

#[alloc_error_handler]
fn alloc_error_handler(_: alloc::alloc::Layout) -> ! {
    panic!("alloc error")
}
