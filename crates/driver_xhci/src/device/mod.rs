use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::ptr::NonNull;

use crate::device::context::DeviceContext;
use crate::registers::doorbell::Doorbell;
use crate::registers::runtime::Runtime;

pub mod context;


pub struct DeviceManager {
    device_contexts: Vec<DeviceContext>,
    devices: Vec<Device>,
    doorbells: Arc<NonNull<Doorbell>>,
}

unsafe impl Sync for DeviceManager {}

unsafe impl Send for DeviceManager {}

impl DeviceManager {
    pub fn new(max_slot: usize, db_addr: usize) -> Self {
        Self {
            device_contexts: DeviceContext::aligned_vec(max_slot + 1),
            devices: vec![],
            doorbells: Arc::new(NonNull::new(db_addr as *mut u8).unwrap().cast()),
        }
    }

    ///  Device Context Base Address Array Pointer
    pub fn dcbaap(&self) -> usize {
        self.device_contexts.as_ptr().addr()
    }

    pub fn device_contexts(&self) -> &Vec<DeviceContext> {
        &self.device_contexts
    }
    #[inline]
    pub fn doorbells(&self) -> &Doorbell {
        unsafe { self.doorbells.as_ref().as_ref() }
    }
}

#[derive(Debug, Default)]
pub struct Device {
    slot_id: u8,
}