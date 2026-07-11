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

        eprintln!("Collected files: {}", self.collected_files);
        eprintln!("Skipped files: {}", self.skipped_files);
        eprintln!("Output size: {}", format_size(self.output_size));

        if self.destinations.is_empty() {
            eprintln!("Output destination: none");
        } else {
            eprintln!("Output destination: {}", self.destinations.join(", "));
        }

        match self.clipboard {
            ClipboardStatus::NotRequested => eprintln!("Clipboard: not requested"),
            ClipboardStatus::Success => eprintln!("Clipboard: success"),
            ClipboardStatus::Failed => eprintln!("Clipboard: failed"),
        }
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
