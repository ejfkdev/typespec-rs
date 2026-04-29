//! Text utilities for TypeSpec-Rust
//!
//! This module provides text processing utilities.

/// Remove common leading indentation from a multiline string.
/// Strips the first and last empty lines, then removes the indentation
/// level of the first non-empty line from all lines.
pub fn dedent(s: &str) -> String {
    // Remove leading and trailing line breaks and spaces
    // Equivalent to JavaScript's: str.replace(/^\n|\n[ ]*$/g, "")
    let s = s.trim_start_matches('\n').trim_end_matches(['\n', ' ']);

    // Find the indent of the first line
    let first_line_indent = s
        .lines()
        .next()
        .map(|line| line.len() - line.trim_start().len())
        .unwrap_or(0);

    if first_line_indent == 0 {
        return s.to_string();
    }

    // Remove the indent from each line
    s.lines()
        .map(|line| {
            if line.len() >= first_line_indent
                && line
                    .chars()
                    .take(first_line_indent)
                    .all(|c| c == ' ' || c == '\t')
            {
                &line[first_line_indent..]
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// A tagged template literal that removes common indentation from a multiline string.
pub fn tag_dedent(strings: &[&str], values: &[&str]) -> String {
    let result = strings
        .iter()
        .zip(values.iter().chain(std::iter::once(&"")))
        .map(|(s, v)| format!("{}{}", s, v))
        .collect::<String>();
    dedent(&result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dedent_simple() {
        let input = "  line one\n    indented\n  line three";
        let expected = "line one\n  indented\nline three";
        assert_eq!(dedent(input), expected);
    }

    #[test]
    fn test_dedent_with_newline() {
        let input = "\n  line one\n  line two\n";
        let expected = "line one\nline two";
        assert_eq!(dedent(input), expected);
    }

    #[test]
    fn test_dedent_no_indent() {
        let input = "line one\nline two";
        assert_eq!(dedent(input), "line one\nline two");
    }

    #[test]
    fn test_dedent_single_line() {
        let input = "  hello  ";
        assert_eq!(dedent(input), "hello");
    }

    #[test]
    fn test_dedent_with_tabs() {
        let input = "\t\tline one\n\t\tline two";
        assert_eq!(dedent(input), "line one\nline two");
    }

    #[test]
    fn test_dedent_empty() {
        assert_eq!(dedent(""), "");
        assert_eq!(dedent("   "), "");
        assert_eq!(dedent("\n\n"), "");
    }

    #[test]
    fn test_tag_dedent() {
        let strings = &["  line one\n    indented\n  line three"];
        let values: &[&str] = &[];
        let result = tag_dedent(strings, values);
        assert_eq!(result, "line one\n  indented\nline three");
    }

    #[test]
    fn test_tag_dedent_with_interpolation() {
        let strings = &["  model Foo {\n    prop: ", ";\n  }"];
        let values = &["string"];
        let result = tag_dedent(strings, values);
        assert_eq!(result, "model Foo {\n  prop: string;\n}");
    }
}
