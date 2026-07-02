use std::fs;
use std::path::{Path, PathBuf};

use crate::filter::Rules;

/// Recursively collect all regular files under a directory.
///
/// Security:
/// - Does not follow symbolic links.
/// - Skips symlinked files and symlinked directories.
/// - Directory filtering is applied during traversal to avoid unnecessary work.
pub fn collect(root: &Path, rules: &Rules) -> Vec<PathBuf> {
    let mut files = Vec::new();

    if super::filter::should_skip_dir(root, rules) {
        return files;
    }

    visit(root, rules, &mut files);
    files
}

/// Depth-first traversal.
///
/// Skips:
/// - hidden directories
/// - directories matched by `--exclude-dir`
/// - symbolic links
fn visit(dir: &Path, rules: &Rules, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => continue,
        };

        // Important: never follow symlinks.
        //
        // This prevents a repository from containing links such as:
        //   project/secrets -> /home/user/.ssh
        //   project/passwd  -> /etc/passwd
        //
        // and having harvcode collect files outside the requested tree.
        if file_type.is_symlink() {
            continue;
        }

        if file_type.is_dir() {
            if super::filter::should_skip_dir(&path, rules) {
                continue;
            }

            visit(&path, rules, out);
        } else if file_type.is_file() {
            out.push(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_test_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let dir = std::env::temp_dir().join(format!("harvcode-walker-test-{}-{}", name, unique));

        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[cfg(unix)]
    #[test]
    fn collect_does_not_follow_symlinked_directory() {
        use std::os::unix::fs::symlink;

        let base = temp_test_dir("symlink-dir");
        let project = base.join("project");
        let outside = base.join("outside");

        fs::create_dir_all(&project).unwrap();
        fs::create_dir_all(&outside).unwrap();

        let secret = outside.join("secret.rs");
        fs::write(&secret, "secret").unwrap();

        let link = project.join("linked-outside");
        symlink(&outside, &link).unwrap();

        let files = collect(&project, &Rules::default());

        assert!(
            !files.iter().any(|path| path.ends_with("secret.rs")),
            "collector should not follow symlinked directories"
        );

        let _ = fs::remove_dir_all(base);
    }

    #[cfg(unix)]
    #[test]
    fn collect_does_not_include_symlinked_file() {
        use std::os::unix::fs::symlink;

        let base = temp_test_dir("symlink-file");
        let project = base.join("project");
        let outside = base.join("outside");

        fs::create_dir_all(&project).unwrap();
        fs::create_dir_all(&outside).unwrap();

        let secret = outside.join("secret.rs");
        fs::write(&secret, "secret").unwrap();

        let link = project.join("secret_link.rs");
        symlink(&secret, &link).unwrap();

        let files = collect(&project, &Rules::default());

        assert!(
            !files.iter().any(|path| path == &link),
            "collector should not include symlinked files"
        );

        assert!(
            !files.iter().any(|path| path.ends_with("secret.rs")),
            "collector should not collect symlink target"
        );

        let _ = fs::remove_dir_all(base);
    }
}
