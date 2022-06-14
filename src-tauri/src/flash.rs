use std::borrow::Cow;
/// Flash keyboard firmware using dfu-util
///
/// Order of operation:
/// 1. Find supported USB device - either STM32 DFU bootloader or USB keyboard.
/// 2. (if no DFU bootloader) Detach USB keyboard to enter DFU bootloader, then wait.
/// 3. Run dfu-util to flash the device.

use std::{io, process};
use std::io::{Write, BufReader, BufRead};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use thiserror::Error;
use regex;

const DFU_UTIL_BIN: &str = "dfu-util";

const STM32_BOOTLOADER_VID_PID: (u16, u16) = (0x0483, 0xdf11);
const KEYBOARD_VID_PID: (u16, u16) = (0x16c0, 0x27db);

#[derive(Error, Debug)]
pub enum FlashError {
    #[error("dfu-util command failed with output: {0}")]
    CommandFailed(String),
    #[error("path is not valid: {0}")]
    InvalidPath(PathBuf),
    #[error("system IO error")]
    IoError(#[from] io::Error),
    #[error("failed to parse output of dfu-util --list")]
    ListParseError(#[from] DfuListParsingError),
    #[error("no supported devices found")]
    NoDevicesFound,
}

#[derive(Clone)]
pub struct DfuListEntry {
    pub is_dfu: bool,
    pub vid: u16,
    pub pid: u16,
    pub devnum: usize,
    pub alt: usize,
}

#[derive(Error, Debug)]
pub enum DfuListParsingError {
    #[error("missing DFU/Rutime type")]
    MissingType(String),
    #[error("missing [vid:pid]")]
    MissingVidPid(String),
    #[error("missing devnum=N")]
    MissingDevNum(String),
    #[error("missing alt=N")]
    MissingAlt(String),
    #[error("failed to parse number: {0}")]
    InvalidNumber(#[from] std::num::ParseIntError),
}

impl std::str::FromStr for DfuListEntry {
    type Err = DfuListParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let typ = regex::RegexBuilder::new(r"found (dfu|runtime)")
            .case_insensitive(true)
            .build()
            .unwrap();
        let vid_pid = regex::Regex::new(r"\[([[:xdigit:]]+):([[:xdigit:]]+)\]").unwrap();
        let devnum = regex::Regex::new(r"devnum=(\d+)").unwrap();
        let alt = regex::Regex::new(r"alt=(\d+)").unwrap();

        let typ = typ.captures(s).ok_or(DfuListParsingError::MissingType(s.into()))?;
        let vid_pid = vid_pid.captures(s).ok_or(DfuListParsingError::MissingVidPid(s.into()))?;
        let devnum = devnum.captures(s).ok_or(DfuListParsingError::MissingDevNum(s.into()))?;
        let alt = alt.captures(s).ok_or(DfuListParsingError::MissingAlt(s.into()))?;

        let typ = typ.get(1).unwrap();
        let (vid, pid) = (vid_pid.get(1).unwrap(), vid_pid.get(2).unwrap());
        let devnum = devnum.get(1).unwrap();
        let alt = alt.get(1).unwrap();

        Ok(Self {
            is_dfu: typ.as_str().to_lowercase() == "dfu",
            vid: u16::from_str_radix(vid.as_str(), 16)?,
            pid: u16::from_str_radix(pid.as_str(), 16)?,
            devnum: devnum.as_str().parse::<usize>()?,
            alt: alt.as_str().parse::<usize>()?,
        })
    }
}

pub fn dfu_list() -> io::Result<process::Child> {
    process::Command::new(DFU_UTIL_BIN)
        .arg("--list")
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .spawn()
}

pub fn find_devices() -> Result<Vec<DfuListEntry>, FlashError> {
    let output = dfu_list()?.wait_with_output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Combine stdout with stderr before parsing. From tests dfu-utils outputs on stderr,
    // but parse both just to be sure.
    let out = format!("{}\n{}", stdout, stderr);

    if !output.status.success() {
        return Err(FlashError::CommandFailed(out));
    }

    let lines = out.split("\n").map(|l| l.trim());
    let entries = lines.filter_map(|l| l.parse::<DfuListEntry>().ok());

    Ok(entries.collect())
}

pub fn find_device() -> Result<DfuListEntry, FlashError> {
    let entries = find_devices()?;

    let bootloaders: Vec<_> = entries.iter().filter(|e| (e.vid, e.pid) == STM32_BOOTLOADER_VID_PID).collect();
    let keyboards: Vec<_> = entries.iter().filter(|e| (e.vid, e.pid) == KEYBOARD_VID_PID).collect();

    match (bootloaders.len(), keyboards.len()) {
        (0, 0) => Err(FlashError::NoDevicesFound),
        (1, 0) => Ok(bootloaders[0].clone()),
        (0, 1) => Ok(keyboards[0].clone()),
        (b, k) => {
            // TODO: select the one with highest devnum (most likely connected most recently)
            println!("Found multiple supported entries: {} bootloaders, {} keyboards", b, k);
            let dev = bootloaders[0];
            println!("Selecting bootloader: devnum={}, vid:pid={:04x}:{:04x}", dev.devnum, dev.vid, dev.pid);
            Ok(dev.clone())
        }
    }
}

pub fn dfu_flash(dev_num: usize, firmware: &Path) -> Result<process::Child, FlashError> {
    let mut cmd = process::Command::new(DFU_UTIL_BIN);
    cmd.args(["--devnum", &dev_num.to_string()])
        .args(["--alt", "0"])
        .args(["--dfuse-address", "0x08000000:leave"])
        .args(["--download", firmware.to_str().ok_or_else(|| FlashError::InvalidPath(firmware.to_path_buf()))?])
        .arg("--reset")
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped());
    println!("Executing command:\n{:?}", cmd);
    let child = cmd.spawn()?;
    Ok(child)
}

// Store firmware received from the frontend as file for dfu-util to download.
// We cannot use path from frontend directly as uploaded file paths are mangled.
pub fn firmware_as_file(bytes: &[u8]) -> io::Result<NamedTempFile> {
    let mut file = NamedTempFile::new()?;
    file.write_all(bytes)?;
    file.flush()?;
    Ok(file)
}

// Wrapper needed to avoid deletion of temporary file until the process finishes
struct DfuFlasher {
    proc: process::Child,
    file: NamedTempFile,
}

fn dfu_flash_firmware(dev_num: usize, firmware: &[u8]) -> Result<DfuFlasher, FlashError> {
    let file = firmware_as_file(firmware)?;
    let proc = dfu_flash(dev_num, file.path())?;
    Ok(DfuFlasher { proc, file })
}

// TODO: detach

/// Buffered stream reader that treats both \r and \n as line termination.
/// This allows to iterate in real time over output of commands with progress bars.
///
/// Based on https://stackoverflow.com/a/55145242 and implementations in std.
#[derive(Debug)]
struct ProgressLines<B> {
    reader: B
}

impl<B: BufRead> ProgressLines<B> {
    pub fn new(reader: B) -> Self {
        Self { reader }
    }

