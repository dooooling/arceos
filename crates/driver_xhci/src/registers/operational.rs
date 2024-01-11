use tock_registers::{register_bitfields, register_structs};
use tock_registers::registers::{ReadOnly, ReadWrite};

register_structs! {
    pub Operational{
        (0x00 => pub usbcmd:ReadWrite<u32, USBCMD::Register>),
        (0x04 => pub usbsts:ReadWrite<u32, USBSTS::Register>),
        (0x08 => pub pagesize:ReadOnly<u32>),
        (0x0C => _rsvd0),
        (0x14 => pub dnctrl:ReadWrite<u32>),
        (0x18 => pub crcr:ReadWrite<u64, CRCR::Register>),
        (0x20 => _rsvd1),
        (0x30 => pub dcbaap:ReadWrite<u64, DCBAAP::Register>),
        (0x38 => pub config:ReadWrite<u32, CONFIG::Register>),
        (0x3C => @END),
    }
}


register_bitfields! {
    u32,
    pub USBCMD [
        RS OFFSET(0) NUMBITS(1) [],
        HCRST OFFSET(1) NUMBITS(1) [],
        INTE OFFSET(2) NUMBITS(1) [],
        HSEE OFFSET(3) NUMBITS(1) [],
        LHCRST OFFSET(7) NUMBITS(1) [],
        CSS OFFSET(8) NUMBITS(1) [],
        CRS OFFSET(9) NUMBITS(1) [],
        EWE OFFSET(10) NUMBITS(1) [],
        EU3S OFFSET(11) NUMBITS(1) [],
        CME OFFSET(13) NUMBITS(1) [],
        ETE OFFSET(14) NUMBITS(1) [],
        TSC_EN OFFSET(15) NUMBITS(1) [],
        VTIOE OFFSET(16) NUMBITS(1) [],
    ],
    pub USBSTS [
        HCH OFFSET(0) NUMBITS(1) [],
        HSE OFFSET(2) NUMBITS(1) [],
        EINT OFFSET(3) NUMBITS(1) [],
        PCD OFFSET(4) NUMBITS(1) [],
        SSS OFFSET(8) NUMBITS(1) [],
        RSS OFFSET(9) NUMBITS(1) [],
        SRE OFFSET(10) NUMBITS(1) [],
        CNR OFFSET(11) NUMBITS(1) [],
        HCE OFFSET(12) NUMBITS(1) [],
    ],
    pub CONFIG [
        MAXSLOTEN OFFSET(0) NUMBITS(8) [],
        U3E OFFSET(8) NUMBITS(1) [],
        CIE OFFSET(9) NUMBITS(1) [],
    ],
}
register_bitfields! {
    u64,
    pub DCBAAP [
        PONITER OFFSET(6) NUMBITS(58) [],
    ],
    pub CRCR[
        RCS OFFSET(0) NUMBITS(1) [],
        CS OFFSET(1) NUMBITS(1) [],
        CA OFFSET(2) NUMBITS(1) [],
        CRR OFFSET(3) NUMBITS(1) [],
        CRP OFFSET(6) NUMBITS(58) [],
    ]
}