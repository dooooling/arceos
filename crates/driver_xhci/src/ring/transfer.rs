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
            buf: GenericTrb::aligned_vec(64, capacity),
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
        link_trb.0.set_pcs(self.cycle_bit);
        self.buf[self.write_idx] = link_trb.cast_trb();
        self.write_idx = 0;
        self.cycle_bit = !self.cycle_bit;
    }
}

pub struct Normal(GenericTrb);

impl Normal {
    pub fn new() -> Self {
        let mut generic_trb = GenericTrb::default();
        generic_trb.set_trb_type(TrbType::Normal);
        Self(generic_trb)
    }

    pub fn set_data_buffer_pointer(&mut self, data_buffer_pointer: u64) {
        self.0.data_low |= (data_buffer_pointer & 0xFFFFFFFF) as u32;
        self.0.data_high |= (data_buffer_pointer >> 32) as u32;
    }
    pub fn set_interrupt_on_short_packet(&mut self, interrupt_on_short_packet: bool) {
        self.0.control |= if interrupt_on_short_packet { 1 << 2 } else { 0 << 2 };
    }
    pub fn set_trb_transfer_length(&mut self, trb_transfer_length: u32) {
        self.0.status |= trb_transfer_length;
    }
    pub fn set_interrupt_on_completion(&mut self, interrupt_on_completion: bool) {
        self.0.control |= if interrupt_on_completion { 1 } else { 0 } << 5;
    }
}

impl From<Normal> for GenericTrb {
    fn from(value: Normal) -> Self {
        value.0
    }
}

pub struct SetupStage(GenericTrb);


impl From<SetupStage> for GenericTrb {
    fn from(value: SetupStage) -> Self {
        value.0
    }
}

impl SetupStage {
    pub(crate) fn get_descriptor(desc_type: u16, desc_index: u16, buf_len: u16) -> Self {
        let mut setup_stage = SetupStage::new();
        // request type
        setup_stage.set_request_type(0b10000000);
        // request
        setup_stage.set_request(6);
        // value
        setup_stage.set_value((desc_type << 8 | desc_index));
        // index
        setup_stage.set_index(0);
        // buf len
        setup_stage.set_length(buf_len);
        // transfer type
        setup_stage.set_transfer_type(3);
        setup_stage
    }

    pub fn new() -> Self {
        let mut trb = GenericTrb::default();
        trb.set_trb_type(TrbType::SetupStage);

        let mut val = Self(trb);
        val.set_idt(true);
        val.set_trb_transfer_length();
        val
    }
    pub fn set_idt(&mut self, idt: bool) {
        let idt = if idt { 1 } else { 0 };
        self.0.control |= idt << 6;
    }
    pub fn set_trb_transfer_length(&mut self) {
        self.0.status |= 8;
    }
    pub fn set_request_type(&mut self, request_type: u8) {
        self.0.data_low |= request_type as u32;
    }
    pub fn set_request(&mut self, request: u8) {
        self.0.data_low |= (request as u32) << 8;
    }
    pub fn set_value(&mut self, value: u16) {
        self.0.data_low |= (value as u32) << 16;
    }
    pub fn set_index(&mut self, index: u16) {
        self.0.data_high |= index as u32;
    }
    pub fn set_length(&mut self, length: u16) {
        self.0.data_high |= (length as u32) << 16;
    }
    pub fn set_interrupter_target(&mut self, interrupter_target: u16) {
        self.0.status |= (interrupter_target as u32) << 22;
    }
    pub fn set_transfer_type(&mut self, transfer_type: u8) {
        let transfer_type = transfer_type & 0b11;
        self.0.control |= (transfer_type as u32) << 16;
    }
}

pub struct DataStage(GenericTrb);

impl DataStage {
    pub fn data_stage(buf_addr: u64, buf_len: u16, direction: u8, interrupt_on_completion: u8) -> Self {
        let mut trb = GenericTrb::default();
        trb.set_trb_type(TrbType::DataStage);
        // buf addr
        trb.data_low = (buf_addr & 0xFFFFFFFF) as u32;
        trb.data_high = (buf_addr >> 32) as u32;
        // trb len
        trb.status = buf_len as u32 & 0x1FFFF;
        // td size
        trb.status |= 0 << 17;
        // dir
        trb.control |= (direction as u32) << 16;
        //ioc
        trb.control |= (interrupt_on_completion as u32) << 5;
        Self(trb)
    }
}

impl From<DataStage> for GenericTrb {
    fn from(value: DataStage) -> Self {
        value.0
    }
}

pub struct StatusStage(GenericTrb);

impl StatusStage {
    pub fn new() -> Self {
        let mut trb = GenericTrb::default();
        trb.set_trb_type(TrbType::StatusStage);
        Self(trb)
    }

    pub fn set_interrupt_on_completion(&mut self, interrupt_on_completion: bool) {
        self.0.control |= if interrupt_on_completion { 1 } else { 0 } << 5;
    }
    pub fn set_direction(&mut self, direction: u8) {
        let direction = direction & 0b1;
        self.0.control |= (direction as u32) << 16;
    }
}

impl From<StatusStage> for GenericTrb {
    fn from(value: StatusStage) -> Self {
        value.0
    }
}
