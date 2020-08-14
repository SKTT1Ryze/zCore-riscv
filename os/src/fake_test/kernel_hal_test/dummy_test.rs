use alloc::vec::Vec;
use crate::kernel_hal::fill_random;
pub fn fill_random_test() {
    println!("test fill_random");
    let mut buffer = Vec::new();
    for i in 0u8..5u8 {
        buffer.push(i);
    }
    fill_random(&mut buffer);
}