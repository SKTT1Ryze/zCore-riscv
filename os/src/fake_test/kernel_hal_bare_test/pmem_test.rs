use crate::kernel_hal_bare::{
    pmem_read,
    pmem_write
};
use crate::kernel_hal::defs::*;
use alloc::vec::Vec;
use crate::{print, println};

pub fn pmem_test() {
    //let paddr: PhysAddr = 0x80220000;
    let paddr: PhysAddr = 0x1;
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

