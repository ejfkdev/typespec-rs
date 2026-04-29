//! Scanner module for TypeSpec tokenization

mod lexer;
pub(crate) mod nonascii;

use crate::charcode;
pub use lexer::{Lexer, Position, Span, TokenFlags, TokenKind};

/// Skip trivia (whitespace and comments) forward from the given position.
/// Ported from TS scanner.ts skipTrivia().
///
/// Returns the position of the first non-trivia character.
/// If no non-trivia is found before end_position, returns end_position.
pub fn skip_trivia(input: &str, position: usize, end_position: Option<usize>) -> usize {
    let end = end_position.unwrap_or(input.len()).min(input.len());
    let bytes = input.as_bytes();
    let mut pos = position;

    while pos < end {
        let ch = bytes[pos] as u32;

        if charcode::is_whitespace(ch) {
            pos += 1;
            continue;
        }

        if ch == charcode::CharCode::Slash as u32 && pos + 1 < end {
            let next = bytes[pos + 1] as u32;
            if next == charcode::CharCode::Slash as u32 {
                // Single-line comment
                pos = skip_single_line_comment(input, pos, end);
                continue;
            }
            if next == charcode::CharCode::Asterisk as u32 {
                // Multi-line comment
                pos = skip_multi_line_comment(input, pos, end);
                continue;
            }
        }

        break;
    }

    pos
}

/// Skip whitespace only (not comments) forward from the given position.
/// Ported from TS scanner.ts skipWhiteSpace().
pub fn skip_whitespace(input: &str, position: usize, end_position: Option<usize>) -> usize {
    let end = end_position.unwrap_or(input.len()).min(input.len());
    let mut pos = position;

    while pos < end {
        let ch = input.as_bytes()[pos] as u32;
        if !charcode::is_whitespace(ch) {
            break;
        }
        pos += 1;
    }

    pos
}

/// Skip a continuous run of identifier characters from the given position.
/// Ported from TS scanner.ts skipContinuousIdentifier().
///
/// If is_backward is true, scans backward; otherwise forward.
/// Returns the position after/before the identifier run.
pub fn skip_continuous_identifier(input: &str, position: usize, is_backward: bool) -> usize {
    let mut cur = position;
    let bytes = input.as_bytes();

    if is_backward {
        while cur > 0 {
            // Look at the character before cur
            let prev = cur - 1;
            let ch = bytes[prev] as u32;
            if !charcode::is_identifier_continue(ch) {
                break;
            }
            cur = prev;
        }
    } else {
        while cur < input.len() {
            let ch = bytes[cur] as u32;
            if !charcode::is_identifier_continue(ch) {
                break;
            }
            cur += 1;
        }
    }

    cur
}

/// Get a display string for a token kind.
/// Ported from TS scanner.ts tokenToString() / Token display mapping.
pub fn token_to_string(token: &TokenKind) -> &'static str {
    match token {
        TokenKind::None => "none",
        TokenKind::Invalid => "invalid",
        TokenKind::EndOfFile => "end of file",
        TokenKind::Identifier => "identifier",
        TokenKind::NumericLiteral => "numeric literal",
        TokenKind::StringLiteral => "string literal",
        TokenKind::StringTemplateHead => "string template head",
        TokenKind::StringTemplateMiddle => "string template middle",
        TokenKind::StringTemplateTail => "string template tail",
        TokenKind::SingleLineComment => "single-line comment",
        TokenKind::MultiLineComment => "multi-line comment",
        TokenKind::NewLine => "newline",
        TokenKind::Whitespace => "whitespace",
        TokenKind::ConflictMarker => "conflict marker",
        TokenKind::DocText => "doc text",
        TokenKind::DocCodeSpan => "doc code span",
        TokenKind::DocCodeFenceDelimiter => "doc code fence delimiter",
        TokenKind::OpenBrace => "'{'",
        TokenKind::CloseBrace => "'}'",
        TokenKind::OpenParen => "'('",
        TokenKind::CloseParen => "')'",
        TokenKind::OpenBracket => "'['",
        TokenKind::CloseBracket => "']'",
        TokenKind::Dot => "'.'",
        TokenKind::Ellipsis => "'...'",
        TokenKind::Semicolon => "';'",
        TokenKind::Comma => "','",
        TokenKind::LessThan => "'<'",
        TokenKind::GreaterThan => "'>'",
        TokenKind::Equals => "'='",
        TokenKind::Ampersand => "'&'",
        TokenKind::Bar => "'|'",
        TokenKind::Question => "'?'",
        TokenKind::Colon => "':'",
        TokenKind::ColonColon => "'::'",
        TokenKind::At => "'@'",
        TokenKind::AtAt => "'@@'",
        TokenKind::Hash => "'#'",
        TokenKind::HashBrace => "'#{'",
        TokenKind::HashBracket => "'#['",
        TokenKind::Star => "'*'",
        TokenKind::ForwardSlash => "'/'",
        TokenKind::Plus => "'+'",
        TokenKind::Hyphen => "'-'",
        TokenKind::Exclamation => "'!'",
        TokenKind::LessThanEquals => "'<='",
        TokenKind::GreaterThanEquals => "'>='",
        TokenKind::AmpersandAmpersand => "'&&'",
        TokenKind::BarBar => "'||'",
        TokenKind::EqualsEquals => "'=='",
        TokenKind::ExclamationEquals => "'!='",
        TokenKind::EqualsGreaterThan => "'=>'",
        TokenKind::ImportKeyword => "'import'",
        TokenKind::ModelKeyword => "'model'",
        TokenKind::ScalarKeyword => "'scalar'",
        TokenKind::NamespaceKeyword => "'namespace'",
        TokenKind::UsingKeyword => "'using'",
        TokenKind::OpKeyword => "'op'",
        TokenKind::EnumKeyword => "'enum'",
        TokenKind::AliasKeyword => "'alias'",
        TokenKind::IsKeyword => "'is'",
        TokenKind::InterfaceKeyword => "'interface'",
        TokenKind::UnionKeyword => "'union'",
        TokenKind::ProjectionKeyword => "'projection'",
        TokenKind::ElseKeyword => "'else'",
        TokenKind::IfKeyword => "'if'",
        TokenKind::DecKeyword => "'dec'",
        TokenKind::ConstKeyword => "'const'",
        TokenKind::InitKeyword => "'init'",
        TokenKind::ExternKeyword => "'extern'",
        TokenKind::InternalKeyword => "'internal'",
        TokenKind::ExtendsKeyword => "'extends'",
        TokenKind::FnKeyword => "'fn'",
        TokenKind::TrueKeyword => "'true'",
        TokenKind::FalseKeyword => "'false'",
        TokenKind::ReturnKeyword => "'return'",
        TokenKind::VoidKeyword => "'void'",
        TokenKind::NeverKeyword => "'never'",
        TokenKind::UnknownKeyword => "'unknown'",
        TokenKind::ValueOfKeyword => "'valueof'",
        TokenKind::TypeOfKeyword => "'typeof'",
        _ => "token",
    }
}

