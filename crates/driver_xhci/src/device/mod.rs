use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::alloc::Layout;
use core::mem::{size_of, size_of_val};
use core::ptr::NonNull;

use log::{error, info};
use tock_registers::interfaces::{ReadWriteable, Writeable};

use crate::device::context::{DeviceContext, InputContext, SlotContext};
use crate::device::descriptor::{ConfigurationDescriptor, ConfigurationDescriptorPack, Descriptor, DescriptorSet, DeviceDescriptor, InterfaceDescriptor};
use crate::registers::doorbell::{Doorbell, DOORBELL};
use crate::ring::event::TransferEvent;
use crate::ring::transfer::{DataStage, Normal, SetupStage, StatusStage, TransferRing};
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
        let layout = Layout::from_size_align(size_of::<u64>() * max_slot + 1, 64).unwrap();
        unsafe {
            let addr = alloc::alloc::alloc(layout).cast();
            let vec = Vec::from_raw_parts(addr, max_slot + 1, max_slot + 1);
            Self {
                device_contexts: vec,
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

    pub fn device_contexts(&self) -> usize {
        self.device_contexts.as_ptr().addr()
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

    pub fn enable_device(&mut self, slot_id: usize) -> &mut Device {
        self.devices[slot_id] = Some(Device::new(slot_id as u8, self.doorbell_at(slot_id)));
        self.device_contexts[slot_id] = virt_to_phys(core::ptr::addr_of_mut!(self.devices[slot_id].as_mut().unwrap().device_context).addr()) as u64;
        self.devices[slot_id].as_mut().unwrap()
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

    pub fn process_transfer_event(&mut self, transfer_event: &TransferEvent) {
        let device = self.devices.get_mut(transfer_event.slot_id() as usize)
            .unwrap()
            .as_mut()
            .unwrap();
        if device.init_phase == InitPhase::SetConfiguration {
            device.device_init_configuration();
            // device
        } else { device.receive_transfer_event(transfer_event); }
    }
}

pub(crate) const DATA_BUFF_SIZE: usize = 256;

#[derive(Clone, Debug, PartialOrd, PartialEq)]
pub enum InitPhase {
    Uninitialized,
    GetDeviceDescriptor,
    GetConfigurationDescriptor,
    SetConfiguration,
    WaitConfigureCommand(u8),
    Finish(u8),
}

#[derive(Debug, Clone)]
pub struct Device {
    pub(crate) slot_id: u8,
    pub(crate) input_context: InputContext,
    pub(crate) device_context: DeviceContext,
    pub(crate) doorbell: NonNull<Doorbell>,
    // pub(crate) transfer_ring: TransferRing,
    pub(crate) device_descriptor_buf: [u8; DATA_BUFF_SIZE],
    pub(crate) init_phase: InitPhase,
    pub(crate) descriptor_set: Option<DescriptorSet>,
    pub(crate) transfer_rings: Vec<Option<TransferRing>>,
    pub(crate) receive_data_buf: [u8; 16],
}

impl Device {
    pub fn new(slot_id: u8, doorbell: NonNull<Doorbell>) -> Self {
        let input_context = InputContext::default();
        let device_context = DeviceContext::default();
        Device {
            slot_id,
            input_context,
            device_context,
            doorbell,
            device_descriptor_buf: [0; 256],
            init_phase: InitPhase::Uninitialized,
            descriptor_set: None,
            transfer_rings: vec![None; 32],
            receive_data_buf: [0; 16],
        }
    }
    fn control_transfer_ring(&mut self) -> &mut TransferRing {
        self.get_transfer_ring(0)
    }
    fn get_transfer_ring(&mut self, dci: usize) -> &mut TransferRing {
        if let Some(_) = self.transfer_rings.get(dci).unwrap() {
            self.transfer_rings[dci].as_mut().unwrap()
        } else {
            self.transfer_rings[dci] = Some(TransferRing::with_capacity(32));
            self.transfer_rings[dci].as_mut().unwrap()
        }
    }
    fn buf_addr(&mut self) -> usize {
        self.device_descriptor_buf.as_ptr().addr()
    }
    pub fn init_default_control_endpoint(&mut self, max_packet_size: u16) {
        let ring_buf_addr = self.control_transfer_ring().buf.as_ptr().addr();
        self.input_context.init_default_control_endpoint(max_packet_size, virt_to_phys(ring_buf_addr) as u64);
    }
    pub fn init(&mut self) {
        self.get_device_descriptor();
    }
    pub fn receive_transfer_event(&mut self, transfer_event: &TransferEvent) {
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
                self.configuration(transfer_event);
                unsafe {
                    let descriptor_set = DescriptorSet::new(self.device_descriptor_buf.as_mut_ptr(), self.device_descriptor_buf.len() - transfer_event.transfer_length() as usize);
                    self.descriptor_set = Some(descriptor_set);
                }
            }
            InitPhase::Finish(dci) => unsafe {
                info!("{:?}",self.receive_data_buf);
                let mut normal = Normal::new();
                normal.set_data_buffer_pointer(virt_to_phys(self.receive_data_buf.as_mut_ptr().addr()) as u64);
                normal.set_interrupt_on_short_packet(true);
                normal.set_trb_transfer_length(self.receive_data_buf.len() as u32);
                normal.set_interrupt_on_completion(true);

                let transfer_ring = self.get_transfer_ring(dci as usize);
                transfer_ring.push_command(normal.into());

                self.doorbell.as_mut().doorbell.modify(DOORBELL::DB_STREAM_ID.val(0));
                self.doorbell.as_mut().doorbell.modify(DOORBELL::DB_TARGET.val(dci as u32));
            }
            InitPhase::SetConfiguration => { error!("{:?}",InitPhase::SetConfiguration) }
            InitPhase::WaitConfigureCommand(dci) => { error!("{:?}",InitPhase::WaitConfigureCommand(dci)) }
        }
    }
    fn get_device_descriptor(&mut self) {
        let buf_len = self.device_descriptor_buf.len() as u16;
        let buf_addr = self.device_descriptor_buf.as_ptr() as usize;
        self.control_transfer_ring().push_command(SetupStage::get_descriptor(1, 0, buf_len).into());
        self.control_transfer_ring().push_command(DataStage::data_stage(virt_to_phys(buf_addr) as u64, buf_len, 1, 1).into());
        let mut status_stage = StatusStage::new();
        status_stage.set_direction(1);
        self.control_transfer_ring().push_command(status_stage.into());
        self.ring();
        self.init_phase = InitPhase::GetDeviceDescriptor;
    }
    fn get_configuration_descriptor(&mut self) {
        let buf_len = self.device_descriptor_buf.len() as u16;
        let buf_addr = self.device_descriptor_buf.as_ptr() as usize;
        self.control_transfer_ring().push_command(SetupStage::get_descriptor(2, 0, buf_len).into());
        self.control_transfer_ring().push_command(DataStage::data_stage(virt_to_phys(buf_addr) as u64, buf_len, 1, 1).into());
        let mut status_stage = StatusStage::new();
        status_stage.set_direction(1);
        self.control_transfer_ring().push_command(status_stage.into());
        self.ring();
        self.init_phase = InitPhase::GetConfigurationDescriptor;
    }

    pub fn on_endpoints_configured(&mut self) {
        //设置引导协议
        let mut setup_stage = SetupStage::new();
        setup_stage.set_value(0);
        setup_stage.set_request_type(0b00100001);
        setup_stage.set_request(11);
        setup_stage.set_index(0);
        self.control_transfer_ring().push_command(setup_stage.into());

        let mut status_stage = StatusStage::new();
        status_stage.set_direction(1);
        status_stage.set_interrupt_on_completion(true);
        self.control_transfer_ring().push_command(status_stage.into());
        self.ring();
    }

    fn configuration(&mut self, transfer_event: &TransferEvent) {
        unsafe {
            let mut descriptor_set = DescriptorSet::new(self.device_descriptor_buf.as_mut_ptr(), self.device_descriptor_buf.len() - transfer_event.transfer_length() as usize);

            let config = &descriptor_set.find(|desc| {
                match desc {
                    Descriptor::Configuration(_) => { true }
                    _ => false
                }
            }).map(|desc| {
                match desc {
                    Descriptor::Configuration(config) => { Some(config) }
                    _ => None
                }
            }).unwrap().unwrap();
            let mut setup_state = SetupStage::new();
            setup_state.set_request(9);
            setup_state.set_request_type(0);
            setup_state.set_value(config.b_configuration_value as _);
            setup_state.set_index(0);
            setup_state.set_length(0);
            setup_state.set_transfer_type(0);
            self.control_transfer_ring().push_command(setup_state.into());

            let mut status_stage = StatusStage::new();
            status_stage.set_interrupt_on_completion(true);
            status_stage.set_direction(1);
            self.control_transfer_ring().push_command(status_stage.into());
            self.ring();
            self.init_phase = InitPhase::SetConfiguration;
        }
    }

    fn device_init_configuration(&mut self) {
        self.input_context.clear_add_context();
        unsafe {
            core::ptr::copy(&self.device_context.slot_ctx as *const SlotContext,
                            self.input_context.mut_slot() as *mut SlotContext, 1);
        }
        self.input_context.enable_slot_context();

        self.input_context.mut_slot().set_context_entries(31);
        for desc in self.descriptor_set.unwrap() {
            info!("desc :{:?}", desc);
        }

        let endpoint_desc = self.descriptor_set.unwrap().filter(|desc| match desc {
            Descriptor::Endpoint(_) => { true }
            _ => { false }
        }).find_map(|desc| if let Descriptor::Endpoint(endpoint_desc) = desc {
            Some(endpoint_desc)
        } else {
            None
        }).unwrap();
        let dci = Self::calculate_dci(endpoint_desc.b_endpoint_address);
        self.input_context.set_add_context(dci as u32, 1);
        let transfer_ring = self.get_transfer_ring(dci as _);
        let ring_buf_addr = virt_to_phys(transfer_ring.buf.as_ptr().addr()) as u64;
        let endpoint = self.input_context.ep_ctxs.get_mut((dci - 1) as usize).unwrap();


        endpoint.set_max_packet_size(endpoint_desc.w_max_packet_size);
        endpoint.set_interval(endpoint_desc.b_interval - 1);
        endpoint.set_average_trb_length(1);
        endpoint.set_endpoint_state(1);
        endpoint.set_error_count(3);
        endpoint.set_transfer_ring_buffer(ring_buf_addr);
        endpoint.set_endpoint_type(7);
        endpoint.set_dequeue_cycle_state(1);
        self.init_phase = InitPhase::WaitConfigureCommand(dci);
    }
    fn ring(&mut self) {
        unsafe {
            self.doorbell.as_mut().doorbell.modify(DOORBELL::DB_STREAM_ID.val(0));
            self.doorbell.as_mut().doorbell.modify(DOORBELL::DB_TARGET.val(1));
        }
    }

    fn calculate_dci(b_endpoint_address: u8) -> u8 {
        2 * (b_endpoint_address & 0xF) + (b_endpoint_address >> 7)
    }
}
