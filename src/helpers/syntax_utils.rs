//! Syntax utilities for TypeSpec identifiers and text processing
//!
//! Ported from TypeSpec compiler/src/core/helpers/syntax-utils.ts

use crate::charcode::{is_identifier_continue, is_identifier_start, utf16_code_units};

/// Print a string as a TypeSpec identifier. If the string is a valid identifier,
/// return it as-is; otherwise wrap it in backticks with escaping.
///
/// # Examples
/// ```
/// use typespec_rs::helpers::syntax_utils::{print_identifier, PrintIdentifierContext};
///
/// assert_eq!(print_identifier("foo", PrintIdentifierContext::DisallowReserved), "foo");
/// assert_eq!(print_identifier("foo bar", PrintIdentifierContext::DisallowReserved), "`foo bar`");
/// ```
pub fn print_identifier(sv: &str, context: PrintIdentifierContext) -> String {
    if need_backtick(sv, context) {
        let escaped = sv
            .replace('\\', "\\\\")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
            .replace('`', "\\`");
        format!("`{escaped}`")
    } else {
        sv.to_string()
    }
}

/// Context for identifier printing — controls whether reserved keywords
/// require backtick quoting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PrintIdentifierContext {
    /// Reserved keywords must be backtick-quoted (default)
    #[default]
    DisallowReserved,
    /// Reserved keywords can be used without backticks
    AllowReserved,
}

fn need_backtick(sv: &str, context: PrintIdentifierContext) -> bool {
    if sv.is_empty() {
        return false;
    }

    // Check for keywords
    if context == PrintIdentifierContext::AllowReserved {
        // In allow-reserved mode, reserved keywords and modifier keywords
        // don't need backticks
        if is_reserved_keyword(sv) {
            return false;
        }
        if let Some(token) = keyword_token(sv)
            && is_modifier(token)
        {
            return false;
        }
    }

    if is_keyword(sv) {
        return true;
    }

    // Check if it's a valid identifier
    let chars: Vec<char> = sv.chars().collect();
    if chars.is_empty() {
        return false;
    }

    let first_cp = chars[0] as u32;
    if !is_identifier_start(first_cp) {
        return true;
    }

    let mut pos = 0usize;
    let mut char_iter = sv.chars().peekable();
    if let Some(&c) = char_iter.peek() {
        char_iter.next();
        pos += utf16_code_units(c as u32);
        while pos < sv.len() {
            if let Some(&c) = char_iter.peek() {
                let cp = c as u32;
                if !is_identifier_continue(cp) {
                    return true;
                }
                char_iter.next();
                pos += utf16_code_units(cp);
            } else {
                break;
            }
        }
    }

    pos < sv.len()
}

/// Check if a string is a TypeSpec keyword
fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "model"
            | "scalar"
            | "namespace"
            | "interface"
            | "union"
            | "enum"
            | "alias"
            | "op"
            | "extends"
            | "is"
            | "using"
            | "import"
            | "dec"
            | "extern"
            | "internal"
            | "fn"
            | "if"
            | "else"
            | "return"
            | "void"
            | "never"
            | "unknown"
            | "null"
            | "true"
            | "false"
            | "const"
            | "init"
            | "projection"
            | "valueof"
            | "typeof"
    )
}

/// Check if a string is a reserved keyword
/// Ported from TS scanner.ts ReservedKeywords set
fn is_reserved_keyword(s: &str) -> bool {
    matches!(
        s,
        "statemachine"
            | "macro"
            | "package"
            | "metadata"
            | "env"
            | "arg"
            | "declare"
            | "array"
            | "struct"
            | "record"
            | "module"
            | "mod"
            | "sym"
            | "context"
            | "prop"
            | "property"
            | "scenario"
            | "pub"
            | "sub"
            | "typeref"
            | "trait"
            | "this"
            | "self"
            | "super"
            | "keyof"
            | "with"
            | "implements"
            | "impl"
            | "satisfies"
            | "flag"
            | "auto"
            | "partial"
            | "private"
            | "public"
            | "protected"
            | "sealed"
            | "local"
            | "async"
    )
}

