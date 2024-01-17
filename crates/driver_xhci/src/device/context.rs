use alloc::vec::Vec;
use core::alloc::Layout;
use crate::ring::GenericTrb;

#[repr(C, align(64))]
#[derive(Default, Clone)]
pub struct DeviceContext {
    pub slot: SlotContext,
    pub endpoints: [EndpointContext; 31],
}

impl DeviceContext {
    pub fn aligned_vec(capacity: usize) -> Vec<DeviceContext> {
        unsafe {
            let layout = Layout::array::<DeviceContext>(capacity).unwrap().align_to(64).unwrap();
            let addr = alloc::alloc::alloc(layout).cast();
            Vec::from_raw_parts(addr, capacity, capacity)
        }
    }
}

#[repr(C, align(32))]
#[derive(Default, Clone)]
struct SlotContext {
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    _rsvd: [u32; 4],
}

#[repr(C, align(32))]
#[derive(Default, Clone)]
pub struct EndpointContext {
    a: u32,
    b: u32,
    trdpl: u32,
    trdph: u32,
    c: u32,
    _rsvd: [u32; 3],
}
