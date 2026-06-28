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
