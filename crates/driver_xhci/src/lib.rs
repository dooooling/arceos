#![feature(strict_provenance)]
#![no_std]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use core::alloc::GlobalAlloc;
use core::num::NonZeroUsize;

use log::{debug, error};
use tock_registers::interfaces::Readable;

use device::context::DeviceContext;
use driver_common::{BaseDriverOps, DeviceType};

use crate::registers::{port, Registers};
use crate::ring::{CommandRing, EventRing, EventRingSegmentTableEntry};

mod device;
mod registers;
mod ring;

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

            let dcbaa = vec![DeviceContext::default(); (max_slots + 1) as usize];
            // register.set_device_context_base_address_array(&dcbaa);

            let mut command_ring = CommandRing::with_capacity(32);
            command_ring.cycle_bit = true;
            register.set_command_ring(&command_ring);

            let mut event_ring = EventRing::with_capacity(32);
            event_ring.cycle_bit = true;

            let mut ers_table = vec![EventRingSegmentTableEntry::default(); 1];
            assert!(
                (ers_table.as_slice().as_ptr() as u64) & 0x3F > 0,
                "event ring segment table entry not aligned"
            );
            assert!(
                (event_ring.buf.as_slice().as_ptr() as u64) & 0x3F > 0,
                "event ring segment not aligned"
            );

            ers_table[0].data[0] = (event_ring.buf.as_slice().as_ptr() as u64) >> 4;
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
                    port.as_ref().reset();
                    error!("ccs = {:?}", ccs);
                    error!("port address {:#X}", port.addr().get());
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

    pub fn new_xhci(address: usize) -> Self {
        unsafe {
            let mut registers = xhci::Registers::new(address, IdentityMapper());
            registers.operational.usbcmd.update_volatile(|u| {
                u.clear_run_stop();
            });
            registers.capability.hccparams1.read_volatile();

            registers.operational.usbcmd.update_volatile(|usb_cmd| {
                usb_cmd.clear_interrupter_enable();
                usb_cmd.clear_host_system_error_enable();
                usb_cmd.clear_enable_wrap_event();
            });
            error!("new_xhci!!!!!!!!!!!!!!!!!!!!!!!!!!!");
            while !registers.operational.usbsts.read_volatile().hc_halted() {}
            error!("test!!!!!!!!!!!!!!!!!!!1");
            registers.operational.usbcmd.update_volatile(|usb_cmd| {
                usb_cmd.set_host_controller_reset();
            });
            while registers
                .operational
                .usbsts
                .read_volatile()
                .controller_not_ready()
            {}
            error!("test!!!!!!!!!!!!!!!!!!!2");

            registers.operational
                .config
                .update_volatile(|config| {
                    config.set_max_device_slots_enabled(8);
                });

            let dcbaa = vec![DeviceContext::default(); (8 + 1) as usize];

            registers
                .operational
                .dcbaap
                .update_volatile(|device_context| device_context.set(dcbaa.as_slice().as_ptr() as u64));

            let mut command_ring = CommandRing::with_capacity(32);
            command_ring.cycle_bit = true;
            registers.operational.crcr.update_volatile(|crcr| {
                crcr.set_ring_cycle_state();
                crcr.set_command_ring_pointer(command_ring.buf.as_slice().as_ptr() as u64);
            });

            let mut event_ring = EventRing::with_capacity(32);
            event_ring.cycle_bit = true;

            let mut ers_table = vec![EventRingSegmentTableEntry::default(); 1];
            ers_table[0].data[0] = (event_ring.buf.as_slice().as_ptr() as u64);
            ers_table[0].data[1] = (event_ring.buf.len() & 0xFFFF) as u64;

            registers.interrupter_register_set
                .interrupter_mut(0)
                .erstsz
                .update_volatile(|e| e.set(1));

            registers.interrupter_register_set
                .interrupter_mut(0)
                .erdp
                .update_volatile(|erdp| erdp.set_event_ring_dequeue_pointer(event_ring.buf.as_slice().as_ptr() as u64));
            registers.interrupter_register_set
                .interrupter_mut(0)
                .erstba
                .update_volatile(|erstba| erstba.set(ers_table.as_slice().as_ptr() as u64));
            registers.interrupter_register_set
                .interrupter_mut(0)
                .iman
                .update_volatile(|iman| {
                    iman.set_0_interrupt_pending();
                });
            registers.interrupter_register_set
                .interrupter_mut(0)
                .iman
                .update_volatile(|iman| {
                    iman.set_interrupt_enable();
                });

            registers.operational.usbcmd.update_volatile(|u| {
                u.set_interrupter_enable();
            });
            registers
                .interrupter_register_set
                .interrupter_mut(0)
                .imod
                .update_volatile(|moderation| {
                    moderation.set_interrupt_moderation_interval(4000);
                });
            registers.operational.usbcmd.update_volatile(|u| {
                u.set_run_stop();
            });

            while registers.operational.usbsts.read_volatile().hc_halted() {}
            error!("test!!!!!!!!!!!!!!!!!!!3");
            let connect_index = registers
                .port_register_set
                .into_iter()
                .position(|p| p.portsc.current_connect_status())
                .unwrap();

            registers
                .port_register_set
                .update_volatile_at(connect_index, |p| {
                    p.portsc.set_port_reset();
                    p.portsc.set_wake_on_connect_enable();
                });
            error!("test!!!!!!!!!!!!!!!!!!!4");
            while registers
                .port_register_set
                .read_volatile_at(connect_index)
                .portsc
                .port_reset()
            {}

            error!("test!!!!!!!!!!!!!!!!!!!5");
            loop {
                if let Some(trb) = event_ring.next_xhci(&mut registers) {
                    error!("trb - >>>>>>>>>>>>>>>>>>>>>> : {:?}", trb);
                }
            }

            Self {
                base_addr: address,
                register: Registers::from(address),
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

#[derive(Clone, Debug)]
struct IdentityMapper();

impl xhci::accessor::Mapper for IdentityMapper {
    unsafe fn map(&mut self, phys_start: usize, _bytes: usize) -> NonZeroUsize {
        return NonZeroUsize::new_unchecked(phys_start);
    }

    fn unmap(&mut self, _virt_start: usize, _bytes: usize) {}
}
