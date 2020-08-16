pub mod paged_test;
pub mod physical_test;
pub mod slice_test;

use crate::zircon_object::vm::*;

pub use {
    paged_test::*,
    physical_test::*,
    slice_test::*,
};

pub fn read_write(vmo: &VmObject) {
    let mut buf = [0u8; 4];
    vmo.write(0, &[0, 1, 2, 3]).unwrap();
    vmo.read(0, &mut buf).unwrap();
    assert_eq!(&buf, &[0, 1, 2, 3]);
}