//! Character code utilities for TypeSpec-Rust
//!
//! Ported from TypeSpec compiler/src/core/charcode.ts
//!
//! This module provides character code constants and utility functions
//! for character classification and manipulation.

/// Character codes matching the TypeSpec compiler
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharCode {
    Null = 0x00,
    MaxAscii = 0x7f,
    ByteOrderMark = 0xfeff,

    // Line breaks
    LineFeed = 0x0a,
    CarriageReturn = 0x0d,

    // ASCII whitespace excluding line breaks
    Space = 0x20,
    Tab = 0x09,
    VerticalTab = 0x0b,
    FormFeed = 0x0c,

    // Non-ASCII whitespace excluding line breaks
    NextLine = 0x0085,
    LeftToRightMark = 0x200e,
    RightToLeftMark = 0x200f,
    LineSeparator = 0x2028,
    ParagraphSeparator = 0x2029,

    // ASCII Digits
    _0 = 0x30,
    _1 = 0x31,
    _2 = 0x32,
    _3 = 0x33,
    _4 = 0x34,
    _5 = 0x35,
    _6 = 0x36,
    _7 = 0x37,
    _8 = 0x38,
    _9 = 0x39,

    // ASCII lowercase letters
    a = 0x61,
    b = 0x62,
    c = 0x63,
    d = 0x64,
    e = 0x65,
    f = 0x66,
    g = 0x67,
    h = 0x68,
    i = 0x69,
    j = 0x6a,
    k = 0x6b,
    l = 0x6c,
    m = 0x6d,
    n = 0x6e,
    o = 0x6f,
    p = 0x70,
    q = 0x71,
    r = 0x72,
    s = 0x73,
    t = 0x74,
    u = 0x75,
    v = 0x76,
    w = 0x77,
    x = 0x78,
    y = 0x79,
    z = 0x7a,

    // ASCII uppercase letters
    A = 0x41,
    B = 0x42,
    C = 0x43,
    D = 0x44,
    E = 0x45,
    F = 0x46,
    G = 0x47,
    H = 0x48,
    I = 0x49,
    J = 0x4a,
    K = 0x4b,
    L = 0x4c,
    M = 0x4d,
    N = 0x4e,
    O = 0x4f,
    P = 0x50,
    Q = 0x51,
    R = 0x52,
    S = 0x53,
    T = 0x54,
    U = 0x55,
    V = 0x56,
    W = 0x57,
    X = 0x58,
    Y = 0x59,
    Z = 0x5a,

    // Non-letter, non-digit ASCII characters that are valid in identifiers
    Underscore = 0x5f,
    Dollar = 0x24,

    // ASCII punctuation
    Ampersand = 0x26,
    Asterisk = 0x2a,
    At = 0x40,
    Backslash = 0x5c,
    Backtick = 0x60,
    Bar = 0x7c,
    Caret = 0x5e,
    CloseBrace = 0x7d,
    CloseBracket = 0x5d,
    CloseParen = 0x29,
    Colon = 0x3a,
    Comma = 0x2c,
    Dot = 0x2e,
    DoubleQuote = 0x22,
    Equals = 0x3d,
    Exclamation = 0x21,
    GreaterThan = 0x3e,
    Hash = 0x23,
    LessThan = 0x3c,
    Minus = 0x2d,
    OpenBrace = 0x7b,
    OpenBracket = 0x5b,
    OpenParen = 0x28,
    Percent = 0x25,
    Plus = 0x2b,
    Question = 0x3f,
    Semicolon = 0x3b,
    SingleQuote = 0x27,
    Slash = 0x2f,
    Tilde = 0x7e,
}

/// Get the number of UTF-16 code units for a code point
pub fn utf16_code_units(code_point: u32) -> usize {
    if code_point >= 0x10000 { 2 } else { 1 }
}

/// Check if a character is a high surrogate
pub fn is_high_surrogate(ch: u32) -> bool {
    (0xd800..=0xdbff).contains(&ch)
}

/// Check if a character is a low surrogate
pub fn is_low_surrogate(ch: u32) -> bool {
    (0xdc00..=0xdfff).contains(&ch)
}

