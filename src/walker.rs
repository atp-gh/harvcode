use std::fs;
use std::path::{Path, PathBuf};

/// Recursively collect all files under a directory
pub fn collect(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    visit(root, &mut files);
    files
}

/// Depth-first traversal
/// Skips hidden files and directories
fn visit(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if super::filter::is_hidden(&path) {
                continue;
            }

            if path.is_dir() {
                visit(&path, out);
            } else {
                out.push(path);
            }
        }
    }
}
