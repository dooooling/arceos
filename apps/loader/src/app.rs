use alloc::vec;
use alloc::vec::Vec;
use crate::config::{APP_COUNT_WIDTH, BLOCK_COUNT_WIDTH, BLOCK_SIZE, MARGIN_SIZE_WIDTH};

pub struct AppManager<'a> {
    pub apps: Vec<App<'a>>,
}

pub struct App<'a> {
    pub size: usize,
    pub code: &'a [u8],
}

impl AppManager<'_> {
    /// 解析app
    pub fn parse(apps_start: *const u8) -> Self {
        let mut apps = vec![];

        let app_count = unsafe { core::slice::from_raw_parts(apps_start, APP_COUNT_WIDTH) };
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

            apps.push(App {
                size,
                code,
            });
        }
        Self {
            apps
        }
    }
}