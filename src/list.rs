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
    let listed_files = select_files(files, cfg, report);

    report.set_output_size(list_output_size(&listed_files));

    write_stdout(&listed_files)?;

    report.add_destination("stdout");

    Ok(())
}

/// Apply file filters and stable ordering to the candidate file list.
///
/// Extracted from `run` so the filtering and sorting behavior can be tested
/// without capturing stdout.
fn select_files(files: Vec<PathBuf>, cfg: &Config, report: &mut Report) -> Vec<PathBuf> {
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
    //
    // Compare the lossy representations in place: `to_string_lossy` borrows
    // for valid UTF-8 paths, so no intermediate `String` is allocated per
    // path during the sort.
    listed_files.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));

    listed_files
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::OutputConfig;
    use crate::args::PickerKind;
    use crate::filter::Rules;
    use std::path::PathBuf;

    fn config_with_rules(rules: Rules) -> Config {
        Config {
            roots: vec![PathBuf::from(".")],
            pick: false,
            picker: None::<PickerKind>,
            rules,
            output: OutputConfig::default(),
            quiet: false,
            verbose: false,
            list: true,
            max_file_size: 0,
            max_total_size: 0,
        }
    }

    #[test]
    fn select_files_applies_filters_and_counts() {
        let rules = Rules {
            include_ext: vec!["rs".to_string()],
            ..Rules::default()
        };
        let cfg = config_with_rules(rules);

        let files = vec![
            PathBuf::from("a.rs"),
            PathBuf::from("b.md"),
            PathBuf::from(".hidden.rs"),
        ];

        let mut report = Report::new();
        let selected = select_files(files, &cfg, &mut report);

        assert_eq!(selected, vec![PathBuf::from("a.rs")]);

        let rendered = report.render();
        assert!(rendered.contains("Collected files: 1"));
        assert!(rendered.contains("Skipped files: 2"));
    }

    #[test]
    fn select_files_sorts_paths_stably() {
        let cfg = config_with_rules(Rules::default());

        let files = vec![
            PathBuf::from("c.rs"),
            PathBuf::from("a.rs"),
            PathBuf::from("b.rs"),
        ];

        let mut report = Report::new();
        let selected = select_files(files, &cfg, &mut report);

        let names: Vec<String> = selected
            .iter()
            .map(|path| path.file_name().unwrap().to_string_lossy().to_string())
            .collect();

        assert_eq!(names, vec!["a.rs", "b.rs", "c.rs"]);
    }

    #[test]
    fn list_output_size_includes_newlines() {
        let files = vec![PathBuf::from("a.rs"), PathBuf::from("b.rs")];

        // "a.rs\n" (5) + "b.rs\n" (5) = 10
        assert_eq!(list_output_size(&files), 10);
    }
}
