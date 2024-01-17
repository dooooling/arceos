use alloc::vec::Vec;

use crate::ring::{GenericTrb, LinkTrb, TrbType};
use crate::virt_to_phys;

#[derive(Default)]
pub struct CommandRing {
    pub buf: Vec<GenericTrb>,
    pub cycle_bit: bool,
    pub write_idx: usize,
}


impl CommandRing {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: GenericTrb::aligned_vec(64, capacity),
            cycle_bit: true,
            write_idx: 0,
        }
    }

    pub fn push_enable_slot_command(&mut self) {
        let mut trb = GenericTrb::default();
        trb.set_trb_type(TrbType::EnableSlot);
        trb.set_pcs(self.cycle_bit);
        self.buf[self.write_idx] = trb;
        self.write_idx += 1;
        if self.write_idx + 1 == self.buf.len() {
            self.back_to_head();
        }
    }

    fn back_to_head(&mut self) {
        let mut link_trb = LinkTrb::new(virt_to_phys(self.buf.as_ptr().addr()));
        link_trb.set_tc(true);
        self.buf[self.write_idx] = link_trb.cast_trb();
        self.write_idx = 0;
        self.cycle_bit = !self.cycle_bit;
    }
}


