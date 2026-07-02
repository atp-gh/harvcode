# Testing

This document describes the testing strategy for **harvcode**.

harvcode uses Rust's built-in test framework and the standard library only. No additional test dependencies are required.

The goal is to keep the test suite simple, stable, and aligned with harvcode's dependency-light design.

## Goals

The test suite is designed to verify:

- CLI argument parsing
- File filtering rules
- Markdown output formatting
- Deterministic output behavior
- File listing mode behavior at the argument-parsing level
- CLI error handling

The tests focus on behavior that can be reliably verified across local machines, CI environments, containers, and different operating systems.

## Running Tests

Run all tests:

```bash
cargo test
```

Run only CLI integration tests:

```bash
cargo test --test cli
```

Run a specific test:

```bash
cargo test stdout_outputs_collected_files
```

Show test output:

```bash
cargo test -- --nocapture
```

## Test Structure

```text
src/
├── args.rs        # Unit tests for CLI argument parsing
├── filter.rs      # Unit tests for file filtering rules
├── formatter.rs   # Unit tests for Markdown formatting
└── ...

tests/
└── cli.rs         # Integration tests for real CLI behavior
```

## Zero-Dependency Test Design

The test suite intentionally avoids extra test dependencies.

It does not use crates such as:

- `assert_cmd`
- `predicates`
- `tempfile`

Instead, tests use Rust's standard library:

- `std::process::Command` for running the compiled CLI binary
- `std::env::temp_dir` for temporary test directories
- `std::fs` for creating test files and reading output files
- `assert!` and `assert_eq!` for assertions

This keeps the project simple and avoids introducing dependencies only for tests.

## Unit Tests

Unit tests are used for small, deterministic pieces of logic.

### Argument Parsing

Argument parsing tests are placed in:

```text
src/args.rs
```

They verify that CLI options are parsed correctly without needing to run the full binary.

Covered behavior includes:

- Default path handling
- Default output mode
- Explicit stdout output
- Explicit clipboard output
- File output
- Combined output modes
- List mode parsing
- List mode disabling implicit clipboard output
- List mode preserving explicitly parsed output flags
- Invalid flags
- Missing flag values
- Include and exclude filter parsing
- Picker backend parsing
- Quiet and verbose mode parsing

### File Listing Mode

File listing mode is enabled with:

```bash
harvcode --list
```

At the argument-parsing level, tests verify that:

- `--list` enables list mode
- `--list` does not enable the default implicit clipboard output
- Explicit output flags are still recorded by the parser when combined with `--list`
- Runtime behavior in `main.rs` is responsible for making list mode override clipboard, stdout, and file output behavior

This keeps parser tests focused on parsing rather than full application execution.

The runtime behavior of list mode is implemented in:

```text
src/list.rs
```

List mode is designed to:

- Print only file paths
- Print one valid path per line
- Apply existing file filters
- Use stable path ordering
- Avoid reading file contents
- Avoid copying to the clipboard
- Avoid writing to output files

### File Filtering

Filtering tests are placed in:

```text
src/filter.rs
```

They verify which files and directories should be included or skipped.

Covered behavior includes:

- Hidden files
- Hidden directories
- Default binary/archive extension skipping
- Include extension rules
- Exclude extension rules
- Exclude file rules
- Exclude directory rules
- Extensionless files

### Markdown Formatting

Formatting tests are placed in:

```text
src/formatter.rs
```

They verify that file contents are converted into Markdown code blocks correctly.

Covered behavior includes:

- Opening Markdown fences
- File path labels
- Preserving file content
- Handling missing trailing newlines
- Ending each block with a blank line

## Integration Tests

Integration tests are placed in:

```text
tests/cli.rs
```

These tests run the real compiled `harvcode` binary.

They verify behavior from the user's perspective, including:

- Reading files from a temporary project directory
- Writing generated output to stdout
- Writing generated output to a file
- Combining stdout and file output
- Applying CLI filtering options
- Returning correct errors for invalid arguments

