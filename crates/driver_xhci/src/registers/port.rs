use core::ptr::NonNull;

use tock_registers::{register_bitfields, register_structs};
use tock_registers::interfaces::{Readable, ReadWriteable, Writeable};
use tock_registers::registers::ReadWrite;

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
        self.portsc.modify(PORTSC::PR.val(1));
        // self.portsc.write(PORTSC::PED.val(1));
        self.portsc.modify(PORTSC::WCE.val(1));
        while self.portsc.read(PORTSC::PR) == 1 {}
    }

    pub fn speed(&self) -> u32 {
        self.portsc.read(port::PORTSC::SPEED)
    }

    pub fn max_packet_size(&self) -> u32 {
        match self.speed() {
            3 => 64,
            4 => 512,
            _ => 8,
        }
    }
}

register_bitfields! {
    u32,
    pub PORTSC [
        CCS OFFSET(0) NUMBITS(1) [],
        PED OFFSET(1) NUMBITS(1) [],
        OCA OFFSET(3) NUMBITS(1) [],
        PR OFFSET(4) NUMBITS(1) [],
        SPEED OFFSET(10) NUMBITS(4) [],
        PRC OFFSET(21) NUMBITS(1) [],
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
    addr: *mut Port,
    index: u32,
}

impl PortSet {
    pub fn new(max_ports: u32, operational_addr: usize) -> Self {
        let addr = unsafe { (operational_addr as *mut u8).offset(0x400) }.cast();
        Self {
            max_ports,
            addr,
            index: 0,
        }
    }

    pub fn enable_port(&self, port_id: u8) {
        unsafe { self.addr.add(Self::id_to_index(port_id)).read().portsc.modify(PORTSC::PRC::SET); }
    }

    pub fn id_to_index(id: u8) -> usize {
        (id - 1) as usize
    }

    pub fn get_by_id(&self, port_id: u8) -> &Port {
        unsafe {
            self.addr.add(Self::id_to_index(port_id)).as_ref().unwrap()
        }
    }
}

impl Iterator for &mut PortSet {
    type Item = NonNull<Port>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.max_ports {
            let addr = unsafe {
                self.addr.offset(self.index as isize)
            };
            self.index += 1;
            return Some(NonNull::new(addr).unwrap().cast());
        }
        None
    }
}
