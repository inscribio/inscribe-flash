#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::process;

mod flash;

#[tauri::command(async)]
fn flash(window: tauri::Window, firmware: Vec<u8>) -> Result<usize, String> {
    let n_bytes = firmware.len();

    let device = flash::find_device().map_err(|e| e.to_string())?;

    // TODO: detach

    flash::flash_firmware(device.devnum, &firmware, |progress| {
        window.emit("flash-progress", progress.clone());
    }).map_err(|e| e.to_string())?;

    Ok(n_bytes)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![flash])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
