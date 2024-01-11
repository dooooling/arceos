use tock_registers::{register_bitfields, register_structs};
use tock_registers::registers::ReadWrite;

register_structs! {
    pub Runtime{
        (0x00 => pub mfindex:ReadWrite<u32,MFINDEX::Register>),
        (0x04 => _rsvd),
        (0x20 => pub ints: [Interrupter;1024]),
        (0x8020 => @END),
    }
}

register_structs! {
    pub Interrupter{
        (0x00 => pub iman: ReadWrite<u32, IMAN::Register>),
        (0x04 => pub imod: ReadWrite<u32, IMOD::Register>),
        (0x08 => pub erstsz: ReadWrite<u32,ERSTSZ::Register>),
        (0x0C => _rsvd),
        (0x10 => pub erstba: ReadWrite<u64,ERSTBA::Register>),
        (0x18 => pub erdp: ReadWrite<u64,ERDP::Register>),
        (0x20 => @END),
    }
}

register_bitfields! {
    u32,
    pub MFINDEX [
        MFINDEX OFFSET(0) NUMBITS(14) [],
    ],
    pub IMAN [
        IP OFFSET(0) NUMBITS(1) [],
        IE OFFSET(1) NUMBITS(1) [],
    ],
    pub IMOD [
        IMODI OFFSET(0) NUMBITS(16) [],
        IMODC OFFSET(16) NUMBITS(16) [],
    ],
    pub ERSTSZ [
        ERSTSZ OFFSET(0) NUMBITS(16) [],
    ],
}

register_bitfields! {
    u64,
    pub ERSTBA [
        ERSTBA OFFSET(6) NUMBITS(58) [],
    ],
    pub ERDP [
        DESI OFFSET(0) NUMBITS(3) [],
        EHB OFFSET(3) NUMBITS(1) [],
        ERDP OFFSET(4) NUMBITS(60) [],
    ],
}
