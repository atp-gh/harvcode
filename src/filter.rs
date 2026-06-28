use std::path::Path;

/// File extensions to exclude by default (binary / non-text).
///
/// These exclusions are always applied, even when `--include-ext` is used.
const SKIP_EXT: [&str; 16] = [
    "png", "jpg", "jpeg", "gif", "bmp", "ico", "mp3", "mp4", "avi", "exe", "dll", "so", "zip",
    "tar", "gz", "lock",
];

/// User-defined filtering rules from CLI arguments.
#[derive(Default)]
pub struct Rules {
    pub include_ext: Vec<String>,
    pub exclude_ext: Vec<String>,
    pub exclude_dirs: Vec<String>,
    pub exclude_files: Vec<String>,
}

/// Check if file or directory is hidden (starts with '.').
pub fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|n| n.starts_with('.'))
        .unwrap_or(false)
}

/// Check if a directory should be skipped during traversal.
///
/// Rules:
/// - Hidden directories are skipped
/// - Directories listed in `--exclude-dir` are skipped
pub fn should_skip_dir(path: &Path, rules: &Rules) -> bool {
    is_hidden(path) || matches_name(path, &rules.exclude_dirs)
}

/// Final file filter.
///
/// Rules:
/// - Skip hidden files
/// - Skip default binary/archive extensions
/// - Skip files listed in `--exclude-file`
/// - Skip extensions listed in `--exclude-ext`
/// - If `--include-ext` is provided, only include matching extensions
pub fn is_valid(path: &Path, rules: &Rules) -> bool {
    if is_hidden(path) {
        return false;
    }

    if matches_name(path, &rules.exclude_files) {
        return false;
    }

    let ext = match extension(path) {
        Some(ext) => ext,
        None => return rules.include_ext.is_empty(),
    };

    if SKIP_EXT.contains(&ext.as_str()) {
        return false;
    }

    if rules.exclude_ext.contains(&ext) {
        return false;
    }

    if !rules.include_ext.is_empty() && !rules.include_ext.contains(&ext) {
        return false;
    }

    true
}

/// Return lowercase file extension.
fn extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
}

/// Match file or directory name case-insensitively.
fn matches_name(path: &Path, names: &[String]) -> bool {
    let name = match path.file_name().and_then(|s| s.to_str()) {
        Some(name) => name.to_ascii_lowercase(),
        None => return false,
    };

    names.contains(&name)
}
#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create default filtering rules.
    ///
    /// Most tests start from the default behavior and then override
    /// only the specific rule being tested.
    fn rules() -> Rules {
        Rules::default()
    }

    #[test]
    fn hidden_file_is_invalid() {
        // Files whose names start with "." should not be included.
        // This prevents accidentally collecting files such as .env.
        assert!(!is_valid(Path::new(".env"), &rules()));
    }

    #[test]
    fn hidden_directory_should_be_skipped() {
        // Hidden directories should be skipped during traversal.
        // This includes directories like .git, .github, .vscode, etc.
        assert!(should_skip_dir(Path::new(".git"), &rules()));
    }

    #[test]
    fn default_binary_extensions_are_invalid() {
        // Binary/archive files are always skipped by default.
        // These exclusions are applied even if the user provides include rules.
        assert!(!is_valid(Path::new("image.png"), &rules()));
        assert!(!is_valid(Path::new("archive.zip"), &rules()));
        assert!(!is_valid(Path::new("Cargo.lock"), &rules()));
    }

    #[test]
    fn normal_text_files_are_valid() {
        // Common source/documentation files should be included by default.
        assert!(is_valid(Path::new("main.rs"), &rules()));
        assert!(is_valid(Path::new("README.md"), &rules()));
    }

    #[test]
    fn file_without_extension_is_valid_by_default() {
        // Extensionless files such as Makefile are treated as text by default.
        assert!(is_valid(Path::new("Makefile"), &rules()));
    }

    #[test]
    fn include_ext_allows_only_matching_extensions() {
        // When include_ext is set, only matching extensions should pass.
        let mut rules = Rules::default();
        rules.include_ext = vec!["rs".to_string()];

        assert!(is_valid(Path::new("main.rs"), &rules));
        assert!(!is_valid(Path::new("README.md"), &rules));
    }

    #[test]
    fn include_ext_rejects_extensionless_files() {
        // If include_ext is provided, files without extensions should be excluded.
        //
        // Example:
        // `harvcode --include-ext rs`
        // should not include "Makefile".
        let mut rules = Rules::default();
        rules.include_ext = vec!["rs".to_string()];

        assert!(!is_valid(Path::new("Makefile"), &rules));
    }

    #[test]
    fn include_ext_does_not_override_default_skip_ext() {
        // Default skip extensions are always applied.
        // Even if the user explicitly includes "png", binary files stay excluded.
        let mut rules = Rules::default();
        rules.include_ext = vec!["png".to_string()];

        assert!(!is_valid(Path::new("image.png"), &rules));
    }

    #[test]
    fn exclude_ext_rejects_matching_extension() {
        // exclude_ext should reject files with matching extensions.
        let mut rules = Rules::default();
        rules.exclude_ext = vec!["json".to_string()];

        assert!(!is_valid(Path::new("config.json"), &rules));
        assert!(is_valid(Path::new("main.rs"), &rules));
    }

    #[test]
    fn exclude_file_rejects_matching_filename_case_insensitively() {
        // exclude_file compares only the file name, not the whole path.
        // Matching is case-insensitive.
        let mut rules = Rules::default();
        rules.exclude_files = vec!["secret.rs".to_string()];

        assert!(!is_valid(Path::new("secret.rs"), &rules));
        assert!(!is_valid(Path::new("SECRET.RS"), &rules));
        assert!(is_valid(Path::new("main.rs"), &rules));
    }

    #[test]
    fn exclude_dir_rejects_matching_directory_case_insensitively() {
        // exclude_dir compares directory names case-insensitively.
        let mut rules = Rules::default();
        rules.exclude_dirs = vec!["target".to_string()];

        assert!(should_skip_dir(Path::new("target"), &rules));
        assert!(should_skip_dir(Path::new("TARGET"), &rules));
        assert!(!should_skip_dir(Path::new("src"), &rules));
    }

    #[test]
    fn exclude_file_matches_file_name_not_full_path() {
        // The rule should match the final file name.
        // This means "src/secret.rs" should be excluded when "secret.rs" is listed.
        let mut rules = Rules::default();
        rules.exclude_files = vec!["secret.rs".to_string()];

        assert!(!is_valid(Path::new("src/secret.rs"), &rules));
    }

    #[test]
    fn exclude_dir_matches_directory_name_not_full_path() {
        // The rule should match the final directory name.
        // This means "build/target" should be skipped when "target" is listed.
        let mut rules = Rules::default();
        rules.exclude_dirs = vec!["target".to_string()];

        assert!(should_skip_dir(Path::new("build/target"), &rules));
    }
}
