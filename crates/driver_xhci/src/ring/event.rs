use crate::registers::Registers;
use crate::ring::GenericTrb;
use crate::virt_to_phys;
use alloc::vec;
use alloc::vec::Vec;
use core::alloc::Layout;
use core::mem::size_of;
use log::debug;

#[derive(Default, Debug)]
pub struct EventRing {
    pub buf: Vec<GenericTrb>,
    pub cycle_bit: bool,
    pub write_idx: usize,
}

impl EventRing {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: vec![GenericTrb::default(); capacity],
            cycle_bit: true,
            write_idx: 0,
        }
    }

    pub fn next(&mut self, registers: &Registers) -> Option<&GenericTrb> {
        let trb = &self.buf[self.write_idx];
        debug!("pcs : {:?}", trb.pcs());
        if trb.pcs() != self.cycle_bit {
            return None;
        }
        debug!(
            "trb address : {:?}",
            (&self.buf[self.write_idx] as *const GenericTrb).addr()
        );
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
            Layout::from_size_align(size_of::<EventRingSegmentTableEntry>() * capacity, 64)
                .unwrap();

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

impl Drop for EventRingSegmentTable {
    fn drop(&mut self) {
        unsafe { alloc::alloc::dealloc(self.entries.as_mut_ptr().cast(), self.entries_layout) }
    }
}
