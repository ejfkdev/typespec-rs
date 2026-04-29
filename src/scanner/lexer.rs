//! Lexer implementation for TypeSpec

use crate::charcode;
use std::iter::Peekable;
use std::str::Chars;

// Thin wrappers that convert char → u32 and delegate to the canonical charcode module.
// This avoids duplicating the CharCodes constants and classification logic.

#[inline]
fn is_line_break(c: char) -> bool {
    charcode::is_line_break(c as u32)
}

#[inline]
fn is_ascii_whitespace_single_line(c: char) -> bool {
    charcode::is_ascii_whitespace_single_line(c as u32)
}

#[inline]
fn is_non_ascii_whitespace_single_line(c: char) -> bool {
    charcode::is_non_ascii_whitespace_single_line(c as u32)
}

#[inline]
fn is_digit(c: char) -> bool {
    charcode::is_digit(c as u32)
}

#[inline]
fn is_hex_digit(c: char) -> bool {
    charcode::is_hex_digit(c as u32)
}

#[inline]
fn is_binary_digit(c: char) -> bool {
    charcode::is_binary_digit(c as u32)
}

#[inline]
fn is_lowercase_ascii_letter(c: char) -> bool {
    charcode::is_lowercase_ascii_letter(c as u32)
}

#[inline]
fn is_ascii_identifier_start(c: char) -> bool {
    charcode::is_ascii_identifier_start(c as u32)
}

#[inline]
fn is_ascii_identifier_continue(c: char) -> bool {
    charcode::is_ascii_identifier_continue(c as u32)
}

/// Token flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TokenFlags(u8);

impl TokenFlags {
    pub const NONE: TokenFlags = TokenFlags(0);
    pub const ESCAPED: TokenFlags = TokenFlags(1 << 0);
    pub const TRIPLE_QUOTED: TokenFlags = TokenFlags(1 << 1);
    pub const DOC_COMMENT: TokenFlags = TokenFlags(1 << 4);

    pub fn contains(self, other: TokenFlags) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn insert(&mut self, other: TokenFlags) {
        self.0 |= other.0;
    }
}

/// Token kinds matching TypeSpec's Token enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    None,
    Invalid,
    EndOfFile,
    Identifier,
    NumericLiteral,
    StringLiteral,
    StringTemplateHead,
    StringTemplateMiddle,
    StringTemplateTail,

    // Trivia
    SingleLineComment,
    MultiLineComment,
    NewLine,
    Whitespace,
    ConflictMarker,

    // Doc comment content
    DocText,
    DocCodeSpan,
    DocCodeFenceDelimiter,

    // Punctuation
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    Dot,
    Ellipsis,
    Semicolon,
    Comma,
    LessThan,
    GreaterThan,
    Equals,
    Ampersand,
    Bar,
    Question,
    Colon,
    ColonColon,
    At,
    AtAt,
    Hash,
    HashBrace,
    HashBracket,
    Star,
    ForwardSlash,
    Plus,
    Hyphen,
    Exclamation,
    LessThanEquals,
    GreaterThanEquals,
    AmpersandAmpersand,
    BarBar,
    EqualsEquals,
    ExclamationEquals,
    EqualsGreaterThan,

    // Statement keywords
    ImportKeyword,
    ModelKeyword,
    ScalarKeyword,
    NamespaceKeyword,
    UsingKeyword,
    OpKeyword,
    EnumKeyword,
    AliasKeyword,
    IsKeyword,
    InterfaceKeyword,
    UnionKeyword,
    ProjectionKeyword,
    ElseKeyword,
    IfKeyword,
    DecKeyword,
    ConstKeyword,
    InitKeyword,

    // Modifier keywords
    ExternKeyword,
    InternalKeyword,

    // Other keywords
    ExtendsKeyword,
    FnKeyword,
    TrueKeyword,
    FalseKeyword,
    ReturnKeyword,
    VoidKeyword,
    NeverKeyword,
    UnknownKeyword,
    ValueOfKeyword,
    TypeOfKeyword,

    // Reserved keywords
    StatemachineKeyword,
    MacroKeyword,
    PackageKeyword,
    MetadataKeyword,
    EnvKeyword,
    ArgKeyword,
    DeclareKeyword,
    ArrayKeyword,
    StructKeyword,
    RecordKeyword,
    ModuleKeyword,
    ModKeyword,
    SymKeyword,
    ContextKeyword,
    PropKeyword,
    PropertyKeyword,
    ScenarioKeyword,
    PubKeyword,
    SubKeyword,
    TypeRefKeyword,
    TraitKeyword,
    ThisKeyword,
    SelfKeyword,
    SuperKeyword,
    KeyofKeyword,
    WithKeyword,
    ImplementsKeyword,
    ImplKeyword,
    SatisfiesKeyword,
    FlagKeyword,
    AutoKeyword,
    PartialKeyword,
    PrivateKeyword,
    PublicKeyword,
    ProtectedKeyword,
    SealedKeyword,
    LocalKeyword,
    AsyncKeyword,
}

impl TokenKind {
    pub fn is_trivia(&self) -> bool {
        matches!(
            self,
            TokenKind::SingleLineComment
                | TokenKind::MultiLineComment
                | TokenKind::NewLine
                | TokenKind::Whitespace
                | TokenKind::ConflictMarker
        )
    }

    pub fn is_keyword(&self) -> bool {
        self.is_statement_keyword()
            || self.is_modifier()
            || matches!(
                self,
                // Other keywords
                TokenKind::IsKeyword
                    | TokenKind::ExtendsKeyword
                    | TokenKind::FnKeyword
                    | TokenKind::TrueKeyword
                    | TokenKind::FalseKeyword
                    | TokenKind::ReturnKeyword
                    | TokenKind::VoidKeyword
                    | TokenKind::NeverKeyword
                    | TokenKind::UnknownKeyword
                    | TokenKind::ValueOfKeyword
                    | TokenKind::TypeOfKeyword
            )
            || self.is_reserved_keyword()
    }

