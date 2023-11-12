#![no_std]
extern crate alloc;

use alloc::vec::Vec;

mod dtb;
pub use dtb::DtbParseError;

pub struct DtbInfo {
    pub memory_addr: usize,
    pub memory_size: usize,
    pub mmio_regions: Vec<(usize, usize)>,
}

pub fn parse_dtb(dtb_pa: usize) -> Result<DtbInfo, DtbParseError> {
    let wrapper = dtb::DtbWrapper::parse(dtb_pa)?;
    let memory = wrapper.memory_node();
    Ok(DtbInfo {
        memory_addr: memory.0,
        memory_size: memory.1,
        mmio_regions: wrapper.virtio_mmio_node(),
    })
}
