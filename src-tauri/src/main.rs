#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{io::{self, Write}, path::Path, collections::HashMap};

use tauri::{Manager, PathResolver};
use tempfile::NamedTempFile;

use app::dfu_util;

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

fn dfu_util_env(path_resolver: PathResolver) -> HashMap<String, String> {
    let mut env = HashMap::new();
    if let Some(resource_dir) = path_resolver.resource_dir() {
        let libs_dir = Path::join(&resource_dir, "bin/libs");
        if cfg!(target_os = "macos") {
            let var = "DYLD_LIBRARY_PATH";
            let dir = if cfg!(target_arch = "aarch64") {
                "aarch64-apple-darwin"
            } else {
                "x86_64-apple-darwin"
            };
            env.insert(var.to_string(), Path::join(&libs_dir, dir).to_string_lossy().to_string());
       }
       // env.insert("LD_LIBRARY_PATH", );
    }
    env
}

#[tauri::command]
fn list(app_handle: tauri::AppHandle) -> Result<Vec<dfu_util::DfuListEntry>, String> {
    let env = dfu_util_env(app_handle.path_resolver());
    dfu_util::list(env).map_err(|e| e.to_string())
}

#[tauri::command]
fn detach(app_handle: tauri::AppHandle, dev_num: usize) -> Result<(), String> {
    let env = dfu_util_env(app_handle.path_resolver());
    dfu_util::detach(env, dev_num)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn flash(window: tauri::Window, firmware: Vec<u8>, dev_num: usize) -> Result<usize, String> {
    let n_bytes = firmware.len();
    let file = bytes_as_file(&firmware)
        .map_err(|e| e.to_string())?;

    let config = dfu_util::DownloadConfig {
        dev_num,
        alt: ALT_SETTING,
        address: DOWNLOAD_ADDRESS,
        firmware: file.path().to_path_buf(),
        reset: false,
    };

    let env = dfu_util_env(window.app_handle().path_resolver());
    let _stderr = dfu_util::download_with_progress(env, config, |progress| {
        window.emit("flash-progress", progress.clone()).ok();
    }).await.map_err(|e| e.to_string())?;

    Ok(n_bytes)
}

#[tauri::command]
fn has_winusb() -> bool {
    cfg!(windows)
}

#[cfg(windows)]
fn tauri_main() {
    use app::winusb;
    winusb::run(|installer| {
        tauri::Builder::default()
            .manage(installer)
            .invoke_handler(tauri::generate_handler![
                list,
                detach,
                flash,
                has_winusb,
                winusb::winusb_install,
                winusb::winusb_candidates,
            ])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    });
}

#[cfg(not(windows))]
fn tauri_main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            list,
            detach,
            flash,
            has_winusb,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format_timestamp(None)
        .init();
    log::info!("Starting");
    log::trace!("Starting trace");

    tauri_main();
}