    /// Check if this token is a comment (single-line or multi-line).
    /// Ported from TS scanner.ts isComment().
    pub fn is_comment(&self) -> bool {
        matches!(
            self,
            TokenKind::SingleLineComment | TokenKind::MultiLineComment
        )
    }

    /// Check if this token is a modifier keyword (extern, internal).
    /// Ported from TS scanner.ts isModifier().
    pub fn is_modifier(&self) -> bool {
        matches!(self, TokenKind::ExternKeyword | TokenKind::InternalKeyword)
    }

    /// Check if this token is a statement-level keyword (import, model, namespace, etc.).
    /// Ported from TS scanner.ts isStatementKeyword().
    pub fn is_statement_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::ImportKeyword
                | TokenKind::ModelKeyword
                | TokenKind::ScalarKeyword
                | TokenKind::NamespaceKeyword
                | TokenKind::UsingKeyword
                | TokenKind::OpKeyword
                | TokenKind::EnumKeyword
                | TokenKind::AliasKeyword
                | TokenKind::InterfaceKeyword
                | TokenKind::UnionKeyword
                | TokenKind::ProjectionKeyword
                | TokenKind::IfKeyword
                | TokenKind::ElseKeyword
                | TokenKind::DecKeyword
                | TokenKind::ConstKeyword
                | TokenKind::InitKeyword
                | TokenKind::FnKeyword
        )
    }

    /// Check if this token is a reserved keyword (not currently used but reserved for future use).
    /// Ported from TS scanner.ts isReservedKeyword().
    pub fn is_reserved_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::StatemachineKeyword
                | TokenKind::MacroKeyword
                | TokenKind::PackageKeyword
                | TokenKind::MetadataKeyword
                | TokenKind::EnvKeyword
                | TokenKind::ArgKeyword
                | TokenKind::DeclareKeyword
                | TokenKind::ArrayKeyword
                | TokenKind::StructKeyword
                | TokenKind::RecordKeyword
                | TokenKind::ModuleKeyword
                | TokenKind::ModKeyword
                | TokenKind::SymKeyword
                | TokenKind::ContextKeyword
                | TokenKind::PropKeyword
                | TokenKind::PropertyKeyword
                | TokenKind::ScenarioKeyword
                | TokenKind::PubKeyword
                | TokenKind::SubKeyword
                | TokenKind::TypeRefKeyword
                | TokenKind::TraitKeyword
                | TokenKind::ThisKeyword
                | TokenKind::SelfKeyword
                | TokenKind::SuperKeyword
                | TokenKind::KeyofKeyword
                | TokenKind::WithKeyword
                | TokenKind::ImplementsKeyword
                | TokenKind::ImplKeyword
                | TokenKind::SatisfiesKeyword
                | TokenKind::FlagKeyword
                | TokenKind::AutoKeyword
                | TokenKind::PartialKeyword
                | TokenKind::PrivateKeyword
                | TokenKind::PublicKeyword
                | TokenKind::ProtectedKeyword
                | TokenKind::SealedKeyword
                | TokenKind::LocalKeyword
                | TokenKind::AsyncKeyword
        )
    }
}

/// Position in the source file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

impl Default for Position {
    fn default() -> Self {
        Position { line: 1, column: 0 }
    }
}

/// Span representing a range in the source
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Span { start, end }
    }
}

/// Keywords lookup map (computed once, shared across all Lexer instances)
static KEYWORDS: std::sync::LazyLock<std::collections::HashMap<&'static str, TokenKind>> =
    std::sync::LazyLock::new(|| {
        let mut keywords = std::collections::HashMap::new();
        keywords.insert("import", TokenKind::ImportKeyword);
        keywords.insert("model", TokenKind::ModelKeyword);
        keywords.insert("scalar", TokenKind::ScalarKeyword);
        keywords.insert("namespace", TokenKind::NamespaceKeyword);
        keywords.insert("interface", TokenKind::InterfaceKeyword);
        keywords.insert("union", TokenKind::UnionKeyword);
        keywords.insert("if", TokenKind::IfKeyword);
        keywords.insert("else", TokenKind::ElseKeyword);
        keywords.insert("projection", TokenKind::ProjectionKeyword);
        keywords.insert("using", TokenKind::UsingKeyword);
        keywords.insert("op", TokenKind::OpKeyword);
        keywords.insert("extends", TokenKind::ExtendsKeyword);
        keywords.insert("is", TokenKind::IsKeyword);
        keywords.insert("enum", TokenKind::EnumKeyword);
        keywords.insert("alias", TokenKind::AliasKeyword);
        keywords.insert("dec", TokenKind::DecKeyword);
        keywords.insert("fn", TokenKind::FnKeyword);
        keywords.insert("valueof", TokenKind::ValueOfKeyword);
        keywords.insert("typeof", TokenKind::TypeOfKeyword);
        keywords.insert("const", TokenKind::ConstKeyword);
        keywords.insert("init", TokenKind::InitKeyword);
        keywords.insert("true", TokenKind::TrueKeyword);
        keywords.insert("false", TokenKind::FalseKeyword);
        keywords.insert("return", TokenKind::ReturnKeyword);
        keywords.insert("void", TokenKind::VoidKeyword);
        keywords.insert("never", TokenKind::NeverKeyword);
        keywords.insert("unknown", TokenKind::UnknownKeyword);
        keywords.insert("extern", TokenKind::ExternKeyword);
        keywords.insert("internal", TokenKind::InternalKeyword);
        keywords.insert("prop", TokenKind::PropKeyword);
        keywords.insert("property", TokenKind::PropertyKeyword);

        // Reserved keywords
        keywords.insert("statemachine", TokenKind::StatemachineKeyword);
        keywords.insert("macro", TokenKind::MacroKeyword);
        keywords.insert("package", TokenKind::PackageKeyword);
        keywords.insert("metadata", TokenKind::MetadataKeyword);
        keywords.insert("env", TokenKind::EnvKeyword);
        keywords.insert("arg", TokenKind::ArgKeyword);
        keywords.insert("declare", TokenKind::DeclareKeyword);
        keywords.insert("array", TokenKind::ArrayKeyword);
        keywords.insert("struct", TokenKind::StructKeyword);
        keywords.insert("record", TokenKind::RecordKeyword);
        keywords.insert("module", TokenKind::ModuleKeyword);
        keywords.insert("mod", TokenKind::ModKeyword);
        keywords.insert("sym", TokenKind::SymKeyword);
        keywords.insert("context", TokenKind::ContextKeyword);
        keywords.insert("scenario", TokenKind::ScenarioKeyword);
        keywords.insert("pub", TokenKind::PubKeyword);
        keywords.insert("sub", TokenKind::SubKeyword);
        keywords.insert("typeref", TokenKind::TypeRefKeyword);
        keywords.insert("trait", TokenKind::TraitKeyword);
        keywords.insert("this", TokenKind::ThisKeyword);
        keywords.insert("self", TokenKind::SelfKeyword);
        keywords.insert("super", TokenKind::SuperKeyword);
        keywords.insert("keyof", TokenKind::KeyofKeyword);
        keywords.insert("with", TokenKind::WithKeyword);
        keywords.insert("implements", TokenKind::ImplementsKeyword);
        keywords.insert("impl", TokenKind::ImplKeyword);
        keywords.insert("satisfies", TokenKind::SatisfiesKeyword);
        keywords.insert("flag", TokenKind::FlagKeyword);
        keywords.insert("auto", TokenKind::AutoKeyword);
        keywords.insert("partial", TokenKind::PartialKeyword);
        keywords.insert("private", TokenKind::PrivateKeyword);
        keywords.insert("public", TokenKind::PublicKeyword);
        keywords.insert("protected", TokenKind::ProtectedKeyword);
        keywords.insert("sealed", TokenKind::SealedKeyword);
        keywords.insert("local", TokenKind::LocalKeyword);
        keywords.insert("async", TokenKind::AsyncKeyword);

        keywords
    });

