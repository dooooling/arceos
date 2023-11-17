use core::mem::size_of;

pub const BLOCK_SIZE: u32 = 1024;
pub const BLOCK_COUNT_WIDTH: u32 = size_of::<u32>() as u32;
pub const MARGIN_SIZE_WIDTH: u32 = size_of::<u32>() as u32;
