use axhal::mem::{phys_to_virt, PhysAddr};
use driver_pci::{DeviceFunction, DeviceFunctionInfo};
use crate::usb::xhci::XHCIRegister;

pub struct HCD {}

pub fn parse_hci(bdf: &DeviceFunction, dev_info: &DeviceFunctionInfo) -> bool {
    if dev_info.class == 0xc && dev_info.subclass == 0x03 {
        /// bar地址屏蔽低位
        let base1_addr = read_word(bdf, 0x10) & 0xFFFFFFF0;
        let gsi = read_word(bdf, 0x3C) & 0xFF;
        let pin = read_word(bdf, 0x3C) & 0xFF00 >> 8;
        info!(
            "found pic usb device  {} - {:#X}   base addr1  : {:#X} , irq : {:#X} , pin : {:#X}",
            dev_info, dev_info.prog_if, base1_addr,gsi,pin
        );

        if base1_addr != 0 {
            let add = phys_to_virt(PhysAddr::from(base1_addr as usize)).as_mut_ptr();

            if dev_info.prog_if == 0x30 {
                let registers = XHCIRegister::new(add);
                info!("usb controller capability :{}", registers);
                registers.init(gsi);
            }
        }
        true
    } else {
        false
    }
}

fn read_word(bdf: &DeviceFunction, offset: u32) -> u32 {
    let addr = phys_to_virt(axconfig::PCI_ECAM_BASE.into());
    let mmio_base = addr.as_ptr() as *mut u32;
    unsafe { (mmio_base.add((ecma_addr(bdf, offset) >> 2) as usize)).read_volatile() }
}

fn ecma_addr(bdf: &DeviceFunction, offset: u32) -> u32 {
    let bdf = (bdf.bus as u32) << 8 | (bdf.device as u32) << 3 | bdf.function as u32;
    let address = bdf << 12 | offset;
    address
}

fn print_uhc_info(dev_info: &DeviceFunctionInfo, version: u16) {
    let usb_type = match dev_info.prog_if {
        0x10 => "USB1",
        0x20 => "USB2",
        0x30 => "USB3",
        _ => "UNKNOWN",
    };
    let a = (version >> 8) & 0xFF;
    let b = (version >> 4) & 0xF;
    let c = (version) & 0xF;
    info!("usb controller version : {}-{}.{}.{}", usb_type, a, b, c);
}
