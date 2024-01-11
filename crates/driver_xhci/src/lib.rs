#![feature(strict_provenance)]
#![no_std]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use core::alloc::GlobalAlloc;

use log::{debug, error};
use tock_registers::interfaces::Readable;

use device::context::DeviceContext;
use driver_common::{BaseDriverOps, DeviceType};

use crate::registers::{port, Registers};
use crate::ring::{CommandRing, EventRing, EventRingSegmentTableEntry};

mod registers;
mod ring;
mod device;

pub struct XhciController {
    base_addr: usize,
    register: Registers,
    command_ring: CommandRing,
    event_ring: EventRing,
    device_contexts: Vec<DeviceContext>,
    ers_table: Vec<EventRingSegmentTableEntry>,
}

impl XhciController {
    pub fn new(address: usize, cap_addr: usize) -> Self {
        unsafe {
            let register = Registers::from(address);
            register.reset();
            register.set_max_slot_enable(register.max_slots());

            let max_slots = register.max_slots();
            debug!("xhci max slots : {}", max_slots);

            let mut max_scratchpad_buffer = register.max_scratchpad_buffer();
            debug!("xhci max scratch pad : {}", max_scratchpad_buffer);
            if max_scratchpad_buffer > 0 {
                unimplemented!("not implemented scratchpad")
            }

            let ac64 = register.ac64();
            debug!("xhci 64-bit Addressing Capability : {}", ac64);
            assert_eq!(ac64, 1, "not implemented 32-bit addressing");

            let page_size = register.page_size();
            debug!("xhci page size : {}", page_size);

            let context_size = register.context_size();
            assert_eq!(context_size, 0, "not support 64-bit context size");


            let dcbaa = vec![DeviceContext::default(); (max_slots + 1) as usize];
            register.set_device_context_base_address_array(&dcbaa);

            let mut command_ring = CommandRing::with_capacity(32);
            command_ring.cycle_bit = true;
            register.set_command_ring(&command_ring);

            let mut event_ring = EventRing::with_capacity(32);
            event_ring.cycle_bit = true;

            let mut ers_table = vec![EventRingSegmentTableEntry::default(); 1];
            assert!((ers_table.as_slice().as_ptr() as u64) & 0x3F > 0, "event ring segment table entry not aligned");
            assert!((event_ring.buf.as_slice().as_ptr() as u64) & 0x3F > 0, "event ring segment not aligned");
            ers_table[0].data[0] = event_ring.buf.as_slice().as_ptr() as u64;
            ers_table[0].data[1] = (event_ring.buf.len() & 0xFFFF) as u64;

            register.set_primary_interrupter(&ers_table);

            // let mut capability_iter = PciCapabilityIterator::new(cap_addr);
            // let mut pci_cap = capability_iter.find(|c| c.as_ref().capability_id == 5).unwrap();
            // let msi_cap = pci_cap.as_mut().as_msi();
            //
            // msi_cap.set_address(current_cpu_id() as u8, 0, 0);
            // msi_cap.set_data(1, 0x60, 0, 1);
            // msi_cap.set_control(1, 0);

            register.run();

            let port_set = port::PortSet::new(register.max_ports(), register.operational_addr());

            for port in port_set {
                let ccs = port.as_ref().connected();
                if ccs {
                    error!("ccs = {:?}", ccs);
                    error!("port address {:#X}", port.addr().get());
                    port.as_ref().reset();
                }
            }

            loop {
                if let Some(trb) = event_ring.next(&register) {
                    error!("trb - >>>>>>>>>>>>>>>>>>>>>> : {:?}", trb);
                }
            }

            Self {
                base_addr: address,
                register,
                command_ring,
                event_ring,
                device_contexts: dcbaa,
                ers_table,
            }
        }
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