/// Check if a character is a line break
pub fn is_line_break(ch: u32) -> bool {
    ch == CharCode::LineFeed as u32 || ch == CharCode::CarriageReturn as u32
}

/// Check if a character is ASCII whitespace (single line)
pub fn is_ascii_whitespace_single_line(ch: u32) -> bool {
    ch == CharCode::Space as u32
        || ch == CharCode::Tab as u32
        || ch == CharCode::VerticalTab as u32
        || ch == CharCode::FormFeed as u32
}

/// Check if a character is non-ASCII whitespace (single line)
pub fn is_non_ascii_whitespace_single_line(ch: u32) -> bool {
    ch == CharCode::NextLine as u32
        || ch == CharCode::LeftToRightMark as u32
        || ch == CharCode::RightToLeftMark as u32
        || ch == CharCode::LineSeparator as u32
        || ch == CharCode::ParagraphSeparator as u32
}

/// Check if a character is whitespace
pub fn is_whitespace(ch: u32) -> bool {
    is_whitespace_single_line(ch) || is_line_break(ch)
}

/// Check if a character is whitespace (single line)
pub fn is_whitespace_single_line(ch: u32) -> bool {
    is_ascii_whitespace_single_line(ch)
        || (ch > CharCode::MaxAscii as u32 && is_non_ascii_whitespace_single_line(ch))
}

/// Check if a character is a digit
pub fn is_digit(ch: u32) -> bool {
    ch >= CharCode::_0 as u32 && ch <= CharCode::_9 as u32
}

/// Check if a character is a hex digit
pub fn is_hex_digit(ch: u32) -> bool {
    is_digit(ch)
        || (ch >= CharCode::A as u32 && ch <= CharCode::F as u32)
        || (ch >= CharCode::a as u32 && ch <= CharCode::f as u32)
}

/// Check if a character is a binary digit
pub fn is_binary_digit(ch: u32) -> bool {
    ch == CharCode::_0 as u32 || ch == CharCode::_1 as u32
}

/// Check if a character is a lowercase ASCII letter
pub fn is_lowercase_ascii_letter(ch: u32) -> bool {
    ch >= CharCode::a as u32 && ch <= CharCode::z as u32
}

/// Check if a character can start an ASCII identifier
pub fn is_ascii_identifier_start(ch: u32) -> bool {
    (ch >= CharCode::A as u32 && ch <= CharCode::Z as u32)
        || (ch >= CharCode::a as u32 && ch <= CharCode::z as u32)
        || ch == CharCode::Dollar as u32
        || ch == CharCode::Underscore as u32
}

/// Check if a character can continue an ASCII identifier
pub fn is_ascii_identifier_continue(ch: u32) -> bool {
    (ch >= CharCode::A as u32 && ch <= CharCode::Z as u32)
        || (ch >= CharCode::a as u32 && ch <= CharCode::z as u32)
        || (ch >= CharCode::_0 as u32 && ch <= CharCode::_9 as u32)
        || ch == CharCode::Dollar as u32
        || ch == CharCode::Underscore as u32
}

/// Check if a character can start an identifier
pub fn is_identifier_start(code_point: u32) -> bool {
    is_ascii_identifier_start(code_point)
        || (code_point > CharCode::MaxAscii as u32 && is_non_ascii_identifier_character(code_point))
}

/// Check if a character can continue an identifier
pub fn is_identifier_continue(code_point: u32) -> bool {
    is_ascii_identifier_continue(code_point)
        || (code_point > CharCode::MaxAscii as u32 && is_non_ascii_identifier_character(code_point))
}

