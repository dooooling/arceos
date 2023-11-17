#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(asm_const)]

#![allow(dead_code)]
extern crate alloc;

#[cfg(feature = "axstd")]
use axstd::println;

use crate::abi::{abi_init, abi_table_ptr};
use crate::app::AppManager;

mod config;
mod app;
mod abi;

const PLASH_START: usize = 0x22000000;

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let load_start = PLASH_START as *const u8;
    // let load_size = 32; // Dangerous!!! We need to get accurate size of apps.

    println!("Load payload ...");
    let app_manager = AppManager::parse(load_start);
    println!("Load payload ok!");

    // app running aspace
    // SBI(0x80000000) -> App <- Kernel(0x80200000)
    // 0xffff_ffc0_0000_0000


    println!("Execute app ...");

    const RUN_START: usize = 0xffff_ffc0_8010_0000;
    abi_init();
    let abi_addr = abi_table_ptr();
    for app in app_manager.apps {
        let run_code = unsafe {
            core::slice::from_raw_parts_mut(RUN_START as *mut u8, app.size)
        };
        run_code.copy_from_slice(app.code);
        println!("run code {:?}; address [{:?}]", run_code, run_code.as_ptr());

        // execute app
        unsafe {
            core::arch::asm!("
        mv      a0, {abi_table}
        li      t2, {run_start}
        jalr    t2",
            run_start = const RUN_START,
            abi_table = in(reg) abi_addr,
            )
        }
    }
}







