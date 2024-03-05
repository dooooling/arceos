use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::alloc::Layout;
use core::mem::size_of_val;
use core::ptr::NonNull;

use log::info;
use tock_registers::interfaces::Writeable;

use crate::device::context::{DeviceContext, InputContext};
use crate::device::descriptor::{ConfigurationDescriptor, ConfigurationDescriptorPack, DeviceDescriptor, InterfaceDescriptor};
use crate::registers::doorbell::{Doorbell, DOORBELL};
use crate::ring::transfer::{DataStage, SetupStage, StatusStage, TransferRing};
use crate::virt_to_phys;

pub mod context;
mod phase;
pub mod descriptor;


pub struct DeviceManager {
    device_contexts: Vec<u64>,
    devices: Vec<Option<Device>>,
    doorbells: Arc<NonNull<Doorbell>>,
    addressing_port: Option<u8>,
}

unsafe impl Sync for DeviceManager {}

unsafe impl Send for DeviceManager {}

impl DeviceManager {
    pub fn new(max_slot: usize, db_addr: usize) -> Self {
        let layout = Layout::array::<u64>(max_slot + 1).unwrap().align_to(64).unwrap();
        unsafe {
            let addr = alloc::alloc::alloc(layout).cast();
            Self {
                device_contexts: Vec::from_raw_parts(addr, max_slot + 1, max_slot + 1),
                devices: vec![None; max_slot + 1],
                doorbells: Arc::new(NonNull::new(db_addr as *mut u8).unwrap().cast()),
                addressing_port: None,
            }
        }
    }

    ///  Device Context Base Address Array Pointer
    pub fn dcbaap(&self) -> usize {
        self.device_contexts.as_ptr().addr()
    }

    pub fn device_contexts(&self) -> &Vec<u64> {
        &self.device_contexts
    }
    pub fn set_context(&mut self, slot_id: usize, device_context_addr: u64) {
        self.device_contexts[slot_id] = device_context_addr;
    }
    #[inline]
    pub fn doorbells(&self) -> &Doorbell {
        unsafe { self.doorbells.as_ref().as_ref() }
    }
    #[inline]
    pub fn doorbell_at(&self, index: usize) -> NonNull<Doorbell> {
        unsafe { NonNull::new(self.doorbells.as_ptr().offset(index as isize)).unwrap() }
    }

    #[inline]
    pub fn devices(&self) -> &Vec<Option<Device>> {
        &self.devices
    }
    #[inline]
    pub fn devices_mut(&mut self) -> &mut Vec<Option<Device>> {
        &mut self.devices
    }

    #[inline]
    pub fn set_device(&mut self, slot_id: usize, device: Device) {
        self.devices[slot_id] = Some(device)
    }

    pub fn set_addressing_port(&mut self, port: u8) -> Option<u8> {
        self.addressing_port.replace(port)
    }
    pub fn get_addressing_port(&self) -> Option<u8> {
        self.addressing_port
    }

    pub fn init_device(&mut self, slot_id: usize) {
        self.devices.get_mut(slot_id)
            .unwrap()
            .as_mut()
            .unwrap()
            .init();
    }

    pub fn process_transfer_event(&mut self, slot_id: usize) {
        self.devices.get_mut(slot_id)
            .unwrap()
            .as_mut()
            .unwrap()
            .receive_transfer_event();
    }
}

pub(crate) const DATA_BUFF_SIZE: usize = 256;

#[derive(Clone, Debug)]
pub enum InitPhase {
    Uninitialized,
    GetDeviceDescriptor,
    GetConfigurationDescriptor,
    Finish,
}

#[derive(Debug, Clone)]
pub struct Device {
    pub(crate) slot_id: u8,
    pub(crate) input_context: InputContext,
    pub(crate) device_context: DeviceContext,
    pub(crate) doorbell: NonNull<Doorbell>,
    pub(crate) transfer_ring: TransferRing,
    pub(crate) device_descriptor_buf: [u8; DATA_BUFF_SIZE],
    pub(crate) init_phase: InitPhase,
}

impl Device {
    pub fn init(&mut self) {
        self.get_device_descriptor();
    }
    pub fn receive_transfer_event(&mut self) {
        match self.init_phase {
            InitPhase::Uninitialized => {
                panic!("usb device not init!");
            }
            InitPhase::GetDeviceDescriptor => {
                unsafe {
                    let dec: DeviceDescriptor = (self.device_descriptor_buf.as_ptr() as *const DeviceDescriptor).read();
                    info!("device descriptor:\n{:?}",dec);
                }
                self.get_configuration_descriptor();
            }
            InitPhase::GetConfigurationDescriptor => {
                unsafe {
                    let mut addr = self.device_descriptor_buf.as_ptr();
                    let pack: ConfigurationDescriptorPack = addr.into();
                    info!("configuration descriptor:\n{:?}",pack);


                }
            }
            InitPhase::Finish => {}
        }
    }
    fn get_device_descriptor(&mut self) {
        let buf_len = self.device_descriptor_buf.len() as u32;
        self.transfer_ring.push_command(SetupStage::get_descriptor(1, 0, buf_len).into());
        self.transfer_ring.push_command(DataStage::data_stage(virt_to_phys(self.device_descriptor_buf.as_ptr() as usize) as u64, buf_len, 1, 1).into());
        self.transfer_ring.push_command(StatusStage::status_stage().into());
        self.ring();
        self.init_phase = InitPhase::GetDeviceDescriptor;
    }
    fn get_configuration_descriptor(&mut self) {
        let buf_len = self.device_descriptor_buf.len() as u32;
        self.transfer_ring.push_command(SetupStage::get_descriptor(2, 0, buf_len).into());
        self.transfer_ring.push_command(DataStage::data_stage(virt_to_phys(self.device_descriptor_buf.as_ptr() as usize) as u64, buf_len, 1, 1).into());
        self.transfer_ring.push_command(StatusStage::status_stage().into());
        self.ring();
        self.init_phase = InitPhase::GetConfigurationDescriptor;
    }

    fn ring(&mut self) {
        unsafe {
            self.doorbell.as_mut().doorbell.write(DOORBELL::DB_STREAM_ID.val(0));
            self.doorbell.as_mut().doorbell.write(DOORBELL::DB_TARGET.val(1));
        }
    }
}
