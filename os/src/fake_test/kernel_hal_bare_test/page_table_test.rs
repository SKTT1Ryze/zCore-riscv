use crate::kernel_hal_bare::PageTableImpl;
use crate::kernel_hal::defs::*;
use crate::kernel_hal::PageTableTrait;

pub fn page_table_test() {
    let mut new_page_table = PageTableImpl::new();
    let vaddr: VirtAddr = 0x1;
    let paddr: PhysAddr = 0x80201000;
    let flags = MMUFlags::USER;
    let map_result = new_page_table.map(vaddr, paddr, flags);
    println!("page table test pass here");
    match map_result {
        Ok(_) => {
            println!("map succeed");
        },
        Err(_err_mes) => {
            panic!("map error");
        }
    }
    let new_flags = MMUFlags::WRITE;
    let protect_result = new_page_table.protect(vaddr, new_flags);
    match protect_result {
        Ok(_) => {
            println!("protect succeed");
        },
        Err(_err_mes) => {
            panic!("protect error");
        }       
    }
    let query_result = new_page_table.query(vaddr);
    match query_result {
        Ok(phy_addr) => {
            println!("query succeed with Physical Address: {}", phy_addr);
        },
        Err(_err_mes) => {
            println!("query error");
        },
    }
    let table_paddr = new_page_table.table_phys();
    println!("get the physical address of root page table: {}", table_paddr);
    println!("page table test pass");
}

#[no_mangle]
pub extern "C" fn hal_pt_map_kernel(pt: &mut u8, current: *const u8) {
    println!("running hal_pt_map_kernel with pt: {} and current: {:?}", pt, current);
}