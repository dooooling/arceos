use tock_registers::{register_bitfields, register_structs};
use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::registers::ReadOnly;

register_structs! {
     pub Capability{
        (0x00 => pub dw1: ReadOnly<u32, CAPABILITY_DW1::Register>),
        (0x04 => pub hcsparams1:ReadOnly<u32, HCSPARAMS1::Register>),
        (0x08 => pub hcsparams2:ReadOnly<u32, HCSPARAMS2::Register>),
        (0x0C => pub hcsparams3:ReadOnly<u32, HCSPARAMS3::Register>),
        (0x10 => pub hccparams1:ReadOnly<u32,HCCPARAMS1::Register>),
        (0x14 => pub dboff:ReadOnly<u32, DBOFF::Register>),
        (0x18 => pub rtsoff:ReadOnly<u32, RTSOFF::Register>),
        (0x1C => pub hccparams2:ReadOnly<u32, HCCPARAMS2::Register>),
        (0x20 => @END),
    }
}

register_bitfields![
    u8,
    CAPLENGTH [
        LEN OFFSET(0) NUMBITS(8) [],
    ],
];

register_bitfields![
    u16,
    HCIVERSION [
        VER OFFSET(0) NUMBITS(16) [],
    ],
];

register_bitfields! {
    u32,
    pub CAPABILITY_DW1[
        CAPLENGTH OFFSET(0) NUMBITS(8) [],
        HCIVERSION OFFSET(16) NUMBITS(16) [],
    ],
    pub HCSPARAMS1[
        MaxSlots OFFSET(0) NUMBITS(8) [],
        MaxIntrs OFFSET(8) NUMBITS(11) [],
        MaxPorts OFFSET(24) NUMBITS(8) [],
    ],
    pub HCSPARAMS2[
        IST OFFSET(0) NUMBITS(4) [],
        ERSTMAX OFFSET(4) NUMBITS(4) [],
        MSB_H5 OFFSET(21) NUMBITS(5) [],
        SPR OFFSET(26) NUMBITS(1) [],
        MSB_L5 OFFSET(27) NUMBITS(5) [],
    ],
    pub HCSPARAMS3[
        U1DEL OFFSET(0) NUMBITS(8) [],
        U2DEL OFFSET(16) NUMBITS(16) [],
    ],
    pub HCCPARAMS1[
        AC64 OFFSET(0) NUMBITS(1) [],
        BNC OFFSET(1) NUMBITS(1) [],
        CSZ OFFSET(2) NUMBITS(1) [],
        PPC OFFSET(3) NUMBITS(1) [],
        PIND OFFSET(4) NUMBITS(1) [],
        LHRC OFFSET(5) NUMBITS(1) [],
        LTC OFFSET(6) NUMBITS(1) [],
        NSS OFFSET(7) NUMBITS(1) [],
        PAE OFFSET(8) NUMBITS(1) [],
        SPC OFFSET(9) NUMBITS(1) [],
        SEC OFFSET(10) NUMBITS(1) [],
        CFC OFFSET(11) NUMBITS(1) [],
        MaxPSASize OFFSET(12) NUMBITS(4) [],
        XECP OFFSET(16) NUMBITS(16) [],
    ],
    pub DBOFF[
       OFFSET OFFSET(2) NUMBITS(30) [],
    ],
    pub RTSOFF[
       OFFSET OFFSET(5) NUMBITS(27) [],
    ],
    pub HCCPARAMS2[
        U3C OFFSET(0) NUMBITS(1) [],
        CMC OFFSET(1) NUMBITS(1) [],
        FSC OFFSET(2) NUMBITS(1) [],
        CTC OFFSET(3) NUMBITS(1) [],
        LEC OFFSET(4) NUMBITS(1) [],
        CIC OFFSET(5) NUMBITS(1) [],
    ],
}