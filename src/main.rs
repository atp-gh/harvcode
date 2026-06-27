mod clipboard;
mod filter;
mod formatter;
mod picker;
mod walker;

use std::env;
use std::path::PathBuf;

/// Runtime configuration
struct Config {
    roots: Vec<PathBuf>,
    pick: bool,
}

/// Parse CLI arguments
/// --pick enables interactive file selection
/// Remaining args are treated as input paths
fn parse_args(args: Vec<String>) -> Config {
    let mut pick = false;
    let mut roots = Vec::new();

    for arg in args {
        match arg.as_str() {
            "--pick" => pick = true,
            _ => roots.push(PathBuf::from(arg)),
        }
    }

    // Default to current directory if no paths are provided
    if roots.is_empty() {
        roots.push(PathBuf::from("."));
    }

    Config { roots, pick }
}

/// Expand input paths into a flat list of files
/// - Files are added directly
/// - Directories are recursively traversed
fn expand_roots(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for root in roots {
        if root.is_file() {
            files.push(root.clone());
        } else if root.is_dir() {
            files.extend(walker::collect(root));
        }
    }

    files
}

/// Resolve final file list
/// - Expands directories
/// - Optionally applies interactive picker
fn resolve_files(cfg: &Config) -> Option<Vec<PathBuf>> {
    let all_files = expand_roots(&cfg.roots);

    if cfg.pick {
        let list = all_files
            .into_iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join("\n");

        return picker::pick(&list).map(|v| v.into_iter().map(PathBuf::from).collect());
    }

    Some(all_files)
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let cfg = parse_args(args);

    let files = match resolve_files(&cfg) {
        Some(f) => f,
        None => {
            eprintln!("No picker available (sk / fzf required)");
            return;
        }
    };

    let mut output = String::with_capacity(1024 * 1024);

    for path in files {
        if !filter::is_valid(&path) {
            continue;
        }

        if let Ok(content) = std::fs::read_to_string(&path) {
            output.push_str(&formatter::format(&path, &content));
        }
    }

    // Try to copy to clipboard, fallback to stdout
    if clipboard::copy(&output) {
        eprintln!("Copied to clipboard");
    } else {
        print!("{}", output);
    }
}
