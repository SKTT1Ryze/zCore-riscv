use crate::zircon_object::signal::*;
use crate::zircon_object::ZxError;
pub fn test_port_new() {
    assert!(Port::new(0).is_ok());
    assert!(Port::new(1).is_ok());
    assert_eq!(Port::new(2).unwrap_err(), ZxError::INVALID_ARGS);
    println!("port_new pass");
}


/* #[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    #[async_std::test]
    async fn wait() {
        let port = Port::new(0).unwrap();
        let object = DummyObject::new() as Arc<dyn KernelObject>;
        object.send_signal_to_port_async(Signal::READABLE, &port, 1);

        let packet_repr2 = PortPacketRepr {
            key: 2,
            status: ZxError::OK,
            data: PayloadRepr::Signal(PacketSignal {
                trigger: Signal::WRITABLE,
                observed: Signal::WRITABLE,
                count: 1,
                timestamp: 0,
                _reserved1: 0,
            }),
        };
        async_std::task::spawn({
            let port = port.clone();
            let object = object.clone();
            let packet2 = packet_repr2.clone();
            async move {
                // Assert an irrelevant signal to test the `false` branch of the callback for `READABLE`.
                object.signal_set(Signal::USER_SIGNAL_0);
                object.signal_clear(Signal::USER_SIGNAL_0);
                object.signal_set(Signal::READABLE);
                async_std::task::sleep(Duration::from_millis(1)).await;
                port.push(packet2);
            }
        });

        let packet = port.wait().await;
        let packet_repr = PortPacketRepr {
            key: 1,
            status: ZxError::OK,
            data: PayloadRepr::Signal(PacketSignal {
                trigger: Signal::READABLE,
                observed: Signal::READABLE,
                count: 1,
                timestamp: 0,
                _reserved1: 0,
            }),
        };
        assert_eq!(PortPacketRepr::from(&packet), packet_repr);

        let packet = port.wait().await;
        assert_eq!(PortPacketRepr::from(&packet), packet_repr2);

        // Test asserting signal before `send_signal_to_port_async`.
        let port = Port::new(0).unwrap();
        let object = DummyObject::new() as Arc<dyn KernelObject>;
        object.signal_set(Signal::READABLE);
        object.send_signal_to_port_async(Signal::READABLE, &port, 1);
        let packet = port.wait().await;
        assert_eq!(PortPacketRepr::from(&packet), packet_repr);
    }
}
 */