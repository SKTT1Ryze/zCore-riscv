use crate::kernel_hal_bare::{
    pmem_read,
    pmem_write
};
use crate::kernel_hal::defs::*;
use alloc::vec::Vec;

pub fn pmem_test() {
    let paddr: PhysAddr = 0x80220000;
    let mut buf = Vec::new();
    for _ in 0..5 {
        buf.push(0u8);
    }
    pmem_read(paddr, &mut buf);
    println!("pmem test pass here");
    for item in &buf {
        print!("{} ",item);
    }
    pmem_write(paddr, &buf);
    println!("test pmem");
}

const PHYSICAL_MEMORY_OFFSET: usize = 0xffff8000_00000000;
#[export_name = "hal_pmem_base"]
static PMEM_BASE: usize = PHYSICAL_MEMORY_OFFSET;