use std::fs;
use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// A temporary project directory for CLI integration tests.
///
/// `tempfile::TempDir` handles both uniqueness and cleanup automatically,
/// including when an assertion panics, so tests never leak directories.
struct TestDir {
    dir: TempDir,
}

impl TestDir {
    /// Create a new unique temporary test directory.
    fn new(name: &str) -> Self {
        let dir = tempfile::Builder::new()
            .prefix(&format!("harvcode-test-{}-", name))
            .tempdir()
            .unwrap();

        Self { dir }
    }

    /// Return the root path of this test directory.
    fn path(&self) -> &Path {
        self.dir.path()
    }

    /// Write a file under the test directory.
    ///
    /// Parent directories are created automatically so tests can use paths like:
    ///
    /// - `src/main.rs`
    /// - `.git/config`
    /// - `target/generated.rs`
    fn write(&self, relative: &str, content: &str) {
        let path = self.path().join(relative);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        fs::write(path, content).unwrap();
    }

    /// Create a directory under the test directory.
    fn mkdir(&self, relative: &str) {
        fs::create_dir_all(self.path().join(relative)).unwrap();
    }
}

/// Build a command running the compiled harvcode binary.
fn harvcode() -> Command {
    Command::cargo_bin("harvcode").unwrap()
}

/// Run harvcode inside a specific working directory with the provided args.
fn run_in(dir: &Path, args: &[&str]) -> assert_cmd::assert::Assert {
    harvcode().current_dir(dir).args(args).assert()
}

#[test]
fn stdout_outputs_collected_files() {
    // This test exercises the deterministic `--stdout` output mode.
    // It verifies that multiple files under the current directory are collected
    // and formatted into the final stdout output.
    let dir = TestDir::new("stdout-outputs-collected-files");

    dir.write("src/main.rs", "fn main() {}\n");
    dir.write("README.md", "# Test\n");

    run_in(dir.path(), &["--stdout"])
        .success()
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("fn main() {}"))
        .stdout(predicate::str::contains("README.md"))
        .stdout(predicate::str::contains("# Test"));
}

#[test]
fn stdout_skips_hidden_files() {
    // Hidden files should be skipped.
    //
    // This is especially important for files like `.env`,
    // which may contain secrets and should not be copied into AI context.
    let dir = TestDir::new("stdout-skips-hidden-files");

    dir.write(".env", "SECRET=value\n");
    dir.write("main.rs", "fn main() {}\n");

    run_in(dir.path(), &["--stdout"])
        .success()
        .stdout(predicate::str::contains("fn main() {}"))
        .stdout(predicate::str::contains("SECRET=value").not());
}

#[test]
fn stdout_skips_hidden_directories() {
    // Hidden directories should be skipped during traversal.
    //
    // This prevents collecting contents from directories like:
    // - .git
    // - .github
    // - .vscode
    let dir = TestDir::new("stdout-skips-hidden-directories");

    dir.write(".git/config", "hidden git config\n");
    dir.write("main.rs", "fn main() {}\n");

    run_in(dir.path(), &["--stdout"])
        .success()
        .stdout(predicate::str::contains("fn main() {}"))
        .stdout(predicate::str::contains("hidden git config").not());
}

#[test]
fn stdout_skips_binary_extensions() {
    // Binary and archive extensions are excluded by default.
    //
    // The test writes plain text into fake `.png` and `.zip` files,
    // but filtering is based on extension, not file content.
    let dir = TestDir::new("stdout-skips-binary-extensions");

    dir.write("main.rs", "fn main() {}\n");
    dir.write("image.png", "fake image content\n");
    dir.write("archive.zip", "fake zip content\n");

    run_in(dir.path(), &["--stdout"])
        .success()
        .stdout(predicate::str::contains("fn main() {}"))
        .stdout(predicate::str::contains("fake image content").not())
        .stdout(predicate::str::contains("fake zip content").not());
}

#[test]
fn output_writes_to_file() {
    // `--output <file>` should write the generated context to a file.
    //
    // This mode is deterministic and does not depend on system clipboard tools,
    // making it appropriate for automated tests and CI.
    let dir = TestDir::new("output-writes-to-file");

    dir.write("main.rs", "fn main() {}\n");

    let output_path = dir.path().join("context.md");
    let output_path_string = output_path.to_string_lossy().to_string();

    run_in(dir.path(), &["--output", &output_path_string])
        .success()
        .stderr(predicate::str::contains("Wrote output to"));

    let content = fs::read_to_string(output_path).unwrap();

    assert!(content.contains("main.rs"));
    assert!(content.contains("fn main() {}"));
    assert!(content.contains("```"));
}