// Internal helpers for skip_trivia

fn skip_single_line_comment(input: &str, position: usize, end: usize) -> usize {
    let mut pos = position + 2; // consume '//'
    let bytes = input.as_bytes();
    while pos < end {
        if charcode::is_line_break(bytes[pos] as u32) {
            break;
        }
        pos += 1;
    }
    pos
}

fn skip_multi_line_comment(input: &str, position: usize, end: usize) -> usize {
    let mut pos = position + 2; // consume '/*'
    let bytes = input.as_bytes();
    while pos + 1 < end {
        if bytes[pos] as u32 == charcode::CharCode::Asterisk as u32
            && bytes[pos + 1] as u32 == charcode::CharCode::Slash as u32
        {
            return pos + 2;
        }
        pos += 1;
    }
    end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip_trivia_whitespace() {
        assert_eq!(skip_trivia("   hello", 0, None), 3);
        assert_eq!(skip_trivia("\t\n hello", 0, None), 3);
    }

    #[test]
    fn test_skip_trivia_comments() {
        // "// comment\n" is 11 bytes; \n is also whitespace so gets skipped
        assert_eq!(skip_trivia("// comment\nhello", 0, None), 11);
        assert_eq!(skip_trivia("/* comment */hello", 0, None), 13);
    }

    #[test]
    fn test_skip_trivia_mixed() {
        assert_eq!(skip_trivia("  // comment\n  /* x */ hello", 0, None), 23);
    }

    #[test]
    fn test_skip_trivia_no_trivia() {
        assert_eq!(skip_trivia("hello", 0, None), 0);
    }

    #[test]
    fn test_skip_trivia_end_position() {
        assert_eq!(skip_trivia("   hello", 0, Some(2)), 2);
    }

    #[test]
    fn test_skip_whitespace_only() {
        assert_eq!(skip_whitespace("   hello", 0, None), 3);
        // skip_whitespace does NOT skip comments
        assert_eq!(skip_whitespace("// hello", 0, None), 0);
    }

    #[test]
    fn test_skip_continuous_identifier_forward() {
        assert_eq!(skip_continuous_identifier("fooBar123+", 0, false), 9);
        assert_eq!(skip_continuous_identifier("foo bar", 0, false), 3);
    }

    #[test]
    fn test_skip_continuous_identifier_backward() {
        assert_eq!(skip_continuous_identifier("fooBar123+", 9, true), 0);
        assert_eq!(skip_continuous_identifier("foo bar", 7, true), 4);
    }

    #[test]
    fn test_token_to_string() {
        assert_eq!(token_to_string(&TokenKind::ModelKeyword), "'model'");
        assert_eq!(token_to_string(&TokenKind::OpenBrace), "'{'");
        assert_eq!(token_to_string(&TokenKind::EndOfFile), "end of file");
        assert_eq!(token_to_string(&TokenKind::StringLiteral), "string literal");
    }
}
