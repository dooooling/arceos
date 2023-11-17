#![feature(asm_const)]
#![no_std]
#![no_main]

use core::panic::PanicInfo;

use crate::abi::{exit, hello, init_abi, putchar};

mod abi;

#[no_mangle]
unsafe extern "C" fn _start(abi_addr: usize) {
    init_abi(abi_addr);
    hello();
    puts("put test ok!");
    exit();
}

fn puts(s: &str) {
    for c in s.chars() {
        putchar(c);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}