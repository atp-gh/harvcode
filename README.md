# harvcode

**harvcode** means **harvest code**.

A small terminal tool written in Rust that collects source files from the current directory, formats them as Markdown code blocks, and copies the result to your clipboard.

It is useful when you want to quickly share an entire project, selected files, or code context with AI tools, documentation, teammates, or issue reports.

## Features

- Recursively collects files from the current directory
- Copies formatted code directly to the system clipboard
- Supports multiple input paths
- Supports interactive file selection with `sk` or `fzf`
- Skips hidden files and directories
- Skips common binary and archive file types
- Falls back to stdout if clipboard copy is unavailable
- Simple, fast, and dependency-light


## Design Goals

* Keep the CLI simple
* Avoid unnecessary dependencies
* Work well in terminal workflows
* Produce AI-friendly code context
* Prefer readable and predictable output

## Requirements

* Rust stable toolchain
* clipboard command:
  * `wl-copy`
  * `xclip`
  * `pbcopy`
  * `clip`
* Optional picker command:
  * `sk`
  * `fzf`


## Installation

Clone the repository:

```bash
git clone https://github.com/atp-gh/harvcode.git
cd harvcode
```

Build with Cargo:

```bash
cargo build --release
```

The compiled binary will be available at:

```bash
target/release/harvcode
```

You can move it to a directory in your `PATH`:

```bash
cp target/release/harvcode /usr/local/bin/harvcode
```

## Usage

Run in the current directory:

```bash
harvcode
```

This collects all valid files under the current directory and copies them to your clipboard.

## Examples

### Copy all code from the current directory

```bash
harvcode
```

### Copy code from a specific directory

```bash
harvcode src
```

### Copy specific files

```bash
harvcode src/main.rs src/filter.rs
```

### Copy multiple directories

```bash
harvcode src tests examples
```

### Use interactive selection

```bash
harvcode --pick
```

Interactive mode requires either:

* `sk`
* `fzf`

The tool will prefer `sk` if available, otherwise it will use `fzf`.

## Output Format

Each file is formatted as a Markdown code block:

````md
```src/main.rs
fn main() {
    println!("Hello, world!");
}
```
````

Multiple files are concatenated into a single output.

## Clipboard Support

harvcode tries to use the following clipboard commands:

| Platform      | Command   |
| ------------- | --------- |
| Linux Wayland | `wl-copy` |
| Linux X11     | `xclip`   |
| macOS         | `pbcopy`  |
| Windows       | `clip`    |

If no clipboard command is available, harvcode prints the output to stdout instead.

## File Filtering

harvcode skips hidden files and directories.

It also skips files with the following extensions:

```text
png, jpg, jpeg, gif, bmp, ico,
mp3, mp4, avi,
exe, dll, so,
zip, tar, gz,
lock
```

Files without extensions are treated as text files.

## Project Structure

```text
src/
├── main.rs        # CLI entry point and application flow
├── walker.rs      # Recursive file collection
├── filter.rs      # File filtering rules
├── formatter.rs   # Markdown output formatting
├── clipboard.rs   # Clipboard integration
└── picker.rs      # Interactive file selection
```

## Development

Run the project locally:

```bash
cargo run
```

Run with arguments:

```bash
cargo run -- src
```

Run interactive mode:

```bash
cargo run -- --pick
```

Build release version:

```bash
cargo build --release
```
