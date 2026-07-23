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
pub fn append(output: &mut String, path: &Path, content: &str) {
    // Remove a leading "." component when possible so paths such as
    // "./src/main.rs" are displayed as "src/main.rs".
    let relative_path = path.strip_prefix(".").unwrap_or(path);

    // Write directly into the final output buffer instead of creating an
    // intermediate String and copying it into the final buffer afterward.
    output.push_str("```");
    output.push_str(&relative_path.to_string_lossy());
    output.push('\n');

    // Append the file content without modifying it.
    output.push_str(content);

    // Ensure the closing Markdown fence starts on its own line.
    if !content.ends_with('\n') {
        output.push('\n');
    }

    // Add a blank line after the block to separate it from the next file.
    output.push_str("```\n\n");
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
}