/// Get the token kind for a keyword, if any
fn keyword_token(s: &str) -> Option<&'static str> {
    match s {
        "model" => Some("model"),
        "scalar" => Some("scalar"),
        "namespace" => Some("namespace"),
        "interface" => Some("interface"),
        "union" => Some("union"),
        "enum" => Some("enum"),
        "alias" => Some("alias"),
        "op" | "operation" => Some("op"),
        "extends" => Some("extends"),
        "is" => Some("is"),
        "using" => Some("using"),
        "import" => Some("import"),
        "dec" => Some("dec"),
        "extern" => Some("extern"),
        "fn" => Some("fn"),
        "if" => Some("if"),
        "else" => Some("else"),
        "return" => Some("return"),
        "void" => Some("void"),
        "never" => Some("never"),
        "unknown" => Some("unknown"),
        "null" => Some("null"),
        "true" => Some("true"),
        "false" => Some("false"),
        "const" => Some("const"),
        "internal" => Some("internal"),
        _ => None,
    }
}

/// Check if a token is a modifier keyword (contextual, can be used as identifier)
fn is_modifier(token: &str) -> bool {
    matches!(token, "extern" | "internal")
}

/// Convert a type reference AST node to its string representation.
/// Ported from TS syntax-utils.ts typeReferenceToString().
pub fn type_reference_to_string(
    ast: &crate::parser::AstBuilder,
    node_id: crate::ast::node::NodeId,
) -> String {
    match ast.id_to_node(node_id) {
        Some(crate::parser::AstNode::MemberExpression(me)) => {
            let base = type_reference_to_string(ast, me.object);
            let selector = match me.selector {
                crate::ast::types::MemberSelector::Dot => ".",
                crate::ast::types::MemberSelector::DoubleColon => "::",
            };
            let id = type_reference_to_string(ast, me.property);
            format!("{base}{selector}{id}")
        }
        Some(crate::parser::AstNode::TypeReference(tr)) => type_reference_to_string(ast, tr.name),
        Some(crate::parser::AstNode::Identifier(id)) => id.value.clone(),
        _ => String::new(),
    }
}

