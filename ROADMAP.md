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

- [ ] Add `--help`
- [ ] Add `--version`
- [ ] Add short aliases:
  - `-h`
  - `-V`
- [ ] Display usage examples
- [ ] Display available options clearly
- [ ] Return proper exit codes for invalid arguments

### Example

```bash
harvcode --help
harvcode --version
````

## v0.3.0 - Enhanced Filtering

Add more precise control over what files are collected.

### Planned

* Include files by extension
* Exclude files by extension
* Exclude specific directories
* Exclude specific files
* Support multiple filter values
* Make filtering case-insensitive where appropriate
* Keep default binary/archive exclusions

### Example

```bash
harvcode --include-ext rs,toml,md
harvcode --exclude-ext lock,json
harvcode --exclude-dir target,node_modules
harvcode --exclude-file Cargo.lock
```

## v0.4.0 - Output System Refactor

Make output behavior explicit and predictable.

### Planned

* Explicitly output to clipboard
* Explicitly output to stdout
* Save output to a file
* Allow combining output modes where reasonable
* Improve fallback behavior
* Add clear error messages when output fails

### Example

```bash
harvcode --clipboard
harvcode --stdout
harvcode --output context.md
```

### Design Direction

Default behavior may remain clipboard-first, but explicit output flags should make automation easier.

## v0.5.0 - Git Integration

Add Git-aware collection modes for repository workflows.

### Planned

* Respect `.gitignore`
* Collect only Git-tracked files
* Collect only modified files
* Collect only staged files
* Optionally include untracked files
* Skip `.git` internals automatically
* Gracefully fallback when not inside a Git repository

### Example

```bash
harvcode --git-tracked
harvcode --git-modified
harvcode --git-staged
harvcode --respect-gitignore
```

### Use Cases

* Create AI context only from changed files
* Review staged changes before committing
* Export clean repository source without build artifacts

## v0.6.0 - Interactive Picker Improvements

Improve file selection experience in interactive mode.

### Planned

* Add file preview in picker
* Allow picker command configuration
* Define picker priority strategy
* Support custom picker arguments
* Improve behavior when no picker is installed
* Keep `sk` and `fzf` support

### Picker Priority

Default priority:

```text
sk > fzf
```

Future configurable behavior:

```bash
harvcode --pick --picker fzf
harvcode --pick --picker sk
```

### Preview Example

Possible preview behavior:

```bash
fzf --multi --preview 'bat --style=numbers --color=always {}'
```

or fallback:

```bash
fzf --multi --preview 'sed -n "1,120p" {}'
```

## v0.7.0 - Configuration File

Allow project-level and user-level configuration.

### Planned

* Support config file
* Define default include rules
* Define default exclude rules
* Define output preference
* Define picker preference
* Define max file size
* Allow CLI arguments to override config values

### Possible Config Files

```text
harvcode.toml
.harvcode.toml
```

### Example

```toml
[filter]
include_ext = ["rs", "toml", "md"]
exclude_ext = ["lock", "png", "jpg"]
exclude_dirs = ["target", "node_modules", ".git"]
exclude_files = ["Cargo.lock"]

[output]
mode = "clipboard"
file = "context.md"

[picker]
command = "fzf"
preview = true

[limits]
max_file_size = "1MB"

[git]
respect_gitignore = true
tracked_only = false
modified_only = false
```

## v0.8.0 - File Size Limits

Avoid collecting huge files accidentally.

### Planned

* Add max file size limit
* Skip files larger than configured size
* Show skipped file count
* Optionally print skipped file paths
* Support human-readable size values

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

## v0.9.0 - Better Reporting

Improve user feedback without making normal output noisy.

### Planned

* Report copied file count
* Report skipped file count
* Report total output size
* Report clipboard success/failure
* Add quiet mode
* Add verbose mode

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

## v1.0.0 - Stable CLI

Prepare the first stable release.

### Goals

* Stable command-line interface
* Stable configuration format
* Reliable Git integration
* Reliable filtering behavior
* Clear documentation
* Cross-platform clipboard support
* Good error messages
* Minimal dependencies
* Predictable output format

## Future Ideas

These are not required for v1.0.0, but may be considered later.

* Token estimation for AI context windows
* Output templates
* JSON output mode
* XML output mode
* Tree view summary
* File ordering options
* Language-aware formatting
* Ignore file similar to `.harvcodeignore`
* Shell completion scripts
* Homebrew package
* Cargo install release workflow

## Priority Order

Recommended implementation order:

1. CLI help and version output
2. Enhanced filtering
3. Output system refactor
4. File size limits
5. Git integration
6. Interactive picker improvements
7. Configuration file
8. Reporting and diagnostics
9. Stable v1.0.0 release

## Design Principles

* Keep the tool simple
* Prefer explicit CLI behavior
* Avoid unnecessary dependencies
* Make output predictable
* Respect project ignore rules
* Never silently include obviously unwanted files
* Keep configuration optional
