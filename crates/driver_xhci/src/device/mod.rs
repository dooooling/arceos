use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::ptr::NonNull;

use crate::device::context::DeviceContext;
use crate::registers::doorbell::Doorbell;

pub mod context;
mod phase;


pub struct DeviceManager {
    device_contexts: Vec<DeviceContext>,
    devices: Vec<Device>,
    doorbells: Arc<NonNull<Doorbell>>,
    addressing_port: Option<u8>,
}

unsafe impl Sync for DeviceManager {}

unsafe impl Send for DeviceManager {}

impl DeviceManager {
    pub fn new(max_slot: usize, db_addr: usize) -> Self {
        Self {
            device_contexts: DeviceContext::aligned_vec(max_slot + 1),
            devices: vec![Device::default(); max_slot + 1],
            doorbells: Arc::new(NonNull::new(db_addr as *mut u8).unwrap().cast()),
            addressing_port: None,
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

    #[inline]
    pub fn devices(&self) -> &Vec<Device> {
        &self.devices
    }

    pub fn set_addressing_port(&mut self, port: u8) -> Option<u8> {
        self.addressing_port.replace(port)
    }
    pub fn get_addressing_port(&self) -> Option<u8> {
        self.addressing_port
    }
}

#[derive(Debug, Default, Clone)]
pub struct Device {
    slot_id: u8,
}