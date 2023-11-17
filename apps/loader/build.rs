#![allow(dead_code)]

use std::fs::DirEntry;
use std::io::{BufReader, BufWriter, Read, Write};
use std::process::Command;

use crate::config::BLOCK_SIZE;

const APP_DIR: &str = "../../payload";
const APP_SUFFIX: &str = ".app";

const BIN_PATH: &str = "../../payload/apps.bin";
const BIN_TEMP: &str = "../../payload/app.temp";


#[path = "src/config.rs"]
mod config;

///
///         从 payload下查找后缀为 .app 的文件合并成 image
///         |----|------------|------------|<------N------->|
///          8bit      32bit       32bit
///      app count | block count |  margin |   app_data
///
///
///
fn main() {
    let apps_bin = std::fs::File::create(BIN_TEMP).unwrap();
    let mut apps_wbuf = BufWriter::new(apps_bin);

    let apps = find_apps();

    assert!(apps.len() <= u8::MAX as usize);
    // 写入app数量
    apps_wbuf
        .write((apps.len() as u8).to_le_bytes().as_ref())
        .unwrap();

    let mut buf = [0u8; 1024];
    for dir in apps {
        let app = std::fs::File::open(dir.path()).unwrap();
        let app_size = app.metadata().unwrap().len();

        assert!(app_size <= BLOCK_SIZE as u64 * u32::MAX as u64);

        let count = (app_size / BLOCK_SIZE as u64) as u32;
        let margin = (app_size % BLOCK_SIZE as u64) as u32;

        // 写入32位 块数量
        apps_wbuf.write(count.to_le_bytes().as_ref()).unwrap();
        // 写入32位 余量
        apps_wbuf.write(margin.to_le_bytes().as_ref()).unwrap();

        let mut app_rbuf = BufReader::new(app);
        loop {
            let len = app_rbuf.read(&mut buf).unwrap();
            if len <= 0 {
                break;
            }
            apps_wbuf.write(&buf[..len]).unwrap();
        }
    }
    apps_wbuf.flush().unwrap();
    create_app_bin();
    write_bin_data();
}

/// 查找app
fn find_apps() -> Vec<DirEntry> {
    std::fs::read_dir(APP_DIR)
        .unwrap()
        .filter(|f| {
            f.as_ref()
                .unwrap()
                .file_name()
                .to_str()
                .unwrap()
                .ends_with(APP_SUFFIX)
        })
        .map(|f| f.unwrap())
        .collect()
}

fn create_app_bin() {
    Command::new("dd")
        .args([
            "if=/dev/zero",
            format!("of={}", BIN_PATH).as_str(),
            "bs=1M",
            "count=32",
        ])
        .status()
        .unwrap();
}

fn write_bin_data() {
    Command::new("dd")
        .args([
            format!("if={}", BIN_TEMP).as_str(),
            format!("of={}", BIN_PATH).as_str(),
            "conv=notrunc",
        ])
        .status()
        .unwrap();
}