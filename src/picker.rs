use std::io::Write;
use std::process::{Command, Stdio};

use crate::args::PickerKind;

/// Check if a command exists in PATH
fn exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Select available fuzzy finder
/// Priority: sk > fzf
fn picker_cmd(picker: Option<PickerKind>) -> Option<Vec<&'static str>> {
    match picker {
        Some(PickerKind::Sk) => {
            if exists("sk") {
                Some(vec!["sk", "-m"])
            } else {
                None
            }
        }
        Some(PickerKind::Fzf) => {
            if exists("fzf") {
                Some(vec!["fzf", "-m"])
            } else {
                None
            }
        }
        None => {
            if exists("sk") {
                Some(vec!["sk", "-m"])
            } else if exists("fzf") {
                Some(vec!["fzf", "-m"])
            } else {
                None
            }
        }
    }
}

/// Run interactive selection
/// Input: newline-separated file paths
/// Output: selected file paths
pub fn pick(input: &str, picker: Option<PickerKind>) -> Option<Vec<String>> {
    let cmd = picker_cmd(picker)?;

    let mut child = Command::new(cmd[0])
        .args(&cmd[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .ok()?;

    // Feed file list into picker
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes()).ok()?;
    }

    let output = child.wait_with_output().ok()?;

    if !output.status.success() {
        return None;
    }

    let result = String::from_utf8_lossy(&output.stdout);

    let files = result
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>();

    if files.is_empty() {
        None
    } else {
        Some(files)
    }
}
