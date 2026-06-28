use std::path::Path;

/// Format file content into Markdown-style code block
/// Includes relative file path as language/tag
pub fn format(path: &Path, content: &str) -> String {
    let rel = path.strip_prefix(".").unwrap_or(path);

    let mut s = String::new();

    s.push_str("```");
    s.push_str(&rel.to_string_lossy());
    s.push('\n');

    s.push_str(content);

    if !content.ends_with('\n') {
        s.push('\n');
    }

    s.push_str("```\n\n");

    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_markdown_code_block() {
        // Formatter should wrap file content in a Markdown code block.
        //
        // The opening fence includes the file path:
        //
        // ```src/main.rs
        // ...
        // ```
        let result = format(Path::new("src/main.rs"), "fn main() {}\n");

        assert!(result.starts_with("```"));
        assert!(result.contains("src/main.rs"));
        assert!(result.contains("fn main() {}"));
        assert!(result.ends_with("```\n\n"));
    }

    #[test]
    fn adds_missing_trailing_newline() {
        // If file content does not end with a newline,
        // formatter should add one before the closing Markdown fence.
        //
        // This prevents malformed output like:
        //
        // fn main() {}```
        let result = format(Path::new("src/main.rs"), "fn main() {}");

        assert!(result.contains("fn main() {}\n```"));
    }

    #[test]
    fn keeps_existing_trailing_newline() {
        // If content already ends with a newline,
        // formatter should not need special handling beyond closing the block.
        let result = format(Path::new("src/main.rs"), "fn main() {}\n");

        assert!(result.contains("fn main() {}\n```"));
    }

    #[test]
    fn appends_blank_line_after_each_code_block() {
        // Multiple formatted files are concatenated together.
        // Ending each block with an extra blank line improves readability.
        let result = format(Path::new("src/main.rs"), "fn main() {}\n");

        assert!(result.ends_with("```\n\n"));
    }
}
