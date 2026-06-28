use std::path::PathBuf;
use std::process;

use crate::filter::Rules;

/// Runtime configuration derived from CLI arguments.
///
/// - `roots`: input files/directories
/// - `pick`: whether interactive selection is enabled
/// - `rules`: file filtering rules
/// - `output`: output behavior
pub struct Config {
    pub roots: Vec<PathBuf>,
    pub pick: bool,
    pub rules: Rules,
    pub output: OutputConfig,
}

/// Output behavior derived from CLI arguments.
///
/// Default:
/// - If no output flags are specified, clipboard is enabled.
///
/// Explicit output flags:
/// - `--clipboard` copy to clipboard
/// - `--stdout` write to stdout
/// - `--output <file>` write to file
#[derive(Default)]
pub struct OutputConfig {
    pub clipboard: bool,
    pub stdout: bool,
    pub file: Option<PathBuf>,

    /// Whether the user explicitly selected at least one output mode.
    ///
    /// Used to distinguish:
    /// - implicit default clipboard behavior
    /// - explicit `--clipboard`
    pub explicit: bool,
}

/// Version string from Cargo.toml at compile time.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Parse command-line arguments into a `Config`.
///
/// Supported forms:
/// - `--clipboard`
/// - `--stdout`
/// - `--output context.md`
/// - `--output=context.md`
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
/// - Defaults to clipboard output if no output mode is specified
pub fn parse_args(args: Vec<String>) -> Result<Config, ()> {
    let mut pick = false;
    let mut roots = Vec::new();
    let mut rules = Rules::default();
    let mut output = OutputConfig::default();

    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        match arg.as_str() {
            "--pick" => pick = true,

            "--clipboard" => {
                output.clipboard = true;
                output.explicit = true;
            }

            "--stdout" => {
                output.stdout = true;
                output.explicit = true;
            }

            "--output" => {
                let value = require_value(&args, &mut i, "--output")?;
                output.file = Some(PathBuf::from(value));
                output.explicit = true;
            }

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

            _ if arg.starts_with("--output=") => {
                let value = after_equal(arg);

                if value.is_empty() {
                    eprintln!("Missing value for --output");
                    return Err(());
                }

                output.file = Some(PathBuf::from(value));
                output.explicit = true;
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

    // v0.4.0 default behavior:
    // If no output mode is specified, keep the old clipboard-first behavior.
    if !output.explicit {
        output.clipboard = true;
    }

    Ok(Config {
        roots,
        pick,
        rules,
        output,
    })
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

Output:
      --clipboard               Copy output to clipboard
      --stdout                  Write output to stdout
      --output <file>           Write output to file

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
  harvcode --clipboard
  harvcode --stdout
  harvcode --output context.md
  harvcode --stdout --output context.md
  harvcode --clipboard --output context.md
  harvcode --include-ext rs,toml,md
  harvcode --exclude-ext lock,json
  harvcode --exclude-dir target,node_modules
  harvcode --exclude-file Cargo.lock

Default:
  If no output option is specified, harvcode copies to clipboard.
"#,
        version = VERSION
    );
}

/// Print version information.
fn print_version() {
    println!("harvcode {}", VERSION);
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Helper: parse a list of string slices into Config.
    ///
    /// This keeps individual tests short and focused on assertions,
    /// instead of repeatedly converting `&str` into `String`.
    fn parse(input: &[&str]) -> Config {
        parse_args(input.iter().map(|s| s.to_string()).collect()).unwrap()
    }

    #[test]
    fn defaults_to_current_dir() {
        // When no path is provided, harvcode should behave as if "." was given.
        // This preserves the original "run in current directory" workflow.
        let cfg = parse(&[]);

        assert_eq!(cfg.roots, vec![PathBuf::from(".")]);
    }

    #[test]
    fn defaults_to_clipboard_when_no_output_flag_is_given() {
        // v0.4.0 keeps backward compatibility:
        // if the user does not specify --clipboard, --stdout, or --output,
        // the default output mode remains clipboard.
        let cfg = parse(&[]);

        assert!(cfg.output.clipboard);
        assert!(!cfg.output.stdout);
        assert!(cfg.output.file.is_none());

        // `explicit` remains false so main.rs can distinguish:
        // - implicit default clipboard behavior
        // - explicit `--clipboard`
        //
        // This matters because implicit clipboard failure may fall back to stdout,
        // while explicit clipboard failure should be treated as an error.
        assert!(!cfg.output.explicit);
    }

    #[test]
    fn parses_stdout_output() {
        // `--stdout` should enable stdout output only.
        // It should not also enable clipboard implicitly.
        let cfg = parse(&["--stdout"]);

        assert!(!cfg.output.clipboard);
        assert!(cfg.output.stdout);
        assert!(cfg.output.file.is_none());
        assert!(cfg.output.explicit);
    }

    #[test]
    fn parses_clipboard_output() {
        // Explicit `--clipboard` should enable clipboard output.
        // It is marked explicit so failures can be reported clearly.
        let cfg = parse(&["--clipboard"]);

        assert!(cfg.output.clipboard);
        assert!(!cfg.output.stdout);
        assert!(cfg.output.file.is_none());
        assert!(cfg.output.explicit);
    }

    #[test]
    fn parses_output_file() {
        // `--output context.md` should write to the provided file path.
        // It should not enable clipboard or stdout unless requested separately.
        let cfg = parse(&["--output", "context.md"]);

        assert!(!cfg.output.clipboard);
        assert!(!cfg.output.stdout);
        assert_eq!(cfg.output.file, Some(PathBuf::from("context.md")));
        assert!(cfg.output.explicit);
    }

    #[test]
    fn parses_output_file_with_equal_syntax() {
        // The `--output=file` form should behave the same as `--output file`.
        let cfg = parse(&["--output=context.md"]);

        assert!(!cfg.output.clipboard);
        assert!(!cfg.output.stdout);
        assert_eq!(cfg.output.file, Some(PathBuf::from("context.md")));
        assert!(cfg.output.explicit);
    }

    #[test]
    fn parses_combined_output_modes() {
        // v0.4.0 allows output modes to be combined.
        // This test ensures the parser does not treat them as mutually exclusive.
        let cfg = parse(&["--clipboard", "--stdout", "--output", "context.md"]);

        assert!(cfg.output.clipboard);
        assert!(cfg.output.stdout);
        assert_eq!(cfg.output.file, Some(PathBuf::from("context.md")));
        assert!(cfg.output.explicit);
    }

    #[test]
    fn returns_error_for_missing_output_value() {
        // `--output` requires a following file path.
        // Missing values should be rejected during argument parsing.
        let result = parse_args(vec!["--output".to_string()]);

        assert!(result.is_err());
    }

    #[test]
    fn returns_error_for_empty_output_value() {
        // `--output=` is syntactically present but semantically empty.
        // Treating this as an error avoids accidentally writing to an invalid path.
        let result = parse_args(vec!["--output=".to_string()]);

        assert!(result.is_err());
    }

    #[test]
    fn returns_error_for_unknown_option() {
        // Unknown flags should fail fast instead of being treated as paths.
        let result = parse_args(vec!["--unknown".to_string()]);

        assert!(result.is_err());
    }

    #[test]
    fn parses_include_extensions() {
        // Extension lists are comma-separated and normalized to lowercase.
        let cfg = parse(&["--include-ext", "rs,toml,MD"]);

        assert_eq!(cfg.rules.include_ext, vec!["rs", "toml", "md"]);
    }

    #[test]
    fn parses_exclude_extensions() {
        // Excluded extensions should also be normalized to lowercase.
        let cfg = parse(&["--exclude-ext", "LOCK,JSON"]);

        assert_eq!(cfg.rules.exclude_ext, vec!["lock", "json"]);
    }

    #[test]
    fn parses_exclude_dirs() {
        // Directory names are stored as lowercase values for case-insensitive matching.
        let cfg = parse(&["--exclude-dir", "target,node_modules"]);

        assert_eq!(cfg.rules.exclude_dirs, vec!["target", "node_modules"]);
    }

    #[test]
    fn parses_exclude_files() {
        // File names are stored as lowercase values for case-insensitive matching.
        let cfg = parse(&["--exclude-file", "Cargo.lock,secret.rs"]);

        assert_eq!(cfg.rules.exclude_files, vec!["cargo.lock", "secret.rs"]);
    }
}
