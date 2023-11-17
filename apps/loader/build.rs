use std::fs::DirEntry;
use std::io::{BufReader, BufWriter, Read, Write};
use std::process::Command;

use crate::config::BLOCK_SIZE;

const APP_PATH: &str = "../../payload/hello_app.app";

const BIN_PATH: &str = "../../payload/apps.bin";
const BIN_TEMP: &str = "../../payload/apps.temp";


#[path = "src/config.rs"]
mod config;

///
///         |----|------------|------------|------N-------|
///          8bit      32bit       32bit      app_data
///      app count | block count |  margin
///
///
///
fn main() {
    let apps_bin = std::fs::File::create(BIN_TEMP).unwrap();
    let mut apps_wbuf = BufWriter::new(apps_bin);

    let app = std::fs::File::open(APP_PATH).unwrap();
    let app_size = app.metadata().unwrap().len();

    assert!(app_size <= BLOCK_SIZE as u64 * u32::MAX  as u64);

    let count = (app_size / BLOCK_SIZE as u64) as u32;
    let margin = (app_size % BLOCK_SIZE as u64) as u32;

    let mut buf = [0u8; 1024];

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
    apps_wbuf.flush().unwrap();
    create_app_bin();
    write_bin_data();


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