fn get_keywords() -> &'static std::collections::HashMap<&'static str, TokenKind> {
    &KEYWORDS
}

/// The Lexer for TypeSpec
pub struct Lexer<'a> {
    source: &'a str,
    chars: Peekable<Chars<'a>>,
    position: Position,
    token_start: Position,
    current_span_start: usize,
    chars_consumed: usize,
    current_token_kind: TokenKind,
    current_token_flags: TokenFlags,
    keywords: &'static std::collections::HashMap<&'static str, TokenKind>,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer from source code
    pub fn new(source: &'a str) -> Self {
        let mut chars = source.chars().peekable();

        // Skip BOM if present
        if let Some(&'\u{feff}') = chars.peek() {
            chars.next();
        }

        Lexer {
            source,
            chars,
            position: Position::default(),
            token_start: Position::default(),
            current_span_start: 0,
            chars_consumed: 0,
            current_token_kind: TokenKind::None,
            current_token_flags: TokenFlags::NONE,
            keywords: get_keywords(),
        }
    }

    /// Get the current position in the source
    pub fn position(&self) -> Position {
        self.position
    }

    /// Get the current byte offset in the source
    pub fn offset(&self) -> usize {
        self.chars_consumed
    }

    /// Get the byte offset of the current token's start
    pub fn token_start_offset(&self) -> usize {
        self.current_span_start
    }

    /// Get the current token
    pub fn token(&self) -> &TokenKind {
        &self.current_token_kind
    }

    /// Get the token text (slice of source)
    pub fn token_text(&self) -> &str {
        let start = self.current_span_start;
        let end = self.chars_consumed;
        &self.source[start..end]
    }

    /// Get the token value (processed value)
    pub fn token_value(&self) -> String {
        let text = self.token_text();
        match self.current_token_kind {
            TokenKind::StringLiteral => self.unescape_string(text),
            TokenKind::StringTemplateHead => {
                // token_text is like "Start ${ — strip opening quote and trailing ${
                // First strip opening quote
                let s = self.strip_opening_quote(text);
                // Then strip trailing ${
                if s.ends_with("${") {
                    s[..s.len() - 2].to_string()
                } else {
                    s
                }
            }
            TokenKind::StringTemplateMiddle => {
                // token_text is like }mid${ — strip leading } and trailing ${
                let mut s = text.to_string();
                if s.starts_with('}') {
                    s = s[1..].to_string();
                }
                // Strip opening quote (from re_scan_string_template for triple-quoted)
                if s.starts_with('"') {
                    s = s[1..].to_string();
                    if s.starts_with('"') {
                        s = s[1..].to_string();
                    }
                }
                if s.ends_with("${") {
                    s = s[..s.len() - 2].to_string();
                }
                s
            }
            TokenKind::StringTemplateTail => {
                // token_text is like } tail" or just "" for empty tail
                let mut s = text.to_string();
                if s.starts_with('}') {
                    s = s[1..].to_string();
                }
                // Strip opening quote (from re_scan_string_template)
                if s.starts_with('"') {
                    s = s[1..].to_string();
                }
                // Strip closing quote
                if s.ends_with('"') {
                    s.pop();
                }
                // Handle triple-quoted
                if s.ends_with("\"\"") {
                    s.pop();
                    s.pop();
                }
                s
            }
            TokenKind::Identifier => text.to_string(),
            _ => text.to_string(),
        }
    }

    /// Strip opening quote(s) from a string literal
    fn strip_opening_quote(&self, text: &str) -> String {
        let mut chars = text.chars().peekable();
        if chars.peek() == Some(&'"') {
            chars.next();
            if chars.peek() == Some(&'"') {
                chars.next();
                if chars.peek() == Some(&'"') {
                    chars.next();
                }
            }
        }
        chars.collect()
    }

    /// Re-scan after `}` in a string template to get the next middle/tail token.
    /// This is called when the parser encounters a `}` inside a string template
    /// expression like `$"hello {name} world"`.
    pub fn re_scan_string_template(&mut self, _flags: TokenFlags) -> TokenKind {
        self.token_start = self.position;
        self.current_span_start = self.chars_consumed;
        self.current_token_flags = TokenFlags::NONE;

        // Check for triple-quoted template
        // Look ahead for """ to determine if this is a tail or middle
        let is_triple = if self.chars.peek() == Some(&'"') {
            let mut lookahead = self.chars.clone();
            lookahead.next();
            if lookahead.peek() == Some(&'"') {
                lookahead.next();
                lookahead.peek() == Some(&'"')
            } else {
                false
            }
        } else {
            false
        };

        if is_triple {
            // Consume the opening """
            self.chars.next();
            self.chars_consumed += 1;
            self.position.column += 1;
            self.chars.next();
            self.chars_consumed += 1;
            self.position.column += 1;
            self.chars.next();
            self.chars_consumed += 1;
            self.position.column += 1;
            self.current_token_flags = TokenFlags::TRIPLE_QUOTED;
        }
        // For single-quoted templates, do NOT consume the closing `"` here.
        // The `"` after `}` is the closing quote, not an opening quote.
        // It will be handled correctly in the scan loop below.

        // Scan the template content
        loop {
            if self.eof() {
                return self.finish_token(TokenKind::StringTemplateTail);
            }

            let ch = self.chars.next().expect("checked eof above");
            self.chars_consumed += ch.len_utf8();

            match ch {
                '\\' => {
                    if let Some(esc_ch) = self.chars.next() {
                        self.chars_consumed += esc_ch.len_utf8();
                        self.position.column += 1;
                    }
                    self.current_token_flags.insert(TokenFlags::ESCAPED);
                }
                '"' => {
                    if is_triple {
                        let mut lookahead = self.chars.clone();
                        if lookahead.peek() == Some(&'"') {
                            lookahead.next();
                            if lookahead.peek() == Some(&'"') {
                                lookahead.next();
                                self.chars = lookahead;
                                self.chars_consumed += 2;
                                self.position.column += 2;
                                return self.finish_token(TokenKind::StringTemplateTail);
                            }
                        }
                    } else {
                        return self.finish_token(TokenKind::StringTemplateTail);
                    }
                }
                '$' => {
                    if let Some(&'{') = self.chars.peek() {
                        self.chars.next();
                        self.chars_consumed += 1;
                        self.position.column += 1;
                        return self.finish_token(TokenKind::StringTemplateMiddle);
                    }
                }
                '\n' | '\r' => {
                    if !is_triple {
                        self.position.line += 1;
                        self.position.column = 0;
                        return self.finish_token(TokenKind::StringTemplateTail);
                    }
                    if ch == '\r' {
                        if let Some(&'\n') = self.chars.peek() {
                            self.chars.next();
                            self.chars_consumed += 1;
                        }
                        self.position.line += 1;
                        self.position.column = 0;
                    } else {
                        self.position.line += 1;
                        self.position.column = 0;
                    }
                }
                _ => {
                    self.position.column += 1;
                }
            }
        }
    }

    /// Check if we've reached the end of input
    pub fn eof(&mut self) -> bool {
        self.chars.peek().is_none()
    }

    /// Scan the next token (skipping trivia - whitespace, comments, newlines)
    pub fn scan(&mut self) -> TokenKind {
        self.token_start = self.position;
        self.current_span_start = self.chars_consumed;
        self.current_token_flags = TokenFlags::NONE;

        if self.eof() {
            return self.finish_token(TokenKind::EndOfFile);
        }

        let ch = self.chars.next().expect("checked eof above");
        self.chars_consumed += ch.len_utf8();

        match ch {
            '\n' | '\r' => {
                if ch == '\r'
                    && let Some(&'\n') = self.chars.peek()
                {
                    self.chars.next();
                    self.chars_consumed += 1;
                }
                self.position.line += 1;
                self.position.column = 0;
                // Skip newline and continue to next token
                self.scan()
            }

            ' ' | '\t' | '\u{0b}' | '\u{0c}' => {
                // Skip whitespace and continue to next token
                self.skip_trivia();
                self.scan()
            }

            '(' => self.finish_token(TokenKind::OpenParen),
            ')' => self.finish_token(TokenKind::CloseParen),
            ',' => self.finish_token(TokenKind::Comma),

            ':' => {
                if let Some(&':') = self.chars.peek() {
                    self.chars.next();
                    self.chars_consumed += 1;
                    self.finish_token(TokenKind::ColonColon)
                } else {
                    self.finish_token(TokenKind::Colon)
                }
            }

            ';' => self.finish_token(TokenKind::Semicolon),
            '[' => self.finish_token(TokenKind::OpenBracket),
            ']' => self.finish_token(TokenKind::CloseBracket),
            '{' => self.finish_token(TokenKind::OpenBrace),
            '}' => self.finish_token(TokenKind::CloseBrace),

            '@' => {
                if let Some(&'@') = self.chars.peek() {
                    self.chars.next();
                    self.chars_consumed += 1;
                    self.finish_token(TokenKind::AtAt)
                } else {
                    self.finish_token(TokenKind::At)
                }
            }

            '#' => match self.chars.peek().copied() {
                Some('{') => {
                    self.chars.next();
                    self.chars_consumed += 1;
                    self.finish_token(TokenKind::HashBrace)
                }
                Some('[') => {
                    self.chars.next();
                    self.chars_consumed += 1;
                    self.finish_token(TokenKind::HashBracket)
                }
                _ => self.finish_token(TokenKind::Hash),
            },

            '+' => {
                if self.chars.peek().map(|c| is_digit(*c)).unwrap_or(false) {
                    self.scan_signed_number()
                } else {
                    self.finish_token(TokenKind::Plus)
                }
            }

            '-' => {
                if self.chars.peek().map(|c| is_digit(*c)).unwrap_or(false) {
                    self.scan_signed_number()
                } else {
                    self.finish_token(TokenKind::Hyphen)
                }
            }

            '*' => self.finish_token(TokenKind::Star),
            '?' => self.finish_token(TokenKind::Question),

            '&' => {
                if let Some(&'&') = self.chars.peek() {
                    self.chars.next();
                    self.chars_consumed += 1;
                    self.finish_token(TokenKind::AmpersandAmpersand)
                } else {
                    self.finish_token(TokenKind::Ampersand)
                }
            }

            '.' => {
                // Check for ellipsis (...) — must see two more dots before consuming them.
                // If we only see two dots (..), don't consume the second one; just return Dot
                // so the next scan() call will see the remaining '.'.
                let mut lookahead = self.chars.clone();
                if lookahead.next() == Some('.') && lookahead.next() == Some('.') {
                    // Confirmed three dots total: consume the extra two
                    self.chars.next();
                    self.chars_consumed += 1;
                    self.chars.next();
                    self.chars_consumed += 1;
                    return self.finish_token(TokenKind::Ellipsis);
                }
                self.finish_token(TokenKind::Dot)
            }

            '/' => match self.chars.peek().copied() {
                Some('/') => self.scan_single_line_comment(),
                Some('*') => self.scan_multi_line_comment(),
                _ => self.finish_token(TokenKind::ForwardSlash),
            },

            '0' => match self.chars.peek().copied() {
                Some('x') | Some('X') => self.scan_hex_number(),
                Some('b') | Some('B') => self.scan_binary_number(),
                _ => self.scan_number(),
            },

            '1'..='9' => self.scan_number(),

            '<' => {
                if self.at_conflict_marker() {
                    self.scan_conflict_marker()
                } else if let Some(&'=') = self.chars.peek() {
                    self.chars.next();
                    self.chars_consumed += 1;
                    self.finish_token(TokenKind::LessThanEquals)
                } else {
                    self.finish_token(TokenKind::LessThan)
                }
            }

            '>' => {
                if self.at_conflict_marker() {
                    self.scan_conflict_marker()
                } else if let Some(&'=') = self.chars.peek() {
                    self.chars.next();
                    self.chars_consumed += 1;
                    self.finish_token(TokenKind::GreaterThanEquals)
                } else {
                    self.finish_token(TokenKind::GreaterThan)
                }
            }

            '=' => {
                if self.at_conflict_marker() {
                    self.scan_conflict_marker()
                } else {
                    match self.chars.peek().copied() {
                        Some('=') => {
                            self.chars.next();
                            self.chars_consumed += 1;
                            self.finish_token(TokenKind::EqualsEquals)
                        }
                        Some('>') => {
                            self.chars.next();
                            self.chars_consumed += 1;
                            self.finish_token(TokenKind::EqualsGreaterThan)
                        }
                        _ => self.finish_token(TokenKind::Equals),
                    }
                }
            }

            '|' => {
                if self.at_conflict_marker() {
                    self.scan_conflict_marker()
                } else if let Some(&'|') = self.chars.peek() {
                    self.chars.next();
                    self.chars_consumed += 1;
                    self.finish_token(TokenKind::BarBar)
                } else {
                    self.finish_token(TokenKind::Bar)
                }
            }

            '"' => {
                // Check for triple-quoted string
                let mut lookahead = self.chars.clone();
                let is_triple = lookahead.peek() == Some(&'"');
                if is_triple {
                    lookahead.next();
                    if lookahead.peek() == Some(&'"') {
                        lookahead.next();
                        self.chars = lookahead;
                        self.chars_consumed += 2;
                        self.current_token_flags = TokenFlags::TRIPLE_QUOTED;
                        self.scan_string(TokenFlags::TRIPLE_QUOTED)
                    } else {
                        self.finish_token(TokenKind::StringLiteral)
                    }
                } else {
                    self.scan_string(TokenFlags::NONE)
                }
            }

            '!' => {
                if let Some(&'=') = self.chars.peek() {
                    self.chars.next();
                    self.chars_consumed += 1;
                    self.finish_token(TokenKind::ExclamationEquals)
                } else {
                    self.finish_token(TokenKind::Exclamation)
                }
            }

            '`' => self.scan_backticked_identifier(),

            _ => {
                if is_lowercase_ascii_letter(ch) {
                    self.scan_identifier_or_keyword()
                } else if is_ascii_identifier_start(ch) {
                    self.scan_identifier()
                } else if (ch as u32) <= charcode::CharCode::MaxAscii as u32 {
                    self.scan_invalid_character()
                } else {
                    self.scan_non_ascii_token()
                }
            }
        }
    }

    fn scan_whitespace(&mut self) -> TokenKind {
        while let Some(&ch) = self.chars.peek() {
            if is_ascii_whitespace_single_line(ch) {
                self.chars.next();
                self.chars_consumed += ch.len_utf8();
                self.position.column += 1;
            } else {
                break;
            }
        }
        self.finish_token(TokenKind::Whitespace)
    }

    fn skip_trivia(&mut self) {
        while let Some(&ch) = self.chars.peek() {
            if is_ascii_whitespace_single_line(ch) {
                self.chars.next();
                self.chars_consumed += ch.len_utf8();
                self.position.column += 1;
            } else if ch == '/' {
                let mut lookahead = self.chars.clone();
                lookahead.next();
                match lookahead.peek().copied() {
                    Some('/') => {
                        // Single line comment
                        drop(lookahead);
                        self.chars.next();
                        self.chars_consumed += 1;
                        self.position.column += 1;
                        while let Some(&ch) = self.chars.peek() {
                            if is_line_break(ch) {
                                break;
                            }
                            self.chars.next();
                            self.chars_consumed += ch.len_utf8();
                            self.position.column += 1;
                        }
                    }
                    Some('*') => {
                        // Multi-line comment
                        drop(lookahead);
                        self.chars.next();
                        self.chars_consumed += 1;
                        self.position.column += 1;
                        self.chars.next(); // consume *
                        self.chars_consumed += 1;
                        while let Some(ch) = self.chars.next() {
                            self.chars_consumed += ch.len_utf8();
                            self.position.column += 1;
                            if ch == '*' {
                                if let Some(&'/') = self.chars.peek() {
                                    self.chars.next();
                                    self.chars_consumed += 1;
                                    break;
                                }
                            } else if is_line_break(ch) {
                                self.position.line += 1;
                                self.position.column = 0;
                            }
                        }
                    }
                    _ => break,
                }
            } else {
                break;
            }
        }
    }

    fn scan_single_line_comment(&mut self) -> TokenKind {
        self.chars.next(); // consume second /
        self.chars_consumed += 1;

        // Note: In TypeSpec, this does NOT consume the newline character.
        // The newline is left for the next scan() call to return as NewLine.
        while let Some(&ch) = self.chars.peek() {
            if is_line_break(ch) {
                break;
            }
            self.chars.next();
            self.chars_consumed += ch.len_utf8();
        }

        self.finish_token(TokenKind::SingleLineComment)
    }

    fn scan_multi_line_comment(&mut self) -> TokenKind {
        self.chars.next(); // consume *
        self.chars_consumed += 1;

        // Check for doc comment: /**
        if let Some(&'*') = self.chars.peek() {
            // This is a doc comment /** ... */
            self.current_token_flags.insert(TokenFlags::DOC_COMMENT);
        }

        while let Some(ch) = self.chars.next() {
            self.chars_consumed += ch.len_utf8();
            if ch == '*' {
                if let Some(&'/') = self.chars.peek() {
                    self.chars.next();
                    self.chars_consumed += 1;
                    return self.finish_token(TokenKind::MultiLineComment);
                }
            } else if is_line_break(ch) {
                self.position.line += 1;
                self.position.column = 0;
            }
        }

        // Unterminated comment
        self.finish_token(TokenKind::MultiLineComment)
    }

    fn scan_number(&mut self) -> TokenKind {
        // Scan integer part
        while let Some(&ch) = self.chars.peek() {
            if is_digit(ch) {
                self.chars.next();
                self.chars_consumed += ch.len_utf8();
                self.position.column += 1;
            } else {
                break;
            }
        }

        // Check for decimal point
        if let Some(&'.') = self.chars.peek() {
            self.chars.next();
            self.chars_consumed += 1;
            self.position.column += 1;
            self.scan_required_digits();
        }

        // Check for exponent
        if let Some(&'e') | Some(&'E') = self.chars.peek() {
            self.chars.next();
            self.chars_consumed += 1;
            self.position.column += 1;
            if let Some(&'+') | Some(&'-') = self.chars.peek() {
                self.chars.next();
                self.chars_consumed += 1;
                self.position.column += 1;
            }
            self.scan_required_digits();
        }

        self.finish_token(TokenKind::NumericLiteral)
    }

    fn scan_required_digits(&mut self) {
        if let Some(&ch) = self.chars.peek()
            && is_digit(ch)
        {
            self.chars.next();
            self.chars_consumed += ch.len_utf8();
            self.position.column += 1;
            while let Some(&ch) = self.chars.peek() {
                if is_digit(ch) {
                    self.chars.next();
                    self.chars_consumed += ch.len_utf8();
                    self.position.column += 1;
                } else {
                    break;
                }
            }
        }
    }

    fn scan_signed_number(&mut self) -> TokenKind {
        // +/- already consumed by scan()
        self.scan_number()
    }

    fn scan_hex_number(&mut self) -> TokenKind {
        self.chars.next(); // consume x/X
        self.chars_consumed += 1;
        self.position.column += 1;

        if let Some(&ch) = self.chars.peek()
            && !is_hex_digit(ch)
        {
            // Error: hex digit expected
            return self.finish_token(TokenKind::NumericLiteral);
        }

        while let Some(&ch) = self.chars.peek() {
            if is_hex_digit(ch) {
                self.chars.next();
                self.chars_consumed += ch.len_utf8();
                self.position.column += 1;
            } else {
                break;
            }
        }

        self.finish_token(TokenKind::NumericLiteral)
    }

    fn scan_binary_number(&mut self) -> TokenKind {
        self.chars.next(); // consume b/B
        self.chars_consumed += 1;
        self.position.column += 1;

        if let Some(&ch) = self.chars.peek()
            && !is_binary_digit(ch)
        {
            // Error: binary digit expected
            return self.finish_token(TokenKind::NumericLiteral);
        }

        while let Some(&ch) = self.chars.peek() {
            if is_binary_digit(ch) {
                self.chars.next();
                self.chars_consumed += ch.len_utf8();
                self.position.column += 1;
            } else {
                break;
            }
        }

        self.finish_token(TokenKind::NumericLiteral)
    }

    fn scan_string(&mut self, flags: TokenFlags) -> TokenKind {
        let is_triple = flags.contains(TokenFlags::TRIPLE_QUOTED);

        loop {
            if self.eof() {
                return self.finish_token(TokenKind::StringLiteral);
            }

            let ch = self.chars.next().expect("checked eof above");
            self.chars_consumed += ch.len_utf8();

            match ch {
                '\\' => {
                    if let Some(esc_ch) = self.chars.next() {
                        self.chars_consumed += esc_ch.len_utf8();
                    }
                    self.current_token_flags.insert(TokenFlags::ESCAPED);
                }
                '"' => {
                    if is_triple {
                        let mut lookahead = self.chars.clone();
                        if lookahead.peek() == Some(&'"') {
                            lookahead.next();
                            if lookahead.peek() == Some(&'"') {
                                lookahead.next();
                                self.chars = lookahead;
                                self.chars_consumed += 2;
                                return self.finish_token(TokenKind::StringLiteral);
                            }
                        }
                    } else {
                        return self.finish_token(TokenKind::StringLiteral);
                    }
                }
                '$' => {
                    if let Some(&'{') = self.chars.peek() {
                        self.chars.next();
                        self.chars_consumed += 1;
                        return self.finish_token(TokenKind::StringTemplateHead);
                    }
                }
                '\n' | '\r' => {
                    if !is_triple {
                        self.position.line += 1;
                        self.position.column = 0;
                        return self.finish_token(TokenKind::StringLiteral);
                    }
                    if ch == '\r'
                        && let Some(&'\n') = self.chars.peek()
                    {
                        self.chars.next();
                        self.chars_consumed += 1;
                    }
                    self.position.line += 1;
                    self.position.column = 0;
                }
                _ => {}
            }
        }
    }

    fn scan_backticked_identifier(&mut self) -> TokenKind {
        loop {
            if self.eof() {
                return self.finish_token(TokenKind::Identifier);
            }

            let ch = self.chars.next().expect("checked eof above");
            self.chars_consumed += ch.len_utf8();

            match ch {
                '`' => {
                    return self.finish_token(TokenKind::Identifier);
                }
                '\\' => {
                    if let Some(esc_ch) = self.chars.next() {
                        self.chars_consumed += esc_ch.len_utf8();
                    }
                }
                '\n' | '\r' => {
                    self.position.line += 1;
                    self.position.column = 0;
                    return self.finish_token(TokenKind::Identifier);
                }
                _ => {}
            }
        }
    }

    fn scan_identifier_or_keyword(&mut self) -> TokenKind {
        let mut count = 0;

        loop {
            count += 1;

            if self.eof() {
                break;
            }

            if let Some(&ch) = self.chars.peek() {
                if count < 12 && is_lowercase_ascii_letter(ch) {
                    self.chars.next();
                    self.chars_consumed += ch.len_utf8();
                    continue;
                }

                if is_ascii_identifier_continue(ch) {
                    self.chars.next();
                    self.chars_consumed += ch.len_utf8();
                    return self.scan_identifier();
                }
            }

            break;
        }

        let text = self.token_text();

        // Check for keyword
        if (2..=12).contains(&count)
            && let Some(keyword) = self.keywords.get(text)
        {
            return self.finish_token(*keyword);
        }

        self.finish_token(TokenKind::Identifier)
    }

    fn scan_identifier(&mut self) -> TokenKind {
        while let Some(&ch) = self.chars.peek() {
            if is_ascii_identifier_continue(ch) {
                self.chars.next();
                self.chars_consumed += ch.len_utf8();
            } else {
                break;
            }
        }
        self.finish_token(TokenKind::Identifier)
    }

    fn at_conflict_marker(&self) -> bool {
        let pos = self.current_span_start;
        if pos == 0 {
            return false;
        }

        // Must be at start of line
        if self.source.as_bytes()[pos - 1] == b'\n' || self.source.as_bytes()[pos - 1] == b'\r' {
            // Check for 7 repeated characters
            let marker_len = 7;
            if pos + marker_len > self.source.len() {
                return false;
            }

            let marker_char = self.source.as_bytes()[pos];
            for i in 0..marker_len {
                if self.source.as_bytes()[pos + i] != marker_char {
                    return false;
                }
            }

            // Valid conflict markers: <<<<<<< or >>>>>>> followed by space
            // or ======= alone, or ||||||| followed by space
            matches!(marker_char, b'<' | b'>' | b'=' | b'|')
        } else {
            false
        }
    }

    fn scan_conflict_marker(&mut self) -> TokenKind {
        // The first character of the 7-char marker was already consumed by scan().
        // Consume the remaining 6 same characters.
        let marker_char = self.chars.next().expect("conflict marker has 7 chars");
        self.chars_consumed += marker_char.len_utf8();
        for _ in 0..5 {
            if let Some(ch) = self.chars.next() {
                self.chars_consumed += ch.len_utf8();
            }
        }

        if marker_char == '<' || marker_char == '>' {
            // Consume to end of line
            while let Some(ch) = self.chars.next() {
                self.chars_consumed += ch.len_utf8();
                if is_line_break(ch) {
                    if ch == '\r'
                        && let Some(&'\n') = self.chars.peek()
                    {
                        let crlf = self.chars.next().unwrap();
                        self.chars_consumed += crlf.len_utf8();
                    }
                    self.position.line += 1;
                    self.position.column = 0;
                    break;
                }
            }
        } else {
            // Consume until next ======= or >>>>>>>
            while let Some(ch) = self.chars.next() {
                self.chars_consumed += ch.len_utf8();
                if is_line_break(ch) {
                    if ch == '\r'
                        && let Some(&'\n') = self.chars.peek()
                    {
                        let crlf = self.chars.next().unwrap();
                        self.chars_consumed += crlf.len_utf8();
                    }
                    self.position.line += 1;
                    self.position.column = 0;
                }
            }
        }

        self.finish_token(TokenKind::ConflictMarker)
    }

    fn scan_non_ascii_token(&mut self) -> TokenKind {
        if let Some(&ch) = self.chars.peek()
            && is_non_ascii_whitespace_single_line(ch)
        {
            return self.scan_whitespace();
        }
        self.scan_identifier()
    }

    fn scan_invalid_character(&mut self) -> TokenKind {
        self.finish_token(TokenKind::Invalid)
    }

    fn finish_token(&mut self, kind: TokenKind) -> TokenKind {
        self.current_token_kind = kind;
        // Note: token_flags should be set before calling finish_token if needed
        kind
    }

    fn unescape_string(&self, text: &str) -> String {
        let mut result = String::new();
        let mut chars = text.chars().peekable();

        // Skip opening quote(s)
        if let Some(&'"') = chars.peek() {
            chars.next();
            if let Some(&'"') = chars.peek() {
                chars.next();
                if let Some(&'"') = chars.peek() {
                    chars.next();
                }
            }
        }

        let mut chars_vec: Vec<char> = chars.collect();
        chars_vec.pop(); // Remove trailing quote(s)
        if chars_vec.len() > 1
            && chars_vec[chars_vec.len() - 1] == '"'
            && chars_vec[chars_vec.len() - 2] == '"'
        {
            chars_vec.pop();
            chars_vec.pop();
        } else if !chars_vec.is_empty() && chars_vec[chars_vec.len() - 1] == '"' {
            chars_vec.pop();
        }

        let mut chars = chars_vec.into_iter().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\\' {
                if let Some(&next) = chars.peek() {
                    chars.next();
                    match next {
                        'r' => result.push('\r'),
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        '$' => result.push('$'),
                        '@' => result.push('@'),
                        '`' => result.push('`'),
                        _ => {
                            result.push('\\');
                            result.push(next);
                        }
                    }
                }
            } else {
                result.push(ch);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let mut lexer = Lexer::new("");
        assert!(lexer.eof());
    }

    #[test]
    fn test_keywords() {
        let keywords = vec![
            "model",
            "enum",
            "interface",
            "union",
            "namespace",
            "using",
            "import",
            "scalar",
            "op",
            "alias",
            "is",
            "extends",
            "if",
            "else",
            "projection",
            "fn",
            "valueof",
            "typeof",
            "const",
            "init",
            "dec",
            "true",
            "false",
            "return",
            "void",
            "never",
            "unknown",
            "extern",
            "internal",
        ];

        for kw in keywords {
            let mut lexer = Lexer::new(kw);
            let kind = lexer.scan();
            assert!(
                kind.is_keyword(),
                "Expected keyword for '{}', got {:?}",
                kw,
                kind
            );
            assert!(lexer.eof(), "Expected EOF after keyword '{}'", kw);
        }
    }

    #[test]
    fn test_identifier() {
        let mut lexer = Lexer::new("myIdentifier");
        let kind = lexer.scan();
        assert_eq!(kind, TokenKind::Identifier);
        assert_eq!(lexer.token_text(), "myIdentifier");
    }

    #[test]
    fn test_model_with_string_property() {
        let source = "model Car { make: string }";
        let mut lexer = Lexer::new(source);

        let mut tokens: Vec<(TokenKind, String)> = Vec::new();
        loop {
            let token = lexer.scan();
            let text = lexer.token_text().to_string();
            tokens.push((token, text));
            if token == TokenKind::EndOfFile {
                break;
            }
        }

        // Verify first token is model keyword
        assert!(tokens[0].0.is_keyword(), "First token should be keyword");
        assert_eq!(tokens[0].1, "model");

        // Second token should be identifier (Car)
        assert_eq!(
            tokens[1].0,
            TokenKind::Identifier,
            "Second token should be Identifier"
        );
        assert_eq!(tokens[1].1, "Car");

        // Find the 'string' token - it should be Identifier
        let string_tokens: Vec<_> = tokens.iter().filter(|t| t.1 == "string").collect();
        assert!(!string_tokens.is_empty(), "Should have 'string' token");
        assert_eq!(
            string_tokens[0].0,
            TokenKind::Identifier,
            "string should be Identifier, got {:?}",
            string_tokens[0].0
        );
    }

    #[test]
    fn test_ellipsis_token() {
        let source = "...BaseCar";
        let mut lexer = Lexer::new(source);

        loop {
            let token = lexer.scan();
            let text = lexer.token_text().to_string();
            let _ = (token, text);
            if token == TokenKind::EndOfFile {
                break;
            }
        }

        // Verify first token is Ellipsis
        let mut lexer2 = Lexer::new("...BaseCar");
        let first = lexer2.scan();
        assert_eq!(
            first,
            TokenKind::Ellipsis,
            "First token should be Ellipsis, got {:?}",
            first
        );
    }

    #[test]
    fn test_string_literal() {
        let mut lexer = Lexer::new("\"hello world\"");
        let kind = lexer.scan();
        assert_eq!(kind, TokenKind::StringLiteral);
    }

    #[test]
    fn test_numeric_literal() {
        let numbers = vec!["42", "3.14", "1e10", "0xFF", "0b1010"];

        for num in numbers {
            let mut lexer = Lexer::new(num);
            let kind = lexer.scan();
            assert_eq!(
                kind,
                TokenKind::NumericLiteral,
                "Expected numeric literal for '{}'",
                num
            );
        }
    }

    #[test]
    fn test_punctuation() {
        let cases = vec![
            ("(", TokenKind::OpenParen),
            (")", TokenKind::CloseParen),
            ("{", TokenKind::OpenBrace),
            ("}", TokenKind::CloseBrace),
            ("[", TokenKind::OpenBracket),
            ("]", TokenKind::CloseBracket),
            (",", TokenKind::Comma),
            (";", TokenKind::Semicolon),
            (":", TokenKind::Colon),
            ("::", TokenKind::ColonColon),
            ("...", TokenKind::Ellipsis),
            ("<", TokenKind::LessThan),
            (">", TokenKind::GreaterThan),
            ("<=", TokenKind::LessThanEquals),
            (">=", TokenKind::GreaterThanEquals),
            ("==", TokenKind::EqualsEquals),
            ("!=", TokenKind::ExclamationEquals),
            ("=>", TokenKind::EqualsGreaterThan),
            ("+", TokenKind::Plus),
            ("-", TokenKind::Hyphen),
            ("*", TokenKind::Star),
            ("/", TokenKind::ForwardSlash),
            ("&&", TokenKind::AmpersandAmpersand),
            ("||", TokenKind::BarBar),
            ("@", TokenKind::At),
            ("@@", TokenKind::AtAt),
            ("#", TokenKind::Hash),
            ("#{", TokenKind::HashBrace),
            ("#[", TokenKind::HashBracket),
        ];

        for (text, expected) in cases {
            let mut lexer = Lexer::new(text);
            let kind = lexer.scan();
            assert_eq!(
                kind, expected,
                "Expected {:?} for '{}', got {:?}",
                expected, text, kind
            );
        }
    }

    #[test]
    fn test_single_line_comment() {
        let mut lexer = Lexer::new("// this is a comment\nmodel");
        let kind = lexer.scan();
        assert_eq!(kind, TokenKind::SingleLineComment);

        // After comment, trivia (newlines, whitespace) is skipped to next token
        let kind = lexer.scan();
        assert_eq!(kind, TokenKind::ModelKeyword);
    }

    #[test]
    fn test_multi_line_comment() {
        let mut lexer = Lexer::new("/* multi\nline\ncomment */model");
        let kind = lexer.scan();
        assert_eq!(kind, TokenKind::MultiLineComment);

        // After comment, trivia is skipped to model
        let kind = lexer.scan();
        assert_eq!(kind, TokenKind::ModelKeyword);
    }

    #[test]
    fn test_whitespace() {
        let mut lexer = Lexer::new("   model");
        // Whitespace is skipped, first token is ModelKeyword
        let kind = lexer.scan();
        assert_eq!(kind, TokenKind::ModelKeyword);
    }
}
