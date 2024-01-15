use alloc::vec;
use alloc::vec::Vec;
use core::alloc::Layout;
use core::fmt;
use core::fmt::{Formatter, UpperHex};
use core::mem::size_of;

pub mod event;

pub type CommandRing = Ring;

#[derive(Default)]
pub struct Ring {
    pub buf: Vec<GenericTrb>,
    pub cycle_bit: bool,
    pub write_idx: usize,
}

impl Ring {
    pub fn with_capacity(capacity: usize) -> Self {
        unsafe {
            // let layout = Layout::from_size_align(size_of::<GenericTrb>() * capacity, 64).unwrap();
            // let ada = alloc::alloc::alloc(layout).cast::<GenericTrb>();

            Self {
                buf: Vec::from_raw_parts(ada, capacity, capacity),
                // buf: vec![GenericTrb::default(); capacity],
                cycle_bit: true,
                write_idx: 0,
            }
        }
    }
}

#[repr(C, align(16))]
#[derive(Clone, Default, Debug)]
pub struct GenericTrb {
    data_low: u32,
    data_high: u32,
    status: u32,
    control: u32,
}

impl UpperHex for GenericTrb {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let data = self.data_low as u128
            | (self.data_high as u128) << 32
            | (self.status as u128) << 64
            | (self.control as u128) << 96;
        core::fmt::UpperHex::fmt(&data, f)
    }
}

impl GenericTrb {
    /// cycle bit
    pub fn pcs(&self) -> bool {
        self.control & 0b1 == 1
    }
}

unsafe impl Sync for Ring {}

unsafe impl Send for Ring {}
