use std::path::PathBuf;
use std::process;

/// Runtime configuration derived from CLI arguments.
///
/// - `roots`: input files/directories
/// - `pick`: whether interactive selection is enabled
pub struct Config {
    pub roots: Vec<PathBuf>,
    pub pick: bool,
}

/// Version string from Cargo.toml at compile time.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Parse command-line arguments into a `Config`.
///
/// Behavior:
/// - Handles flags (`--help`, `--version`, `--pick`)
/// - Treats non-flag arguments as file/directory paths
/// - Returns error on unknown flags
/// - Defaults to current directory if no paths provided
///
/// Exit conditions:
/// - `--help` or `--version` will print and terminate the process
pub fn parse_args(args: Vec<String>) -> Result<Config, ()> {
    let mut pick = false;
    let mut roots = Vec::new();

    for arg in args {
        match arg.as_str() {
            "--pick" => pick = true,

            // Show help and exit immediately
            "-h" | "--help" => {
                print_help();
                process::exit(0);
            }

            // Show version and exit immediately
            "-V" | "--version" => {
                print_version();
                process::exit(0);
            }

            // Unknown flag: report error
            _ if arg.starts_with('-') => {
                eprintln!("Unknown option: {}", arg);
                eprintln!("Use --help for usage.");
                return Err(());
            }

            // Positional argument → treat as input path
            _ => roots.push(PathBuf::from(arg)),
        }
    }

    // Default to current directory
    if roots.is_empty() {
        roots.push(PathBuf::from("."));
    }

    Ok(Config { roots, pick })
}

/// Print CLI usage and examples.
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
"#,
        version = VERSION
    );
}

/// Print version information.
fn print_version() {
    println!("harvcode {}", VERSION);
}
