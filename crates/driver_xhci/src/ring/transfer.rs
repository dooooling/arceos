use alloc::vec::Vec;

use crate::ring::{GenericTrb, LinkTrb, TrbType};
use crate::virt_to_phys;

#[derive(Default, Clone, Debug)]
pub struct TransferRing {
    pub buf: Vec<GenericTrb>,
    pub cycle_bit: bool,
    pub write_idx: usize,
}

impl TransferRing {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: GenericTrb::aligned_vec(16, capacity),
            cycle_bit: true,
            write_idx: 0,
        }
    }
    pub(crate) fn push_command(&mut self, mut trb: GenericTrb) {
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


pub struct SetupStage(GenericTrb);

impl Default for SetupStage {
    fn default() -> Self {
        Self(GenericTrb::default())
    }
}

impl From<SetupStage> for GenericTrb {
    fn from(value: SetupStage) -> Self {
        value.0
    }
}

impl SetupStage {
    pub(crate) fn get_descriptor(desc_type: u16, desc_index: u16, buf_len: u32) -> Self {
        let mut trb = GenericTrb::default();
        trb.set_trb_type(TrbType::SetupStage);
        // request type
        trb.data_low = 0b10000000;
        // request
        trb.data_low |= 6 << 8;
        // value
        trb.data_low |= ((desc_type << 8 | desc_index) as u32) << 16;
        // index
        trb.data_high = 0;
        // buf len
        trb.data_high = buf_len << 16;
        // trb len
        trb.status = 8;
        // idt
        trb.control |= 1 << 6;
        // transfer type
        trb.control |= 3 << 16;
        Self(trb)
    }
}

pub struct DataStage(GenericTrb);

impl DataStage {
    pub fn data_stage(buf_addr: u64, buf_len: u32, direction: u8, interrupt_on_completion: u8) -> Self {
        let mut trb = GenericTrb::default();
        trb.set_trb_type(TrbType::DataStage);
        // buf addr
        trb.data_low = (buf_addr & 0xFFFFFFFF) as u32;
        trb.data_high = (buf_addr >> 32) as u32;
        // trb len
        trb.status = buf_len & 0x1FFFF;
        // td size
        trb.status |= 0 << 17;
        // dir
        trb.control |= (direction as u32) << 16;
        //ioc
        trb.control |= (interrupt_on_completion as u32) << 5;
        Self(trb)
    }
}

impl Default for DataStage {
    fn default() -> Self {
        Self(GenericTrb::default())
    }
}

impl From<DataStage> for GenericTrb {
    fn from(value: DataStage) -> Self {
        value.0
    }
}

pub struct StatusStage(GenericTrb);

impl StatusStage {
    pub fn status_stage() -> Self {
        let mut trb = GenericTrb::default();
        trb.set_trb_type(TrbType::StatusStage);
        // dir
        trb.control |= 1 << 16;
        Self(trb)
    }
}

impl Default for StatusStage {
    fn default() -> Self {
        Self(GenericTrb::default())
    }
}

impl From<StatusStage> for GenericTrb {
    fn from(value: StatusStage) -> Self {
        value.0
    }
}
