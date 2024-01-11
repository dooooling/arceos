#[repr(C, align(64))]
#[derive(Default, Clone)]
pub struct DeviceContext {
    pub slot: SlotContext,
    pub endpoints: [EndpointContext; 31],
}

#[repr(C, align(32))]
#[derive(Default, Clone)]
struct SlotContext {
    data: [u32; 8],
}

#[repr(C, align(32))]
#[derive(Default, Clone)]
struct EndpointContext {
    data: [u32; 8],
}
