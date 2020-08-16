use crate::zircon_object::vm::*;
use crate::{print, println};

pub fn test_read_write_paged() {
    let vmo = VmObject::new_paged(2);
    super::read_write(&*vmo);
    println!("test_read_write_paged pass");
}

pub fn test_create_child_paged() {
    let vmo = VmObject::new_paged(1);
    let child_vmo = vmo.create_child(false, 0, PAGE_SIZE).unwrap();

    // write to parent and make sure clone doesn't see it
    vmo.test_write(0, 1);
    assert_eq!(vmo.test_read(0), 1);
    assert_eq!(child_vmo.test_read(0), 0);

    // write to clone and make sure parent doesn't see it
    child_vmo.test_write(0, 2);
    assert_eq!(vmo.test_read(0), 1);
    assert_eq!(child_vmo.test_read(0), 2);
    println!("test_create_child_paged pass");
}

pub fn test_zero_page_write() {
    let vmo0 = VmObject::new_paged(1);
    let vmo1 = vmo0.create_child(false, 0, PAGE_SIZE).unwrap();
    let vmo2 = vmo0.create_child(false, 0, PAGE_SIZE).unwrap();
    let vmos = [vmo0, vmo1, vmo2];
    let origin = vmo_page_bytes();

    // no committed pages
    for vmo in &vmos {
        assert_eq!(vmo.get_info().committed_bytes(), 0);
    }

    // copy-on-write
    for i in 0..3 {
        vmos[i].test_write(0, i as u8);
        for j in 0..3 {
            assert_eq!(vmos[j].test_read(0), if j <= i { j as u8 } else { 0 });
            assert_eq!(
                vmos[j].get_info().committed_bytes() as usize,
                if j <= i { PAGE_SIZE } else { 0 }
            );
        }
        assert_eq!(vmo_page_bytes() - origin, (i + 1) * PAGE_SIZE);
    }
    println!("test_zero_page_write pass");
}

pub fn test_overflow() {
    let vmo0 = VmObject::new_paged(2);
    vmo0.test_write(0, 1);
    let vmo1 = vmo0.create_child(false, 0, 2 * PAGE_SIZE).unwrap();
    vmo1.test_write(1, 2);
    let vmo2 = vmo1.create_child(false, 0, 3 * PAGE_SIZE).unwrap();
    vmo2.test_write(2, 3);
    assert_eq!(vmo0.get_info().committed_bytes() as usize, PAGE_SIZE);
    assert_eq!(vmo1.get_info().committed_bytes() as usize, PAGE_SIZE);
    assert_eq!(vmo2.get_info().committed_bytes() as usize, PAGE_SIZE);
    println!("test_overflow pass");
}
