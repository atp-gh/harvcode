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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;

    /// Serializes tests that temporarily modify the process `PATH`.
    ///
    /// Rust runs tests in parallel by default, and `pick` resolves its
    /// subprocess by bare name through `PATH`. Mutating `PATH` in one test
    /// would race with another test that also relies on it, so all picker
    /// tests that touch `PATH` take this lock.
    static PATH_LOCK: Mutex<()> = Mutex::new(());

    /// Create a fake executable `sk` in a temporary directory and return the
    /// directory. The fake picker echoes the given output to stdout, which
    /// lets tests exercise the spawn → write → read pipeline without a real
    /// fuzzy finder.
    #[cfg(unix)]
    fn fake_picker(script: &str) -> tempfile::TempDir {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let sk = dir.path().join("sk");
        fs::write(&sk, script).unwrap();

        let mut perms = fs::metadata(&sk).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&sk, perms).unwrap();

        dir
    }

    /// Temporarily replace `PATH` with a single directory, run `body`, then
    /// restore the original `PATH`.
    #[cfg(unix)]
    fn with_path<T>(dir: &std::path::Path, body: impl FnOnce() -> T) -> T {
        let original_path = std::env::var_os("PATH");
        std::env::set_var("PATH", dir);

        let result = body();

        match original_path {
            Some(path) => std::env::set_var("PATH", path),
            None => std::env::remove_var("PATH"),
        }

        result
    }

    #[test]
    fn explicit_sk_selects_only_sk() {
        let commands = picker_commands(Some(PickerKind::Sk));
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].0, "sk");
    }

    #[test]
    fn explicit_fzf_selects_only_fzf() {
        let commands = picker_commands(Some(PickerKind::Fzf));
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].0, "fzf");
    }

    #[test]
    fn automatic_mode_prefers_sk_then_fzf() {
        let commands = picker_commands(None);
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].0, "sk");
        assert_eq!(commands[1].0, "fzf");
    }

    #[cfg(unix)]
    #[test]
    fn pick_returns_selected_lines() {
        let dir = fake_picker("#!/bin/sh\nprintf 'src/main.rs\\nsrc/lib.rs\\n'\n");

        let _guard = PATH_LOCK.lock().unwrap();
        let result = with_path(dir.path(), || pick("ignored input", Some(PickerKind::Sk)));

        assert_eq!(
            result,
            Some(vec!["src/main.rs".to_string(), "src/lib.rs".to_string()])
        );
    }

    #[cfg(unix)]
    #[test]
    fn pick_returns_none_when_picker_unavailable() {
        // Point PATH at an empty directory so no `sk` exists, simulating an
        // unavailable picker.
        let empty = tempfile::tempdir().unwrap();

        let _guard = PATH_LOCK.lock().unwrap();
        let result = with_path(empty.path(), || pick("input", Some(PickerKind::Sk)));

        assert_eq!(result, None);
    }
}
