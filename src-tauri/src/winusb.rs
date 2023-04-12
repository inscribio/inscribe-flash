use std::io;

use winusb_installer::{Mode, Device, Server, InstallConfig};
use tokio::sync::{Mutex, TryLockError};
use serde::{Serialize, Deserialize};

const STM32_BOOTLOADER_VID: u16 = 0x0483;
const STM32_BOOTLOADER_PID: u16 = 0xdf11;
const VENDOR: &str = "inscrib.io";
const DRIVER_PATH: &str = "C:\\usb_driver";
const INF_NAME: &str = "STM32BootloaderWinUSB.inf";

pub struct Installer(Mutex<Server>);

#[derive(Debug, Serialize, Deserialize)]
pub enum InstallError {
    /// Installation has already been started and is ongoing
    Ongoing,
    /// Input/output error
    Io(String),
}

#[tauri::command]
pub async fn winusb_install(state: tauri::State<'_, Installer>, window: tauri::Window, devices: Vec<Device>) -> Result<(), InstallError> {
    let mut server = state.0.try_lock()?;
    let on_progress = |progress| {
        window.emit("winusb-progress", progress).ok();
    };
    Ok(server.install(install_config(), &devices, on_progress).await?)
}

#[tauri::command]
pub async fn winusb_candidates(state: tauri::State<'_, Installer>) -> Result<Vec<Device>, InstallError> {
    let server = state.0.try_lock()?;
    let devices = server.visible_devices()?
        .into_iter()
        .filter(needs_install)
        .collect();
    Ok(devices)
}

/// Runs either main tauri app or winusb installation client. Tauri app must be started in
/// `tauri_main` callback. It must add Installer to managed state and add the command handlers from
/// this module via `.invoke_handler()`.
pub fn run(tauri_main: impl FnOnce(Installer)) {
    let mode = winusb_installer::init();
    match mode {
        Mode::Server(mut server) => {
            server.show_child_window(false);
            tauri_main(Installer::new(server));
        },
        Mode::Client(mut client) => {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed building the Runtime")
                .block_on(client.serve())
                .expect("Error while running wiusb install client");
        },
    }
}

fn needs_install(dev: &Device) -> bool {
    (dev.vid, dev.pid) == (STM32_BOOTLOADER_VID, STM32_BOOTLOADER_PID)
        && !dev.has_winusb()
}

fn install_config() -> InstallConfig {
    InstallConfig {
        vendor: VENDOR.to_string(),
        driver_path: DRIVER_PATH.to_string(),
        inf_name: INF_NAME.to_string(),
    }
}

impl From<io::Error> for InstallError {
    fn from(err: io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<TryLockError> for InstallError {
    fn from(_err: TryLockError) -> Self {
        Self::Ongoing
    }
}

impl Installer {
    pub fn new(server: Server) -> Self {
        Self(Mutex::new(server))
    }
}
