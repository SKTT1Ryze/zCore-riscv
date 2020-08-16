use crate::zircon_object::ipc::*;
use crate::zircon_object::object::*;

const SOCKET_SIZE: usize = 128 * 2048;

/* pub fn test_basics_socket() {
    assert_eq!(Socket::create(1 << 10).unwrap_err(), ZxError::INVALID_ARGS);
    assert_eq!(
        Socket::create(SocketFlags::SOCKET_PEEK.bits).unwrap_err(),
        ZxError::INVALID_ARGS
    );
    let (end0, end1) = Socket::create(1).unwrap();
    assert!(Arc::ptr_eq(
        &end0.peer().unwrap().downcast_arc().unwrap(),
        &end1
    ));
    assert_eq!(end0.related_koid(), end1.id());

    drop(end1);
    assert_eq!(end0.peer().unwrap_err(), ZxError::PEER_CLOSED);
    assert_eq!(end0.related_koid(), 0);
} */

pub fn test_stream() {
    let (end0, end1) = Socket::create(0).unwrap();

    // empty read & write
    assert_eq!(
        end0.read(false, &mut [0; 10]).unwrap_err(),
        ZxError::SHOULD_WAIT
    );
    assert_eq!(end0.write(&[]).unwrap(), 0);

    assert_eq!(end0.write(&[1, 2, 3]), Ok(3));
    let mut buf = [0u8; 4];
    assert_eq!(end1.read(true, &mut buf).unwrap(), 3);
    assert_eq!(buf, [1, 2, 3, 0]);
    buf = [0; 4];
    // can read again
    assert_eq!(end1.read(true, &mut buf).unwrap(), 3);
    assert_eq!(buf, [1, 2, 3, 0]);
    assert_eq!(
        end0.get_info(),
        SocketInfo::new(
            0,
            0,
            SOCKET_SIZE as _,
            0,
            0,
            SOCKET_SIZE as _,
            3
        )
    );

    // use a small buffer now
    let mut buf = [0u8; 2];
    assert_eq!(end1.read(true, &mut buf).unwrap(), 2);
    assert_eq!(buf, [1, 2]);
    // consume
    assert_eq!(end1.read(false, &mut buf).unwrap(), 2);
    assert_eq!(buf, [1, 2]);
    assert_eq!(end1.read(false, &mut buf).unwrap(), 1);
    assert_eq!(buf, [3, 2]);

    end1.write(&[111, 233, 222]).unwrap();
    assert_eq!(
        end0.get_info(),
        SocketInfo::new(
            0,
            0,
            SOCKET_SIZE as _,
            3,
            3,
            SOCKET_SIZE as _,
            0
        )
    );

    // write much data
    assert_eq!(end0.write(&[0; SOCKET_SIZE * 2]).unwrap(), SOCKET_SIZE);
    assert_eq!(end0.write(&[0; 1]).unwrap_err(), ZxError::SHOULD_WAIT);
    assert!(!end0.signal().contains(Signal::WRITABLE));
    end1.read(false, &mut [0; 1]).unwrap();
    assert!(end0.signal().contains(Signal::WRITABLE));
}

/* pub fn test_datagram() {
    let (end0, end1) = Socket::create(1).unwrap();

    // empty read & write
    assert_eq!(
        end0.read(false, &mut [0; 10]).unwrap_err(),
        ZxError::SHOULD_WAIT
    );
    assert_eq!(end0.write(&[]).unwrap_err(), ZxError::INVALID_ARGS);

    assert_eq!(end0.write(&[1, 2, 3]), Ok(3));
    assert_eq!(end0.write(&[4, 5, 6, 7]), Ok(4));

    let mut buf = [0u8; 4];
    assert_eq!(end1.read(true, &mut []).unwrap(), 0);
    assert_eq!(end1.read(true, &mut buf).unwrap(), 3);
    assert_eq!(buf, [1, 2, 3, 0]);
    buf = [0; 4];
    // can read again
    assert_eq!(end1.read(true, &mut buf).unwrap(), 3);
    assert_eq!(buf, [1, 2, 3, 0]);
    assert_eq!(
        end0.get_info(),
        SocketInfo {
            options: SocketFlags::DATAGRAM.bits,
            padding1: 0,
            rx_buf_max: SOCKET_SIZE as _,
            rx_buf_size: 0,
            rx_buf_available: 0,
            tx_buf_max: SOCKET_SIZE as _,
            tx_buf_size: 7,
        }
    );
    assert_eq!(
        end1.get_info(),
        SocketInfo {
            options: SocketFlags::DATAGRAM.bits,
            padding1: 0,
            rx_buf_max: SOCKET_SIZE as _,
            rx_buf_size: 7,
            rx_buf_available: 3,
            tx_buf_max: SOCKET_SIZE as _,
            tx_buf_size: 0,
        }
    );

    // use a small buffer now
    let mut buf = [0u8; 2];
    assert_eq!(end1.read(true, &mut buf).unwrap(), 2);
    assert_eq!(buf, [1, 2]);
    // consume
    assert_eq!(end1.read(false, &mut buf).unwrap(), 2);
    assert_eq!(buf, [1, 2]);
    assert_eq!(end1.read(false, &mut buf).unwrap(), 2);
    assert_eq!(buf, [4, 5]);

    // write much data
    let (end0, end1) = Socket::create(1).unwrap();
    assert_eq!(
        end0.write(&[0; SOCKET_SIZE * 2]).unwrap_err(),
        ZxError::OUT_OF_RANGE
    );
    assert_eq!(end0.write(&[0; SOCKET_SIZE]).unwrap(), SOCKET_SIZE);
    assert!(!end0.signal().contains(Signal::WRITABLE));
    end1.read(false, &mut [0; 1]).unwrap();
    assert!(end0.signal().contains(Signal::WRITABLE));
} */

