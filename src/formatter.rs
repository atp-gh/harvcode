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
