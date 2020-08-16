use {
    crate::zircon_object::object::*,
    alloc::collections::VecDeque,
    alloc::sync::{Arc, Weak},
    alloc::vec::Vec,
    core::convert::TryInto,
    core::sync::atomic::{AtomicU32, Ordering},
    futures::channel::oneshot::{self, Sender},
    hashbrown::HashMap,
    spin::Mutex,
};

/// Bidirectional interprocess communication
///
/// # SYNOPSIS
///
/// A channel is a bidirectional transport of messages consisting of some
/// amount of byte data and some number of handles.
///
/// # DESCRIPTION
///
/// The process of sending a message via a channel has two steps. The first is to
/// atomically write the data into the channel and move ownership of all handles in
/// the message into this channel. This operation always consumes the handles: at
/// the end of the call, all handles either are all in the channel or are all
/// discarded. The second operation, channel read, is similar: on success
/// all the handles in the next message are atomically moved into the
/// receiving process' handle table. On failure, the channel retains ownership.
pub struct Channel {
    base: KObjectBase,
    _counter: CountHelper,
    peer: Weak<Channel>,
    recv_queue: Mutex<VecDeque<T>>,
    call_reply: Mutex<HashMap<TxID, Sender<ZxResult<T>>>>,
    next_txid: AtomicU32,
}

type T = MessagePacket;
type TxID = u32;

impl_kobject!(Channel
    fn peer(&self) -> ZxResult<Arc<dyn KernelObject>> {
        let peer = self.peer.upgrade().ok_or(ZxError::PEER_CLOSED)?;
        Ok(peer)
    }
    fn related_koid(&self) -> KoID {
        self.peer.upgrade().map(|p| p.id()).unwrap_or(0)
    }
);
define_count_helper!(Channel);

impl Channel {
    /// Create a channel and return a pair of its endpoints
    #[allow(unsafe_code)]
    pub fn create() -> (Arc<Self>, Arc<Self>) {
        let mut channel0 = Arc::new(Channel {
            base: KObjectBase::with_signal(Signal::WRITABLE),
            _counter: CountHelper::new(),
            peer: Weak::default(),
            recv_queue: Default::default(),
            call_reply: Default::default(),
            next_txid: AtomicU32::new(0x8000_0000),
        });
        let channel1 = Arc::new(Channel {
            base: KObjectBase::with_signal(Signal::WRITABLE),
            _counter: CountHelper::new(),
            peer: Arc::downgrade(&channel0),
            recv_queue: Default::default(),
            call_reply: Default::default(),
            next_txid: AtomicU32::new(0x8000_0000),
        });
        // no other reference of `channel0`
        unsafe {
            Arc::get_mut_unchecked(&mut channel0).peer = Arc::downgrade(&channel1);
        }
        (channel0, channel1)
    }

    /// Read a packet from the channel if check is ok, otherwise the msg will keep.
    pub fn check_and_read(&self, checker: impl FnOnce(&T) -> ZxResult) -> ZxResult<T> {
        let mut recv_queue = self.recv_queue.lock();
        if let Some(msg) = recv_queue.front() {
            checker(msg)?;
            let msg = recv_queue.pop_front().unwrap();
            if recv_queue.is_empty() {
                self.base.signal_clear(Signal::READABLE);
            }
            return Ok(msg);
        }
        if self.peer_closed() {
            Err(ZxError::PEER_CLOSED)
        } else {
            Err(ZxError::SHOULD_WAIT)
        }
    }

    /// Read a packet from the channel
    pub fn read(&self) -> ZxResult<T> {
        self.check_and_read(|_| Ok(()))
    }

    /// Write a packet to the channel
    pub fn write(&self, msg: T) -> ZxResult {
        let peer = self.peer.upgrade().ok_or(ZxError::PEER_CLOSED)?;
        // check first 4 bytes: whether it is a call reply?
        let txid = msg.get_txid();
        if txid != 0 {
            if let Some(sender) = peer.call_reply.lock().remove(&txid) {
                let _ = sender.send(Ok(msg));
                return Ok(());
            }
        }
        peer.push_general(msg);
        Ok(())
    }

    /// Send a message to a channel and await a reply.
    ///
    /// The first four bytes of the written and read back messages are treated as a
    /// transaction ID.  The kernel generates a txid for the
    /// written message, replacing that part of the message as read from userspace.
    ///
    /// `msg.data` must have at lease a length of 4 bytes.
    pub async fn call(self: &Arc<Self>, mut msg: T) -> ZxResult<T> {
        assert!(msg.data.len() >= 4);
        let peer = self.peer.upgrade().ok_or(ZxError::PEER_CLOSED)?;
        let txid = self.new_txid();
        msg.set_txid(txid);
        peer.push_general(msg);
        let (sender, receiver) = oneshot::channel();
        self.call_reply.lock().insert(txid, sender);
        drop(peer);
        receiver.await.unwrap()
    }

    /// Push a message to general queue, called from peer.
    fn push_general(&self, msg: T) {
        let mut send_queue = self.recv_queue.lock();
        send_queue.push_back(msg);
        if send_queue.len() == 1 {
            self.base.signal_set(Signal::READABLE);
        }
    }

    /// Generate a new transaction ID for `call`.
    fn new_txid(&self) -> TxID {
        self.next_txid.fetch_add(1, Ordering::SeqCst)
    }

    /// Is peer channel closed?
    fn peer_closed(&self) -> bool {
        self.peer.strong_count() == 0
    }
}

impl Drop for Channel {
    fn drop(&mut self) {
        if let Some(peer) = self.peer.upgrade() {
            peer.base
                .signal_change(Signal::WRITABLE, Signal::PEER_CLOSED);
            for (_, sender) in core::mem::take(&mut *peer.call_reply.lock()).into_iter() {
                let _ = sender.send(Err(ZxError::PEER_CLOSED));
            }
        }
    }
}

/// The message transferred in the channel.
/// See [Channel](struct.Channel.html) for details.
#[derive(Default, Debug)]
pub struct MessagePacket {
    /// The data carried by the message packet
    pub data: Vec<u8>,
    /// See [Channel](struct.Channel.html) for details.
    pub handles: Vec<Handle>,
}

impl MessagePacket {
    /// Set txid (the first 4 bytes)
    pub fn set_txid(&mut self, txid: TxID) {
        if self.data.len() >= core::mem::size_of::<TxID>() {
            self.data[..4].copy_from_slice(&txid.to_ne_bytes());
        }
    }

    /// Get txid (the first 4 bytes)
    pub fn get_txid(&self) -> TxID {
        if self.data.len() >= core::mem::size_of::<TxID>() {
            TxID::from_ne_bytes(self.data[..4].try_into().unwrap())
        } else {
            0
        }
    }
}