/// Check if a non-ASCII code point is a valid identifier character.
///
/// Delegates to the full Unicode 15.0 range map in `scanner::nonascii`.
pub fn is_non_ascii_identifier_character(code_point: u32) -> bool {
    crate::scanner::nonascii::is_non_ascii_identifier_character(code_point)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_code_values() {
        assert_eq!(CharCode::Space as u32, 0x20);
        assert_eq!(CharCode::Tab as u32, 0x09);
        assert_eq!(CharCode::LineFeed as u32, 0x0a);
        assert_eq!(CharCode::CarriageReturn as u32, 0x0d);
        assert_eq!(CharCode::A as u32, 0x41);
        assert_eq!(CharCode::a as u32, 0x61);
        assert_eq!(CharCode::_0 as u32, 0x30);
    }

    #[test]
    fn test_is_line_break() {
        assert!(is_line_break(CharCode::LineFeed as u32));
        assert!(is_line_break(CharCode::CarriageReturn as u32));
        assert!(!is_line_break(CharCode::Space as u32));
        assert!(!is_line_break(CharCode::A as u32));
    }

    #[test]
    fn test_is_ascii_whitespace_single_line() {
        assert!(is_ascii_whitespace_single_line(CharCode::Space as u32));
        assert!(is_ascii_whitespace_single_line(CharCode::Tab as u32));
        assert!(!is_ascii_whitespace_single_line(CharCode::LineFeed as u32));
    }

    #[test]
    fn test_is_digit() {
        assert!(is_digit(CharCode::_0 as u32));
        assert!(is_digit(CharCode::_5 as u32));
        assert!(is_digit(CharCode::_9 as u32));
        assert!(!is_digit(CharCode::A as u32));
        assert!(!is_digit(CharCode::a as u32));
    }

    #[test]
    fn test_is_hex_digit() {
        assert!(is_hex_digit(CharCode::_0 as u32));
        assert!(is_hex_digit(CharCode::A as u32));
        assert!(is_hex_digit(CharCode::F as u32));
        assert!(is_hex_digit(CharCode::a as u32));
        assert!(is_hex_digit(CharCode::f as u32));
        assert!(!is_hex_digit(CharCode::G as u32));
    }

    #[test]
    fn test_is_binary_digit() {
        assert!(is_binary_digit(CharCode::_0 as u32));
        assert!(is_binary_digit(CharCode::_1 as u32));
        assert!(!is_binary_digit(CharCode::_2 as u32));
    }

    #[test]
    fn test_is_ascii_identifier_start() {
        assert!(is_ascii_identifier_start(CharCode::A as u32));
        assert!(is_ascii_identifier_start(CharCode::z as u32));
        assert!(is_ascii_identifier_start(CharCode::Dollar as u32));
        assert!(is_ascii_identifier_start(CharCode::Underscore as u32));
        assert!(!is_ascii_identifier_start(CharCode::_0 as u32));
        assert!(!is_ascii_identifier_start(CharCode::Space as u32));
    }

    #[test]
    fn test_is_ascii_identifier_continue() {
        assert!(is_ascii_identifier_continue(CharCode::A as u32));
        assert!(is_ascii_identifier_continue(CharCode::_0 as u32));
        assert!(is_ascii_identifier_continue(CharCode::Dollar as u32));
        assert!(is_ascii_identifier_continue(CharCode::Underscore as u32));
        assert!(!is_ascii_identifier_continue(CharCode::Space as u32));
        assert!(!is_ascii_identifier_continue(CharCode::Ampersand as u32));
    }

    #[test]
    fn test_is_identifier_start() {
        assert!(is_identifier_start(CharCode::A as u32));
        assert!(is_identifier_start(CharCode::a as u32));
        assert!(is_identifier_start(CharCode::Dollar as u32));
        assert!(is_identifier_start(CharCode::Underscore as u32));
    }

    #[test]
    fn test_is_identifier_continue() {
        assert!(is_identifier_continue(CharCode::A as u32));
        assert!(is_identifier_continue(CharCode::_0 as u32));
    }

    #[test]
    fn test_is_high_surrogate() {
        assert!(is_high_surrogate(0xd800));
        assert!(is_high_surrogate(0xdbff));
        assert!(!is_high_surrogate(0xdc00));
        assert!(!is_high_surrogate(0xffff));
    }

    #[test]
    fn test_is_low_surrogate() {
        assert!(is_low_surrogate(0xdc00));
        assert!(is_low_surrogate(0xdfff));
        assert!(!is_low_surrogate(0xd800));
        assert!(!is_low_surrogate(0xffff));
    }

    #[test]
    fn test_utf16_code_units() {
        assert_eq!(utf16_code_units(0x0), 1);
        assert_eq!(utf16_code_units(0xffff), 1);
        assert_eq!(utf16_code_units(0x10000), 2);
        assert_eq!(utf16_code_units(0x1f600), 2);
    }
}
