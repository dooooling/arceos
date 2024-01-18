#![feature(strict_provenance)]
#![feature(pointer_is_aligned)]
// #![feature(non_null_convenience)]
#![no_std]

extern crate alloc;

use alloc::borrow::ToOwned;

use log::{debug, error, warn};
use tock_registers::interfaces::{Readable, Writeable};

use driver_common::{BaseDriverOps, DeviceType};

use crate::device::context::DeviceContext;
use crate::device::DeviceManager;
use crate::registers::{port, Registers};
use crate::registers::capability::DBOFF;
use crate::registers::doorbell::DOORBELL;
use crate::ring::{GenericTrb, TrbType};
use crate::ring::command::CommandRing;
use crate::ring::event::{CommandCompletionEvent, EventRing, EventRingSegmentTable, PortStatusChangeEvent};

mod device;
mod registers;
mod ring;

pub struct XhciController {
    base_addr: usize,
    register: Registers,
    event_ring: EventRing,
    command_ring: CommandRing,
    device_manager: DeviceManager,
    ers_table: EventRingSegmentTable,
}

impl XhciController {
    pub fn new(address: usize, cap_addr: usize) -> Self {
        unsafe {
            let register = Registers::from(address);
            register.reset();

            let max_slots = register.max_slots();
            debug!("xhci max slots : {}", max_slots);
            register.set_max_slot_enable(max_slots);

            let mut max_scratchpad_buffer = register.max_scratchpad_buffer();
            debug!("xhci max scratch pad : {}", max_scratchpad_buffer);
            if max_scratchpad_buffer > 0 {
                unimplemented!("not implemented scratchpad");
            }

            let ac64 = register.ac64();
            debug!("xhci 64-bit Addressing Capability : {}", ac64);
            assert_eq!(ac64, 1, "not implemented 32-bit addressing");

            let page_size = register.page_size();
            debug!("xhci page size : {}", page_size);

            let context_size = register.context_size();
            assert_eq!(context_size, 0, "not support 64-bit context size");

            let dboff = register.capability().dboff.read(DBOFF::OFFSET) << 2;
            let mut device_manager = DeviceManager::new(max_slots as usize, address + dboff as usize);
            register.set_device_context_base_address_array(device_manager.device_contexts());

            let mut command_ring = CommandRing::with_capacity(1);
            command_ring.cycle_bit = true;
            register.set_command_ring(&command_ring);

            let mut event_ring = EventRing::with_capacity(32);
            let mut ers_table = EventRingSegmentTable::with_capacity(1);
            ers_table.set_event_ring_data(0, &event_ring);
            register.set_primary_interrupter(&ers_table);
            // let mut capability_iter = PciCapabilityIterator::new(cap_addr);
            // let mut pci_cap = capability_iter.find(|c| c.as_ref().capability_id == 5).unwrap();
            // let msi_cap = pci_cap.as_mut().as_msi();
            //
            // msi_cap.set_address(current_cpu_id() as u8, 0, 0);
            // msi_cap.set_data(1, 0x60, 0, 1);
            // msi_cap.set_control(1, 0);

            register.run();


            let mut port_set = port::PortSet::new(register.max_ports(), register.operational_addr());
            for port in &mut port_set {
                let ccs = port.as_ref().connected();
                if ccs {
                    port.as_ref().reset();
                    error!("ccs = {:?}", ccs);
                    error!("port address {:#X}", port.addr().get());
                }
            }

            loop {
                if let Some(trb) = event_ring.next(&register) {
                    match trb.trb_type() {
                        TrbType::PortStatusChange => {
                            let port_status_change: PortStatusChangeEvent = trb.clone().into();
                            debug!("{:?}", port_status_change);
                            let port_id = port_status_change.port_id();
                            let completion_code = port_status_change.completion_code();
                            port_set.enable_port(port_id);
                            command_ring.push_enable_slot_command();
                            device_manager.doorbells().doorbell.write(DOORBELL::DB_TARGET.val(0));
                            device_manager.doorbells().doorbell.write(DOORBELL::DB_STREAM_ID.val(0));
                            device_manager.set_addressing_port(port_id);
                        }
                        TrbType::CommandCompletion => {
                            let command_completion: CommandCompletionEvent = trb.clone().into();
                            debug!("{:?}", command_completion);
                            let addr = phys_to_virt(command_completion.command_trb_pointer() as usize);
                            let cmd_trb = &*(addr as *mut GenericTrb);
                            debug!("{:?}", cmd_trb);

                            match cmd_trb.trb_type() {
                                TrbType::Reserved => {}
                                TrbType::Normal => {}
                                TrbType::SetupStage => {}
                                TrbType::DataStage => {}
                                TrbType::StatusStage => {}
                                TrbType::Isoch => {}
                                TrbType::Link => {}
                                TrbType::EventData => {}
                                TrbType::NoOp => {}
                                TrbType::EnableSlot => {
                                    match device_manager.get_addressing_port() {
                                        Some(port_id) => {
                                            let port = port_set.get_by_id(port_id);
                                            let slot_id = command_completion.slot_id();
                                            assert!(slot_id > 0 && slot_id < device_manager.devices().len() as u8, "invalid slot id");
                                            let device_context = DeviceContext::default();

                                        }
                                        None => {
                                            warn!("addressing port is None");
                                        }
                                    }
                                }
                                TrbType::DisableSlot => {}
                                TrbType::AddressDevice => {}
                                TrbType::ConfigureEndpoint => {}
                                TrbType::EvaluateContext => {}
                                TrbType::ResetEndpoint => {}
                                TrbType::StopEndpoint => {}
                                TrbType::SetTrDequeuePointer => {}
                                TrbType::ResetDevice => {}
                                TrbType::ForceEvent => {}
                                TrbType::NegotiateBandwidth => {}
                                TrbType::SetLatencyToleranceValue => {}
                                TrbType::GetPortBandwidth => {}
                                TrbType::ForceHeader => {}
                                TrbType::NoOpCmd => {}
                                TrbType::GetExtendedProperty => {}
                                TrbType::SetExtendedProperty => {}
                                TrbType::Transfer => {}
                                TrbType::CommandCompletion => {}
                                TrbType::PortStatusChange => {}
                                TrbType::BandwidthRequest => {}
                                TrbType::Doorbell => {}
                                TrbType::HostController => {}
                                TrbType::DeviceNotification => {}
                                TrbType::MfindexWrap => {}
                                _ => {}
                            }
                        }
                        _ => {
                            error!(" unimplemented trb type !")
                        }
                    }
                }
            }

            Self {
                base_addr: address,
                register,
                command_ring,
                event_ring,
                device_manager,
                ers_table,
            }
        }
    }
}

impl Drop for XhciController {
    fn drop(&mut self) {
        debug!("123");
    }
}

/// The information of the xhci device.
#[derive(Debug, Clone, Copy)]
pub struct XhciInfo {}

pub trait XhciDriverOps: BaseDriverOps {
    /// Get the xhci information.
    fn info(&self) -> XhciInfo;
}

impl BaseDriverOps for XhciController {
    fn device_name(&self) -> &str {
        "xhci-controller"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Xhci
    }
}

impl XhciDriverOps for XhciController {
    fn info(&self) -> XhciInfo {
        XhciInfo {}
    }
}

fn current_cpu_id() -> usize {
    match raw_cpuid::CpuId::new().get_feature_info() {
        Some(finfo) => finfo.initial_local_apic_id() as usize,
        None => 0,
    }
}

#[inline]
pub const fn virt_to_phys(vaddr: usize) -> usize {
    vaddr - 0xffff_ff80_0000_0000
}

#[inline]
pub const fn phys_to_virt(paddr: usize) -> usize {
    paddr + 0xffff_ff80_0000_0000
}
