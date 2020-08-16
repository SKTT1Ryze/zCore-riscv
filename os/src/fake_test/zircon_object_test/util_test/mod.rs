pub mod block_range_test;
pub mod elf_loader_test;
pub mod kcounter_test;

use crate::{print, println};

use {
    block_range_test::*,
    elf_loader_test::*,
    kcounter_test::*,
};

pub fn test_all_in_util_test() {
    test_block_iter();
    println!("all test in tuil_test pass");
}