#[test]
fn stdout_and_output_can_be_combined() {
    // v0.4.0 allows output modes to be combined.
    //
    // This test verifies that:
    // - stdout receives the generated content
    // - the output file also receives the same generated content
    let dir = TestDir::new("stdout-and-output-can-be-combined");

    dir.write("main.rs", "fn main() {}\n");

    let output_path = dir.path().join("context.md");
    let output_path_string = output_path.to_string_lossy().to_string();

    run_in(dir.path(), &["--stdout", "--output", &output_path_string])
        .success()
        .stdout(predicate::str::contains("fn main() {}"))
        .stderr(predicate::str::contains("Wrote output to"));

    let content = fs::read_to_string(output_path).unwrap();

    assert!(content.contains("fn main() {}"));
}

#[test]
fn include_ext_filters_stdout_output() {
    // `--include-ext rs` should include only `.rs` files.
    //
    // README.md is present but should not appear in stdout.
    let dir = TestDir::new("include-ext-filters-stdout-output");

    dir.write("main.rs", "fn main() {}\n");
    dir.write("README.md", "# Readme\n");

    run_in(dir.path(), &["--stdout", "--include-ext", "rs"])
        .success()
        .stdout(predicate::str::contains("fn main() {}"))
        .stdout(predicate::str::contains("# Readme").not());
}

#[test]
fn exclude_ext_filters_stdout_output() {
    // `--exclude-ext json` should exclude JSON files from output.
    let dir = TestDir::new("exclude-ext-filters-stdout-output");

    dir.write("main.rs", "fn main() {}\n");
    dir.write("config.json", "{\"name\":\"test\"}\n");

    run_in(dir.path(), &["--stdout", "--exclude-ext", "json"])
        .success()
        .stdout(predicate::str::contains("fn main() {}"))
        .stdout(predicate::str::contains("\"name\"").not());
}

#[test]
fn exclude_dir_filters_stdout_output() {
    // `--exclude-dir target` should prevent traversal into the target directory.
    let dir = TestDir::new("exclude-dir-filters-stdout-output");

    dir.mkdir("src");
    dir.mkdir("target");

    dir.write("src/main.rs", "fn main() {}\n");
    dir.write("target/generated.rs", "generated\n");

    run_in(dir.path(), &["--stdout", "--exclude-dir", "target"])
        .success()
        .stdout(predicate::str::contains("fn main() {}"))
        .stdout(predicate::str::contains("generated").not());
}

#[test]
fn exclude_file_filters_stdout_output() {
    // `--exclude-file secret.rs` should exclude that specific file name.
    //
    // This is useful for removing sensitive or irrelevant files from context.
    let dir = TestDir::new("exclude-file-filters-stdout-output");

    dir.write("main.rs", "fn main() {}\n");
    dir.write("secret.rs", "secret\n");

    run_in(dir.path(), &["--stdout", "--exclude-file", "secret.rs"])
        .success()
        .stdout(predicate::str::contains("fn main() {}"))
        .stdout(predicate::str::contains("secret").not());
}

#[test]
fn max_file_size_skips_oversized_files() {
    // `--max-file-size` should skip files larger than the limit without
    // reading them, so an oversized file cannot exhaust memory.
    let dir = TestDir::new("max-file-size-skips-oversized-files");

    dir.write("main.rs", "fn main() {}\n");
    dir.write("big.txt", &"x".repeat(2048));

    run_in(dir.path(), &["--stdout", "--max-file-size", "1024"])
        .success()
        .stdout(predicate::str::contains("fn main() {}"))
        .stdout(predicate::str::contains("xxxx").not());
}

#[test]
fn max_file_size_zero_disables_limit() {
    // A limit of 0 means "no limit", so the file is collected normally.
    let dir = TestDir::new("max-file-size-zero-disables-limit");

    dir.write("big.txt", &"x".repeat(2048));

    run_in(dir.path(), &["--stdout", "--max-file-size", "0"])
        .success()
        .stdout(predicate::str::contains("xxxx"));
}

#[test]
fn max_total_size_aborts_with_error() {
    // When the total output exceeds `--max-total-size`, harvcode should stop
    // and exit with the output-failure code instead of writing everything.
    let dir = TestDir::new("max-total-size-aborts-with-error");

    dir.write("main.rs", "fn main() {}\n");
    dir.write("notes.txt", &"y".repeat(2048));

    run_in(dir.path(), &["--stdout", "--max-total-size", "1024"])
        .failure()
        .code(3)
        .stderr(predicate::str::contains("Output size limit exceeded"));
}

