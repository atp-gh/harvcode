# Roadmap

This roadmap tracks planned improvements for **harvcode**.

harvcode is designed to stay simple, fast, and terminal-friendly while improving filtering, output control, Git integration, picker experience, and project-level configuration.

## v0.1.0 Current Foundation

Core functionality is already available:

- [x] Collect files from the current directory
- [x] Recursively traverse directories
- [x] Skip hidden files and common binary/archive files
- [x] Format file contents as Markdown code blocks
- [x] Copy output to clipboard
- [x] Fallback to stdout when clipboard is unavailable
- [x] Support multiple input paths
- [x] Support interactive selection with `sk` or `fzf`

## v0.2.0 - CLI Help and Version Output

Improve the command-line interface and make the tool easier to discover.

### Planned

- [x] Add `--help`
- [x] Add `--version`
- [x] Add short aliases:
  - `-h`
  - `-V`
- [x] Display usage examples
- [x] Display available options clearly
- [x] Return proper exit codes for invalid arguments

### Example

```bash
harvcode --help
harvcode --version
```

## v0.3.0 - Enhanced Filtering

Add more precise control over what files are collected.

### Planned

- [x] Include files by extension
- [x] Exclude files by extension
- [x] Exclude specific directories
- [x] Exclude specific files
- [x] Support multiple filter values
- [x] Make filtering case-insensitive where appropriate
- [x] Keep default binary/archive exclusions

### Example

```bash
harvcode --include-ext rs,toml,md
harvcode --exclude-ext lock,json
harvcode --exclude-dir target,node_modules
harvcode --exclude-file Cargo.lock
```

## v0.3.1 - Clipboard Fix

### Fixed

- [x] Use `xclip -selection clipboard` on X11
- [x] Fix clipboard output going to primary selection instead of system clipboard

## v0.4.0 - Output System Refactor

Make output behavior explicit and predictable.

### Planned

- [x] Added `--clipboard` for explicit clipboard output.
- [x] Added `--stdout` for explicit stdout output.
- [x] Added `--output <file>` for writing output to a file.
- [x] Output modes can be combined.

### Exit Codes

- `1`: CLI argument error
- `2`: picker unavailable or cancelled
- `3`: output failure

### Example

```bash
harvcode --clipboard
harvcode --stdout
harvcode --output context.md
```

## v0.4.1 - Picker Selection Improvement

Improve interactive picker selection.

### Planned

- [x] Allow users to choose the picker
- [x] Define picker priority strategy
- [x] Improve behavior when no picker is installed
- [x] Keep `sk` and `fzf` support

### Picker Priority

Default priority:

```text
sk > fzf
```

### Examples

Use the default picker:

```bash
harvcode --pick
```

Force a specific picker:

```bash
harvcode --pick --picker fzf
harvcode --pick --picker sk
```

## v0.5.0 - Better Reporting

### Planned

- [x] Report collected file count
- [x] Report skipped file count
- [x] Report total output size
- [x] Report output destination
- [x] Report clipboard success or failure
- [x] Add `--quiet`
- [x] Add `--verbose`

### Example

```bash
harvcode --verbose
harvcode --quiet
```

Possible output:

```text
Collected files: 24
Skipped files: 8
Output size: 132 KB
Copied to clipboard
```

## v0.5.1 - File Listing Mode

Add a simple listing mode to preview which files would be collected.

### Planned

- [ ] Add `--list`
- [ ] Print only collected file paths
- [ ] Do not output file contents in list mode
- [ ] Do not copy to clipboard in list mode
- [ ] Do not write to output file in list mode
- [ ] Respect all existing filters
- [ ] Respect excluded binary/archive files
- [ ] Use stable path ordering

### Example

```bash
harvcode --list
harvcode src --list
harvcode --include-ext rs,md --list
harvcode --exclude-dir target,node_modules --list
```

Possible Output

