use std::io::Write;
use std::process::{Command, Stdio};

use crate::args::PickerKind;

/// Return the picker commands that should be attempted.
///
/// The returned slice contains static command definitions, so selecting a
/// picker does not require allocating a temporary `Vec`.
///
/// Behavior:
/// - An explicitly selected picker produces exactly one candidate.
/// - Automatic selection tries `sk` first, followed by `fzf`.
fn picker_commands(
    picker: Option<PickerKind>,
) -> &'static [(&'static str, &'static [&'static str])] {
    const SK: &[(&str, &[&str])] = &[("sk", &["-m"])];
    const FZF: &[(&str, &[&str])] = &[("fzf", &["-m"])];
    const AUTO: &[(&str, &[&str])] = &[("sk", &["-m"]), ("fzf", &["-m"])];

    match picker {
        Some(PickerKind::Sk) => SK,
        Some(PickerKind::Fzf) => FZF,
        None => AUTO,
    }
}

/// Run an interactive fuzzy finder and return the selected file paths.
///
/// The input must contain newline-separated file paths. Each selected path is
/// returned as an individual `String`.
///
/// Instead of using `which` to check whether a picker is installed, this
/// function directly attempts to spawn each candidate command. A failed spawn
/// indicates that the command is unavailable or could not be started, allowing
/// automatic selection to continue with the next candidate.
///
/// Returns `None` when:
/// - No picker command can be started.
/// - The picker exits unsuccessfully or is cancelled.
/// - Writing the file list to the picker fails.
/// - Reading the picker output fails.
/// - The picker returns no selected files.
pub fn pick(input: &str, picker: Option<PickerKind>) -> Option<Vec<String>> {
    for &(program, args) in picker_commands(picker) {
        // Try starting the picker directly. This avoids launching a separate
        // `which` process before launching the actual picker.
        let mut child = match Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,

            // In automatic mode, continue to the next candidate when the
            // current picker is unavailable or cannot be started.
            Err(_) => continue,
        };

        // Send the newline-separated file list to the picker's standard input.
        //
        // Taking ownership of stdin allows it to be explicitly dropped after
        // writing, which sends EOF so the picker can finish processing input.
        let mut stdin = child.stdin.take()?;
        stdin.write_all(input.as_bytes()).ok()?;
        drop(stdin);

        // Wait for the picker to exit and collect its standard output.
        let output = child.wait_with_output().ok()?;

        // A non-zero status normally means that selection was cancelled or
        // the picker encountered an error.
        if !output.status.success() {
            return None;
        }

        // Convert each non-empty output line into one selected file path.
        let files: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_owned)
            .collect();

        // An empty result is treated as a cancelled selection.
        return (!files.is_empty()).then_some(files);
    }

    // None of the configured picker commands could be started.
    None
}
