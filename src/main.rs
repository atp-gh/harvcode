mod clipboard;
mod filter;
mod formatter;
mod picker;
mod walker;

use std::env;
use std::path::PathBuf;
use std::process;

/// Runtime configuration
struct Config {
    roots: Vec<PathBuf>,
    pick: bool,
}

/// Version
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_help() {
    println!(
        r#"harvcode {version}

Usage:
  harvcode [options] [paths...]

Options:
  -h, --help       Show help
  -V, --version    Show version
      --pick       Interactive selection (sk / fzf)

Examples:
  harvcode
  harvcode src
  harvcode src/main.rs
  harvcode src tests
  harvcode --pick

Description:
  Collect files, format as Markdown code blocks, copy to clipboard."#,
        version = VERSION
    );
}

fn print_version() {
    println!("harvcode {}", VERSION);
}

/// Parse CLI arguments
fn parse_args(args: Vec<String>) -> Result<Config, ()> {
    let mut pick = false;
    let mut roots = Vec::new();

    for arg in args {
        match arg.as_str() {
            "--pick" => pick = true,
            "-h" | "--help" => {
                print_help();
                process::exit(0);
            }
            "-V" | "--version" => {
                print_version();
                process::exit(0);
            }
            _ if arg.starts_with('-') => {
                eprintln!("Unknown option: {}", arg);
                eprintln!("Use --help for usage.");
                return Err(());
            }
            _ => roots.push(PathBuf::from(arg)),
        }
    }

    // Default to current directory if no paths are provided
    if roots.is_empty() {
        roots.push(PathBuf::from("."));
    }

    Ok(Config { roots, pick })
}

/// Expand input paths
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

    let cfg = match parse_args(args) {
        Ok(c) => c,
        Err(_) => process::exit(1),
    };

    let files = match resolve_files(&cfg) {
        Some(f) => f,
        None => {
            eprintln!("No picker available (sk / fzf required)");
            process::exit(2);
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
