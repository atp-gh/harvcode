# harvcode

**harvcode** means **harvest code**.

A small, fast, terminal-friendly Rust tool that collects source files, formats them as Markdown code blocks, and sends the result to your clipboard, stdout, or a file.

It is useful when you want to quickly share project code, selected files, or AI-friendly context with AI tools, documentation, teammates, or issue reports.

## Features

- Recursively collects files from directories
- Supports multiple input paths
- Formats files as Markdown code blocks
- Copies output to the system clipboard by default
- Falls back to stdout when clipboard copy is unavailable
- Supports explicit output modes:
  - clipboard
  - stdout
  - file
- Supports interactive file selection with `sk` or `fzf`
- Allows manually choosing the picker
- Skips hidden files and directories
- Skips common binary and archive file types
- Supports include and exclude filtering
- Simple, fast, and dependency-light

## Design Goals

- Keep the CLI simple
- Avoid unnecessary dependencies
- Work well in terminal workflows
- Produce readable and predictable output
- Generate AI-friendly code context
- Prefer sensible defaults over heavy configuration

## Requirements

### Clipboard support

harvcode uses one of the following clipboard commands when clipboard output is enabled:

- `wl-copy` for wayland
- `xclip` for x11
- `pbcopy` for macos
- `clip` for windows

> [!NOTE]
> **Windows clipboard encoding**
>
> If text copied by harvcode appears garbled on Windows, this is because the
> Windows `clip` command may not handle UTF-8 text correctly.
>
> To resolve this issue, enable Windows UTF-8 globalization support:
>
> 1. Open **Settings**.
> 2. Go to **Time & language** → **Language & region**.
> 3. Open **Administrative language settings**.
> 4. Click **Change system locale...**.
> 5. Enable **Beta: Use Unicode UTF-8 for worldwide language support**.
> 6. Restart Windows for the change to take effect.
>
> This Windows option is marked as a beta feature. Enable it only if it is
> compatible with the other applications you use.

### Optional picker support

Interactive mode requires one of:

- `sk`
- `fzf`

If both are installed, harvcode prefers: `sk > fzf`

## Installation

### Cargo

```bash
cargo install harvcode
```

### Linux x86_64 prebuilt binary

For Linux x86_64, you can install the prebuilt `musl` binary manually:

```bash
curl -L \
  https://github.com/atp-gh/harvcode/releases/download/v0.5.3/harvcode-v0.5.3-x86_64-unknown-linux-musl.tar.gz \
  -o harvcode-v0.5.3-x86_64-unknown-linux-musl.tar.gz
tar -xzf harvcode-v0.5.3-x86_64-unknown-linux-musl.tar.gz
chmod +x harvcode-v0.5.3-x86_64-unknown-linux-musl/harvcode
sudo mv harvcode-v0.5.3-x86_64-unknown-linux-musl/harvcode /usr/local/bin/harvcode


# Verify the installation:
harvcode --version
```

### Windows and FreeBSD

For Windows and FreeBSD, download the appropriate archive from [the GitHub Releases page](https://github.com/atp-gh/harvcode/releases).

After downloading, extract the archive and place the harvcode executable somewhere in your PATH.

### MacOS

Prebuilt binaries for macOS are not provided yet.

Please build from source using the instructions below.

### Build from source

You can build harvcode from source on any supported platform with a stable Rust toolchain.

Clone the repository:

```bash
git clone https://github.com/atp-gh/harvcode.git
cd harvcode
```

Build with Cargo:

```bash
cargo build --release
```

The compiled binary will be available at `target/release/harvcode`.

Optionally install it into your PATH:

```bash
sudo cp target/release/harvcode /usr/local/bin/harvcode
```

## CLI Reference

```text
Usage:
  harvcode [options] [paths...]

Options:
  -h, --help                    Show help
  -V, --version                 Show version
      --pick                    Interactive selection (sk / fzf)
      --picker <sk|fzf>         Choose picker manually; implies --pick
      --list                    List collected file paths only
      --quiet                   Suppress non-error status output
      --verbose                 Print execution report

Output:
      --clipboard               Copy output to clipboard
      --stdout                  Write output to stdout
      --output <file>           Write output to file

Filtering:
      --include-ext <list>      Include only extensions, e.g. rs,toml,md
      --exclude-ext <list>      Exclude extensions, e.g. lock,json
      --exclude-dir <list>      Exclude directories, e.g. target,node_modules
      --exclude-file <list>     Exclude files, e.g. Cargo.lock
```

## Exit Codes

```text
0   Success
1   CLI argument error
2   Picker unavailable or selection cancelled
3   Output failure
```

## Project Structure

```text
src/
├── main.rs        # CLI entry point and application flow
├── args.rs        # CLI argument parsing
├── walker.rs      # Recursive file collection
├── filter.rs      # File filtering rules
├── formatter.rs   # Markdown output formatting
├── clipboard.rs   # Clipboard integration
├── picker.rs      # Interactive file selection
├── report.rs      # Execution reporting
└── list.rs        # File listing mode
```

## CONTRIBUTING

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
