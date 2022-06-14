#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{io::{self, Write}, path::Path};

use tempfile::NamedTempFile;

use app::dfu_util;

const DFU_UTIL_BIN: &str = "dfu-util";
const ALT_SETTING: usize = 0;
const DOWNLOAD_ADDRESS: usize = 0x0800_0000;

// Store file bytes received from the frontend as file, as We cannot use paths from
// frontend directly because they are mangled and don't reflect filesystem paths.
pub fn bytes_as_file(bytes: &[u8]) -> io::Result<NamedTempFile> {
    let mut file = NamedTempFile::new()?;
    file.write_all(bytes)?;
    file.flush()?;
    Ok(file)
}

#[tauri::command]
fn list() -> Result<Vec<dfu_util::DfuListEntry>, String> {
    dfu_util::list(DFU_UTIL_BIN)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn detach(dev_num: usize) -> Result<(), String> {
    dfu_util::detach(DFU_UTIL_BIN, dev_num)
        .map_err(|e| e.to_string())
}

#[tauri::command(async)]
fn flash(window: tauri::Window, firmware: Vec<u8>, dev_num: usize) -> Result<usize, String> {
    let n_bytes = firmware.len();
    let file = bytes_as_file(&firmware)
        .map_err(|e| e.to_string())?;

    let config = dfu_util::DownloadConfig {
        bin: Path::new(DFU_UTIL_BIN),
        dev_num,
        alt: ALT_SETTING,
        address: DOWNLOAD_ADDRESS,
        firmware: file.path(),
        reset: true,
    };

    let _output = dfu_util::download_with_progress(&config, |progress| {
        window.emit("flash-progress", progress.clone()).ok();
    }).map_err(|e| e.to_string())?;

    Ok(n_bytes)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![list, detach, flash])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
