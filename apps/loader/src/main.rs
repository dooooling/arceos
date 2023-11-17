#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
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
    let apps_start = PLASH_START as *const u8;
    println!("Load payload ...");
    let apps = App::parse_apps(apps_start);
    for (idx, app) in apps.iter().enumerate() {
        println!("app {} ,size {} |  content: {:?}", idx, app.size, app.code);
    }
    println!("Load payload ok!");
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
            apps.push(Self {
                size,
                code,
            });
            offset += size;
        }
        apps
    }
}

