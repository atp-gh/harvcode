mod args;
mod clipboard;
mod filter;
mod formatter;
mod picker;
mod walker;

use std::env;
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
/// - `None` if picker is requested but unavailable
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

/// Application entry point.
///
/// Flow:
/// 1. Parse CLI arguments → `Config`
/// 2. Resolve target files
/// 3. Filter and read file contents
/// 4. Format as Markdown code blocks
/// 5. Copy to clipboard (fallback: stdout)
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
            eprintln!("No picker available (sk / fzf required)");
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

    // Attempt clipboard copy, fallback to stdout.
    if clipboard::copy(&output) {
        eprintln!("Copied to clipboard");
    } else {
        print!("{}", output);
    }
}
