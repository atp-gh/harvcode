use std::path::Path;

/// File extensions to exclude (binary / non-text)
const SKIP_EXT: [&str; 16] = [
    "png", "jpg", "jpeg", "gif", "bmp", "ico", "mp3", "mp4", "avi", "exe", "dll", "so", "zip",
    "tar", "gz", "lock",
];

/// Check if file or directory is hidden (starts with '.')
pub fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|n| n.starts_with('.'))
        .unwrap_or(false)
}

/// Heuristic text file detection based on extension
pub fn is_text_file(path: &Path) -> bool {
    match path.extension().and_then(|s| s.to_str()) {
        Some(ext) => !SKIP_EXT.contains(&ext),
        None => true,
    }
}

/// Final filter rule:
/// - Not hidden
/// - Treated as text file
pub fn is_valid(path: &Path) -> bool {
    !is_hidden(path) && is_text_file(path)
}
