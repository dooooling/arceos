#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(asm_const)]

#![allow(dead_code)]
extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

#[cfg(feature = "axstd")]
use axstd::println;

use crate::config::{APP_COUNT_WIDTH, BLOCK_COUNT_WIDTH, BLOCK_SIZE, MARGIN_SIZE_WIDTH};

mod config;

const PLASH_START: usize = 0x22000000;

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let load_start = PLASH_START as *const u8;
    // let load_size = 32; // Dangerous!!! We need to get accurate size of apps.

    println!("Load payload ...");
    let apps = App::parse_apps(load_start);
    println!("Load payload ok!");

    // app running aspace
    // SBI(0x80000000) -> App <- Kernel(0x80200000)
    // 0xffff_ffc0_0000_0000
    const RUN_START: usize = 0xffff_ffc0_8010_0000;
    for app in apps {
        let run_code = unsafe {
            core::slice::from_raw_parts_mut(RUN_START as *mut u8, app.size)
        };
        run_code.copy_from_slice(app.code);
        println!("run code {:?}; address [{:?}]", run_code, run_code.as_ptr());
    }

    register_abi(SYS_HELLO, abi_hello as usize);
    register_abi(SYS_PUTCHAR, abi_putchar as usize);
    register_abi(SYS_EXIT, abi_exit as usize);
    println!("Execute app ...");

    // execute app
    unsafe { core::arch::asm!("
        li      t0, {abi_num}
        slli    t0, t0, 3
        la      t1, {abi_table}
        add     t1, t1, t0
        ld      t1, (t1)
        jalr    t1
        li      t2, {run_start}
        jalr    t2
        j       .",
    run_start = const RUN_START,
    abi_table = sym ABI_TABLE,
    //abi_num = const SYS_HELLO,
    abi_num = const SYS_EXIT,
    in("a0") 0,
    )}
}


struct App<'a> {
    size: usize,
    code: &'a [u8],
}

impl App<'_> {
    /// 解析app
    fn parse_apps(apps_start: *const u8) -> Vec<Self> {
        let mut apps = vec![];

        let app_count = unsafe { core::slice::from_raw_parts(apps_start, APP_COUNT_WIDTH as usize) };
        let app_count = u8::from_le_bytes(app_count.try_into().unwrap());

        let mut offset = APP_COUNT_WIDTH;
        for _ in 0..app_count {
            let block_count = unsafe { core::slice::from_raw_parts(apps_start.offset(offset as isize), BLOCK_COUNT_WIDTH) };
            let block_count = u32::from_le_bytes(block_count.try_into().unwrap());
            offset += BLOCK_COUNT_WIDTH;

            let margin = unsafe {
                core::slice::from_raw_parts(apps_start.offset(offset as isize),
                                            MARGIN_SIZE_WIDTH)
            };
            let margin = u32::from_le_bytes(margin.try_into().unwrap());
            offset += BLOCK_COUNT_WIDTH;

            let size = (block_count * BLOCK_SIZE + margin) as usize;
            let code = unsafe { core::slice::from_raw_parts(apps_start.offset(offset as isize), size) };
            offset += size;

            apps.push(Self {
                size,
                code,
            });
        }
        apps
    }
}

const SYS_HELLO: usize = 1;
const SYS_PUTCHAR: usize = 2;
const SYS_EXIT: usize = 3;

static mut ABI_TABLE: [usize; 16] = [0; 16];

fn register_abi(num: usize, handle: usize) {
    unsafe { ABI_TABLE[num] = handle; }
}

fn abi_hello() {
    println!("[ABI:Hello] Hello, Apps!");
}

fn abi_putchar(c: char) {
    println!("[ABI:Print] {c}");
}
fn abi_exit(code: i32) {
    println!("[ABI:Exit] exit code {code}");
    axstd::process::exit(code)
}



