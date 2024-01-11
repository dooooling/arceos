use core::ptr::NonNull;

use tock_registers::{register_bitfields, register_structs};
use tock_registers::interfaces::Writeable;
use tock_registers::registers::{ReadOnly, ReadWrite};

#[derive(Debug)]
#[repr(C)]
pub struct PciCapability {
    pub capability_id: u8,
    pub next_pointer: u8,
}

impl PciCapability {
    pub fn cast<U>(&mut self) -> &'static U {
        unsafe { NonNull::from(self).cast().as_ref() }
    }
    pub fn as_msi(&mut self) -> MsiCapability {
        MsiCapability::from(NonNull::new(self).unwrap()).unwrap()
    }
}

#[derive(Debug)]
pub enum MsiCapability {
    _32(NonNull<Msi32>),
    _64(NonNull<Msi64>),
    _32Pvm(NonNull<Msi32Pvm>),
    _64Pvm(NonNull<Msi64Pvm>),
}

impl MsiCapability {
    fn from(pci_capability: NonNull<PciCapability>) -> Option<Self> {
        unsafe {
            if pci_capability.as_ref().capability_id != 5 {
                return None;
            }
            let ptr = pci_capability.as_ptr().cast::<u16>();
            let msg_ctrl = ptr.offset(1).read_volatile();
            let is_64bit = msg_ctrl & 0x80 != 0;
            let mask = msg_ctrl & 0x100 != 0;

            let mis_cap = match (is_64bit, mask) {
                (true, false) => MsiCapability::_64(pci_capability.cast()),
                (true, true) => MsiCapability::_64Pvm(pci_capability.cast()),
                (false, false) => MsiCapability::_32(pci_capability.cast()),
                (false, true) => MsiCapability::_32Pvm(pci_capability.cast()),
            };
            Some(mis_cap)
        }
    }

    pub fn set_control(&self, enable: u8, multiple_enable: u8) {
        match self {
            MsiCapability::_32(cap) => unsafe {
                cap.as_ref().dw1.write(CAPABILITY_DW1::enable.val(enable as u32));
                cap.as_ref().dw1.write(CAPABILITY_DW1::multiple_msg_capable.val(multiple_enable as u32));
            }
            MsiCapability::_64(cap) => unsafe {
                cap.as_ref().dw1.write(CAPABILITY_DW1::enable.val(enable as u32));
                cap.as_ref().dw1.write(CAPABILITY_DW1::multiple_msg_capable.val(multiple_enable as u32));
            }
            MsiCapability::_32Pvm(cap) => unsafe {
                cap.as_ref().dw1.write(CAPABILITY_DW1::enable.val(enable as u32));
                cap.as_ref().dw1.write(CAPABILITY_DW1::multiple_msg_capable.val(multiple_enable as u32));
            }
            MsiCapability::_64Pvm(cap) => unsafe {
                cap.as_ref().dw1.write(CAPABILITY_DW1::enable.val(enable as u32));
                cap.as_ref().dw1.write(CAPABILITY_DW1::multiple_msg_capable.val(multiple_enable as u32));
            }
        }
    }

    pub fn set_address(&self, destination_id: u8, redirection_hint: u8, destination_mode: u8) {
        match self {
            MsiCapability::_32(cap) => unsafe {
                cap.as_ref().address.write(CAPABILITY_ADDRESS::destination_id.val(destination_id as u32));
                cap.as_ref().address.write(CAPABILITY_ADDRESS::redirection_hint.val(redirection_hint as u32));
                cap.as_ref().address.write(CAPABILITY_ADDRESS::destination_mode.val(destination_mode as u32));
            }
            MsiCapability::_64(cap) => unsafe {
                cap.as_ref().address.write(CAPABILITY_ADDRESS::destination_id.val(destination_id as u32));
                cap.as_ref().address.write(CAPABILITY_ADDRESS::redirection_hint.val(redirection_hint as u32));
                cap.as_ref().address.write(CAPABILITY_ADDRESS::destination_mode.val(destination_mode as u32));
            }
            MsiCapability::_32Pvm(cap) => unsafe {
                cap.as_ref().address.write(CAPABILITY_ADDRESS::destination_id.val(destination_id as u32));
                cap.as_ref().address.write(CAPABILITY_ADDRESS::redirection_hint.val(redirection_hint as u32));
                cap.as_ref().address.write(CAPABILITY_ADDRESS::destination_mode.val(destination_mode as u32));
            }
            MsiCapability::_64Pvm(cap) => unsafe {
                cap.as_ref().address.write(CAPABILITY_ADDRESS::destination_id.val(destination_id as u32));
                cap.as_ref().address.write(CAPABILITY_ADDRESS::redirection_hint.val(redirection_hint as u32));
                cap.as_ref().address.write(CAPABILITY_ADDRESS::destination_mode.val(destination_mode as u32));
            }
        }
    }

