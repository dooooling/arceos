use alloc::vec::Vec;

pub struct Ring {
    buf: Vec<Trb>,
    cycle_bit: bool,
    write_idx: usize,
}

#[repr(C, align(16))]
#[derive(Clone, Default)]
pub struct Trb {
    pub data: [u32; 4],
}

unsafe impl Sync for Ring {}
unsafe impl Send for Ring {}
