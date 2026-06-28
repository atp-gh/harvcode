mod args;
mod clipboard;
mod filter;
mod formatter;
mod picker;
mod walker;

use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

use args::Config;

/// Expand input roots into a flat list of files.
///
/// Rules:
/// - If path is a file → include directly
/// - If path is a directory → recursively collect files
fn expand_roots(cfg: &Config) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for root in &cfg.roots {
        if root.is_file() {
            files.push(root.clone());
        } else if root.is_dir() {
            files.extend(walker::collect(root, &cfg.rules));
        }
    }

    files
}

/// Resolve the final list of files to process.
///
/// Steps:
/// 1. Expand directories into file list
/// 2. If `--pick` is enabled:
///    - Present list to fuzzy finder (sk / fzf)
///    - Return only selected files
///
/// Returns:
/// - `Some(Vec<PathBuf>)` on success
/// - `None` if picker is requested but unavailable or cancelled
fn resolve_files(cfg: &Config) -> Option<Vec<PathBuf>> {
    let all_files = expand_roots(cfg);

    if cfg.pick {
        // Convert file paths into newline-separated input for picker.
        let list = all_files
            .into_iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join("\n");

        return picker::pick(&list).map(|v| v.into_iter().map(PathBuf::from).collect());
    }

    Some(all_files)
}

/// Write output to stdout.
///
/// Using explicit I/O instead of `print!` so failures can be reported.
fn write_stdout(output: &str) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(output.as_bytes())?;
    stdout.flush()
}

/// Application entry point.
///
/// Flow:
/// 1. Parse CLI arguments → `Config`
/// 2. Resolve target files
/// 3. Filter and read file contents
/// 4. Format as Markdown code blocks
/// 5. Write output to selected destinations
///
/// Output behavior:
/// - No explicit output flags: copy to clipboard
/// - `--clipboard`: copy to clipboard
/// - `--stdout`: write to stdout
/// - `--output <file>`: write to file
/// - Output modes can be combined
fn main() {
    // Collect CLI args and skip program name.
    let args: Vec<String> = env::args().skip(1).collect();

    // Parse CLI arguments.
    let cfg = match args::parse_args(args) {
        Ok(c) => c,
        Err(_) => process::exit(1),
    };

    // Resolve file list, optionally through interactive picker.
    let files = match resolve_files(&cfg) {
        Some(f) => f,
        None => {
            eprintln!("No picker available or selection cancelled (sk / fzf required)");
            process::exit(2);
        }
    };

    // Pre-allocate output buffer with 1MB initial capacity.
    let mut output = String::with_capacity(1024 * 1024);

    // Read, filter, and format files.
    for path in files {
        if !filter::is_valid(&path, &cfg.rules) {
            continue;
        }

        if let Ok(content) = std::fs::read_to_string(&path) {
            output.push_str(&formatter::format(&path, &content));
        }
    }

    let mut output_failed = false;

    // Explicit stdout output.
    if cfg.output.stdout {
        if let Err(err) = write_stdout(&output) {
            eprintln!("Failed to write to stdout: {}", err);
            output_failed = true;
        }
    }

    // File output.
    if let Some(path) = &cfg.output.file {
        match std::fs::write(path, &output) {
            Ok(_) => {
                eprintln!("Wrote output to {}", path.display());
            }
            Err(err) => {
                eprintln!("Failed to write output file {}: {}", path.display(), err);
                output_failed = true;
            }
        }
    }

    // Clipboard output.
    if cfg.output.clipboard {
        if clipboard::copy(&output) {
            eprintln!("Copied to clipboard");
        } else if cfg.output.explicit {
            // Explicit `--clipboard` should fail loudly.
            eprintln!("Failed to copy to clipboard");
            output_failed = true;
        } else {
            // Backward-compatible default behavior:
            // no output flag means clipboard-first, stdout fallback.
            eprintln!("Clipboard unavailable; writing output to stdout");

            if let Err(err) = write_stdout(&output) {
                eprintln!("Failed to write fallback output to stdout: {}", err);
                output_failed = true;
            }
        }
    }

    if output_failed {
        process::exit(3);
    }
}
