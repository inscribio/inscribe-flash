use std::{path::PathBuf, collections::HashMap};
use std::io;

use regex::{Regex, RegexBuilder};
use thiserror::Error;
use tauri::api::process;

type Result<T> = std::result::Result<T, DfuUtilError>;

// Binary embedded in the app
const DFU_UTIL_SIDECAR: &str = "dfu-util";

#[derive(Error, Debug)]
pub enum DfuUtilError {
    #[error("process failed,\n--- STDOUT ---\n{stdout}\n--- STDERR ---\n{stderr}\n")]
    ProcessFailed { stdout: String, stderr: String },
    #[error("failed to parse dfu-util --list output")]
    ListParsingFailed(#[from] DfuListParsingError),
    #[error("I/O error")]
    Io(#[from] io::Error),
    #[error("tauri error")]
    Tauri(#[from] tauri::Error),
    #[error("tauri api error")]
    TauriApi(#[from] tauri::api::Error),
    #[error("invalid utf-8 characters in {0}")]
    InvalidUtf8(String),
}

/// Entry from the output of `dfu-util --list`
#[derive(Clone, serde::Serialize)]
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

/// Progress stage of dfu-util --download
#[derive(Debug, Clone, serde::Serialize)]
pub enum DfuProgress {
    Erase(usize),
    Download(usize),
}

pub struct DownloadConfig {
    pub dev_num: usize,
    pub alt: usize,
    pub address: usize,
    pub firmware: PathBuf,
    pub reset: bool,
}

pub fn list(env: HashMap<String, String>) -> Result<Vec<DfuListEntry>> {
    let output = process::Command::new_sidecar(DFU_UTIL_SIDECAR)?
        .envs(env)
        .args(["--list"])
        .output()?;

    if !output.status.success() {
        return Err(DfuUtilError::ProcessFailed {
            stdout: output.stdout.to_string(),
            stderr: output.stderr.to_string(),
        });
    }

    // Combine stdout with stderr before parsing. From tests dfu-utils outputs on stderr,
    // but parse both just to be sure.
    let combined = format!("{}\n{}", output.stdout, output.stderr);

    let lines = combined.split("\n").map(|l| l.trim());
    let entries = lines.filter_map(|l| l.parse::<DfuListEntry>().ok());

    Ok(entries.collect())
}

pub fn detach(env: HashMap<String, String>, dev_num: usize) -> Result<()> {
    let output = process::Command::new_sidecar(DFU_UTIL_SIDECAR)?
        .envs(env)
        .args(["--devnum", &dev_num.to_string()])
        .args(["--detach"])
        .output()?;

    let stdout = output.stdout.to_string();
    let stderr = output.stderr.to_string();

    match output.status.success() {
        true => if stderr.contains("dfu-util: error detaching") {
            Err(DfuUtilError::ProcessFailed { stdout, stderr })
        } else {
            Ok(())
        },
        false => Err(DfuUtilError::ProcessFailed { stdout, stderr }),
    }
}

fn download<'a>(env: HashMap<String, String>, config: &'a DownloadConfig) -> Result<process::Command> {
    let firmware = config
        .firmware
        .to_str()
        .ok_or_else(|| DfuUtilError::InvalidUtf8(config.firmware.to_string_lossy().to_string()))?;
    let dfuse_address = &format!("0x{:08x}:leave", config.address);

    let mut cmd = process::Command::new_sidecar(DFU_UTIL_SIDECAR)?.envs(env);
    cmd = cmd.args(["--devnum", &config.dev_num.to_string()])
        .args(["--alt", &config.alt.to_string()])
        .args(["--dfuse-address", dfuse_address])
        .args(["--download", firmware]);

    if config.reset {
        cmd = cmd.args(["--reset"]);
    }

    // println!("Executing command:\n{:?}", cmd);
    Ok(cmd)
}

pub async fn download_with_progress<F>(
    env: HashMap<String, String>,
    config: DownloadConfig,
    mut on_progress: F,
) -> Result<String>
where
    F: FnMut(&DfuProgress),
{
    let cmd = download(env, &config)?;

    let (mut rx, _child) = cmd.spawn()?;
    let mut stderr = String::new();

    while let Some(event) = rx.recv().await {
        match event {
            process::CommandEvent::Stdout(line) => {
                if let Ok(progress) = line.parse::<DfuProgress>() {
                    on_progress(&progress)
                }
            }
            process::CommandEvent::Stderr(line) => stderr += &line,
            _ => (),
        }
    }

    Ok(stderr)
}

impl std::str::FromStr for DfuListEntry {
    type Err = DfuListParsingError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let typ = RegexBuilder::new(r"found (dfu|runtime)")
            .case_insensitive(true)
            .build()
            .unwrap();
        let vid_pid = Regex::new(r"\[([[:xdigit:]]+):([[:xdigit:]]+)\]").unwrap();
        let devnum = Regex::new(r"devnum=(\d+)").unwrap();
        let alt = Regex::new(r"alt=(\d+)").unwrap();

        let typ = typ
            .captures(s)
            .ok_or(DfuListParsingError::MissingType(s.into()))?;
        let vid_pid = vid_pid
            .captures(s)
            .ok_or(DfuListParsingError::MissingVidPid(s.into()))?;
        let devnum = devnum
            .captures(s)
            .ok_or(DfuListParsingError::MissingDevNum(s.into()))?;
        let alt = alt
            .captures(s)
            .ok_or(DfuListParsingError::MissingAlt(s.into()))?;

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

impl std::str::FromStr for DfuProgress {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        // e.g.
        // Download  [================         ]  64%        28672 bytes
        const PATTERN: &str = r"([[:alpha:]]+)\s+\[[^]]+\]\s+(\d+)%\s+(\d+)\s+bytes";
        let pattern = Regex::new(PATTERN).unwrap();

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
