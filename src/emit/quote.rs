//! String quoting for the hand-written JSON and YAML emitters.

use std::fmt::Write;

/// A JSON string literal for `s`, quotes included.
pub(crate) fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 || c == '\u{7f}' => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// A YAML scalar (block context) that reads back as the string `s`: plain
/// when that is unambiguous, otherwise double-quoted. A JSON string literal
/// is a valid YAML double-quoted scalar.
pub(crate) fn yaml_scalar(s: &str) -> String {
    if is_plain_safe(s) {
        s.to_string()
    } else {
        json_string(s)
    }
}

/// A single-quoted YAML scalar for `s`.
pub(crate) fn yaml_single_quoted(s: &str) -> String {
    if s.chars().any(|c| c.is_control()) {
        return json_string(s);
    }
    format!("'{}'", s.replace('\'', "''"))
}

/// Whether `s` can be written as a plain YAML scalar and still read back as
/// that same string (not a comment, mapping, flow collection, tag, alias,
/// number, boolean or null).
fn is_plain_safe(s: &str) -> bool {
    let Some(first) = s.chars().next() else {
        return false;
    };
    if !(first.is_alphabetic() || matches!(first, '_' | '$' | '/' | '.')) {
        return false;
    }
    if s.ends_with([' ', ':']) || s.contains(": ") || s.contains(" #") {
        return false;
    }
    if !s.chars().all(|c| {
        c.is_alphanumeric()
            || matches!(
                c,
                '_' | '$' | '/' | '.' | '-' | '{' | '}' | '[' | ']' | '?' | ' ' | '|'
            )
    }) {
        return false;
    }
    let lower = s.to_ascii_lowercase();
    !matches!(
        lower.as_str(),
        "true" | "false" | "null" | "yes" | "no" | "on" | "off" | "y" | "n" | ".inf" | ".nan"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_string_escapes() {
        assert_eq!(json_string(r#"a"b"#), r#""a\"b""#);
        assert_eq!(json_string("a\\b\n\u{1}"), r#""a\\b\n\u0001""#);
    }

    #[test]
    fn yaml_scalar_quotes_only_when_needed() {
        assert_eq!(yaml_scalar("Pet"), "Pet");
        assert_eq!(yaml_scalar("name?"), "name?");
        assert_eq!(yaml_scalar("/pets/{id}"), "/pets/{id}");
        assert_eq!(yaml_scalar("Pet[]"), "Pet[]");
        assert_eq!(yaml_scalar("a | b"), "a | b");
        assert_eq!(yaml_scalar("#c"), r##""#c""##);
        assert_eq!(yaml_scalar(r#"a"b"#), r#""a\"b""#);
        assert_eq!(yaml_scalar("d: e"), r#""d: e""#);
        assert_eq!(yaml_scalar("true"), r#""true""#);
        assert_eq!(yaml_scalar("1"), r#""1""#);
        assert_eq!(yaml_scalar(""), r#""""#);
        assert_eq!(yaml_scalar("-x"), r#""-x""#);
    }

    #[test]
    fn yaml_single_quoted_doubles_quotes() {
        assert_eq!(yaml_single_quoted("it's"), "'it''s'");
    }
}
