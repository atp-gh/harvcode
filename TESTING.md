# Testing

This document describes the testing strategy for **harvcode**.

harvcode uses Rust's built-in test framework and the standard library only. The test suite intentionally avoids additional test dependencies so that it remains lightweight, portable, and consistent with the project's dependency-conscious design.

## Goals

The test suite is designed to verify the following behavior:

- CLI argument parsing and validation
- Default and explicit output mode selection
- File and directory filtering
- Markdown output formatting
- Recursive file collection
- Symbolic-link safety
- Deterministic stdout and file output
- CLI error handling
- List mode configuration

Tests focus on behavior that can be reproduced reliably across local development environments, CI systems, containers, and supported operating systems.

## Running Tests

Run the complete test suite:

```bash
cargo test
```

Run only the CLI integration tests:

```bash
cargo test --test cli
```

Run a specific test by name:

```bash
cargo test stdout_outputs_collected_files
```

Show output captured during tests:

```bash
cargo test -- --nocapture
```

Run tests in a single thread when debugging filesystem-related behavior:

```bash
cargo test -- --test-threads=1
```

Check that the project builds without running tests:

```bash
cargo check
```

Before submitting a change, the recommended verification sequence is:

```bash
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets
```

## Test Organization

Tests are divided into unit tests and integration tests.

```text
src/
├── args.rs        # CLI argument parsing tests
├── filter.rs      # File and directory filtering tests
├── formatter.rs   # Markdown formatting tests
├── main.rs        # Root expansion and symlink safety tests
├── walker.rs      # Recursive traversal and symlink safety tests
└── ...

tests/
└── cli.rs         # End-to-end CLI integration tests
```

### Unit Tests

Unit tests live next to the implementation they verify, inside `#[cfg(test)]` modules.

They are used for logic that can be tested directly and deterministically without launching the compiled CLI binary.

### Integration Tests

Integration tests are located in:

```text
tests/cli.rs
```

These tests execute the compiled `harvcode` binary through `std::process::Command` and verify behavior from the user's perspective.

Cargo provides the binary path through:

```rust
env!("CARGO_BIN_EXE_harvcode")
```

This allows the test suite to exercise the real command-line interface without dependencies such as `assert_cmd`.

## Zero-Dependency Test Design

The test suite intentionally does not use additional testing crates such as:

- `assert_cmd`
- `predicates`
- `tempfile`

Instead, it uses the Rust standard library:

- `std::process::Command` to execute the compiled CLI
- `std::env::temp_dir` to create temporary test directories
- `std::fs` to create files and inspect output
- `assert!` and `assert_eq!` for assertions
- `Drop` implementations for best-effort temporary directory cleanup

This approach keeps the test setup simple and avoids adding dependencies used only during testing.

Each integration test creates a unique temporary directory. Directory names include values such as:

- The test name
- The current process ID
- A nanosecond timestamp

This makes collisions unlikely when tests run in parallel.

Temporary directories are removed when their `TestDir` value is dropped. Cleanup is best-effort so that a cleanup failure does not hide the actual test result.

## Unit Test Coverage

### Argument Parsing

Argument parsing tests are located in:

```text
src/args.rs
```

They verify:

- No path defaults to the current directory
- No explicit output option defaults to clipboard output
- `--stdout` enables stdout output
- `--clipboard` enables explicit clipboard output
- `--output <file>` enables file output
- `--output=<file>` is accepted
- Multiple output destinations can be combined
- `--list` enables list mode
- List mode does not enable implicit clipboard output
- Explicit output flags are still recorded when used with `--list`
- `--picker sk` and `--picker fzf` enable picker mode
- `--picker=<value>` is accepted
- `--quiet` and `--verbose` are parsed
- Include and exclude lists are normalized to lowercase
- Unknown flags return an error
- Unsupported picker backends return an error
- Missing flag values return an error
- Empty `--output=` and `--picker=` values return an error

The parser records configuration only. Runtime precedence rules, such as list mode overriding normal output destinations, are handled elsewhere.

### File Filtering

Filtering tests are located in:

```text
src/filter.rs
```

They verify:

- Hidden files are rejected
- Hidden directories are skipped
- Known binary and archive extensions are rejected
- Lock files are rejected through the default extension skip list
- Normal source and documentation files are accepted
- Extensionless files are accepted by default
- `--include-ext` allows only matching extensions
- `--include-ext` rejects extensionless files
- Default skipped extensions cannot be re-enabled through `--include-ext`
- `--exclude-ext` rejects matching extensions
- `--exclude-file` matches the final file name
- `--exclude-dir` matches the final directory name
- File and directory name matching is case-insensitive

Filtering is performed by extension and file name. The current implementation does not inspect file contents to determine whether a file is binary.

### Markdown Formatting

Formatting tests are located in:

```text
src/formatter.rs
```

They verify:

- File content is wrapped in a Markdown code block
- The file path is included in the opening fence
- Existing content is preserved
- A missing trailing newline is added before the closing fence
- Existing trailing newlines are handled correctly
- Every formatted block ends with a blank line

