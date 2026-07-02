use std::io::{self, Write};
use std::path::PathBuf;

use crate::args::Config;
use crate::filter;
use crate::report::Report;

/// Run `--list` mode.
///
/// This mode prints the final set of files that would be processed by
/// normal content output mode.
///
/// Behavior:
/// - Applies the same file filters used by normal output.
/// - Prints one valid file path per line to stdout.
/// - Sorts paths for stable, deterministic output.
/// - Does not read or print file contents.
/// - Does not write to output files.
/// - Does not copy anything to the clipboard.
///
/// The input `files` should already be expanded from roots and, if enabled,
/// narrowed by the interactive picker.
pub fn run(files: Vec<PathBuf>, cfg: &Config, report: &mut Report) -> io::Result<()> {
    let mut listed_files = files
        .into_iter()
        .filter(|path| {
            let valid = filter::is_valid(path, &cfg.rules);

            if valid {
                report.collect_file();
            } else {
                report.skip_file();
            }

            valid
        })
        .collect::<Vec<_>>();

    // Keep list output stable across filesystems and runs.
    listed_files.sort_by_key(|path| path.to_string_lossy().to_string());

    report.set_output_size(list_output_size(&listed_files));

    write_stdout(&listed_files)?;

    report.add_destination("stdout");

    Ok(())
}

/// Write file paths to stdout, one path per line.
fn write_stdout(files: &[PathBuf]) -> io::Result<()> {
    let mut stdout = io::stdout().lock();

    for path in files {
        writeln!(stdout, "{}", path.display())?;
    }

    stdout.flush()
}

/// Return the number of bytes that will be written by list mode.
///
/// Each listed path is followed by a trailing newline.
fn list_output_size(files: &[PathBuf]) -> usize {
    files
        .iter()
        .map(|path| path.display().to_string().len() + 1)
        .sum()
}