Integration tests use temporary directories created with the standard library.

Each test creates its own isolated directory under the system temp directory and removes it after the test finishes.

## Clipboard Testing Policy

Clipboard output depends on external platform-specific commands:

- `wl-copy`
- `xclip`
- `pbcopy`
- `clip`

These commands may not be available in every environment.

For example:

- A Linux CI container may not have `wl-copy` or `xclip`
- A macOS machine may have `pbcopy`, but Linux does not
- A Windows machine may have `clip`, but Unix systems do not

Because of this, direct clipboard behavior is not tested through integration tests.

Instead:

- Unit tests verify that the default output mode is clipboard.
- Unit tests verify that `--list` disables implicit clipboard output.
- Integration tests use deterministic output modes such as `--stdout` and `--output`.

This keeps the test suite stable and portable.

## Output Mode Testing Strategy

harvcode supports these output-related modes:

```bash
harvcode --clipboard
harvcode --stdout
harvcode --output context.md
harvcode --list
```

The test suite focuses mainly on deterministic modes:

```bash
harvcode --stdout
harvcode --output context.md
harvcode --stdout --output context.md
```

These modes are deterministic because they do not require external clipboard commands.

Default clipboard behavior is tested at the argument-parsing level instead of through direct system clipboard integration.

List mode is also tested at the argument-parsing level. The parser records `--list` and ensures that implicit clipboard output is not enabled when list mode is used.

Runtime list behavior can be verified manually with commands such as:

```bash
harvcode --list
harvcode src --list
harvcode --include-ext rs,md --list
harvcode --exclude-dir target,node_modules --list
```

## Test Coverage Summary

| Area             | Type        | Verified Behavior                                          |
| ---------------- | ----------- | ---------------------------------------------------------- |
| Argument parsing | Unit        | No path defaults to current directory                      |
| Argument parsing | Unit        | No explicit output option defaults to clipboard            |
| Argument parsing | Unit        | `--stdout` enables stdout output only                      |
| Argument parsing | Unit        | `--clipboard` enables explicit clipboard output            |
| Argument parsing | Unit        | `--output <file>` parses file output                       |
| Argument parsing | Unit        | `--output=<file>` parses file output                       |
| Argument parsing | Unit        | Multiple output modes can be combined                      |
| Argument parsing | Unit        | `--list` enables list mode                                 |
| Argument parsing | Unit        | `--list` disables implicit clipboard output                |
| Argument parsing | Unit        | `--list` preserves explicitly parsed output flags          |
| Argument parsing | Unit        | `--list --clipboard` keeps explicit clipboard in parser    |
| Argument parsing | Unit        | Missing `--output` value returns an error                  |
| Argument parsing | Unit        | Empty `--output=` value returns an error                   |
| Argument parsing | Unit        | Unknown options return an error                            |
| Argument parsing | Unit        | Include extension values are parsed and normalized         |
| Argument parsing | Unit        | Exclude extension values are parsed and normalized         |
| Argument parsing | Unit        | Excluded directory names are parsed                        |
| Argument parsing | Unit        | Excluded file names are parsed                             |
| Argument parsing | Unit        | `--picker fzf` enables picker mode                         |
| Argument parsing | Unit        | `--picker sk` enables picker mode                          |
| Argument parsing | Unit        | `--picker=<value>` syntax is supported                     |
| Argument parsing | Unit        | Unknown picker backends return an error                    |
| Argument parsing | Unit        | Empty picker values return an error                        |
| Argument parsing | Unit        | `--quiet` is parsed                                        |
| Argument parsing | Unit        | `--verbose` is parsed                                      |
| Argument parsing | Unit        | `--quiet` and `--verbose` can be parsed together           |
| File filtering   | Unit        | Hidden files are skipped                                   |
| File filtering   | Unit        | Hidden directories are skipped                             |
| File filtering   | Unit        | Binary and archive extensions are skipped                  |
| File filtering   | Unit        | Normal text files are accepted                             |
| File filtering   | Unit        | Extensionless files are accepted by default                |
| File filtering   | Unit        | `--include-ext` only allows matching extensions            |
| File filtering   | Unit        | `--include-ext` excludes extensionless files               |
| File filtering   | Unit        | Default skipped extensions cannot be re-included           |
| File filtering   | Unit        | `--exclude-ext` rejects matching extensions                |
| File filtering   | Unit        | `--exclude-file` matches file names case-insensitively     |
| File filtering   | Unit        | `--exclude-dir` matches directory names case-insensitively |
| Formatting       | Unit        | File content is wrapped in a Markdown code block           |
| Formatting       | Unit        | File path is included in the opening Markdown fence        |
| Formatting       | Unit        | Missing trailing newline is added before the closing fence |
| Formatting       | Unit        | Existing trailing newline is handled correctly             |
| Formatting       | Unit        | Each formatted block ends with a blank line                |
| CLI behavior     | Integration | `--stdout` prints collected files to stdout                |
| CLI behavior     | Integration | Hidden files are not included in stdout output             |
| CLI behavior     | Integration | Hidden directories are not traversed                       |
| CLI behavior     | Integration | Binary and archive extensions are skipped                  |
| CLI behavior     | Integration | `--output <file>` writes generated context to a file       |
| CLI behavior     | Integration | `--stdout` and `--output` can be combined                  |
| CLI behavior     | Integration | `--include-ext` filters stdout output                      |
| CLI behavior     | Integration | `--exclude-ext` filters stdout output                      |
| CLI behavior     | Integration | `--exclude-dir` filters stdout output                      |
| CLI behavior     | Integration | `--exclude-file` filters stdout output                     |
| CLI errors       | Integration | Unknown options exit with code `1`                         |
| CLI errors       | Integration | Missing `--output` value exits with code `1`               |

