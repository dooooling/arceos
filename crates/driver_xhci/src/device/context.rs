use alloc::vec::Vec;
use core::alloc::Layout;

#[repr(C, align(64))]
#[derive(Default, Clone, Debug)]
pub struct DeviceContext {
    pub slot: SlotContext,
    pub endpoints: [EndpointContext; 31],
}

impl DeviceContext {
    pub fn aligned_vec(capacity: usize) -> Vec<DeviceContext> {
        unsafe {
            let layout = Layout::array::<DeviceContext>(capacity).unwrap().align_to(64).unwrap();
            let addr = alloc::alloc::alloc(layout).cast();
            Vec::from_raw_parts(addr, capacity, capacity)
        }
    }
}

#[repr(C, align(32))]
#[derive(Default, Clone, Debug)]
pub struct SlotContext {
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    _rsvd: [u32; 4],
}


impl SlotContext {
    pub fn set_route_string(&mut self, val: u32) {
        self.a |= val & 0xFFFFF;
    }
    pub fn set_speed(&mut self, val: u32) {
        self.a |= (val & 0xF) << 20;
    }

    pub fn set_hub(&mut self, hub: bool) {
        let hub = if hub { 1 } else { 0 };
        self.a |= hub << 26;
    }

    pub fn set_context_entries(&mut self, entries: u32) {
        self.a |= (entries & 0x1F) << 27;
    }
    pub fn set_root_hub_number(&mut self, number: u8) {
        self.b |= (number as u32) << 16;
    }
}

#[repr(C, align(32))]
#[derive(Default, Clone, Debug)]
pub struct EndpointContext {
    a: u32,
    b: u32,
    trdpl: u32,
    trdph: u32,
    c: u32,
    _rsvd: [u32; 3],
}

impl EndpointContext {
    pub fn set_endpoint_type(&mut self, ep_type: u8) {
        self.b |= ((ep_type as u32) & 0x7) << 3
    }

    pub fn set_max_packet_size(&mut self, max_packet_size: u16) {
        self.b |= (max_packet_size as u32) << 16
    }
    pub fn set_max_burst_size(&mut self, max_burst_size: u8) {
        self.b |= (max_burst_size as u32) << 8
    }

    pub fn set_transfer_ring_buffer(&mut self, ring_addr: u64) {
        self.trdpl = (ring_addr & 0xFFFFFFF0) as u32;
        self.trdph = (ring_addr >> 32) as u32;
    }
    pub fn set_dequeue_cycle_state(&mut self, state: u8) {
        self.trdpl |= (state as u32) & 1
    }

    pub fn set_interval(&mut self, interval: u8) {
        self.a |= (interval as u32) << 16
    }
    pub fn set_max_primary_streams(&mut self, streams: u8) {
        self.a |= ((streams as u32) & 0x1F) << 10
    }
    pub fn set_mult(&mut self, mult: u8) {
        self.a |= ((mult as u32) & 0x3) << 8
    }
    pub fn set_error_count(&mut self, count: u8) {
        self.b |= ((count as u32) & 0x3) << 1
    }
}

#[repr(C, align(64))]
#[derive(Default, Clone, Debug)]
pub struct InputContext {
    input_control_ctx: InputControlContext,
    slot_ctx: SlotContext,
    ep_ctxs: [EndpointContext; 31],
}

#[repr(C, align(16))]
#[derive(Default, Debug, Clone)]
struct InputControlContext {
    drop_context_flags: u32,
    add_context_flags: u32,
    _reserved1: [u32; 5],
    configuration_value: u8,
    interface_number: u8,
    alternate_setting: u8,
    _reserved2: u8,
}


impl InputContext {
    /// fix
    pub fn set_add_context(&mut self, index: u32, val: u32) {
        self.input_control_ctx.add_context_flags &= val << index;
    }

    pub fn enable_slot_context(&mut self) {
        self.input_control_ctx.add_context_flags |= 1;
    }

    pub fn init_default_control_endpoint(&mut self, max_packet_size: u16, ring_addr: u64) {
        self.input_control_ctx.add_context_flags |= 1 << 1;
        let ep = &mut self.ep_ctxs[0];
        ep.set_endpoint_type(4);
        ep.set_max_packet_size(max_packet_size);
        ep.set_max_burst_size(0);
        ep.set_transfer_ring_buffer(ring_addr);
        ep.set_dequeue_cycle_state(1);
        ep.set_interval(0);
        ep.set_max_primary_streams(0);
        ep.set_mult(0);
        ep.set_error_count(3);
    }
    pub fn mut_slot(&mut self) -> &mut SlotContext {
        &mut self.slot_ctx
    }
}