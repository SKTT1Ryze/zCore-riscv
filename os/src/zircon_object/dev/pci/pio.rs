use super::super::*;
use crate::kernel_hal::{inpd, outpd};
use spin::Mutex;

static PIO_LOCK: Mutex<()> = Mutex::new(());
const PCI_CONFIG_ADDR: u16 = 0xcf8;
const PCI_CONFIG_DATA: u16 = 0xcfc;
const PCI_CONFIG_ENABLE: u32 = 1 << 31;

pub fn pci_bdf_raw_addr(bus: u8, dev: u8, func: u8, offset: u8) -> u32 {
    ((bus as u32 & 0xff) << 16)         // bits 23-16 bus
        | ((dev as u32 & 0x1f) << 11)   // bits 15-11 device
        | ((func as u32 & 0x7) << 8)    // bits 10-8 func
        | (offset as u32 & 0xff) // bits 7-2 reg, with bottom 2 bits as well
}

pub fn pio_config_read(bus: u8, dev: u8, func: u8, offset: u8, width: usize) -> ZxResult<u32> {
    pio_config_read_addr(pci_bdf_raw_addr(bus, dev, func, offset), width)
}

pub fn pio_config_read_addr(addr: u32, width: usize) -> ZxResult<u32> {
    let _lock = PIO_LOCK.lock();
    let shift = ((addr & 0x3) << 3) as usize;
    if shift + width > 32 {
        return Err(ZxError::INVALID_ARGS);
    }
    outpd(PCI_CONFIG_ADDR, (addr & !0x3) | PCI_CONFIG_ENABLE);
    let tmp_val = u32::from_le(inpd(PCI_CONFIG_DATA));
    Ok((tmp_val >> shift) & (((1u64 << width) - 1) as u32))
}

pub fn pio_config_write(
    bus: u8,
    dev: u8,
    func: u8,
    offset: u8,
    val: u32,
    width: usize,
) -> ZxResult {
    pio_config_write_addr(pci_bdf_raw_addr(bus, dev, func, offset), val, width)
}

pub fn pio_config_write_addr(addr: u32, val: u32, width: usize) -> ZxResult {
    let _lock = PIO_LOCK.lock();
    let shift = ((addr & 0x3) << 3) as usize;
    if shift + width > 32 {
        return Err(ZxError::INVALID_ARGS);
    }
    outpd(PCI_CONFIG_ADDR, (addr & !0x3) | PCI_CONFIG_ENABLE);
    let width_mask = ((1u64 << width) - 1) as u32;
    let val = val & width_mask;
    let tmp_val = if width < 32 {
        (u32::from_le(inpd(PCI_CONFIG_DATA)) & !(width_mask << shift)) | (val << shift)
    } else {
        val
    };
    outpd(PCI_CONFIG_DATA, u32::to_le(tmp_val));
    Ok(())
}
