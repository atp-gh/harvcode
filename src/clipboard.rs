use std::io::Write;
use std::process::{Command, Stdio};

/// Clipboard command definition
struct ClipboardCommand {
    program: &'static str,
    args: &'static [&'static str],
}

/// Copy content to the system clipboard
///
/// Supported tools:
/// - Wayland: wl-copy
/// - X11: xclip -selection clipboard
/// - macOS: pbcopy
/// - Windows: clip
pub fn copy(content: &str) -> bool {
    let commands = [
        ClipboardCommand {
            program: "wl-copy",
            args: &[],
        },
        ClipboardCommand {
            program: "xclip",
            args: &["-selection", "clipboard"],
        },
        ClipboardCommand {
            program: "pbcopy",
            args: &[],
        },
        ClipboardCommand {
            program: "clip",
            args: &[],
        },
    ];

    for cmd in commands {
        if try_copy(cmd.program, cmd.args, content) {
            return true;
        }
    }

    false
}

/// Try copying content using a specific clipboard command
fn try_copy(program: &str, args: &[&str], content: &str) -> bool {
    let mut child = match Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => return false,
    };

    if let Some(mut stdin) = child.stdin.take() {
        if stdin.write_all(content.as_bytes()).is_err() {
            return false;
        }
    } else {
        return false;
    }

    child.wait().map(|status| status.success()).unwrap_or(false)
}
