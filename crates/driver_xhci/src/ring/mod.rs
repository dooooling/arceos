use alloc::vec::Vec;
use core::alloc::Layout;
use core::fmt::{Formatter, UpperHex};

use crate::ring::CompletionCode::Invalid;

pub mod event;
pub mod command;
pub mod transfer;


pub const TRB_CONTROL_TRB_TYPE_SHIFT: u8 = 10;
pub const TRB_CONTROL_TRB_TYPE_MASK: u32 = 0x0000_FC00;


#[derive(Default)]
pub struct Ring {
    pub buf: Vec<GenericTrb>,
    pub cycle_bit: bool,
    pub write_idx: usize,
}

impl Ring {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: GenericTrb::aligned_vec(64, capacity),
            cycle_bit: true,
            write_idx: 0,
        }
    }
}


#[repr(C)]
#[derive(Clone, Default, Debug)]
pub struct GenericTrb {
    data_low: u32,
    data_high: u32,
    status: u32,
    control: u32,
}

impl GenericTrb {
    pub fn aligned_vec(align: usize, capacity: usize) -> Vec<GenericTrb> {
        unsafe {
            let layout = Layout::array::<GenericTrb>(capacity).unwrap().align_to(align).unwrap();
            let addr = alloc::alloc::alloc(layout).cast();
            Vec::from_raw_parts(addr, capacity, capacity)
        }
    }

    pub fn trb_type(&self) -> TrbType {
        (((self.control & TRB_CONTROL_TRB_TYPE_MASK) >> TRB_CONTROL_TRB_TYPE_SHIFT) as u8).into()
    }
    pub fn set_trb_type(&mut self, trb_type: TrbType) {
        self.control |= ((trb_type as u32) << TRB_CONTROL_TRB_TYPE_SHIFT);
    }

    pub fn set_slot_id(&mut self, slot_id: u8) {
        self.control |= (slot_id as u32) << 24;
    }

    pub fn set_pointer(&mut self, pointer: u64) {
        self.data_low |= (pointer as u32) & 0xFFFFFFF0;
        self.data_high |= (pointer >> 32) as u32;
    }

    /// cycle bit
    pub fn pcs(&self) -> bool {
        self.control & 0b1 == 1
    }
    /// set cycle bit
    pub fn set_pcs(&mut self, cycle: bool) {
        if cycle {
            self.control |= 0b1;
        } else {
            self.control |= 0b0;
        }
    }
}

pub struct LinkTrb(GenericTrb);

impl LinkTrb {
    pub fn new(addr: usize) -> Self {
        let mut trb = GenericTrb::default();
        trb.data_low = (addr as u32) & 0xFFFFFFF0;
        trb.data_low = (addr >> 32) as u32;
        Self(trb)
    }
    pub fn cast_trb(self) -> GenericTrb {
        self.0
    }

    /// Toggle Cycle (TC).
    pub fn set_tc(&mut self, toggle: bool) {
        if toggle {
            self.0.control |= 0b10;
        } else {
            self.0.control |= 0b00;
        }
    }
}

impl UpperHex for GenericTrb {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let data = self.data_low as u128
            | (self.data_high as u128) << 32
            | (self.status as u128) << 64
            | (self.control as u128) << 96;
        core::fmt::UpperHex::fmt(&data, f)
    }
}

#[derive(Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum TrbType {
    Reserved = 0,
    /* Transfer */
    Normal = 1,
    SetupStage = 2,
    DataStage = 3,
    StatusStage = 4,
    Isoch = 5,
    Link = 6,
    EventData = 7,
    NoOp = 8,
    /* Command */
    EnableSlot = 9,
    DisableSlot = 10,
    AddressDevice = 11,
    ConfigureEndpoint = 12,
    EvaluateContext = 13,
    ResetEndpoint = 14,
    StopEndpoint = 15,
    SetTrDequeuePointer = 16,
    ResetDevice = 17,
    ForceEvent = 18,
    NegotiateBandwidth = 19,
    SetLatencyToleranceValue = 20,
    GetPortBandwidth = 21,
    ForceHeader = 22,
    NoOpCmd = 23,
    /* Reserved */
    GetExtendedProperty = 24,
    SetExtendedProperty = 25,
    Rsv26 = 26,
    Rsv27 = 27,
    Rsv28 = 28,
    Rsv29 = 29,
    Rsv30 = 30,
    Rsv31 = 31,
    /* Events */
    Transfer = 32,
    CommandCompletion = 33,
    PortStatusChange = 34,
    BandwidthRequest = 35,
    Doorbell = 36,
    HostController = 37,
    DeviceNotification = 38,
    MfindexWrap = 39,
    /* Reserved from 40 to 47, vendor devined from 48 to 63 */
}

