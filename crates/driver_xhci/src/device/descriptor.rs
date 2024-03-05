use core::fmt::{Debug, Display, Formatter};

#[derive(Debug, Default, Copy, Clone)]
#[repr(C)]
pub(crate) struct DeviceDescriptor {
    b_length: u8,
    b_descriptor_type: u8,
    bcd_usb: u16,
    b_device_class: u8,
    b_device_sub_class: u8,
    b_device_protocol: u8,
    b_max_packet_size0: u8,
    id_vendor: u16,
    id_product: u16,
    bcd_device: u16,
    i_manufacturer: u8,
    i_product: u8,
    i_serial_number: u8,
    b_num_configurations: u8,
}

#[derive(Debug, Default, Copy, Clone)]
#[repr(C)]
pub(crate) struct ConfigurationDescriptor {
    pub b_length: u8,
    b_descriptor_type: u8,
    w_total_length: u16,
    b_num_interfaces: u8,
    b_configuration_value: u8,
    i_configuration: u8,
    bm_attributes: u8,
    b_max_power: u8,
}

#[derive(Debug, Default, Copy, Clone)]
#[repr(C)]
pub(crate) struct InterfaceDescriptor {
    pub b_length: u8,
    b_descriptor_type: u8,
    b_interface_number: u8,
    b_alternate_setting: u8,
    b_num_endpoints: u8,
    b_interface_class: u8,
    b_interface_sub_class: u8,
    b_interface_protocol: u8,
    i_interface: u8,
}

#[derive(Debug, Default, Copy, Clone)]
#[repr(C)]
pub(crate) struct HIDDescriptor {
    pub b_length: u8,
    b_descriptor_type: u8,
    bcd_hid: u16,
    b_country_code: u8,
    b_num_descriptors: u8,
    b_type: u8,
    w_descriptor_length: u16,
}

#[derive(Debug, Default, Copy, Clone)]
#[repr(C)]
pub(crate) struct EndpointDescriptor {
    pub b_length: u8,
    b_descriptor_type: u8,
    b_endpoint_address: u8,
    bm_attributes: u8,
    w_max_packet_size: u16,
    b_interval: u8,
}

#[derive(Default)]
#[repr(C)]
pub(crate) struct ConfigurationDescriptorPack {
    pub(crate) configuration: ConfigurationDescriptor,
    pub(crate) interface0: InterfaceDescriptor,
    pub(crate) hid0: HIDDescriptor,
    pub(crate) endpoint0: EndpointDescriptor,
    pub(crate) interface1: InterfaceDescriptor,
    pub(crate) hid1: HIDDescriptor,
    pub(crate) endpoint1: EndpointDescriptor,
}

impl From<*const u8> for ConfigurationDescriptorPack {
    fn from(mut addr: *const u8) -> Self {
        unsafe {
            let configuration_descriptor = (addr as *const ConfigurationDescriptor).read();
            addr = addr.add(configuration_descriptor.b_length as usize);
            let interface_descriptor0 = (addr as *const InterfaceDescriptor).read();
            addr = addr.add(interface_descriptor0.b_length as usize);
            let hid_descriptor0 = (addr as *const HIDDescriptor).read();
            addr = addr.add(hid_descriptor0.b_length as usize);
            let endpoint_descriptor0 = (addr as *const EndpointDescriptor).read();
            addr = addr.add(endpoint_descriptor0.b_length as usize);
            let interface_descriptor1 = (addr as *const InterfaceDescriptor).read();
            addr = addr.add(interface_descriptor1.b_length as usize);
            let hid_descriptor1 = (addr as *const HIDDescriptor).read();
            addr = addr.add(hid_descriptor1.b_length as usize);
            let endpoint_descriptor1 = (addr as *const EndpointDescriptor).read();
            Self {
                configuration: configuration_descriptor,
                interface0: interface_descriptor0,
                hid0: hid_descriptor0,
                endpoint0: endpoint_descriptor0,
                interface1: interface_descriptor1,
                hid1: hid_descriptor1,
                endpoint1: endpoint_descriptor1,
            }
        }
    }
}

impl Debug for ConfigurationDescriptorPack {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n",
               self.configuration,
               self.interface0,
               self.hid0,
               self.endpoint0,
               self.interface1,
               self.hid1,
               self.endpoint1)
    }
}