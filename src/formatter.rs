use std::path::Path;

/// Append file content to an existing output buffer as a Markdown code block.
///
/// Writing directly into the caller-provided `String` avoids allocating a
/// temporary `String` for every file. This is especially useful when many
/// files are processed and appended to one final output buffer.
///
/// The opening Markdown fence includes the relative file path:
///
/// ```src/main.rs
/// fn main() {}
/// ```
///
/// A trailing newline is inserted before the closing fence when the original
/// content does not already end with one. Each code block also ends with one
/// blank line so multiple blocks remain visually separated.
///
/// The fence length is derived from the content: a run of backticks inside
/// the content can never close the block early because the fence is always
/// one backtick longer than the longest run in the content.
pub fn append(output: &mut String, path: &Path, content: &str) {
    // Remove a leading "." component when possible so paths such as
    // "./src/main.rs" are displayed as "src/main.rs".
    let relative_path = path.strip_prefix(".").unwrap_or(path);

    let fence = "`".repeat(fence_len(content));
    let path_label = sanitize_path_label(&relative_path.to_string_lossy());

    // Write directly into the final output buffer instead of creating an
    // intermediate String and copying it into the final buffer afterward.
    output.push_str(&fence);
    output.push_str(&path_label);
    output.push('\n');

    // Append the file content without modifying it.
    output.push_str(content);

    // Ensure the closing Markdown fence starts on its own line.
    if !content.ends_with('\n') {
        output.push('\n');
    }

    // Add a blank line after the block to separate it from the next file.
    output.push_str(&fence);
    output.push_str("\n\n");
}

/// Return the number of backticks the code fence needs for the given content.
///
/// Markdown closes a fenced code block with a run of backticks at least as
/// long as the opening fence. If the file content contains a run of backticks
/// that is as long as the opening fence, that run closes the block early and
/// breaks the output structure.
///
/// Using a fence one backtick longer than the longest run in the content
/// guarantees the block cannot be closed by the content itself. The minimum
/// is three backticks, the CommonMark fence length for an ordinary block.
fn fence_len(content: &str) -> usize {
    let mut longest_run = 0;
    let mut current_run = 0;

    for ch in content.chars() {
        if ch == '`' {
            current_run += 1;
            longest_run = longest_run.max(current_run);
        } else {
            current_run = 0;
        }
    }

    (longest_run + 1).max(3)
}

/// Replace characters in a path label that could break the Markdown fence.
///
/// Newlines, carriage returns, tabs, and backticks would either split the
/// opening fence onto multiple lines or terminate it early, so they are
/// replaced with an underscore. Other control characters are replaced as well
/// so the label stays safe to print in terminals and inside the fence line.
fn sanitize_path_label(label: &str) -> String {
    label
        .chars()
        .map(|c| match c {
            '\n' | '\r' | '\t' | '`' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Format one file into a newly created buffer for concise test setup.
    ///
    /// Production code reuses a shared output buffer, while this helper keeps
    /// individual tests focused on the resulting Markdown.
    fn format(path: &Path, content: &str) -> String {
        let mut output = String::new();
        append(&mut output, path, content);
        output
    }

    #[test]
    fn formats_markdown_code_block() {
        let result = format(Path::new("src/main.rs"), "fn main() {}\n");

        assert_eq!(result, "```src/main.rs\nfn main() {}\n```\n\n");
    }

    #[test]
    fn adds_missing_trailing_newline() {
        let result = format(Path::new("src/main.rs"), "fn main() {}");

        // The formatter must insert a newline before the closing fence.
        assert_eq!(result, "```src/main.rs\nfn main() {}\n```\n\n");
    }

    #[test]
    fn keeps_existing_trailing_newline() {
        let result = format(Path::new("src/main.rs"), "fn main() {}\n");

        // Existing trailing newlines must not be duplicated.
        assert_eq!(result, "```src/main.rs\nfn main() {}\n```\n\n");
    }

    #[test]
    fn appends_blank_line_after_each_code_block() {
        let result = format(Path::new("src/main.rs"), "fn main() {}\n");

        // Two trailing newlines leave one empty line between code blocks.
        assert!(result.ends_with("```\n\n"));
    }

    #[test]
    fn appends_multiple_files_to_the_same_buffer() {
        let mut output = String::new();

        append(&mut output, Path::new("src/main.rs"), "fn main() {}\n");

        append(&mut output, Path::new("src/lib.rs"), "pub fn run() {}\n");

        // Both files should be stored in the same buffer without replacing
        // content that was appended by an earlier call.
        assert_eq!(
            output,
            concat!(
                "```src/main.rs\n",
                "fn main() {}\n",
                "```\n\n",
                "```src/lib.rs\n",
                "pub fn run() {}\n",
                "```\n\n",
            )
        );
    }

    #[test]
    fn removes_leading_current_directory_component() {
        let result = format(Path::new("./src/main.rs"), "fn main() {}\n");

        assert_eq!(result, "```src/main.rs\nfn main() {}\n```\n\n");
    }

    #[test]
    fn uses_longer_fence_when_content_contains_backticks() {
        // Content containing triple backticks must not close the block early.
        // The fence must be longer than any run of backticks in the content.
        let result = format(Path::new("src/main.rs"), "```\nmalicious\n```\n");

        assert!(result.starts_with("````src/main.rs\n"));
        assert!(result.ends_with("````\n\n"));
    }

    #[test]
    fn fence_length_scales_with_longest_backtick_run() {
        // A run of two backticks still fits under the minimum three-backtick
        // fence and must not change the ordinary output.
        let short = format(Path::new("src/main.rs"), "a `` b\n");
        assert!(short.starts_with("```src/main.rs\n"));

        // A run of five backticks requires a six-backtick fence.
        let long = format(Path::new("src/main.rs"), "`````\n");
        assert!(long.starts_with("``````src/main.rs\n"));
        assert!(long.ends_with("``````\n\n"));
    }

    #[test]
    fn sanitizes_path_label_with_newlines_and_backticks() {
        // Path labels with newlines, carriage returns, tabs, backticks, or
        // control characters must not be able to break the fence line.
        let label = sanitize_path_label("evil\n```\t\x1b");

        assert!(!label.contains('\n'));
        assert!(!label.contains('\r'));
        assert!(!label.contains('\t'));
        assert!(!label.contains('`'));
        assert!(!label.contains('\x1b'));
    }

    #[test]
    fn sanitized_path_label_is_used_in_output() {
        // A hostile file name must not appear verbatim inside the fence line.
        let result = format(Path::new("evil\n```.rs"), "fn main() {}\n");

        // The opening fence line must be a single line: the hostile newline
        // and backticks are replaced with underscores.
        assert_eq!(result.lines().next(), Some("```evil____.rs"));
    }
}