impl From<u8> for TrbType {
    fn from(value: u8) -> Self {
        match value {
            0 => TrbType::Reserved,
            1 => TrbType::Normal,
            2 => TrbType::SetupStage,
            3 => TrbType::DataStage,
            4 => TrbType::StatusStage,
            5 => TrbType::Isoch,
            6 => TrbType::Link,
            7 => TrbType::EventData,
            8 => TrbType::NoOp,
            9 => TrbType::EnableSlot,
            10 => TrbType::DisableSlot,
            11 => TrbType::AddressDevice,
            12 => TrbType::ConfigureEndpoint,
            13 => TrbType::EvaluateContext,
            14 => TrbType::ResetEndpoint,
            15 => TrbType::StopEndpoint,
            16 => TrbType::SetTrDequeuePointer,
            17 => TrbType::ResetDevice,
            18 => TrbType::ForceEvent,
            19 => TrbType::NegotiateBandwidth,
            20 => TrbType::SetLatencyToleranceValue,
            21 => TrbType::GetPortBandwidth,
            22 => TrbType::ForceHeader,
            23 => TrbType::NoOpCmd,
            24 => TrbType::GetExtendedProperty,
            25 => TrbType::SetExtendedProperty,
            26 => TrbType::Rsv26,
            27 => TrbType::Rsv27,
            28 => TrbType::Rsv28,
            29 => TrbType::Rsv29,
            30 => TrbType::Rsv30,
            31 => TrbType::Rsv31,
            32 => TrbType::Transfer,
            33 => TrbType::CommandCompletion,
            34 => TrbType::PortStatusChange,
            35 => TrbType::BandwidthRequest,
            36 => TrbType::Doorbell,
            37 => TrbType::HostController,
            38 => TrbType::DeviceNotification,
            39 => TrbType::MfindexWrap,
            _ => TrbType::Reserved
        }
    }
}

pub(crate) enum CompletionCode {
    Invalid = 0,
    Success = 1,
    DataBufferError = 2,
    BabbleDetectedError = 3,
    USBTransactionError = 4,
    TRBError = 5,
    StallError = 6,
    ResourceError = 7,
    BandwidthError = 8,
    NoSlotsAvailableError = 9,
    InvalidStreamTypeError = 10,
    SlotNotEnabledError = 11,
    EndpointNotEnabledError = 12,
    ShortPacket = 13,
    RingUnderrun = 14,
    RingOverrun = 15,
    VFEventRingFullError = 16,
    ParameterError = 17,
    BandwidthOverrunError = 18,
    ContextStateError = 19,
    NoPingResponseError = 20,
    EventRingFullError = 21,
    IncompatibleDeviceError = 22,
    MissedServiceError = 23,
    CommandRingStopped = 24,
    CommandAborted = 25,
    Stopped = 26,
    StoppedLengthInvalid = 27,
    StoppedShortPacket = 28,
    MaxExitLatencyTooLargeError = 29,
    Reserved = 30,
    IsochBufferOverrun = 31,
    EventLostError = 32,
    UndefinedError = 33,
    InvalidStreamIDError = 34,
    SecondaryBandwidthError = 35,
    SplitTransactionError = 36,
    Reserved1 = 37,
}

impl From<u8> for CompletionCode {
    fn from(value: u8) -> Self {
        use crate::ring::CompletionCode::*;
        match value {
            0 => Invalid,
            1 => Success,
            2 => DataBufferError,
            3 => BabbleDetectedError,
            4 => USBTransactionError,
            5 => TRBError,
            6 => StallError,
            7 => ResourceError,
            8 => BandwidthError,
            9 => NoSlotsAvailableError,
            10 => InvalidStreamTypeError,
            11 => SlotNotEnabledError,
            12 => EndpointNotEnabledError,
            13 => ShortPacket,
            14 => RingUnderrun,
            15 => RingOverrun,
            16 => VFEventRingFullError,
            17 => ParameterError,
            18 => BandwidthOverrunError,
            19 => ContextStateError,
            20 => NoPingResponseError,
            21 => EventRingFullError,
            22 => IncompatibleDeviceError,
            23 => MissedServiceError,
            24 => CommandRingStopped,
            25 => CommandAborted,
            26 => Stopped,
            27 => StoppedLengthInvalid,
            28 => StoppedShortPacket,
            29 => MaxExitLatencyTooLargeError,
            30 => Reserved,
            31 => IsochBufferOverrun,
            32 => EventLostError,
            33 => UndefinedError,
            34 => InvalidStreamIDError,
            35 => SecondaryBandwidthError,
            36 => SplitTransactionError,
            _ => Reserved1
        }
    }
}



