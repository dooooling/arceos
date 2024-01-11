use core::ptr::NonNull;

use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::registers::ReadWrite;
use tock_registers::{register_bitfields, register_structs};

use crate::registers::port;

register_structs! {
    pub Port{
        (0x00 => pub portsc:ReadWrite<u32, PORTSC::Register>),
        (0x04 => pub portpmsc:ReadWrite<u32, PORTPMSC::Register>),
        (0x08 => pub portli:ReadWrite<u32, PORTLI::Register>),
        (0x0C => pub porthlpmc:ReadWrite<u32, PORTHLPMC::Register>),
        (0x10 => @END),
    }
}

impl Port {
    pub fn connected(&self) -> bool {
        self.portsc.read(port::PORTSC::CCS) == 1
    }
    pub fn reset(&self) {
        self.portsc.write(PORTSC::PR.val(1));
        // self.portsc.write(PORTSC::PED.val(1));
        self.portsc.write(PORTSC::WCE.val(1));
        while self.portsc.read(PORTSC::PR) != 0 {}
    }
}

register_bitfields! {
    u32,
    pub PORTSC [
        CCS OFFSET(0) NUMBITS(1) [],
        PED OFFSET(1) NUMBITS(1) [],
        OCA OFFSET(3) NUMBITS(1) [],
        PR OFFSET(4) NUMBITS(1) [],
        WCE OFFSET(25) NUMBITS(1) [],
    ],
    pub PORTPMSC [
        NONE OFFSET(0) NUMBITS(1) [],
    ],
    pub PORTLI [
        NONE OFFSET(0) NUMBITS(16) [],
    ],
    pub PORTHLPMC [
        NONE OFFSET(0) NUMBITS(16) [],
    ],
}

pub struct PortSet {
    max_ports: u32,
    operational_addr: usize,
    index: u32,
}

impl PortSet {
    pub fn new(max_ports: u32, operational_addr: usize) -> Self {
        Self {
            max_ports,
            operational_addr,
            index: 0,
        }
    }
}

impl Iterator for PortSet {
    type Item = NonNull<Port>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.max_ports {
            let addr = unsafe {
                (self.operational_addr as *mut u8).offset(0x400 + (0x10 * self.index) as isize)
            };
            self.index += 1;
            return Some(NonNull::new(addr).unwrap().cast());
        }
        None
    }
}