```text
Cargo.toml
README.md
src/main.rs
src/cli.rs
src/output.rs
```

## v0.6.0 - File Size Limits

Avoid collecting huge files accidentally.

### Planned

- Add max file size limit
- Skip files larger than configured size
- Show skipped file count
- Optionally print skipped file paths
- Support human-readable size values

### Example

```bash
harvcode --max-size 512KB
harvcode --max-size 1MB
harvcode --max-size 5MB
```

### Behavior

Files larger than the limit should be skipped safely.

```text
Skipped large file: data/output.log
```

## Commit and Tag Release Convention

harvcode follows a simple and consistent commit and tag convention to keep development history readable and releases predictable.

### Commit Message Convention

Commit messages should follow a lightweight Conventional Commits style.

### Commit Format

```text
<type>(optional scope): <short description>
```

### Commit Types

Common commit types:

- `feat`: Add a new feature
- `fix`: Fix a bug
- `docs`: Update documentation only
- `refactor`: Refactor code without changing behavior
- `style`: Formatting or style-only changes
- `test`: Add or update tests
- `chore`: Maintenance tasks
- `build`: Build system or dependency changes
- `ci`: CI/CD workflow changes
- `perf`: Performance improvements
- `release`: Release-related changes

### Commit Examples

```bash
git commit -m "feat(filter): add include extension option"
git commit -m "fix(output): fallback to stdout when clipboard fails"
git commit -m "docs: update roadmap"
git commit -m "refactor(cli): simplify argument parsing"
git commit -m "chore: update dependencies"
git commit -m "release: prepare v0.3.0"
```

### Commit Guidelines

- Use imperative mood where possible.
- Keep the subject line short and clear.
- Prefer one logical change per commit.
- Use a scope when it improves clarity.
- Avoid vague messages such as `update`, `fix stuff`, or `changes`.

### Version Tag Convention

Release tags should use semantic versioning.

### Tag Format

```text
vMAJOR.MINOR.PATCH
```

### Tag Examples

```text
v0.1.0
v0.2.0
v0.3.0
v1.0.0
```

### Versioning Rules

- Increment `MAJOR` for breaking CLI or configuration changes.
- Increment `MINOR` for new backward-compatible features.
- Increment `PATCH` for backward-compatible bug fixes.
- Pre-`v1.0.0` releases may still introduce breaking changes, but they should be documented clearly.

### Tagging Workflow

Recommended release workflow:

```bash
git checkout main
git pull
cargo test
git tag -a v0.3.0 -m "Release v0.3.0"
git push origin v0.3.0
```

### Release Guidelines

Before creating a release tag:

- Ensure tests pass.
- Update `ROADMAP.md` if milestone status changed.
- Update documentation if CLI behavior changed.
- Confirm the version shown by `harvcode --version`.
- Use annotated tags for official releases.
- Avoid moving or rewriting published release tags.

### Release Branch Policy

The default release branch is:

```text
main
```

Official release tags should be created from `main` after the release commit has been merged.

## v1.0.0 - Stable CLI

Prepare the first stable release.

### Goals

- Stable command-line interface
- Stable configuration format
- Reliable Git integration
- Reliable filtering behavior
- Clear documentation
- Cross-platform clipboard support
- Good error messages
- Minimal dependencies
- Predictable output format
- No required configuration file
- No required Git integration

## Future Ideas

These are not required for v1.0.0, but may be considered later.

- Token estimation for AI context windows
- Output templates
- JSON output mode
- XML output mode
- Tree view summary
- File ordering options
- Language-aware formatting
- Integration git
- Configuration File
- Ignore file similar to `.harvcodeignore`
- Shell completion scripts
- Homebrew package
- Cargo install release workflow

## Design Principles

- Keep the tool simple
- Prefer explicit CLI behavior
- Avoid unnecessary dependencies
- Make output predictable
- Respect project ignore rules
- Never silently include obviously unwanted files
- Keep configuration optional
