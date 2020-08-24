use crate::zircon_object::signal::*;
use crate::zircon_object::ZxError;
use alloc::format;
use crate::{print, println};

pub fn test_port_packet_size() {
    use core::mem::size_of;
    assert_eq!(size_of::<PacketUser>(), 32);
    assert_eq!(size_of::<PacketSignal>(), 32);
    assert_eq!(size_of::<PacketGuestBell>(), 32);
    assert_eq!(size_of::<PacketGuestMem>(), 32);
    assert_eq!(size_of::<PacketGuestIo>(), 32);
    assert_eq!(size_of::<PacketGuestVcpu>(), 32);
    assert_eq!(size_of::<PacketInterrupt>(), 32);
    println!("test_port_packet_size pass");
}

pub fn test_encdec(data: PayloadRepr) {
    let repr = PortPacketRepr {
        key: 0,
        status: ZxError::OK,
        data: data.clone(),
    };
    let packet = PortPacket {
        key: 0,
        type_: data.type_test(),
        status: ZxError::OK,
        data: data.encode_test(),
    };
    assert_eq!(repr, PortPacketRepr::from(&packet));
    assert_eq!(repr.clone(), PortPacketRepr::from(&PortPacket::from(repr)));
    //println!("test_encdec pass");
}

pub fn test_user() {
    let user = PacketUser::default();
    test_encdec(PayloadRepr::User(user));
    println!("test_user pass");
}

pub fn test_guest_bell() {
    let guest_bell = PacketGuestBell::default();
    assert_eq!(guest_bell.addr, 0);
    test_encdec(PayloadRepr::GuestBell(guest_bell));
    println!("guest_bell pass");
}

pub fn test_guest_mem() {
    let guest_mem = PacketGuestMem::default();
    assert_eq!(guest_mem.addr, 0);
    test_encdec(PayloadRepr::GuestMem(guest_mem));
    println!("guest_mem pass");
}

pub fn test_guest_io() {
    let guest_io = PacketGuestIo::default();
    assert_eq!(guest_io.port, 0);
    assert_eq!(guest_io.input, false);
    test_encdec(PayloadRepr::GuestIo(guest_io));
    println!("test_guest_io pass");
}

pub fn test_interrupt() {
    let interrupt = PacketInterrupt {
        timestamp: 12345,
        _reserved0: 0,
        _reserved1: 0,
        _reserved2: 0,
    };
    test_encdec(PayloadRepr::Interrupt(interrupt));
    println!("test_interrupt pass");
}

pub fn test_all_in_port_packet_test() {
    test_port_packet_size();
    test_user();
    test_guest_bell();
    test_guest_mem();
    test_guest_io();
    test_interrupt();
}
/* pub fn test_guest_vcpu() {
    let interrupt = PacketGuestVcpuInterrupt { mask: 0, vector: 0 };
    let guest_vcpu1 = PacketGuestVcpu {
        data: PacketGuestVcpuData { interrupt },
        type_: PacketGuestVcpuType::VcpuInterrupt,
        _padding1: Default::default(),
        _reserved: 0,
    };
    let startup = PacketGuestVcpuStartup { id: 0, entry: 0 };
    let guest_vcpu2 = PacketGuestVcpu {
        data: PacketGuestVcpuData { startup },
        type_: PacketGuestVcpuType::VcpuStartup,
        _padding1: Default::default(),
        _reserved: 0,
    };
    test_encdec(PayloadRepr::GuestVcpu(guest_vcpu1));
    test_encdec(PayloadRepr::GuestVcpu(guest_vcpu2));

    let packet = PortPacket {
        key: 1,
        type_: PacketType::GuestVcpu,
        status: ZxError::OK,
        data: Payload {
            guest_vcpu: guest_vcpu2,
        },
    };
    assert_eq!(
        format!("{:?}", packet),
        "PortPacketRepr { key: 1, status: OK, data: GuestVcpu(PacketGuestVcpu { data: PacketGuestVcpuStartup { id: 0, entry: 0 }, type_: VcpuStartup, _padding1: [0, 0, 0, 0, 0, 0, 0], _reserved: 0 }) }"
    );

    assert!(!guest_vcpu1.eq(&guest_vcpu2));
    let guest_vcpu3 = PacketGuestVcpu {
        data: PacketGuestVcpuData {
            startup: PacketGuestVcpuStartup { id: 0, entry: 1 },
        },
        type_: PacketGuestVcpuType::VcpuStartup,
        _padding1: Default::default(),
        _reserved: 0,
    };
    assert!(!guest_vcpu2.eq(&guest_vcpu3));
}
 */

/* #[test]
#[should_panic(expected = "not implemented")]
fn page_request() {
    let data: PacketUser = [0u8; 32];
    PayloadRepr::decode(PacketType::PageRequest, &Payload { user: data });
} */