These tests ensure that multiple formatted files can be concatenated into one readable context document.

### Root Expansion and Symlink Safety

Root expansion tests are located in:

```text
src/main.rs
```

On Unix platforms, they verify that explicitly supplied symbolic-link roots are not followed.

Covered cases include:

- A symbolic link to a regular file
- A symbolic link to a directory

These tests protect against collecting data outside the paths explicitly intended by the user.

The tests are guarded with:

```rust
#[cfg(unix)]
```

because symbolic-link creation uses Unix-specific standard library APIs.

### Recursive Traversal and Symlink Safety

Walker tests are located in:

```text
src/walker.rs
```

On Unix platforms, they verify that recursive traversal:

- Does not follow symbolic links to directories
- Does not include symbolic links to files
- Does not collect the target of a symbolic link

The walker uses directory entry file types to reject symlinks before recursion or file collection.

## Integration Test Coverage

CLI integration tests are located in:

```text
tests/cli.rs
```

They execute the real binary inside isolated temporary project directories.

### Stdout Output

Integration tests verify that `--stdout`:

- Collects files recursively
- Formats collected files into Markdown blocks
- Includes file paths and file contents
- Skips hidden files
- Skips hidden directories
- Skips default binary and archive extensions

### File Output

Integration tests verify that `--output <file>`:

- Creates the requested output file
- Writes formatted context into the file
- Reports the destination on stderr
- Exits successfully

### Combined Output Destinations

Integration tests verify that:

```bash
harvcode --stdout --output context.md
```

writes the generated context to both stdout and the requested file.

Output modes are not mutually exclusive.

### Filtering Options

Integration tests verify the runtime behavior of:

```bash
--include-ext
--exclude-ext
--exclude-dir
--exclude-file
```

The tests create both matching and non-matching files and inspect stdout to confirm that only valid files are included.

### CLI Errors

Integration tests verify that:

- Unknown options exit with code `1`
- Missing values for `--output` exit with code `1`
- Error messages are written to stderr

## Output Mode Testing Strategy

harvcode supports the following output-related modes:

```bash
harvcode --clipboard
harvcode --stdout
harvcode --output context.md
harvcode --list
```

Output destinations can be combined, except that list mode takes runtime precedence and writes only the final file list to stdout.

The automated integration suite primarily uses deterministic output destinations:

```bash
harvcode --stdout
harvcode --output context.md
harvcode --stdout --output context.md
```

These modes do not depend on external platform tools and are therefore suitable for CI.

Default clipboard selection is tested at the argument-parsing level rather than through direct clipboard integration.

## List Mode

List mode is enabled with:

```bash
harvcode --list
```

Its intended runtime behavior is:

- Print file paths only
- Print one valid path per line
- Apply the normal file filters
- Use stable path ordering
- Avoid reading file contents
- Avoid copying data to the clipboard
- Ignore normal output destinations such as `--output`

At the argument-parsing level, automated tests currently verify that:

- `--list` enables list mode
- List mode disables implicit clipboard selection
- Explicit output flags are still recorded by the parser
- Runtime code remains responsible for applying list mode precedence

The end-to-end runtime behavior of list mode is not yet covered by CLI integration tests.

Manual verification can be performed with:

```bash
harvcode --list
harvcode src --list
harvcode --include-ext rs,md --list
harvcode --exclude-dir target,node_modules --list
```

List mode precedence can be checked with:

```bash
harvcode --list --stdout
harvcode --list --clipboard
harvcode --list --output context.md
```

In each case, the expected behavior is:

- Only file paths are written to stdout
- File contents are not written
- Clipboard tools are not invoked
- The file passed to `--output` is not created or modified

## Clipboard Testing Policy

Clipboard support depends on external platform-specific commands, including:

- `wl-copy`
- `xclip`
- `pbcopy`
- `clip`

These commands are not available consistently across operating systems and CI environments.

For example:

- A Linux CI container may not provide `wl-copy` or `xclip`
- macOS commonly provides `pbcopy`
- Windows commonly provides `clip`
- Headless environments may not have a usable clipboard session

Because of this, clipboard execution is not tested directly through the current integration suite.

Instead:

- Unit tests verify that clipboard is the default output mode
- Unit tests distinguish implicit clipboard behavior from explicit `--clipboard`
- Unit tests verify that list mode does not enable implicit clipboard output
- Integration tests use stdout and file output for deterministic assertions

Clipboard behavior can be checked manually with:

```bash
harvcode
harvcode --clipboard
harvcode --clipboard --output context.md
```

Expected behavior:

- With implicit clipboard mode, clipboard failure falls back to stdout
- With explicit `--clipboard`, clipboard failure is treated as an output error
- Clipboard output can be combined with file output

If stronger automated clipboard coverage is needed, the clipboard implementation should first be moved behind an injectable or mockable abstraction.

## Security-Oriented Tests

harvcode processes repository contents that may be untrusted. The test suite therefore includes checks for behavior that could otherwise expose files outside the requested tree.

Current security-focused coverage includes:

