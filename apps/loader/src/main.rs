#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(asm_const)]
#![allow(dead_code)]
extern crate alloc;

#[cfg(feature = "axstd")]
use axstd::println;

use crate::abi::{abi_init, abi_table_ptr};
use crate::app::AppManager;

mod abi;
mod app;
mod config;

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

    abi_init();
    let abi_addr = abi_table_ptr();

    // switch aspace from kernel to app
    unsafe {
        init_app_page_table();
    }
    unsafe {
        switch_app_aspace();
    }
    const RUN_START: usize = 0x4010_0000;

    for app in app_manager.apps {
        let run_code = unsafe { core::slice::from_raw_parts_mut(RUN_START as *mut u8, app.size) };
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

#[link_section = ".data.app_page_table"]
static mut APP_PT_SV39: [u64; 512] = [0; 512];

unsafe fn init_app_page_table() {
    // 0x8000_0000..0xc000_0000, VRWX_GAD, 1G block
    APP_PT_SV39[2] = (0x80000 << 10) | 0xef;
    // 0xffff_ffc0_8000_0000..0xffff_ffc0_c000_0000, VRWX_GAD, 1G block
    APP_PT_SV39[0x102] = (0x80000 << 10) | 0xef;

    // 0x0000_0000..0x4000_0000, VRWX_GAD, 1G block
    APP_PT_SV39[0] = (0x00000 << 10) | 0xef;

    // For App aspace!
    // 0x4000_0000..0x8000_0000, VRWX_GAD, 1G block
    APP_PT_SV39[1] = (0x80000 << 10) | 0xef;
}

unsafe fn switch_app_aspace() {
    use riscv::register::satp;
    let page_table_root = APP_PT_SV39.as_ptr() as usize - axconfig::PHYS_VIRT_OFFSET;
    satp::set(satp::Mode::Sv39, 0, page_table_root >> 12);
    riscv::asm::sfence_vma_all();
}
