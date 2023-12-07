#![no_std]

extern crate alloc;

use tock_registers::interfaces::Readable;

use driver_common::{BaseDriverOps, DeviceType};

use crate::registers::Registers;

mod registers;


pub struct XhciController {
    base_addr: usize,
    register: Registers,
}

impl XhciController {
    pub fn init(address: usize) -> Self {
        unsafe {
            let register = Registers::from(address);
            register.reset().unwrap();
            Self {
                base_addr: address,
                register,
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