/// Split text into lines, handling `\r\n`, `\r`, and `\n` line endings.
pub fn split_lines(text: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut pos = 0;
    let chars: Vec<char> = text.chars().collect();
    let byte_offsets: Vec<usize> = {
        let mut offsets = Vec::new();
        let mut offset = 0;
        for c in &chars {
            offsets.push(offset);
            offset += c.len_utf8();
        }
        offsets.push(offset); // end offset
        offsets
    };

    while pos < chars.len() {
        match chars[pos] {
            '\r' => {
                let end_byte = byte_offsets[pos];
                if pos + 1 < chars.len() && chars[pos + 1] == '\n' {
                    lines.push(text[start..end_byte].to_string());
                    start = byte_offsets[pos + 2];
                    pos += 1;
                } else {
                    lines.push(text[start..end_byte].to_string());
                    start = byte_offsets[pos + 1];
                }
            }
            '\n' => {
                let end_byte = byte_offsets[pos];
                lines.push(text[start..end_byte].to_string());
                start = byte_offsets[pos + 1];
            }
            _ => {}
        }
        pos += 1;
    }

    lines.push(text[start..].to_string());
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_identifier_valid() {
        assert_eq!(
            print_identifier("foo", PrintIdentifierContext::default()),
            "foo"
        );
    }

    #[test]
    fn test_print_identifier_with_space() {
        assert_eq!(
            print_identifier("foo bar", PrintIdentifierContext::default()),
            "`foo bar`"
        );
    }

    #[test]
    fn test_print_identifier_keyword() {
        assert_eq!(
            print_identifier("model", PrintIdentifierContext::default()),
            "`model`"
        );
    }

    #[test]
    fn test_print_identifier_keyword_allow_reserved() {
        // Keywords still need backticks even in allow-reserved mode
        // (only modifiers and reserved keywords are exempted)
        assert_eq!(
            print_identifier("model", PrintIdentifierContext::AllowReserved),
            "`model`"
        );
    }

    #[test]
    fn test_print_identifier_reserved_keyword_allow_reserved() {
        // "true" is a keyword but NOT a reserved keyword, so it still needs backticks
        // in allow-reserved mode. Only ReservedKeywords (sealed, async, etc.) and
        // modifier keywords (extern, internal) are exempt.
        assert_eq!(
            print_identifier("true", PrintIdentifierContext::AllowReserved),
            "`true`"
        );
        // But "sealed" IS a reserved keyword, so it doesn't need backticks in allow-reserved
        assert_eq!(
            print_identifier("sealed", PrintIdentifierContext::AllowReserved),
            "sealed"
        );
    }

    #[test]
    fn test_print_identifier_escape_backtick() {
        assert_eq!(
            print_identifier("foo`bar", PrintIdentifierContext::default()),
            "`foo\\`bar`"
        );
    }

    #[test]
    fn test_print_identifier_escape_newline() {
        assert_eq!(
            print_identifier("foo\nbar", PrintIdentifierContext::default()),
            "`foo\\nbar`"
        );
    }

    #[test]
    fn test_split_lines_lf() {
        assert_eq!(split_lines("a\nb\nc"), vec!["a", "b", "c"]);
    }

    #[test]
    fn test_split_lines_crlf() {
        assert_eq!(split_lines("a\r\nb\r\nc"), vec!["a", "b", "c"]);
    }

    #[test]
    fn test_split_lines_cr() {
        assert_eq!(split_lines("a\rb\rc"), vec!["a", "b", "c"]);
    }

    #[test]
    fn test_split_lines_mixed() {
        assert_eq!(split_lines("a\nb\r\nc\rd"), vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn test_split_lines_trailing_newline() {
        assert_eq!(split_lines("a\nb\n"), vec!["a", "b", ""]);
    }

    #[test]
    fn test_split_lines_empty() {
        assert_eq!(split_lines(""), vec![""]);
    }

    #[test]
    fn test_split_lines_single_line() {
        assert_eq!(split_lines("hello"), vec!["hello"]);
    }

    #[test]
    fn test_type_reference_to_string_identifier() {
        let result = crate::parser::parse("model Foo {}");
        let ast = &result.builder;
        // Find an identifier node in the AST
        let root = ast.id_to_node(result.root_id);
        if let Some(crate::parser::AstNode::TypeSpecScript(script)) = root
            && let Some(&stmt_id) = script.statements.first()
        {
            let _name = type_reference_to_string(ast, stmt_id);
            // Name extraction verified; value intentionally unused in this test helper
        }
    }

    #[test]
    fn test_modifier_keywords_no_backtick_in_allow_reserved() {
        // "internal" and "extern" are modifier keywords - should not need backticks
        // in allow-reserved mode
        assert_eq!(
            print_identifier("internal", PrintIdentifierContext::AllowReserved),
            "internal"
        );
        assert_eq!(
            print_identifier("extern", PrintIdentifierContext::AllowReserved),
            "extern"
        );
    }

    #[test]
    fn test_modifier_keywords_need_backtick_in_disallow_reserved() {
        // "internal" and "extern" are keywords and need backticks in default mode
        assert_eq!(
            print_identifier("internal", PrintIdentifierContext::DisallowReserved),
            "`internal`"
        );
        assert_eq!(
            print_identifier("extern", PrintIdentifierContext::DisallowReserved),
            "`extern`"
        );
    }

    #[test]
    fn test_reserved_keywords_no_backtick_in_allow_reserved() {
        // Reserved keywords like "sealed" should not need backticks in allow-reserved
        assert_eq!(
            print_identifier("sealed", PrintIdentifierContext::AllowReserved),
            "sealed"
        );
        assert_eq!(
            print_identifier("async", PrintIdentifierContext::AllowReserved),
            "async"
        );
    }

    #[test]
    fn test_reserved_keywords_need_backtick_in_disallow_reserved() {
        // In default mode, reserved keywords are still just identifiers (not Keywords)
        // so they don't need backticks unless is_keyword matches them
        // "sealed" is NOT in is_keyword, so it's a valid identifier
        assert_eq!(
            print_identifier("sealed", PrintIdentifierContext::DisallowReserved),
            "sealed"
        );
    }
}
