use bitflags::bitflags;
use numeric_enum_macro::numeric_enum;

bitflags! {
    pub struct MMUFlags: usize {
        #[allow(clippy::identity_op)]
        const CACHE_1   = 1 << 0;
        const CACHE_2   = 1 << 1;
        const READ      = 1 << 2;
        const WRITE     = 1 << 3;
        const EXECUTE   = 1 << 4;
        const USER      = 1 << 5;
    }
}
numeric_enum! {
    #[repr(u32)]
    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum CachePolicy {
        Cached = 0,
        Uncached = 1,
        UncachedDevice = 2,
        WriteCombining = 3,
    }
}
pub const CACHE_POLICY_MASK: u32 = 3;

pub type PhysAddr = usize;
pub type VirtAddr = usize;
pub type DevVAddr = usize;
pub const PAGE_SIZE: usize = 0x1000;