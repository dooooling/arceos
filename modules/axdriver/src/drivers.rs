//! Defines types and probe methods of all supported devices.

#![allow(unused_imports)]

use axalloc::{global_allocator, GlobalAllocator};
use driver_common::DeviceType;
use driver_pci::capability::PciCapabilityIterator;
use driver_pci::{Command, Status};
#[cfg(feature = "bus-pci")]
use driver_pci::{DeviceFunction, DeviceFunctionInfo, PciRoot};
use driver_virtio::pci;
use driver_xhci::XhciController;

#[cfg(feature = "virtio")]
use crate::virtio::{self, VirtIoDevMeta};
use crate::AxDeviceEnum;

pub use super::dummy::*;

pub trait DriverProbe {
    fn probe_global() -> Option<AxDeviceEnum> {
        None
    }

    #[cfg(bus = "mmio")]
    fn probe_mmio(_mmio_base: usize, _mmio_size: usize) -> Option<AxDeviceEnum> {
        None
    }

    #[cfg(bus = "pci")]
    fn probe_pci(
        _root: &mut PciRoot,
        _bdf: DeviceFunction,
        _dev_info: &DeviceFunctionInfo,
    ) -> Option<AxDeviceEnum> {
        None
    }
}

#[cfg(net_dev = "virtio-net")]
register_net_driver!(
    <virtio::VirtIoNet as VirtIoDevMeta>::Driver,
    <virtio::VirtIoNet as VirtIoDevMeta>::Device
);

#[cfg(block_dev = "virtio-blk")]
register_block_driver!(
    <virtio::VirtIoBlk as VirtIoDevMeta>::Driver,
    <virtio::VirtIoBlk as VirtIoDevMeta>::Device
);

#[cfg(display_dev = "virtio-gpu")]
register_display_driver!(
    <virtio::VirtIoGpu as VirtIoDevMeta>::Driver,
    <virtio::VirtIoGpu as VirtIoDevMeta>::Device
);

cfg_if::cfg_if! {
    if #[cfg(block_dev = "ramdisk")] {
        pub struct RamDiskDriver;
        register_block_driver!(RamDiskDriver, driver_block::ramdisk::RamDisk);

        impl DriverProbe for RamDiskDriver {
            fn probe_global() -> Option<AxDeviceEnum> {
                // TODO: format RAM disk
                Some(AxDeviceEnum::from_block(
                    driver_block::ramdisk::RamDisk::new(0x100_0000), // 16 MiB
                ))
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(block_dev = "bcm2835-sdhci")]{
        pub struct BcmSdhciDriver;
        register_block_driver!(MmckDriver, driver_block::bcm2835sdhci::SDHCIDriver);

        impl DriverProbe for BcmSdhciDriver {
            fn probe_global() -> Option<AxDeviceEnum> {
                debug!("mmc probe");
                driver_block::bcm2835sdhci::SDHCIDriver::try_new().ok().map(AxDeviceEnum::from_block)
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(net_dev = "ixgbe")] {
        use crate::ixgbe::IxgbeHalImpl;
        use axhal::mem::phys_to_virt;
        pub struct IxgbeDriver;
        register_net_driver!(IxgbeDriver, driver_net::ixgbe::IxgbeNic<IxgbeHalImpl, 1024, 1>);
        impl DriverProbe for IxgbeDriver {
            fn probe_pci(
                    root: &mut driver_pci::PciRoot,
                    bdf: driver_pci::DeviceFunction,
                    dev_info: &driver_pci::DeviceFunctionInfo,
                ) -> Option<crate::AxDeviceEnum> {
                    use crate::ixgbe::IxgbeHalImpl;
                    use driver_net::ixgbe::{INTEL_82599, INTEL_VEND, IxgbeNic};
                    if dev_info.vendor_id == INTEL_VEND && dev_info.device_id == INTEL_82599 {
                        // Intel 10Gb Network
                        info!("ixgbe PCI device found at {:?}", bdf);

                        // Initialize the device
                        // These can be changed according to the requirments specified in the ixgbe init function.
                        const QN: u16 = 1;
                        const QS: usize = 1024;
                        let bar_info = root.bar_info(bdf, 0).unwrap();
                        match bar_info {
                            driver_pci::BarInfo::Memory {
                                address,
                                size,
                                ..
                            } => {
                                let ixgbe_nic = IxgbeNic::<IxgbeHalImpl, QS, QN>::init(
                                    phys_to_virt((address as usize).into()).into(),
                                    size as usize
                                )
                                .expect("failed to initialize ixgbe device");
                                return Some(AxDeviceEnum::from_net(ixgbe_nic));
                            }
                            driver_pci::BarInfo::IO { .. } => {
                                error!("ixgbe: BAR0 is of I/O type");
                                return None;
                            }
                        }
                    }
                    None
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(xhci_dev = "xhci")]{
        use axhal::mem::phys_to_virt;
        pub struct XhciDriver;
        register_xhci_driver!(XhciDriver, driver_xhci::XhciController);
        impl DriverProbe for XhciDriver {
            fn probe_pci(
                root: &mut PciRoot,
                bdf: DeviceFunction,
                dev_info: &DeviceFunctionInfo,
            ) -> Option<AxDeviceEnum> {
                return match ( dev_info.class,dev_info.subclass,dev_info.prog_if){
                    (0xC, 0x3, 0x30)=>{
                        let bar_info = root.bar_info(bdf, 0).unwrap();
                        let (_status, _command) = root.get_status_command(bdf);

                        let bdf_addr = bdf_addr(&bdf) as usize;
                        debug!("bdf_addr : {:#X}", bdf_addr);
                        if let driver_pci::BarInfo::Memory{address,size, address_type,..} = bar_info{
                            info!("found a USB compatible device entry. (xHCI)");
                            info!("bus = {}, device = {}, function = {} io base: {:#X}, type : {:?}", bdf.bus, bdf.device, bdf.function, address, address_type);
                            Some(AxDeviceEnum::Xhci(XhciController::new(phys_to_virt((address as usize).into()).into(), bdf_addr)))
                        }else{
                            None
                        }
                    },
                    _=>{
                         None
                    },
                }
            }
        }
    }
}

fn bdf_addr(bdf: &DeviceFunction) -> *mut u32 {
    let addr = phys_to_virt(axconfig::PCI_ECAM_BASE.into());
    let mmio_base = addr.as_ptr() as *mut u32;
    unsafe { (mmio_base.add((ecma_addr(bdf, 0) >> 2) as usize)) }
}

fn read_word(bdf: &DeviceFunction, offset: u32) -> u32 {
    let addr = phys_to_virt(axconfig::PCI_ECAM_BASE.into());
    let mmio_base = addr.as_ptr() as *mut u32;
    unsafe { (mmio_base.add((ecma_addr(bdf, offset) >> 2) as usize)).read_volatile() }
}

fn ecma_addr(bdf: &DeviceFunction, offset: u32) -> u32 {
    let bdf = (bdf.bus as u32) << 8 | (bdf.device as u32) << 3 | bdf.function as u32;
    let address = bdf << 12 | offset;
    address
}
