use alloc::vec::Vec;
use crate::kernel_hal::fill_random;
use crate::{print, println};

pub fn fill_random_test() {
    let mut buffer = Vec::new();
    for i in 0u8..5u8 {
        buffer.push(i);
    }
    fill_random(&mut buffer);
    println!("fill_random_test pass");
}