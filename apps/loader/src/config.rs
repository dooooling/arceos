use core::mem::size_of;

pub const BLOCK_SIZE: u32 = 1024;
pub const APP_COUNT_WIDTH: usize = size_of::<u8>() ;
pub const BLOCK_COUNT_WIDTH: usize = size_of::<u32>();
pub const MARGIN_SIZE_WIDTH: usize = size_of::<u32>();
