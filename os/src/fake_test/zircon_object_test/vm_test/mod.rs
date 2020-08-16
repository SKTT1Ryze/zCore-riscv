pub mod vmo_test;
pub mod stream_test;
pub mod vmar_test;

use crate::{print, println};

use {
    vmo_test::*,
    stream_test::*,
    vmar_test::*,
    crate::zircon_object::vm::*,
};

pub fn test_all_in_vm_test() {
    test_read_write_paged();
    test_create_child_paged();
    test_zero_page_write();
    test_overflow();
    test_read_write_physical();
    test_round_pages();
    test_create_child_vmar();
    test_map();
    test_unmap_vmar();
    test_destroy();
    test_unmap_mapping();
    println!("all test in vm_test pass");
}

pub fn test_round_pages() {
    assert_eq!(roundup_pages(0), 0);
    assert_eq!(roundup_pages(core::usize::MAX), 0);
    assert_eq!(
        roundup_pages(core::usize::MAX - PAGE_SIZE + 1),
        core::usize::MAX - PAGE_SIZE + 1
    );
    assert_eq!(roundup_pages(PAGE_SIZE * 3 - 1), PAGE_SIZE * 3);
    println!("test_round_pages pass");
}