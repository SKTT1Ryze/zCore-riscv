use crate::zircon_object::ipc::*;
use crate::zircon_object::object::*;
use alloc::sync::Arc;
use alloc::{vec, vec::Vec};
pub fn test_test_basics_fifo() {
    let (end0, end1) = Fifo::create(10, 5);
    assert!(Arc::ptr_eq(
        &end0.peer().unwrap().downcast_arc().unwrap(),
        &end1
    ));
    assert_eq!(end0.related_koid(), end1.id());

    drop(end1);
    assert_eq!(end0.peer().unwrap_err(), ZxError::PEER_CLOSED);
    assert_eq!(end0.related_koid(), 0);
    println!("test_test_basics_fifo pass");
}

pub fn test_read_write_fifo() {
    let (end0, end1) = Fifo::create(2, 5);

    assert_eq!(
        end0.write(4, &[0; 9], 1).unwrap_err(),
        ZxError::OUT_OF_RANGE
    );
    assert_eq!(
        end0.write(5, &[0; 0], 0).unwrap_err(),
        ZxError::OUT_OF_RANGE
    );
    let data = (0..15).collect::<Vec<u8>>();
    assert_eq!(end0.write(5, data.as_slice(), 3).unwrap(), 2);
    assert_eq!(
        end0.write(5, data.as_slice(), 3).unwrap_err(),
        ZxError::SHOULD_WAIT
    );

    let mut buf = [0; 15];
    assert_eq!(
        end1.read(4, &mut [0; 4], 1).unwrap_err(),
        ZxError::OUT_OF_RANGE
    );
    assert_eq!(end1.read(5, &mut [], 0).unwrap_err(), ZxError::OUT_OF_RANGE);
    assert_eq!(end1.read(5, &mut buf, 3).unwrap(), 2);
    let mut data = (0..10).collect::<Vec<u8>>();
    data.append(&mut vec![0; 5]);
    assert_eq!(buf, data.as_slice());
    assert_eq!(end1.read(5, &mut buf, 3).unwrap_err(), ZxError::SHOULD_WAIT);

    drop(end1);
    assert_eq!(
        end0.write(5, data.as_slice(), 3).unwrap_err(),
        ZxError::PEER_CLOSED
    );
    assert_eq!(end0.read(5, &mut buf, 3).unwrap_err(), ZxError::PEER_CLOSED);
    println!("test_read_write_fifo pass");
}