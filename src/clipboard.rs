use std::io::Write;
use std::process::{Command, Stdio};

/// Definition of an external clipboard command.
struct ClipboardCommand {
    /// Executable name resolved through the system PATH.
    program: &'static str,

    /// Arguments passed to the clipboard command.
    args: &'static [&'static str],
}

/// Clipboard commands supported on Linux.
///
/// Wayland uses `wl-copy`, while X11 commonly uses `xclip`.
/// The commands are attempted in order, so Wayland is preferred when both
/// tools are installed.
#[cfg(target_os = "linux")]
const COMMANDS: &[ClipboardCommand] = &[
    ClipboardCommand {
        program: "wl-copy",
        args: &[],
    },
    ClipboardCommand {
        program: "xclip",
        args: &["-selection", "clipboard"],
    },
];

/// Clipboard command supported on macOS.
#[cfg(target_os = "macos")]
const COMMANDS: &[ClipboardCommand] = &[ClipboardCommand {
    program: "pbcopy",
    args: &[],
}];

/// Clipboard command supported on Windows.
#[cfg(target_os = "windows")]
const COMMANDS: &[ClipboardCommand] = &[ClipboardCommand {
    program: "clip.exe",
    args: &[],
}];

/// No external clipboard commands are configured for unsupported targets.
///
/// Defining an empty command list allows the rest of the clipboard logic to
/// compile without introducing platform-specific behavior for unknown targets.
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows",)))]
const COMMANDS: &[ClipboardCommand] = &[];

/// Copy content to the system clipboard.
///
/// Only commands supported by the current compilation target are included in
/// the binary. Commands are attempted in order until one completes
/// successfully.
///
/// Returns `true` when a clipboard command succeeds and `false` when no
/// supported command is available or every command fails.
pub fn copy(content: &str) -> bool {
    COMMANDS
        .iter()
        .any(|command| try_copy(command.program, command.args, content))
}

/// Try to copy content using one external clipboard command.
///
/// The command receives the content through standard input. Its standard
/// output and standard error are discarded because callers only need to know
/// whether the operation succeeded.
fn try_copy(program: &str, args: &[&str], content: &str) -> bool {
    let mut child = match Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,

        // A failed spawn usually means that the command is unavailable or
        // could not be started. The caller may then try the next command.
        Err(_) => return false,
    };

    let Some(mut stdin) = child.stdin.take() else {
        return false;
    };

    // Send all content to the clipboard process through standard input.
    if stdin.write_all(content.as_bytes()).is_err() {
        return false;
    }

    // Close the standard input pipe explicitly before waiting. This sends EOF
    // to commands that wait for the complete input before exiting.
    drop(stdin);

    // Treat the operation as successful only when the process exits with a
    // successful status code.
    matches!(child.wait(), Ok(status) if status.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_fails_when_no_commands_are_configured() {
        // On targets without a clipboard command table, copy must report
        // failure instead of panicking.
        if COMMANDS.is_empty() {
            assert!(!copy("hello"));
        }
    }

    #[test]
    fn try_copy_fails_when_program_does_not_exist() {
        // Spawning a program that does not exist is the normal way a
        // clipboard command is reported as unavailable.
        assert!(!try_copy(
            "harvcode-no-such-clipboard-program",
            &[],
            "hello"
        ));
    }

    #[cfg(unix)]
    #[test]
    fn try_copy_succeeds_when_program_reads_stdin_and_exits_ok() {
        // `cat` reads stdin and exits successfully. This exercises the
        // write-to-stdin → close → wait pipeline on a stable command.
        assert!(try_copy("cat", &[], "hello"));
    }

    #[cfg(unix)]
    #[test]
    fn try_copy_fails_when_program_exits_nonzero() {
        // A command that exits with a failure status must be reported as a
        // failed copy, even though the spawn and stdin write succeeded.
        assert!(!try_copy("false", &[], "hello"));
    }
}
