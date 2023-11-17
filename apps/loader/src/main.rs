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
    // let load_code = unsafe { core::slice::from_raw_parts(load_start, load_size) };
    // println!("load code {:?}; address [{:?}]", load_code, load_code.as_ptr());

    // app running aspace
    // SBI(0x80000000) -> App <- Kernel(0x80200000)
    // 0xffff_ffc0_0000_0000
    const RUN_START: usize = 0xffff_ffc0_8010_0000;
    for app in apps {
        let run_code = unsafe {
            core::slice::from_raw_parts_mut(RUN_START as *mut u8, app.size)
        };
        // run_code.fill(0);
        run_code.copy_from_slice(app.code);
        println!("run code {:?}; address [{:?}]", run_code, run_code.as_ptr());
        println!("Execute app ...");

        // execute app
        unsafe {
            core::arch::asm!("
                li      t2, {run_start}
                jalr    t2",
            run_start = const RUN_START,
            )
        }
    }
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

