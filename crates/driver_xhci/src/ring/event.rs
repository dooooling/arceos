use alloc::vec::Vec;
use core::alloc::Layout;
use core::fmt::{Debug, Formatter};

use crate::registers::Registers;
use crate::ring::{GenericTrb, TrbType};
use crate::virt_to_phys;

#[derive(Default, Debug)]
pub struct EventRing {
    pub buf: Vec<GenericTrb>,
    pub cycle_bit: bool,
    pub write_idx: usize,
}

impl EventRing {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: GenericTrb::aligned_vec(64, capacity),
            cycle_bit: true,
            write_idx: 0,
        }
    }

    pub fn next(&mut self, registers: &Registers) -> Option<&GenericTrb> {
        let trb = &self.buf[self.write_idx];
        if trb.pcs() != self.cycle_bit {
            return None;
        }
        self.write_idx += 1;
        if self.write_idx == self.buf.len() {
            self.write_idx = 0;
            self.cycle_bit = !self.cycle_bit;
        }
        registers.set_erdp(
            ((virt_to_phys(self.buf.as_slice().as_ptr() as usize) >> 4) + self.write_idx) as u64,
        );
        Some(trb)
    }
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn addr(&self) -> usize {
        self.buf.as_ptr().addr()
    }
}


#[derive(Default)]
pub struct PortStatusChangeEvent(GenericTrb);

impl PortStatusChangeEvent {
    pub fn port_id(&self) -> u8 {
        (self.0.data_low >> 24) as u8
    }
    pub fn completion_code(&self) -> u8 {
        (self.0.status >> 24) as u8
    }
}

impl Debug for PortStatusChangeEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "port status change event\n\tport id : {}  completion code : {}", self.port_id(), self.completion_code())
    }
}


impl From<GenericTrb> for PortStatusChangeEvent {
    fn from(value: GenericTrb) -> Self {
        assert_eq!(value.trb_type(), TrbType::PortStatusChange);
        Self(value)
    }
}

#[derive(Default)]
pub struct CommandCompletionEvent(GenericTrb);

impl CommandCompletionEvent {
    pub fn slot_id(&self) -> u8 {
        (self.0.control >> 24) as u8
    }
    pub fn command_trb_pointer(&self) -> u64 {
        self.0.data_low as u64 | (self.0.data_high as u64) << 32
    }
    pub fn completion_code(&self) -> u8 {
        (self.0.status >> 24) as u8
    }
}

impl Debug for CommandCompletionEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "command completion event\n\tslot id : {}  completion_code : {}  command trb pointer : {:#X}",
               self.slot_id(),
               self.completion_code(),
               self.command_trb_pointer())
    }
}


impl From<GenericTrb> for CommandCompletionEvent {
    fn from(value: GenericTrb) -> Self {
        assert_eq!(value.trb_type(), TrbType::CommandCompletion);
        Self(value)
    }
}


#[derive(Default, Debug)]
pub struct TransferEvent(GenericTrb);

#[repr(C)]
#[derive(Default, Clone, Debug)]
pub struct EventRingSegmentTableEntry {
    address: u64,
    size: u16,
    _rsvd: u16,
    _rsvd2: u32,
}

impl EventRingSegmentTableEntry {
    fn set_address(&mut self, address: u64) {
        self.address = address;
    }
    fn set_size(&mut self, size: u16) {
        self.size = size;
    }
}

pub struct EventRingSegmentTable {
    entries: Vec<EventRingSegmentTableEntry>,
    entries_layout: Layout,
}

impl EventRingSegmentTable {
    pub fn with_capacity(capacity: usize) -> Self {
        let layout =
            Layout::array::<EventRingSegmentTableEntry>(capacity)
                .unwrap().align_to(64).unwrap();
        unsafe {
            let addr = alloc::alloc::alloc(layout).cast::<EventRingSegmentTableEntry>();
            Self {
                entries: Vec::from_raw_parts(addr, capacity, capacity),
                entries_layout: layout,
            }
        }
    }

    pub fn set_event_ring_data(&mut self, index: usize, event_ring: &EventRing) {
        self.entries[index].set_address(virt_to_phys(event_ring.addr()) as u64);
        self.entries[index].set_size(event_ring.len() as u16);
    }

    pub fn addr(&self) -> usize {
        self.entries.as_ptr().addr()
    }
    pub fn entry_addr(&self, index: usize) -> u64 {
        self.entries[index].address
    }
    pub fn len(&self) -> u64 {
        self.entries.len() as u64
    }
}