## Manual List Mode Testing

List mode can be checked manually.

Basic list mode:

```bash
harvcode --list
```

List files under a specific directory:

```bash
harvcode src --list
```

List only selected extensions:

```bash
harvcode --include-ext rs,md --list
```

List while excluding generated directories:

```bash
harvcode --exclude-dir target,node_modules --list
```

List mode should:

- Print file paths only
- Print one path per line
- Not print file contents
- Not copy anything to the clipboard
- Not write to files passed through `--output`
- Respect all existing filters
- Use stable path ordering

These commands are useful for verifying the behavior:

```bash
harvcode --list --output context.md
harvcode --list --clipboard
harvcode --list --stdout
```

In all cases, list mode should write only the file list to stdout.

## Manual Clipboard Testing

Clipboard behavior can be checked manually.

Default clipboard behavior:

```bash
harvcode
```

Explicit clipboard behavior:

```bash
harvcode --clipboard
```

Clipboard plus file output:

```bash
harvcode --clipboard --output context.md
```

If clipboard support is unavailable, install one of the supported clipboard commands for your platform.

## Exit Codes Covered by Tests

| Code | Meaning            |
| ---- | ------------------ |
| `0`  | Success            |
| `1`  | CLI argument error |

Other exit codes may exist in the application, but the current integration tests focus on stable argument and output behavior.

## Notes

The test suite prioritizes reproducibility.

Clipboard integration is part of harvcode's runtime behavior, but it is intentionally not covered by direct integration tests because it depends on external commands and platform-specific environments.

List mode is deterministic and could be covered by integration tests in the future. If stronger automated coverage is desired, useful integration tests would include:

- `--list` prints file paths only
- `--list` does not print file contents
- `--list` respects `--include-ext`
- `--list` respects `--exclude-dir`
- `--list --output context.md` does not write `context.md`
- `--list --clipboard` does not require clipboard tools
- `--list` output is sorted deterministically

If clipboard behavior needs stronger automated coverage in the future, the clipboard layer should first be refactored into a mockable abstraction.
