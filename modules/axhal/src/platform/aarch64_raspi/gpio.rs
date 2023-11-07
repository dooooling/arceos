use core::ptr::NonNull;

use aarch64_cpu::asm;
use tock_registers::{register_bitfields, register_structs, registers::ReadWrite};
use tock_registers::interfaces::{ReadWriteable, Writeable};

use memory_addr::PhysAddr;
use spinlock::SpinNoIrq;

use crate::mem::phys_to_virt;

const GPIO_BASE: PhysAddr = PhysAddr::from(axconfig::GPIO_PADDR);

static GPIO: SpinNoIrq<GPIO> =
    SpinNoIrq::new(GPIO::new(phys_to_virt(GPIO_BASE).as_mut_ptr()));

unsafe impl Send for GPIO {}
unsafe impl Sync for GPIO {}

register_bitfields! {
    u32,

    /// GPIO Function Select 1
    GPFSEL1 [
        /// Pin 15
        FSEL15 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AltFunc0 = 0b100  // PL011 UART RX

        ],

        /// Pin 14
        FSEL14 OFFSET(12) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AltFunc0 = 0b100  // PL011 UART TX
        ]
    ],

    /// GPIO Pull-up/down Register
    ///
    /// BCM2837 only.
    GPPUD [
        /// Controls the actuation of the internal pull-up/down control line to ALL the GPIO pins.
        PUD OFFSET(0) NUMBITS(2) [
            Off = 0b00,
            PullDown = 0b01,
            PullUp = 0b10
        ]
    ],

    /// GPIO Pull-up/down Clock Register 0
    ///
    /// BCM2837 only.
    GPPUDCLK0 [
        /// Pin 15
        PUDCLK15 OFFSET(15) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 14
        PUDCLK14 OFFSET(14) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ]
    ],

    /// GPIO Pull-up / Pull-down Register 0
    ///
    /// BCM2711 only.
    GPIO_PUP_PDN_CNTRL_REG0 [
        /// Pin 15
        GPIO_PUP_PDN_CNTRL15 OFFSET(30) NUMBITS(2) [
            NoResistor = 0b00,
            PullUp = 0b01
        ],

        /// Pin 14
        GPIO_PUP_PDN_CNTRL14 OFFSET(28) NUMBITS(2) [
            NoResistor = 0b00,
            PullUp = 0b01
        ]
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    RegisterBlock {
        (0x00 => _reserved1),
        (0x04 => GPFSEL1: ReadWrite<u32, GPFSEL1::Register>),
        (0x08 => _reserved2),
        (0x94 => GPPUD: ReadWrite<u32, GPPUD::Register>),
        (0x98 => GPPUDCLK0: ReadWrite<u32, GPPUDCLK0::Register>),
        (0x9C => _reserved3),
        (0xE4 => GPIO_PUP_PDN_CNTRL_REG0: ReadWrite<u32, GPIO_PUP_PDN_CNTRL_REG0::Register>),
        (0xE8 => @END),
    }
}

struct GPIO {
    base: NonNull<RegisterBlock>,
}

impl GPIO {
    pub const fn new(base: *mut u8) -> Self {
        Self {
            base: NonNull::new(base).unwrap().cast(),
        }
    }
    const fn regs(&self) -> &RegisterBlock {
        unsafe { self.base.as_ref() }
    }

    // #[cfg(feature = "bsp_rpi3")]
    fn disable_pud_14_15_bcm2837(&mut self) {
        self.regs()
            .GPFSEL1
            .modify(GPFSEL1::FSEL15::AltFunc0 + GPFSEL1::FSEL14::AltFunc0);

        // The Linux 2837 GPIO driver waits 1 µs between the steps.

        self.regs().GPPUD.write(GPPUD::PUD::Off);
        spin_for_cycles(2000);

        self.regs()
            .GPPUDCLK0
            .write(GPPUDCLK0::PUDCLK15::AssertClock + GPPUDCLK0::PUDCLK14::AssertClock);
        spin_for_cycles(2000);

        self.regs().GPPUD.write(GPPUD::PUD::Off);
        self.regs().GPPUDCLK0.set(0);
    }
}

fn spin_for_cycles(delay: usize) {
    for _ in 0..delay {
        asm::nop();
    }
}


/// Initialize the UART
pub fn init_early() {
    GPIO.lock().disable_pud_14_15_bcm2837();
}