pub fn test_threshold() {
    let (end0, end1) = Socket::create(0).unwrap();
    assert_eq!(end0.get_rx_tx_threshold(), (0, 0));

    // write
    assert_eq!(
        end0.set_write_threshold(SOCKET_SIZE * 2).unwrap_err(),
        ZxError::INVALID_ARGS
    );
    // have space when setting threshold
    assert!(end0.set_write_threshold(10).is_ok());
    assert!(end0.signal().contains(Signal::SOCKET_WRITE_THRESHOLD));
    assert_eq!(end0.get_rx_tx_threshold(), (0, 10));
    end0.write(&[0; SOCKET_SIZE - 9]).unwrap();
    assert!(!end0.signal().contains(Signal::SOCKET_WRITE_THRESHOLD));
    // no space when setting threshold
    assert!(end0.set_write_threshold(20).is_ok());
    assert!(!end0.signal().contains(Signal::SOCKET_WRITE_THRESHOLD));
    end1.read(false, &mut [0; 10]).unwrap();
    assert!(!end0.signal().contains(Signal::SOCKET_WRITE_THRESHOLD));
    end1.read(false, &mut [0; 1]).unwrap();
    assert!(end0.signal().contains(Signal::SOCKET_WRITE_THRESHOLD));
    // disable threshold
    assert!(end0.set_write_threshold(0).is_ok());
    assert!(!end0.signal().contains(Signal::SOCKET_WRITE_THRESHOLD));

    // read
    assert_eq!(
        end0.set_read_threshold(SOCKET_SIZE * 2).unwrap_err(),
        ZxError::INVALID_ARGS
    );
    // have data when setting threshold
    end1.write(&[0; 10]).unwrap();
    assert!(end0.set_read_threshold(10).is_ok());
    assert!(end0.signal().contains(Signal::SOCKET_READ_THRESHOLD));
    assert_eq!(end0.get_rx_tx_threshold(), (10, 0));
    end0.read(false, &mut [0; 1]).unwrap();
    assert!(!end0.signal().contains(Signal::SOCKET_READ_THRESHOLD));
    // no data when setting threshold
    end0.read(false, &mut [0; 10]).unwrap();
    assert!(end0.set_read_threshold(20).is_ok());
    assert!(!end0.signal().contains(Signal::SOCKET_WRITE_THRESHOLD));
    end1.write(&[0; 10]).unwrap();
    assert!(!end0.signal().contains(Signal::SOCKET_READ_THRESHOLD));
    end1.write(&[0; 10]).unwrap();
    assert!(end0.signal().contains(Signal::SOCKET_READ_THRESHOLD));
    // disable threshold
    assert!(end0.set_read_threshold(0).is_ok());
    assert!(!end0.signal().contains(Signal::SOCKET_READ_THRESHOLD));
    println!("test_threshold pass");
}

pub fn test_shutdown() {
    let (end0, end1) = Socket::create(0).unwrap();
    end0.write(&[0; 10]).unwrap();

    assert!(end1.shutdown(true, false).is_ok());
    assert_eq!(end0.write(&[0; 1]).unwrap_err(), ZxError::BAD_STATE);
    assert!(!end0.signal().contains(Signal::WRITABLE));
    // buffered data can be read
    assert!(end1.signal().contains(Signal::READABLE));
    assert_eq!(end1.read(false, &mut [0; 20]).unwrap(), 10);
    // no more data
    assert!(!end1.signal().contains(Signal::READABLE));
    assert_eq!(
        end1.read(false, &mut [0; 20]).unwrap_err(),
        ZxError::BAD_STATE
    );
    // the opposite direction is still okay
    assert_eq!(end1.write(&[0; 1]).unwrap(), 1);
    assert_eq!(end0.read(false, &mut [0; 10]).unwrap(), 1);
    println!("test_shutdown pass");
}

pub fn test_drop() {
    let (end0, end1) = Socket::create(0).unwrap();
    end0.write(&[0; 10]).unwrap();

    drop(end0);
    assert!(!end1.signal().contains(Signal::WRITABLE));
    assert!(end1.signal().contains(Signal::PEER_CLOSED));
    assert_eq!(end1.write(&[0; 1]).unwrap_err(), ZxError::PEER_CLOSED);
    // buffered data can be read
    assert!(end1.signal().contains(Signal::READABLE));
    assert_eq!(end1.read(false, &mut [0; 20]).unwrap(), 10);
    // no more data
    assert!(!end1.signal().contains(Signal::READABLE));
    assert_eq!(
        end1.read(false, &mut [0; 20]).unwrap_err(),
        ZxError::PEER_CLOSED
    );
    println!("test_drop pass");
}