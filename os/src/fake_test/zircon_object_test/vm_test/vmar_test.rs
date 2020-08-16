use crate::zircon_object::vm::*;
use crate::zircon_object::ZxError;
use alloc::sync::Arc;

use crate::{print, println};

/// A valid virtual address base to mmap.
const MAGIC: usize = 0xdead_beaf;

struct Sample {
    root: Arc<VmAddressRegion>,
    child1: Arc<VmAddressRegion>,
    child2: Arc<VmAddressRegion>,
    grandson1: Arc<VmAddressRegion>,
    grandson2: Arc<VmAddressRegion>,
}

impl Sample {
    fn new() -> Self {
        let root = VmAddressRegion::new_root();
        let child1 = root
            .allocate_at(0, 0x2000, VmarFlags::CAN_MAP_RXW, PAGE_SIZE)
            .unwrap();
        let child2 = root
            .allocate_at(0x2000, 0x1000, VmarFlags::CAN_MAP_RXW, PAGE_SIZE)
            .unwrap();
        let grandson1 = child1
            .allocate_at(0, 0x1000, VmarFlags::CAN_MAP_RXW, PAGE_SIZE)
            .unwrap();
        let grandson2 = child1
            .allocate_at(0x1000, 0x1000, VmarFlags::CAN_MAP_RXW, PAGE_SIZE)
            .unwrap();
        Sample {
            root,
            child1,
            child2,
            grandson1,
            grandson2,
        }
    }
}

pub fn test_create_child_vmar() {
    let root_vmar = VmAddressRegion::new_root();
    let child = root_vmar
        .allocate_at(0, 0x2000, VmarFlags::CAN_MAP_RXW, PAGE_SIZE)
        .expect("failed to create child VMAR");

    // test invalid argument
    assert_eq!(
        root_vmar
            .allocate_at(0x2001, 0x1000, VmarFlags::CAN_MAP_RXW, PAGE_SIZE)
            .err(),
        Some(ZxError::INVALID_ARGS)
    );
    assert_eq!(
        root_vmar
            .allocate_at(0x2000, 1, VmarFlags::CAN_MAP_RXW, PAGE_SIZE)
            .err(),
        Some(ZxError::INVALID_ARGS)
    );
    assert_eq!(
        root_vmar
            .allocate_at(0, 0x1000, VmarFlags::CAN_MAP_RXW, PAGE_SIZE)
            .err(),
        Some(ZxError::INVALID_ARGS)
    );
    assert_eq!(
        child
            .allocate_at(0x1000, 0x2000, VmarFlags::CAN_MAP_RXW, PAGE_SIZE)
            .err(),
        Some(ZxError::INVALID_ARGS)
    );
    println!("test_create_child_vmar pass");
}

pub fn test_map() {
    let vmar = VmAddressRegion::new_root();
    let vmo = VmObject::new_paged(4);
    let flags = MMUFlags::READ | MMUFlags::WRITE;

    // invalid argument
    assert_eq!(
        vmar.map_at(0, vmo.clone(), 0x4000, 0x1000, flags),
        Err(ZxError::INVALID_ARGS)
    );
    assert_eq!(
        vmar.map_at(0, vmo.clone(), 0, 0x5000, flags),
        Err(ZxError::INVALID_ARGS)
    );
    assert_eq!(
        vmar.map_at(0, vmo.clone(), 0x1000, 1, flags),
        Err(ZxError::INVALID_ARGS)
    );
    assert_eq!(
        vmar.map_at(0, vmo.clone(), 1, 0x1000, flags),
        Err(ZxError::INVALID_ARGS)
    );

    vmar.map_at(0, vmo.clone(), 0, 0x4000, flags).unwrap();
    vmar.map_at(0x12000, vmo.clone(), 0x2000, 0x1000, flags)
        .unwrap();

    unsafe {
        ((vmar.addr() + 0x2000) as *mut usize).write(MAGIC);
        assert_eq!(((vmar.addr() + 0x12000) as *const usize).read(), MAGIC);
    }
    println!("test_map pass");
}

pub fn test_unmap_vmar() {
    let s = Sample::new();
    let base = s.root.addr();
    s.child1.unmap(base, 0x1000).unwrap();
    assert!(s.grandson1.is_dead());
    assert!(s.grandson2.is_alive());

    // partial overlap sub-region should fail.
    let s = Sample::new();
    let base = s.root.addr();
    assert_eq!(
        s.root.unmap(base + 0x1000, 0x2000),
        Err(ZxError::INVALID_ARGS)
    );

    // unmap nothing should success.
    let s = Sample::new();
    let base = s.root.addr();
    s.child1.unmap(base + 0x8000, 0x1000).unwrap();
    println!("test_unmap_vmar test");
}

pub fn test_destroy() {
    let s = Sample::new();
    s.child1.destroy().unwrap();
    assert!(s.child1.is_dead());
    assert!(s.grandson1.is_dead());
    assert!(s.grandson2.is_dead());
    assert!(s.child2.is_alive());
    // address space should be released
    assert!(s
        .root
        .allocate_at(0, 0x1000, VmarFlags::CAN_MAP_RXW, PAGE_SIZE)
        .is_ok());
    println!("test_destroy pass");
}

pub fn test_unmap_mapping() {
    //   +--------+--------+--------+--------+--------+
    // 1 [--------------------------|xxxxxxxx|--------]
    // 2 [xxxxxxxx|-----------------]
    // 3          [--------|xxxxxxxx]
    // 4          [xxxxxxxx]
    let vmar = VmAddressRegion::new_root();
    let base = vmar.addr();
    let vmo = VmObject::new_paged(5);
    let flags = MMUFlags::READ | MMUFlags::WRITE;
    vmar.map_at(0, vmo, 0, 0x5000, flags).unwrap();
    assert_eq!(vmar.count(), 1);
    assert_eq!(vmar.used_size(), 0x5000);

    // 0. unmap none.
    vmar.unmap(base + 0x5000, 0x1000).unwrap();
    assert_eq!(vmar.count(), 1);
    assert_eq!(vmar.used_size(), 0x5000);

    // 1. unmap middle.
    vmar.unmap(base + 0x3000, 0x1000).unwrap();
    assert_eq!(vmar.count(), 2);
    assert_eq!(vmar.used_size(), 0x4000);

    // 2. unmap prefix.
    vmar.unmap(base, 0x1000).unwrap();
    assert_eq!(vmar.count(), 2);
    assert_eq!(vmar.used_size(), 0x3000);

    // 3. unmap postfix.
    vmar.unmap(base + 0x2000, 0x1000).unwrap();
    assert_eq!(vmar.count(), 2);
    assert_eq!(vmar.used_size(), 0x2000);

    // 4. unmap all.
    vmar.unmap(base + 0x1000, 0x1000).unwrap();
    assert_eq!(vmar.count(), 1);
    assert_eq!(vmar.used_size(), 0x1000);
    println!("test_unmap_mapping pass");
}