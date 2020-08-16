use crate::zircon_object::util::block_range::*;
use crate::{print, println};

pub fn test_block_iter() {
    let mut iter = BlockIter {
        begin: 0x123,
        end: 0x2018,
        block_size_log2: 12,
    };
    assert_eq!(
        iter.next(),
        Some(BlockRange {
            block: 0,
            begin: 0x123,
            end: 0x1000,
            block_size_log2: 12
        })
    );
    assert_eq!(
        iter.next(),
        Some(BlockRange {
            block: 1,
            begin: 0,
            end: 0x1000,
            block_size_log2: 12
        })
    );
    assert_eq!(
        iter.next(),
        Some(BlockRange {
            block: 2,
            begin: 0,
            end: 0x18,
            block_size_log2: 12
        })
    );
    assert_eq!(iter.next(), None);
    println!("test_block_iter pass");
}