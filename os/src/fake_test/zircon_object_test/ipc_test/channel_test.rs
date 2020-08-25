use crate::zircon_object::ipc::*;
use crate::zircon_object::object::*;
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::boxed::Box;
use core::sync::atomic::{AtomicBool, Ordering};
use crate::{print, println};

pub fn test_basics_channel() {
    let (end0, end1) = Channel::create();
    assert!(Arc::ptr_eq(
        &end0.peer().unwrap().downcast_arc().unwrap(),
        &end1
    ));
    assert_eq!(end0.related_koid(), end1.id());

    drop(end1);
    assert_eq!(end0.peer().unwrap_err(), ZxError::PEER_CLOSED);
    assert_eq!(end0.related_koid(), 0);
    println!("test_basics pass");
}

pub fn test_read_write_channel() {
    let (channel0, channel1) = Channel::create();
    // write a message to each other
    channel0
        .write(MessagePacket {
            data: Vec::from("hello 1"),
            handles: Vec::new(),
        })
        .unwrap();
    channel1
        .write(MessagePacket {
            data: Vec::from("hello 0"),
            handles: Vec::new(),
        })
        .unwrap();

    // read message should success
    let recv_msg = channel1.read().unwrap();
    assert_eq!(recv_msg.data.as_slice(), b"hello 1");
    assert!(recv_msg.handles.is_empty());

    let recv_msg = channel0.read().unwrap();
    assert_eq!(recv_msg.data.as_slice(), b"hello 0");
    assert!(recv_msg.handles.is_empty());

    // read more message should fail.
    assert_eq!(channel0.read().err(), Some(ZxError::SHOULD_WAIT));
    assert_eq!(channel1.read().err(), Some(ZxError::SHOULD_WAIT));
    println!("test_read_write pass");
}

pub fn test_peer_closed_channel() {
    let (channel0, channel1) = Channel::create();
    // write a message from peer, then drop it
    channel1.write(MessagePacket::default()).unwrap();
    drop(channel1);
    // read the first message should success.
    channel0.read().unwrap();
    // read more message should fail.
    assert_eq!(channel0.read().err(), Some(ZxError::PEER_CLOSED));
    // write message should fail.
    assert_eq!(
        channel0.write(MessagePacket::default()),
        Err(ZxError::PEER_CLOSED)
    );
    println!("test_peer_closed pass");
}

pub fn test_signal() {
    let (channel0, channel1) = Channel::create();

    // initial status is writable and not readable.
    let init_signal = channel0.base.signal();
    assert!(!init_signal.contains(Signal::READABLE));
    assert!(init_signal.contains(Signal::WRITABLE));

    // register callback for `Signal::READABLE` & `Signal::PEER_CLOSED`:
    //   set `readable` and `peer_closed`
    let readable = Arc::new(AtomicBool::new(false));
    let peer_closed = Arc::new(AtomicBool::new(false));
    channel0.add_signal_callback(Box::new({
        let readable = readable.clone();
        let peer_closed = peer_closed.clone();
        move |signal| {
            readable.store(signal.contains(Signal::READABLE), Ordering::SeqCst);
            peer_closed.store(signal.contains(Signal::PEER_CLOSED), Ordering::SeqCst);
            false
        }
    }));

    // writing to peer should trigger `Signal::READABLE`.
    channel1.write(MessagePacket::default()).unwrap();
    assert!(readable.load(Ordering::SeqCst));

    // reading all messages should cause `Signal::READABLE` be cleared.
    channel0.read().unwrap();
    assert!(!readable.load(Ordering::SeqCst));

    // peer closed should trigger `Signal::PEER_CLOSED`.
    assert!(!peer_closed.load(Ordering::SeqCst));
    drop(channel1);
    assert!(peer_closed.load(Ordering::SeqCst));
    println!("test_signal pass");
}

/* #[async_std::test]
    async fn call() {
        let (channel0, channel1) = Channel::create();
        async_std::task::spawn({
            let channel1 = channel1.clone();
            async move {
                async_std::task::sleep(Duration::from_millis(10)).await;
                let recv_msg = channel1.read().unwrap();
                let txid = recv_msg.get_txid();
                assert_eq!(txid, 0x8000_0000);
                assert_eq!(txid.to_ne_bytes(), &recv_msg.data[..4]);
                assert_eq!(&recv_msg.data[4..], b"o 0");
                // write an irrelevant message
                channel1
                    .write(MessagePacket {
                        data: Vec::from("hello 1"),
                        handles: Vec::new(),
                    })
                    .unwrap();
                // reply the call
                let mut data: Vec<u8> = vec![];
                data.append(&mut txid.to_ne_bytes().to_vec());
                data.append(&mut Vec::from("hello 2"));
                channel1
                    .write(MessagePacket {
                        data,
                        handles: Vec::new(),
                    })
                    .unwrap();
            }
        });

        let recv_msg = channel0
            .call(MessagePacket {
                data: Vec::from("hello 0"),
                handles: Vec::new(),
            })
            .await
            .unwrap();
        let txid = recv_msg.get_txid();
        assert_eq!(txid, 0x8000_0000);
        assert_eq!(txid.to_ne_bytes(), &recv_msg.data[..4]);
        assert_eq!(&recv_msg.data[4..], b"hello 2");

        // peer dropped when calling
        let (channel0, channel1) = Channel::create();
        async_std::task::spawn({
            async move {
                async_std::task::sleep(Duration::from_millis(10)).await;
                let _ = channel1;
            }
        });
        assert_eq!(
            channel0
                .call(MessagePacket {
                    data: Vec::from("hello 0"),
                    handles: Vec::new(),
                })
                .await
                .unwrap_err(),
            ZxError::PEER_CLOSED
        );
    }
 */