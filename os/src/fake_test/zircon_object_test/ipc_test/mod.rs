pub mod channel_test;
pub mod fifo_test;
pub mod socket_test;
use crate::{print, println};

use {
    channel_test::*,
    fifo_test::*,
    socket_test::*,
};

pub fn test_all_in_ipc_test() {
    test_basics_channel();
    test_read_write_channel();
    test_peer_closed_channel();
    test_test_basics_fifo();
    test_read_write_fifo();
    test_stream();
    test_threshold();
    test_shutdown();
    test_drop();
    println!("all test in ipc_test pass");
}