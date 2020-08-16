pub mod event_test;
pub mod eventpair_test;
pub mod futex_test;
pub mod port_test;
pub mod port_packet_test;
pub mod timer_test;
use crate::{print, println};

pub use {
    event_test::*,
    eventpair_test::*,
    futex_test::*,
    port_test::*,
    port_packet_test::*,
    timer_test::*,
};

pub fn test_all_in_signal_test() {
    test_allowed_signals_1();
    test_allowed_signals_2();
    test_peer_closed();
    test_all_in_port_packet_test();
    println!("all test in signal_test pass");
}