    pub fn set_data(&self, trigger_mode: u8, vector: u8, delivery_mode: u8, level: u8) {
        match self {
            MsiCapability::_32(cap) => unsafe {
                cap.as_ref().data.write(CAPABILITY_DATA::trigger_mode.val(trigger_mode as u32));
                cap.as_ref().data.write(CAPABILITY_DATA::vector.val(vector as u32));
                cap.as_ref().data.write(CAPABILITY_DATA::delivery_mode.val(delivery_mode as u32));
                cap.as_ref().data.write(CAPABILITY_DATA::level.val(level as u32));
            }
            MsiCapability::_64(cap) => unsafe {
                cap.as_ref().data.write(CAPABILITY_DATA::trigger_mode.val(trigger_mode as u32));
                cap.as_ref().data.write(CAPABILITY_DATA::vector.val(vector as u32));
                cap.as_ref().data.write(CAPABILITY_DATA::delivery_mode.val(delivery_mode as u32));
                cap.as_ref().data.write(CAPABILITY_DATA::level.val(level as u32));
            }
            MsiCapability::_32Pvm(cap) => unsafe {
                cap.as_ref().data.write(CAPABILITY_DATA::trigger_mode.val(trigger_mode as u32));
                cap.as_ref().data.write(CAPABILITY_DATA::vector.val(vector as u32));
                cap.as_ref().data.write(CAPABILITY_DATA::delivery_mode.val(delivery_mode as u32));
                cap.as_ref().data.write(CAPABILITY_DATA::level.val(level as u32));
            }
            MsiCapability::_64Pvm(cap) => unsafe {
                cap.as_ref().data.write(CAPABILITY_DATA::trigger_mode.val(trigger_mode as u32));
                cap.as_ref().data.write(CAPABILITY_DATA::vector.val(vector as u32));
                cap.as_ref().data.write(CAPABILITY_DATA::delivery_mode.val(delivery_mode as u32));
                cap.as_ref().data.write(CAPABILITY_DATA::level.val(level as u32));
            }
        }
    }
}

#[derive(Debug)]
pub struct PciCapabilityIterator {
    base_addr: *mut u8,
    next_capability_offset: Option<u8>,
}

impl PciCapabilityIterator {
    pub fn new(base_addr: usize) -> Self {
        let base_addr = base_addr as *mut u8;
        let offset = unsafe {
            base_addr.offset(0x34).read_volatile()
        };
        Self {
            base_addr,
            next_capability_offset: Some(offset),
        }
    }
}

impl Iterator for PciCapabilityIterator {
    type Item = NonNull<PciCapability>;

    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.next_capability_offset?;
        unsafe {
            let cap: NonNull<PciCapability> = NonNull::new(self.base_addr.offset(offset as isize)).unwrap().cast();
            let next_offset = cap.as_ref().next_pointer;
            self.next_capability_offset = if next_offset == 0 {
                None
            } else if next_offset < 64 || next_offset & 0x3 != 0 {
                None
            } else {
                Some(next_offset)
            };
            Some(cap)
        }
    }
}
register_structs! {
     pub Msi32{
        (0x00 => dw1: ReadWrite<u32, CAPABILITY_DW1::Register>),
        (0x04 => address: ReadWrite<u32,CAPABILITY_ADDRESS::Register>),
        (0x08 => data: ReadWrite<u32, CAPABILITY_DATA::Register>),
        (0x0c => @END),
    }
}
register_structs! {
     pub Msi64{
        (0x00 => dw1: ReadWrite<u32, CAPABILITY_DW1::Register>),
        (0x04 => address: ReadWrite<u32,CAPABILITY_ADDRESS::Register>),
        (0x08 => upper_address: ReadWrite<u32>),
        (0x0C => data: ReadWrite<u32, CAPABILITY_DATA::Register>),
        (0x10 => @END),
    }
}


register_structs! {
     pub Msi32Pvm{
        (0x00 => dw1: ReadWrite<u32, CAPABILITY_DW1::Register>),
        (0x04 => address: ReadWrite<u32,CAPABILITY_ADDRESS::Register>),
        (0x08 => data: ReadWrite<u32, CAPABILITY_DATA::Register>),
        (0x0C => mask: ReadOnly<u32>),
        (0x10 => pending: ReadOnly<u32>),
        (0x14 => @END),
    }
}
register_structs! {
     pub Msi64Pvm{
        (0x00 => dw1: ReadWrite<u32, CAPABILITY_DW1::Register>),
        (0x04 => address: ReadWrite<u32,CAPABILITY_ADDRESS::Register>),
        (0x08 => upper_address: ReadWrite<u32>),
        (0x0C => data: ReadWrite<u32, CAPABILITY_DATA::Register>),
        (0x10 => mask: ReadWrite<u32>),
        (0x14 => pending: ReadWrite<u32>),
        (0x18 => @END),
    }
}


impl MsiCapability {}

register_bitfields! {
    u32,
    CAPABILITY_DW1[
        id OFFSET(0) NUMBITS(8) [],
        next_pointer OFFSET(8) NUMBITS(8) [],
        enable OFFSET(16) NUMBITS(1) [],
        multiple_msg_capable OFFSET(17) NUMBITS(3) [],
        multiple_msg_enable OFFSET(20) NUMBITS(3) [],
        b4_bit_capable OFFSET(23) NUMBITS(1) [],
        masking_capable OFFSET(24) NUMBITS(1) [],
    ],
    CAPABILITY_ADDRESS[
        destination_mode OFFSET(2) NUMBITS(1) [],
        redirection_hint OFFSET(3) NUMBITS(1) [],
        destination_id OFFSET(12) NUMBITS(8) [],
    ],
    CAPABILITY_DATA[
        vector OFFSET(0) NUMBITS(8) [],
        delivery_mode OFFSET(8) NUMBITS(3) [],
        level OFFSET(14) NUMBITS(1) [],
        trigger_mode OFFSET(15) NUMBITS(1) [],
    ],
}
