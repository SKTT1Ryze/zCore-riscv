use super::*;
use super::object::*;
use alloc::sync::{Arc, Weak};

/// Mutually signalable pair of events for concurrent programming
///
/// ## SYNOPSIS
///
/// Event Pairs are linked pairs of user-signalable objects. The 8 signal
/// bits reserved for userspace (`ZX_USER_SIGNAL_0` through
/// `ZX_USER_SIGNAL_7`) may be set or cleared on the local or opposing
/// endpoint of an Event Pair.
pub struct EventPair {
    base: KObjectBase,
    _counter: CountHelper,
    peer: Weak<EventPair>,
}

impl_kobject!(EventPair
    fn allowed_signals(&self) -> Signal {
        Signal::USER_ALL | Signal::SIGNALED
    }
    fn peer(&self) -> ZxResult<Arc<dyn KernelObject>> {
        let peer = self.peer.upgrade().ok_or(ZxError::PEER_CLOSED)?;
        Ok(peer)
    }
    fn related_koid(&self) -> KoID {
        self.peer.upgrade().map(|p| p.id()).unwrap_or(0)
    }
);
define_count_helper!(EventPair);

impl EventPair {
    /// Create a pair of event.
    #[allow(unsafe_code)]
    pub fn create() -> (Arc<Self>, Arc<Self>) {
        let mut event0 = Arc::new(EventPair {
            base: KObjectBase::default(),
            _counter: CountHelper::new(),
            peer: Weak::default(),
        });
        let event1 = Arc::new(EventPair {
            base: KObjectBase::default(),
            _counter: CountHelper::new(),
            peer: Arc::downgrade(&event0),
        });
        // no other reference of `channel0`
        unsafe {
            Arc::get_mut_unchecked(&mut event0).peer = Arc::downgrade(&event1);
        }
        (event0, event1)
    }

    /// Get the peer event.
    pub fn peer(&self) -> ZxResult<Arc<Self>> {
        self.peer.upgrade().ok_or(ZxError::PEER_CLOSED)
    }
}

impl Drop for EventPair {
    fn drop(&mut self) {
        if let Some(peer) = self.peer.upgrade() {
            peer.base.signal_set(Signal::PEER_CLOSED);
        }
    }
}