- Hidden files are skipped
- Hidden directories are skipped
- Common binary and archive formats are skipped
- Explicit symbolic-link file roots are rejected
- Explicit symbolic-link directory roots are rejected
- Symbolic links encountered during traversal are skipped
- Files are checked again before being read

The second metadata check before reading reduces the window in which a traversed file could be replaced by a symbolic link.

These checks reduce risk but should not be treated as a complete sandbox. harvcode still runs with the permissions of the current user.

## Exit Codes

The application currently uses the following exit codes:

| Code | Meaning |
| ---: | --- |
| `0` | Successful execution |
| `1` | CLI argument parsing error |
| `2` | Picker unavailable or selection cancelled |
| `3` | Output or list-writing failure |

The current integration suite directly verifies:

- Successful execution
- Exit code `1` for invalid CLI arguments

Exit codes `2` and `3` are implemented but are not yet covered comprehensively by integration tests.

## Current Coverage Summary

### Automatically Tested

- CLI defaults and argument parsing
- Output flag parsing
- Combined output modes
- Picker option parsing
- Quiet and verbose flag parsing
- Include and exclude rule parsing
- Hidden file and directory filtering
- Binary and archive extension filtering
- Extensionless file behavior
- Case-insensitive file and directory exclusion
- Markdown formatting
- Stdout output
- File output
- Combined stdout and file output
- Runtime filtering through the CLI
- Unknown option handling
- Missing output value handling
- Symlink-safe root expansion on Unix
- Symlink-safe recursive traversal on Unix

### Manually Tested or Not Yet Automated

- End-to-end list mode behavior
- Clipboard command execution
- Picker interaction and cancellation
- Exit code `2`
- Output failure exit code `3`
- Broken pipe behavior
- Permission-denied input files
- Permission-denied output destinations
- Deterministic ordering across larger directory trees
- Platform-specific behavior outside Unix symlink tests

## Recommended Future Tests

The following additions would improve coverage while preserving the zero-dependency test design.

### List Mode Integration Tests

Add integration tests verifying that:

- `--list` prints paths only
- `--list` does not print file contents
- `--list` respects `--include-ext`
- `--list` respects `--exclude-dir`
- `--list --output context.md` does not create the output file
- `--list --clipboard` does not require clipboard tools
- List output is sorted deterministically

### Output Failure Tests

Where supported by the test environment, add tests for:

- Writing to an invalid output path
- Writing to a directory instead of a file
- Writing to a destination without permission
- Returning exit code `3` after an output failure

Permission-based tests should be designed carefully because behavior can differ across operating systems and when tests run with elevated privileges.

### Picker Tests

Picker behavior currently depends on external commands and interactive input.

To make it testable, consider separating:

- Picker discovery
- Process execution
- Selected-path parsing

This would allow deterministic unit tests without launching `sk` or `fzf`.

### Ordering Tests

Add tests that create files in a deliberately unsorted order and verify that the final list or formatted output uses the intended stable ordering.

This is especially important because filesystem directory iteration order is not guaranteed.

### Formatter Edge Cases

Potential formatter tests include:

- Empty files
- File paths containing spaces
- Content containing triple backticks
- Non-UTF-8 paths
- Very large text files

Content containing Markdown fences may require a formatter change before it can be represented safely.

### Argument Parsing Edge Cases

Potential parser tests include:

- Missing values for all value-taking options
- Empty values for all `--flag=value` forms
- Repeated flags
- Repeated output file flags
- Values beginning with `-`
- Explicit `--` end-of-options handling, if supported in the future

## Adding New Tests

When adding a feature or fixing a bug:

1. Add a unit test when the behavior can be isolated.
2. Add an integration test when the behavior is visible through the CLI.
3. Prefer deterministic output modes such as `--stdout` or `--output`.
4. Avoid relying on installed clipboard or picker commands.
5. Use a unique temporary directory for filesystem tests.
6. Ensure temporary resources are cleaned up on both success and failure.
7. Use descriptive test names that state the expected behavior.
8. Keep platform-specific tests behind an appropriate `#[cfg(...)]` guard.
9. Test observable behavior instead of internal implementation details.
10. Run the full test suite before submitting the change.

A test name should describe one behavior clearly:

```rust
#[test]
fn exclude_dir_filters_stdout_output() {
    // ...
}
```

When a bug is fixed, add a regression test that fails before the fix and passes afterward.

## CI Recommendations

A minimal CI test job should run:

```bash
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets
```

If the project supports multiple operating systems, run the suite on at least:

- Linux
- macOS
- Windows

Unix-only symlink tests will run where `#[cfg(unix)]` is enabled and will be skipped on Windows.

Clipboard and picker commands should not be required for the default CI test job.

## Notes

The test suite prioritizes:

- Reproducibility
- Portability
- Clear failure diagnostics
- Minimal dependencies
- Security around filesystem traversal
- Testing behavior visible to users

Some runtime integrations are intentionally tested manually because they depend on external commands or operating-system facilities.

As those components evolve, dependency injection or small mockable abstractions can be introduced to make clipboard and picker behavior testable without sacrificing the lightweight design of the project.
