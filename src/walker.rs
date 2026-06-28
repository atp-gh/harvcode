use std::fs;
use std::path::{Path, PathBuf};

use crate::filter::Rules;

/// Recursively collect all files under a directory.
///
/// Directory filtering is applied during traversal to avoid unnecessary work.
pub fn collect(root: &Path, rules: &Rules) -> Vec<PathBuf> {
    let mut files = Vec::new();
    visit(root, rules, &mut files);
    files
}

/// Depth-first traversal.
///
/// Skips:
/// - hidden directories
/// - directories matched by `--exclude-dir`
fn visit(dir: &Path, rules: &Rules, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                if super::filter::should_skip_dir(&path, rules) {
                    continue;
                }

                visit(&path, rules, out);
            } else {
                out.push(path);
            }
        }
    }
}
