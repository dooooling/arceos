use alloc::sync::Arc;
use alloc::vec::Vec;
use core::ptr::NonNull;

use log::debug;
use tock_registers::interfaces::{Readable, Writeable};

use crate::device::context::DeviceContext;
use crate::registers::capability::{
    Capability, CAPABILITY_DW1, HCCPARAMS1, HCSPARAMS1, HCSPARAMS2, RTSOFF,
};
use crate::registers::operational::{CONFIG, CRCR, Operational, USBCMD, USBSTS};
use crate::registers::runtime::{IMAN, IMOD, Runtime};
use crate::registers::runtime::ERDP::ERDP;
use crate::registers::runtime::ERSTBA::ERSTBA;
use crate::registers::runtime::ERSTSZ::ERSTSZ;
use crate::ring::{CommandRing, EventRingSegmentTableEntry};

pub mod capability;
pub mod doorbell;
pub mod operational;
pub mod runtime;
pub mod port;

pub struct Registers {
    capability: Arc<NonNull<Capability>>,
    operational: Arc<NonNull<Operational>>,
    runtime: Arc<NonNull<Runtime>>,
}

impl Registers {
    #[inline]
    pub fn operational(&self) -> &Operational {
        unsafe { self.operational.as_ref().as_ref() }
    }
    #[inline]
    pub fn operational_addr(&self) -> usize {
        self.operational.addr().get()
    }
    #[inline]
    pub fn runtime(&self) -> &Runtime {
        unsafe { self.runtime.as_ref().as_ref() }
    }
    #[inline]
    pub fn capability(&self) -> &Capability {
        unsafe { self.capability.as_ref().as_ref() }
    }

    pub fn max_slots(&self) -> u32 {
        self.capability().hcsparams1.read(HCSPARAMS1::MaxSlots)
    }
    pub fn max_ports(&self) -> u32 {
        self.capability().hcsparams1.read(HCSPARAMS1::MaxPorts)
    }
    pub fn max_scratchpad_buffer(&self) -> u32 {
        let capability = self.capability();
        capability.hcsparams2.read(HCSPARAMS2::MSB_L5)
            | capability.hcsparams2.read(HCSPARAMS2::MSB_H5) << 5
    }
    pub fn page_size(&self) -> u32 {
        (self.operational().pagesize.get() & 0xFFFF) << 12
    }
    pub fn context_size(&self) -> u32 {
        self.capability().hccparams1.read(HCCPARAMS1::CSZ)
    }

    pub fn ac64(&self) -> u32 {
        self.capability().hccparams1.read(HCCPARAMS1::AC64)
    }

    pub fn set_max_slot_enable(&self, max_slot: u32) {
        self.operational().config.write(CONFIG::MAXSLOTEN.val(max_slot));
    }

    pub fn set_command_ring(&self, ring: &CommandRing) {
        let operational = self.operational();
        operational.crcr.write(CRCR::RCS.val(ring.cycle_bit as u64));
        operational.crcr.write(CRCR::CS.val(0));
        operational.crcr.write(CRCR::CA.val(0));
        operational.crcr.write(CRCR::CRP.val(ring.buf.as_slice().as_ptr() as u64));
    }

    pub fn set_device_context_base_address_array(&self, dcbaa: &Vec<DeviceContext>) {
        self.operational().dcbaap.set(dcbaa.as_slice().as_ptr() as u64);
    }
    // pub fn set_device_context_base_address_array(&self, erst: &Vec<EventRingSegmentTableEntry>) {
    //     self.operational.dcbaap.set(dcbaap);
    // }

    pub fn set_primary_interrupter(&self, ers_table: &Vec<EventRingSegmentTableEntry>) {
        let runtime = self.runtime();
        let primary_interrupter = &runtime.ints[0];
        primary_interrupter.erstsz.write(ERSTSZ.val(ers_table.len() as u32));
        primary_interrupter.erdp.write(ERDP.val(ers_table[0].data[0]));
        primary_interrupter.erstba.write(ERSTBA.val(ers_table.as_slice().as_ptr() as u64));


        primary_interrupter.imod.write(IMOD::IMODI.val(4000));
        primary_interrupter.iman.write(IMAN::IE::SET);
        primary_interrupter.iman.write(IMAN::IP::SET);
    }

    pub fn set_erdp(&self, erdp: u64) {
        let runtime = self.runtime();
        let primary_interrupter = &runtime.ints[0];
        primary_interrupter.erdp.write(ERDP.val(erdp));
    }


    pub fn reset(&self) {
        let operational = self.operational();
        operational.usbcmd.write(USBCMD::RS::CLEAR);
        while operational.usbsts.read(USBSTS::HCH) == 0 {}
        debug!("xhci controller stopped!");

        operational.usbcmd.write(USBCMD::HCRST::SET);
        while operational.usbsts.read(USBSTS::CNR) != 0 {}
        debug!("xhci controller reseted!");
    }

    pub fn run(&self) {
        let operational = self.operational();
        operational.usbcmd.write(USBCMD::INTE::SET);
        operational.usbcmd.write(USBCMD::RS::SET);
        while operational.usbsts.read(USBSTS::HCH) == 1 {}
        debug!("xhci controller started!");
    }

    pub fn free_legacy_control(&self) {
        // todo check if legacy has control then release control
        unimplemented!()
    }
}

unsafe impl Sync for Registers {}

unsafe impl Send for Registers {}

impl Registers {
    pub fn from(addr: usize) -> Self {
        let base_addr = addr as *mut u8;
        unsafe {
            let capability: NonNull<Capability> = NonNull::new(base_addr).unwrap().cast();
            let cap_len = capability.as_ref().dw1.read(CAPABILITY_DW1::CAPLENGTH) as isize;
            let operational: NonNull<Operational> =
                NonNull::new(base_addr.offset(cap_len)).unwrap().cast();
            let off_set = capability.as_ref().rtsoff.read(RTSOFF::OFFSET) as isize;
            let runtime: NonNull<Runtime> = NonNull::new(base_addr.offset(off_set)).unwrap().cast();
            Self {
                capability: Arc::new(capability),
                operational: Arc::new(operational),
                runtime: Arc::new(runtime),
            }
        }
    }
}
