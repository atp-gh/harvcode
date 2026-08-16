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

    /// Create a temporary project directory that is cleaned up automatically.
    ///
    /// `tempfile::tempdir` handles both uniqueness and cleanup, so tests do
    /// not need to track or remove directories manually.
    fn temp_project() -> (tempfile::TempDir, PathBuf) {
        let base = tempfile::tempdir().unwrap();
        let project = base.path().join("project");
        fs::create_dir_all(&project).unwrap();
        (base, project)
    }

    #[test]
    fn collect_recursively_finds_regular_files() {
        let (_base, project) = temp_project();
        fs::create_dir_all(project.join("src")).unwrap();

        fs::write(project.join("main.rs"), "fn main() {}\n").unwrap();
        fs::write(project.join("src/lib.rs"), "pub fn run() {}\n").unwrap();

        let files = collect(&project, &Rules::default());

        assert!(files.iter().any(|path| path.ends_with("main.rs")));
        assert!(files.iter().any(|path| path.ends_with("src/lib.rs")));
    }

    #[test]
    fn collect_skips_hidden_directories() {
        let (_base, project) = temp_project();
        fs::create_dir_all(project.join(".git")).unwrap();

        fs::write(project.join(".git/config"), "secret\n").unwrap();
        fs::write(project.join("main.rs"), "fn main() {}\n").unwrap();

        let files = collect(&project, &Rules::default());

        assert!(files.iter().any(|path| path.ends_with("main.rs")));
        assert!(!files.iter().any(|path| path.ends_with("config")));
    }

    #[test]
    fn collect_skips_directories_matched_by_exclude_dir() {
        let (_base, project) = temp_project();
        fs::create_dir_all(project.join("target")).unwrap();

        fs::write(project.join("target/generated.rs"), "generated\n").unwrap();
        fs::write(project.join("main.rs"), "fn main() {}\n").unwrap();

        let rules = Rules {
            exclude_dirs: vec!["target".to_string()],
            ..Rules::default()
        };

        let files = collect(&project, &rules);

        assert!(files.iter().any(|path| path.ends_with("main.rs")));
        assert!(!files.iter().any(|path| path.ends_with("generated.rs")));
    }

    #[cfg(unix)]
    #[test]
    fn collect_does_not_follow_symlinked_directory() {
        use std::os::unix::fs::symlink;

        let (_base, project) = temp_project();
        let outside = _base.path().join("outside");
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
    }

    #[cfg(unix)]
    #[test]
    fn collect_does_not_include_symlinked_file() {
        use std::os::unix::fs::symlink;

        let (_base, project) = temp_project();
        let outside = _base.path().join("outside");
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
    }
}
