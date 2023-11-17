#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;

use crate::config::{BLOCK_COUNT_WIDTH, BLOCK_SIZE, MARGIN_SIZE_WIDTH};

mod config;

const PLASH_START: usize = 0x22000000;

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let apps_start = PLASH_START as *const u8;
    println!("Load payload ...");
    let app = App::parse_app(apps_start);
    println!("content: {:?}", app.code);

    println!("Load payload ok!");
}


struct App<'a> {
    size: u32,
    code: &'a [u8],
}

impl App<'_> {
    /// 解析app
    fn parse_app(apps_start: *const u8) -> Self {
        let block_count = unsafe { core::slice::from_raw_parts(apps_start, BLOCK_COUNT_WIDTH as usize) };
        let block_count = u32::from_le_bytes(block_count.try_into().unwrap());

        let margin = unsafe {
            core::slice::from_raw_parts(apps_start.offset(BLOCK_COUNT_WIDTH as isize),
                                        MARGIN_SIZE_WIDTH as usize)
        };
        let margin = u32::from_le_bytes(margin.try_into().unwrap());

        let size = block_count * BLOCK_SIZE + margin;
        let code = unsafe { core::slice::from_raw_parts(apps_start.offset((BLOCK_COUNT_WIDTH + MARGIN_SIZE_WIDTH) as isize), size as usize) };
        Self {
            size,
            code,
        }
    }
}