    // Keep reading and appending to buf until delimiter is found or we read EOF.
    // Returns true if found delimiter, else (=> found EOF) false.
    fn read_until_delimiter(&mut self, buf: &mut Vec<u8>) -> io::Result<bool> {
        loop {
            // Perform single IO operation and check if we found delimiter
            let (found, consumed) = {
                // Fill the buffer from stream (performs IO)
                let bytes = match self.reader.fill_buf() {
                    Ok(b) => b,
                    Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(e) => return Err(e),
                };

                // Find the delimiter
                let is_delim = |c: &u8| *c == b'\n' || *c == b'\r';
                match bytes.iter().position(is_delim) {
                    Some(n) => {
                        // Drop delimiter byte
                        buf.extend_from_slice(&bytes[..n]);
                        // Consume with the delimiter
                        (true, n + 1)
                    },
                    None => {
                        // Append all data that has been read
                        buf.extend_from_slice(bytes);
                        (false, bytes.len())
                    },
                }
            };

            // Drop used data from reader
            self.reader.consume(consumed);

            // Stop reading if we have a line, or if we encounter EOF
            let end_of_file = consumed == 0;
            if found || end_of_file {
                return Ok(found);
            }
        }
    }
}

impl<B: BufRead> Iterator for ProgressLines<B> {
    type Item = Result<String, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = Vec::<u8>::new();

        match self.read_until_delimiter(&mut buf) {
            Err(e) => Some(Err(e)),
            Ok(false) => None,  // EOF
            Ok(true) => {
                // TODO: it's probably possible to keep Cow by storing `buf` in `self`
                let line = String::from_utf8_lossy(&buf).to_string();
                Some(Ok(line))
            }
        }
    }
}

/// Progress stage of dfu-util download
#[derive(Debug, Clone, serde::Serialize)]
pub enum DfuProgress {
    Erase(usize),
    Download(usize),
}

impl std::str::FromStr for DfuProgress {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // e.g.
        // Download  [================         ]  64%        28672 bytes
        const PATTERN: &str = r"([[:alpha:]]+)\s+\[[^]]+\]\s+(\d+)%\s+(\d+)\s+bytes";
        let pattern = regex::Regex::new(PATTERN).unwrap();

        let groups = pattern.captures(s.trim()).ok_or(())?;
        let stage = groups.get(1).unwrap();
        let _percent = groups.get(2).unwrap();
        let bytes = groups.get(3).unwrap();

        let stage = stage.as_str().to_lowercase();
        let bytes = bytes.as_str().parse::<usize>().map_err(|_| ())?;

        match stage.as_str() {
            "erase" => Ok(Self::Erase(bytes)),
            "download" => Ok(Self::Download(bytes)),
            _ => Err(()),
        }
    }
}

// Use dfu-util to flash firmware, with progress updates.
pub fn flash_firmware<F>(dev_num: usize, firmware: &[u8], mut on_progress: F) -> Result<process::Output, FlashError>
where
    F: FnMut(&DfuProgress)
{
    let flasher = dfu_flash_firmware(dev_num, firmware)?;
    let mut proc = flasher.proc;
    let out = BufReader::new(proc.stdout.as_mut().unwrap());

    for line in ProgressLines::new(out) {
        if let Ok(progress) = line?.parse::<DfuProgress>() {
            on_progress(&progress)
        }
    }

    Ok(proc.wait_with_output()?)
}
