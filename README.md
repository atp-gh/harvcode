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

### Optional picker support

Interactive mode requires one of:

- `sk`
- `fzf`

If both are installed, harvcode prefers: `sk > fzf`

## Installation

### Linux x86_64 prebuilt binary

For Linux x86_64, you can install the prebuilt `musl` binary manually:

```bash
curl -L \
  https://github.com/atp-gh/harvcode/releases/download/v0.4.1/harvcode-v0.4.1-x86_64-unknown-linux-musl.tar.gz \
  -o harvcode-v0.4.1-x86_64-unknown-linux-musl.tar.gz
tar -xzf harvcode-v0.4.1-x86_64-unknown-linux-musl.tar.gz
chmod +x harvcode-v0.4.1-x86_64-unknown-linux-musl/harvcode
sudo mv harvcode-v0.4.1-x86_64-unknown-linux-musl/harvcode /usr/local/bin/harvcode


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
      --picker <sk|fzf>          Choose picker manually; implies --pick

Output:
      --clipboard               Copy output to clipboard
      --stdout                  Write output to stdout
      --output <file>            Write output to file

Filtering:
      --include-ext <list>       Include only extensions, e.g. rs,toml,md
      --exclude-ext <list>       Exclude extensions, e.g. lock,json
      --exclude-dir <list>       Exclude directories, e.g. target,node_modules
      --exclude-file <list>      Exclude files, e.g. Cargo.lock
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
└── picker.rs      # Interactive file selection
```