#[test]
fn max_total_size_zero_disables_limit() {
    // A total limit of 0 means "no limit", so large combined output is fine.
    let dir = TestDir::new("max-total-size-zero-disables-limit");

    dir.write("notes.txt", &"y".repeat(2048));

    run_in(dir.path(), &["--stdout", "--max-total-size", "0"])
        .success()
        .stdout(predicate::str::contains("yyyy"));
}

#[test]
fn list_mode_prints_paths_only() {
    // `--list` should print only the collected file paths, never contents.
    let dir = TestDir::new("list-mode-prints-paths-only");

    dir.write("src/main.rs", "fn main() {}\n");
    dir.write("README.md", "# Readme\n");

    run_in(dir.path(), &["--list"])
        .success()
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("README.md"))
        .stdout(predicate::str::contains("fn main() {}").not())
        .stdout(predicate::str::contains("# Readme").not());
}

#[test]
fn list_mode_respects_filters() {
    // `--list` applies the same filters as content output.
    let dir = TestDir::new("list-mode-respects-filters");

    dir.write("main.rs", "fn main() {}\n");
    dir.write("README.md", "# Readme\n");
    dir.write(".env", "SECRET=value\n");

    run_in(dir.path(), &["--list", "--include-ext", "rs"])
        .success()
        .stdout(predicate::str::contains("main.rs"))
        .stdout(predicate::str::contains("README.md").not())
        .stdout(predicate::str::contains("SECRET=value").not());
}

#[test]
fn list_mode_uses_stable_ordering() {
    // List output should be sorted deterministically regardless of the
    // filesystem's directory iteration order.
    let dir = TestDir::new("list-mode-uses-stable-ordering");

    dir.write("c.txt", "c\n");
    dir.write("a.txt", "a\n");
    dir.write("b.txt", "b\n");

    let assert = run_in(dir.path(), &["--list"]).success();
    let out = String::from_utf8_lossy(&assert.get_output().stdout).to_string();

    // Paths are printed relative to the working directory ("./a.txt"), so
    // compare only the file names in order.
    let names: Vec<&str> = out
        .lines()
        .map(|line| line.trim_start_matches("./"))
        .collect();
    assert_eq!(names, vec!["a.txt", "b.txt", "c.txt"]);
}

#[test]
fn list_mode_ignores_output_file() {
    // In list mode, `--output <file>` must not create or modify the file.
    let dir = TestDir::new("list-mode-ignores-output-file");

    dir.write("main.rs", "fn main() {}\n");

    let output_path = dir.path().join("context.md");
    let output_path_string = output_path.to_string_lossy().to_string();

    run_in(dir.path(), &["--list", "--output", &output_path_string]).success();

    assert!(!output_path.exists());
}

#[test]
fn verbose_prints_execution_report() {
    // `--verbose` prints a report with counts and destinations on stderr.
    let dir = TestDir::new("verbose-prints-execution-report");

    dir.write("main.rs", "fn main() {}\n");

    run_in(dir.path(), &["--stdout", "--verbose"])
        .success()
        .stderr(predicate::str::contains("Collected files: 1"))
        .stderr(predicate::str::contains("Output destination: stdout"));
}

#[test]
fn quiet_suppresses_status_messages() {
    // `--quiet` suppresses non-error status output such as "Wrote output to".
    let dir = TestDir::new("quiet-suppresses-status-messages");

    dir.write("main.rs", "fn main() {}\n");

    let output_path = dir.path().join("context.md");
    let output_path_string = output_path.to_string_lossy().to_string();

    run_in(dir.path(), &["--quiet", "--output", &output_path_string])
        .success()
        .stderr(predicate::str::contains("Wrote output to").not());
}

#[test]
fn help_prints_usage() {
    // `--help` should print usage and exit successfully.
    harvcode()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("--max-file-size"));
}

#[test]
fn version_prints_version() {
    // `--version` should print the version and exit successfully.
    harvcode()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("harvcode "));
}

#[test]
fn unknown_option_exits_with_error() {
    // Unknown options should produce a CLI error and exit with code 1.
    //
    // This protects users from typos silently being interpreted as paths.
    harvcode()
        .arg("--unknown")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Unknown option"));
}

#[test]
fn missing_output_value_exits_with_error() {
    // `--output` requires a file path.
    //
    // Missing the value should fail during argument parsing and exit with code 1.
    harvcode()
        .arg("--output")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Missing value for --output"));
}

#[test]
fn invalid_size_value_exits_with_error() {
    // A non-numeric size value should be a CLI error (exit code 1).
    harvcode()
        .args(["--max-file-size", "abc"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Invalid size value"));
}
