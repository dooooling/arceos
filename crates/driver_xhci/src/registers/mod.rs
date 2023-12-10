use alloc::sync::Arc;
use alloc::vec;
use core::ptr::NonNull;

use log::debug;
use tock_registers::interfaces::{Readable, Writeable};

use crate::context::DeviceContext;
use driver_common::DevResult;

use crate::registers::capability::{
    Capability, CAPABILITY_DW1, HCCPARAMS1, HCSPARAMS1, HCSPARAMS2, RTSOFF,
};
use crate::registers::operational::{Operational, CONFIG, USBCMD, USBSTS};
use crate::registers::runtime::Runtime;

pub mod capability;
pub mod doorbell;
pub mod operational;
pub mod runtime;

pub struct Registers {
    capability: Arc<NonNull<Capability>>,
    operational: Arc<NonNull<Operational>>,
    runtime: Arc<NonNull<Runtime>>,
}

impl Registers {
    pub fn operational(&self) -> &Operational {
        unsafe { self.operational.as_ref().as_ref() }
    }
    pub fn runtime(&self) -> &Runtime {
        unsafe { self.runtime.as_ref().as_ref() }
    }
    pub fn capability(&self) -> &Capability {
        unsafe { self.capability.as_ref().as_ref() }
    }

    pub fn reset(&self) -> DevResult {
        let operational = self.operational();
        operational.usbcmd.write(USBCMD::RS::CLEAR);
        while operational.usbsts.read(USBSTS::HCH) == 0 {}
        debug!("xhci controller stopped!");

        operational.usbcmd.write(USBCMD::HCRST::SET);
        while operational.usbsts.read(USBSTS::CNR) != 0 {}
        debug!("xhci controller rested!");

        let capability = self.capability();
        let max_slots = capability.hcsparams1.read(HCSPARAMS1::MaxSlots);
        debug!("xhci max slots : {}", max_slots);

        operational.config.write(CONFIG::MAXSLOTEN.val(max_slots));

        let mut max_scratchpad_buffer = capability.hcsparams2.read(HCSPARAMS2::MSB_L5);
        max_scratchpad_buffer |= capability.hcsparams2.read(HCSPARAMS2::MSB_H5) << 5;
        debug!("xhci max scratch pad : {}", max_scratchpad_buffer);

        if max_scratchpad_buffer > 0 {
            unimplemented!("not implemented scratchpad")
        }
        let ac64 = capability.hccparams1.read(HCCPARAMS1::AC64);
        debug!("xhci 64-bit Addressing Capability : {}", ac64);
        assert_eq!(ac64, 1, "not implemented 32-bit addressing");

        let page_size = (operational.pagesize.get() & 0xFFFF) << 12;
        debug!("xhci page size : {}", page_size);

        let context_size = capability.hccparams1.read(HCCPARAMS1::CSZ);
        assert_eq!(context_size, 0, "not support 64-bit context size");

        let dcbaa = vec![DeviceContext::default(); (max_slots + 1) as usize];
        let dcbaap = (dcbaa.as_ptr() as *mut DeviceContext) as u64;

        operational.dcbaap.set(dcbaap);

        operational.usbcmd.write(USBCMD::RS::SET);
        while operational.usbsts.read(USBSTS::HCH) == 0 {}
        debug!("xhci controller started!");
        Ok(())
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
