use crate::args::Config;

/// Clipboard reporting state.
#[derive(Clone, Copy, Default)]
pub enum ClipboardStatus {
    #[default]
    NotRequested,
    Success,
    Failed,
}

/// Execution report for verbose output.
#[derive(Default)]
pub struct Report {
    collected_files: usize,
    skipped_files: usize,
    output_size: usize,
    destinations: Vec<String>,
    clipboard: ClipboardStatus,
}

impl Report {
    /// Create a new execution report.
    pub fn new() -> Self {
        Self::default()
    }

    /// Count one collected file.
    pub fn collect_file(&mut self) {
        self.collected_files += 1;
    }

    /// Count one skipped file.
    pub fn skip_file(&mut self) {
        self.skipped_files += 1;
    }

    /// Store final output size in bytes.
    pub fn set_output_size(&mut self, bytes: usize) {
        self.output_size = bytes;
    }

    /// Add a successful output destination.
    pub fn add_destination(&mut self, destination: impl Into<String>) {
        self.destinations.push(destination.into());
    }

    /// Store clipboard result.
    pub fn set_clipboard(&mut self, status: ClipboardStatus) {
        self.clipboard = status;
    }

    /// Print verbose report unless quiet mode is enabled.
    pub fn print_if_enabled(&self, cfg: &Config) {
        if !cfg.verbose || cfg.quiet {
            return;
        }

        eprint!("{}", self.render());
    }

    /// Render the report as a single string.
    ///
    /// Kept separate from `print_if_enabled` so the exact output can be
    /// asserted in unit tests without capturing stderr.
    pub(crate) fn render(&self) -> String {
        let mut out = String::new();

        out.push_str(&format!("Collected files: {}\n", self.collected_files));
        out.push_str(&format!("Skipped files: {}\n", self.skipped_files));
        out.push_str(&format!("Output size: {}\n", format_size(self.output_size)));

        if self.destinations.is_empty() {
            out.push_str("Output destination: none\n");
        } else {
            out.push_str(&format!(
                "Output destination: {}\n",
                self.destinations.join(", ")
            ));
        }

        let clipboard = match self.clipboard {
            ClipboardStatus::NotRequested => "not requested",
            ClipboardStatus::Success => "success",
            ClipboardStatus::Failed => "failed",
        };
        out.push_str(&format!("Clipboard: {}\n", clipboard));

        out
    }
}

/// Format a byte count for human-readable reporting.
fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB ({} bytes)", bytes as f64 / 1024.0, bytes)
    } else {
        format!("{:.1} MB ({} bytes)", bytes as f64 / 1024.0 / 1024.0, bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::Config;
    use crate::args::OutputConfig;
    use crate::args::PickerKind;
    use crate::filter::Rules;
    use std::path::PathBuf;

    fn config(verbose: bool, quiet: bool) -> Config {
        Config {
            roots: vec![PathBuf::from(".")],
            pick: false,
            picker: None::<PickerKind>,
            rules: Rules::default(),
            output: OutputConfig::default(),
            quiet,
            verbose,
            list: false,
            max_file_size: 0,
            max_total_size: 0,
        }
    }

    #[test]
    fn tracks_collected_and_skipped_counts() {
        let mut report = Report::new();
        report.collect_file();
        report.collect_file();
        report.skip_file();

        let rendered = report.render();
        assert!(rendered.contains("Collected files: 2"));
        assert!(rendered.contains("Skipped files: 1"));
    }

    #[test]
    fn reports_output_size_in_human_readable_form() {
        let mut report = Report::new();
        report.set_output_size(1536);

        assert!(report.render().contains("Output size: 1.5 KB (1536 bytes)"));
    }

    #[test]
    fn reports_no_destination_when_none_added() {
        let report = Report::new();
        assert!(report.render().contains("Output destination: none"));
    }

    #[test]
    fn joins_multiple_destinations() {
        let mut report = Report::new();
        report.add_destination("stdout");
        report.add_destination("file: out.md");

        let rendered = report.render();
        assert!(rendered.contains("Output destination: stdout, file: out.md"));
    }

    #[test]
    fn reports_clipboard_status() {
        let mut report = Report::new();
        assert!(report.render().contains("Clipboard: not requested"));

        report.set_clipboard(ClipboardStatus::Success);
        assert!(report.render().contains("Clipboard: success"));

        report.set_clipboard(ClipboardStatus::Failed);
        assert!(report.render().contains("Clipboard: failed"));
    }

    #[test]
    fn print_if_enabled_skips_when_quiet_or_not_verbose() {
        // quiet wins over verbose
        let cfg = config(true, true);
        // The method returns without writing when suppressed; this only
        // verifies the guard does not panic and the report still renders.
        let report = Report::new();
        report.print_if_enabled(&cfg);
        assert!(report.render().contains("Collected files: 0"));
    }

    #[test]
    fn format_size_handles_all_units() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1023), "1023 B");
        assert_eq!(format_size(1024), "1.0 KB (1024 bytes)");
        assert_eq!(format_size(5 * 1024 * 1024), "5.0 MB (5242880 bytes)");
    }
}
