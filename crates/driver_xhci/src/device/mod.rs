use alloc::vec::Vec;

use crate::device::context::DeviceContext;

pub mod context;

pub struct DeviceManager {
    device_contexts: Vec<DeviceContext>,
    max_slots: usize,
    // max_slots: usize,
}
