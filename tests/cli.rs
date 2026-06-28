use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

/// A tiny zero-dependency temporary directory helper.
///
/// This avoids adding `tempfile` as a dev-dependency.
/// Each test gets a unique directory under the system temp directory.
///
/// The directory name includes:
/// - test name
/// - current process id
/// - current timestamp in nanoseconds
///
/// This makes collisions very unlikely, even when tests run in parallel.
struct TestDir {
    path: PathBuf,
}

impl TestDir {
    /// Create a new unique temporary test directory.
    fn new(name: &str) -> Self {
        let mut path = std::env::temp_dir();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!(
            "harvcode-test-{}-{}-{}",
            name,
            std::process::id(),
            now
        ));

        fs::create_dir_all(&path).unwrap();

        Self { path }
    }

    /// Return the root path of this test directory.
    fn path(&self) -> &Path {
        &self.path
    }

    /// Write a file under the test directory.
    ///
    /// Parent directories are created automatically so tests can use paths like:
    ///
    /// - `src/main.rs`
    /// - `.git/config`
    /// - `target/generated.rs`
    fn write(&self, relative: &str, content: &str) {
        let path = self.path.join(relative);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        fs::write(path, content).unwrap();
    }

    /// Create a directory under the test directory.
    fn mkdir(&self, relative: &str) {
        fs::create_dir_all(self.path.join(relative)).unwrap();
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        // Best-effort cleanup.
        //
        // We intentionally ignore cleanup errors here because a failed cleanup
        // should not hide the real test result.
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Create a command pointing to the compiled harvcode binary.
///
/// `CARGO_BIN_EXE_harvcode` is provided by Cargo for integration tests.
/// This lets us test the real CLI binary without extra crates like assert_cmd.
fn harvcode() -> Command {
    Command::new(env!("CARGO_BIN_EXE_harvcode"))
}

/// Run harvcode inside a specific working directory with the provided args.
fn run_in(dir: &Path, args: &[&str]) -> Output {
    harvcode().current_dir(dir).args(args).output().unwrap()
}

/// Decode stdout as UTF-8 text.
///
/// The tool produces text output, so lossy UTF-8 decoding is acceptable here.
/// If invalid bytes ever appear, the test will still produce readable diagnostics.
fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Decode stderr as UTF-8 text.
fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

#[test]
fn stdout_outputs_collected_files() {
    // This test exercises the deterministic `--stdout` output mode.
    // It verifies that multiple files under the current directory are collected
    // and formatted into the final stdout output.
    let dir = TestDir::new("stdout-outputs-collected-files");

    dir.write("src/main.rs", "fn main() {}\n");
    dir.write("README.md", "# Test\n");

    let output = run_in(dir.path(), &["--stdout"]);

    assert!(output.status.success());

    let out = stdout(&output);

    assert!(out.contains("src/main.rs"));
    assert!(out.contains("fn main() {}"));
    assert!(out.contains("README.md"));
    assert!(out.contains("# Test"));
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

    let output = run_in(dir.path(), &["--stdout"]);

    assert!(output.status.success());

    let out = stdout(&output);

    assert!(out.contains("fn main() {}"));
    assert!(!out.contains("SECRET=value"));
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

    let output = run_in(dir.path(), &["--stdout"]);

    assert!(output.status.success());

    let out = stdout(&output);

    assert!(out.contains("fn main() {}"));
    assert!(!out.contains("hidden git config"));
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

    let output = run_in(dir.path(), &["--stdout"]);

    assert!(output.status.success());

    let out = stdout(&output);

    assert!(out.contains("fn main() {}"));
    assert!(!out.contains("fake image content"));
    assert!(!out.contains("fake zip content"));
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

    let output = run_in(dir.path(), &["--output", &output_path_string]);

    assert!(output.status.success());

    let err = stderr(&output);
    assert!(err.contains("Wrote output to"));

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

    let output = run_in(dir.path(), &["--stdout", "--output", &output_path_string]);

    assert!(output.status.success());

    let out = stdout(&output);
    let err = stderr(&output);

    assert!(out.contains("fn main() {}"));
    assert!(err.contains("Wrote output to"));

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

    let output = run_in(dir.path(), &["--stdout", "--include-ext", "rs"]);

    assert!(output.status.success());

    let out = stdout(&output);

    assert!(out.contains("fn main() {}"));
    assert!(!out.contains("# Readme"));
}

#[test]
fn exclude_ext_filters_stdout_output() {
    // `--exclude-ext json` should exclude JSON files from output.
    let dir = TestDir::new("exclude-ext-filters-stdout-output");

    dir.write("main.rs", "fn main() {}\n");
    dir.write("config.json", "{\"name\":\"test\"}\n");

    let output = run_in(dir.path(), &["--stdout", "--exclude-ext", "json"]);

    assert!(output.status.success());

    let out = stdout(&output);

    assert!(out.contains("fn main() {}"));
    assert!(!out.contains("\"name\""));
}

#[test]
fn exclude_dir_filters_stdout_output() {
    // `--exclude-dir target` should prevent traversal into the target directory.
    let dir = TestDir::new("exclude-dir-filters-stdout-output");

    dir.mkdir("src");
    dir.mkdir("target");

    dir.write("src/main.rs", "fn main() {}\n");
    dir.write("target/generated.rs", "generated\n");

    let output = run_in(dir.path(), &["--stdout", "--exclude-dir", "target"]);

    assert!(output.status.success());

    let out = stdout(&output);

    assert!(out.contains("fn main() {}"));
    assert!(!out.contains("generated"));
}

#[test]
fn exclude_file_filters_stdout_output() {
    // `--exclude-file secret.rs` should exclude that specific file name.
    //
    // This is useful for removing sensitive or irrelevant files from context.
    let dir = TestDir::new("exclude-file-filters-stdout-output");

    dir.write("main.rs", "fn main() {}\n");
    dir.write("secret.rs", "secret\n");

    let output = run_in(dir.path(), &["--stdout", "--exclude-file", "secret.rs"]);

    assert!(output.status.success());

    let out = stdout(&output);

    assert!(out.contains("fn main() {}"));
    assert!(!out.contains("secret"));
}

#[test]
fn unknown_option_exits_with_error() {
    // Unknown options should produce a CLI error and exit with code 1.
    //
    // This protects users from typos silently being interpreted as paths.
    let output = harvcode().arg("--unknown").output().unwrap();

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));

    let err = stderr(&output);

    assert!(err.contains("Unknown option"));
}

#[test]
fn missing_output_value_exits_with_error() {
    // `--output` requires a file path.
    //
    // Missing the value should fail during argument parsing and exit with code 1.
    let output = harvcode().arg("--output").output().unwrap();

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));

    let err = stderr(&output);

    assert!(err.contains("Missing value for --output"));
}
