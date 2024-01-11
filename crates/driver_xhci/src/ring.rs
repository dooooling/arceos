use alloc::vec;
use alloc::vec::Vec;

use log::debug;

use crate::registers::Registers;

pub type CommandRing = Ring;

#[derive(Default, Debug)]
pub struct EventRing {
    pub buf: Vec<Trb>,
    pub cycle_bit: bool,
    pub write_idx: usize,
}

#[derive(Default)]
pub struct Ring {
    pub buf: Vec<Trb>,
    pub cycle_bit: bool,
    pub write_idx: usize,
}

impl Ring {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: vec![Trb::default(); capacity],
            cycle_bit: true,
            write_idx: 0,
        }
    }
}

impl EventRing {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: vec![Trb::default(); capacity],
            cycle_bit: true,
            write_idx: 0,
        }
    }

    pub fn next(&mut self, registers: &Registers) -> Option<&Trb> {
        let trb = &self.buf[self.write_idx];
        // debug!("pcs : {:?}",trb.pcs());
        if trb.pcs() != self.cycle_bit {
            return None;
        }
        debug!(
            "trb address : {:?}",
            (&self.buf[self.write_idx] as *const Trb).addr()
        );
        self.write_idx += 1;
        if self.write_idx == self.buf.len() {
            self.write_idx = 0;
            self.cycle_bit = !self.cycle_bit;
        }
        registers.set_erdp((self.buf.as_slice().as_ptr() as u64) >> 4 + self.write_idx);
        Some(trb)
    }
}

#[repr(C, align(16))]
#[derive(Clone, Default, Debug)]
pub struct Trb {
    pub data: [u32; 4],
}

impl Trb {
    pub fn pcs(&self) -> bool {
        self.data[3] & 0x01 == 1
    }
}

#[repr(C, packed(4))]
#[derive(Default, Clone, Debug)]
pub struct EventRingSegmentTableEntry {
    pub data: [u64; 2],
}

unsafe impl Sync for Ring {}

unsafe impl Send for Ring {}
