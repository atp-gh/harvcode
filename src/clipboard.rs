use std::io::Write;
use std::process::{Command, Stdio};

/// Copy content to system clipboard
/// Supports common tools across platforms:
/// - Linux: wl-copy, xclip
/// - macOS: pbcopy
/// - Windows: clip
pub fn copy(content: &str) -> bool {
    let cmds = ["wl-copy", "xclip", "pbcopy", "clip"];

    for cmd in cmds {
        if let Ok(mut child) = Command::new(cmd).stdin(Stdio::piped()).spawn() {
            if let Some(stdin) = child.stdin.as_mut() {
                let _ = stdin.write_all(content.as_bytes());
                return true;
            }
        }
    }

    false
}
