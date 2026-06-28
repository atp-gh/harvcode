use std::path::PathBuf;
use std::process;

use crate::filter::Rules;

/// Runtime configuration derived from CLI arguments.
///
/// - `roots`: input files/directories
/// - `pick`: whether interactive selection is enabled
/// - `rules`: file filtering rules
pub struct Config {
    pub roots: Vec<PathBuf>,
    pub pick: bool,
    pub rules: Rules,
}

/// Version string from Cargo.toml at compile time.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Parse command-line arguments into a `Config`.
///
/// Supported forms:
/// - `--include-ext rs,toml,md`
/// - `--include-ext=rs,toml,md`
/// - `--exclude-ext lock,json`
/// - `--exclude-dir target,node_modules`
/// - `--exclude-file Cargo.lock`
///
/// Behavior:
/// - Treats non-flag arguments as file/directory paths
/// - Returns error on unknown flags or missing values
/// - Defaults to current directory if no paths provided
pub fn parse_args(args: Vec<String>) -> Result<Config, ()> {
    let mut pick = false;
    let mut roots = Vec::new();
    let mut rules = Rules::default();

    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

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

            "--include-ext" => {
                rules.include_ext = parse_values(require_value(&args, &mut i, "--include-ext")?);
            }

            "--exclude-ext" => {
                rules.exclude_ext = parse_values(require_value(&args, &mut i, "--exclude-ext")?);
            }

            "--exclude-dir" => {
                rules.exclude_dirs = parse_values(require_value(&args, &mut i, "--exclude-dir")?);
            }

            "--exclude-file" => {
                rules.exclude_files = parse_values(require_value(&args, &mut i, "--exclude-file")?);
            }

            _ if arg.starts_with("--include-ext=") => {
                rules.include_ext = parse_values(after_equal(arg));
            }

            _ if arg.starts_with("--exclude-ext=") => {
                rules.exclude_ext = parse_values(after_equal(arg));
            }

            _ if arg.starts_with("--exclude-dir=") => {
                rules.exclude_dirs = parse_values(after_equal(arg));
            }

            _ if arg.starts_with("--exclude-file=") => {
                rules.exclude_files = parse_values(after_equal(arg));
            }

            _ if arg.starts_with('-') => {
                eprintln!("Unknown option: {}", arg);
                eprintln!("Use --help for usage.");
                return Err(());
            }

            _ => roots.push(PathBuf::from(arg)),
        }

        i += 1;
    }

    if roots.is_empty() {
        roots.push(PathBuf::from("."));
    }

    Ok(Config { roots, pick, rules })
}

/// Read the next CLI argument as the value for a flag.
fn require_value(args: &[String], index: &mut usize, flag: &str) -> Result<String, ()> {
    if *index + 1 >= args.len() {
        eprintln!("Missing value for {}", flag);
        return Err(());
    }

    *index += 1;
    Ok(args[*index].clone())
}

/// Return the part after `=` in `--flag=value`.
fn after_equal(arg: &str) -> String {
    arg.split_once('=')
        .map(|(_, value)| value.to_string())
        .unwrap_or_default()
}

/// Parse comma-separated values.
///
/// Values are:
/// - trimmed
/// - converted to lowercase
/// - empty entries ignored
fn parse_values(value: String) -> Vec<String> {
    value
        .split(',')
        .map(|v| v.trim().to_ascii_lowercase())
        .filter(|v| !v.is_empty())
        .collect()
}

/// Print CLI usage and examples.
fn print_help() {
    println!(
        r#"harvcode {version}

Usage:
  harvcode [options] [paths...]

Options:
  -h, --help                    Show help
  -V, --version                 Show version
      --pick                    Interactive selection (sk / fzf)

Filtering:
      --include-ext <list>      Include only extensions, e.g. rs,toml,md
      --exclude-ext <list>      Exclude extensions, e.g. lock,json
      --exclude-dir <list>      Exclude directories, e.g. target,node_modules
      --exclude-file <list>     Exclude files, e.g. Cargo.lock

Examples:
  harvcode
  harvcode src
  harvcode src/main.rs
  harvcode --pick
  harvcode --include-ext rs,toml,md
  harvcode --exclude-ext lock,json
  harvcode --exclude-dir target,node_modules
  harvcode --exclude-file Cargo.lock
"#,
        version = VERSION
    );
}

/// Print version information.
fn print_version() {
    println!("harvcode {}", VERSION);
}
