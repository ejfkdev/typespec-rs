//! Lexer implementation for TypeSpec

use std::iter::Peekable;
use std::str::Chars;

/// Character code constants
struct CharCodes;

impl CharCodes {
    const NULL: u32 = 0x00;
    const MAX_ASCII: u32 = 0x7f;
    const BYTE_ORDER_MARK: u32 = 0xfeff;

    // Line breaks
    const LINE_FEED: u32 = 0x0a;
    const CARRIAGE_RETURN: u32 = 0x0d;

    // ASCII whitespace excluding line breaks
    const SPACE: u32 = 0x20;
    const TAB: u32 = 0x09;
    const VERTICAL_TAB: u32 = 0x0b;
    const FORM_FEED: u32 = 0x0c;

    // Non-ASCII whitespace excluding line breaks
    const NEXT_LINE: u32 = 0x0085;
    const LEFT_TO_RIGHT_MARK: u32 = 0x200e;
    const RIGHT_TO_LEFT_MARK: u32 = 0x200f;
    const LINE_SEPARATOR: u32 = 0x2028;
    const PARAGRAPH_SEPARATOR: u32 = 0x2029;

    // ASCII Digits
    const DIGIT_0: u32 = 0x30;
    const DIGIT_9: u32 = 0x39;

    // ASCII lowercase letters
    const LETTER_A: u32 = 0x61;
    const LETTER_Z: u32 = 0x7a;

    // ASCII uppercase letters
    const LETTER_A_UPPER: u32 = 0x41;
    const LETTER_Z_UPPER: u32 = 0x5a;

    // Non-letter, non-digit ASCII characters valid in identifiers
    const UNDERSCORE: u32 = 0x5f;
    const DOLLAR: u32 = 0x24;

    // ASCII punctuation
    const AMPERSAND: u32 = 0x26;
    const ASTERISK: u32 = 0x2a;
    const AT: u32 = 0x40;
    const BACKSLASH: u32 = 0x5c;
    const BACKTICK: u32 = 0x60;
    const BAR: u32 = 0x7c;
    const CLOSE_BRACE: u32 = 0x7d;
    const CLOSE_BRACKET: u32 = 0x5d;
    const CLOSE_PAREN: u32 = 0x29;
    const COLON: u32 = 0x3a;
    const COMMA: u32 = 0x2c;
    const DOT: u32 = 0x2e;
    const DOUBLE_QUOTE: u32 = 0x22;
    const EQUALS: u32 = 0x3d;
    const EXCLAMATION: u32 = 0x21;
    const GREATER_THAN: u32 = 0x3e;
    const HASH: u32 = 0x23;
    const LESS_THAN: u32 = 0x3c;
    const MINUS: u32 = 0x2d;
    const OPEN_BRACE: u32 = 0x7b;
    const OPEN_BRACKET: u32 = 0x5b;
    const OPEN_PAREN: u32 = 0x28;
    const PLUS: u32 = 0x2b;
    const QUESTION: u32 = 0x3f;
    const SEMICOLON: u32 = 0x3b;
    const SLASH: u32 = 0x2f;
}

fn char_code(c: char) -> u32 {
    c as u32
}

fn is_line_break(c: char) -> bool {
    let cc = char_code(c);
    cc == CharCodes::LINE_FEED || cc == CharCodes::CARRIAGE_RETURN
}

fn is_ascii_whitespace_single_line(c: char) -> bool {
    let cc = char_code(c);
    cc == CharCodes::SPACE
        || cc == CharCodes::TAB
        || cc == CharCodes::VERTICAL_TAB
        || cc == CharCodes::FORM_FEED
}

fn is_non_ascii_whitespace_single_line(c: char) -> bool {
    let cc = char_code(c);
    cc == CharCodes::NEXT_LINE
        || cc == CharCodes::LEFT_TO_RIGHT_MARK
        || cc == CharCodes::RIGHT_TO_LEFT_MARK
        || cc == CharCodes::LINE_SEPARATOR
        || cc == CharCodes::PARAGRAPH_SEPARATOR
}

fn is_whitespace(c: char) -> bool {
    is_ascii_whitespace_single_line(c) || is_line_break(c)
}

fn is_digit(c: char) -> bool {
    let cc = char_code(c);
    cc >= CharCodes::DIGIT_0 && cc <= CharCodes::DIGIT_9
}

fn is_hex_digit(c: char) -> bool {
    let cc = char_code(c);
    is_digit(c)
        || (cc >= CharCodes::LETTER_A_UPPER && cc <= CharCodes::LETTER_Z_UPPER)
        || (cc >= CharCodes::LETTER_A && cc <= CharCodes::LETTER_Z)
}

fn is_binary_digit(c: char) -> bool {
    c == '0' || c == '1'
}

fn is_lowercase_ascii_letter(c: char) -> bool {
    let cc = char_code(c);
    cc >= CharCodes::LETTER_A && cc <= CharCodes::LETTER_Z
}

fn is_ascii_identifier_start(c: char) -> bool {
    let cc = char_code(c);
    (cc >= CharCodes::LETTER_A_UPPER && cc <= CharCodes::LETTER_Z_UPPER)
        || (cc >= CharCodes::LETTER_A && cc <= CharCodes::LETTER_Z)
        || cc == CharCodes::DOLLAR
        || cc == CharCodes::UNDERSCORE
}

fn is_ascii_identifier_continue(c: char) -> bool {
    let cc = char_code(c);
    (cc >= CharCodes::LETTER_A_UPPER && cc <= CharCodes::LETTER_Z_UPPER)
        || (cc >= CharCodes::LETTER_A && cc <= CharCodes::LETTER_Z)
        || (cc >= CharCodes::DIGIT_0 && cc <= CharCodes::DIGIT_9)
        || cc == CharCodes::DOLLAR
        || cc == CharCodes::UNDERSCORE
}

/// Token flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TokenFlags(u8);

impl TokenFlags {
    pub const NONE: TokenFlags = TokenFlags(0);
    pub const ESCAPED: TokenFlags = TokenFlags(1 << 0);
    pub const TRIPLE_QUOTED: TokenFlags = TokenFlags(1 << 1);
    pub const UNTERMINATED: TokenFlags = TokenFlags(1 << 2);
    pub const NON_ASCII: TokenFlags = TokenFlags(1 << 3);
    pub const DOC_COMMENT: TokenFlags = TokenFlags(1 << 4);
    pub const BACKTICKED: TokenFlags = TokenFlags(1 << 5);

    pub fn contains(self, other: TokenFlags) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn insert(&mut self, other: TokenFlags) {
        self.0 |= other.0;
    }
}

/// Token kinds matching TypeSpec's Token enum
#[derive(Debug, Clone, PartialEq, Eq)]
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
        !self.is_trivia()
            && !matches!(
                self,
                TokenKind::None
                    | TokenKind::Invalid
                    | TokenKind::EndOfFile
                    | TokenKind::Identifier
                    | TokenKind::NumericLiteral
                    | TokenKind::StringLiteral
                    | TokenKind::StringTemplateHead
                    | TokenKind::StringTemplateMiddle
                    | TokenKind::StringTemplateTail
                    | TokenKind::SingleLineComment
                    | TokenKind::MultiLineComment
                    | TokenKind::NewLine
                    | TokenKind::Whitespace
                    | TokenKind::ConflictMarker
                    | TokenKind::DocText
                    | TokenKind::DocCodeSpan
                    | TokenKind::DocCodeFenceDelimiter
                    | TokenKind::OpenBrace
                    | TokenKind::CloseBrace
                    | TokenKind::OpenParen
                    | TokenKind::CloseParen
                    | TokenKind::OpenBracket
                    | TokenKind::CloseBracket
                    | TokenKind::Dot
                    | TokenKind::Ellipsis
                    | TokenKind::Semicolon
                    | TokenKind::Comma
                    | TokenKind::LessThan
                    | TokenKind::GreaterThan
                    | TokenKind::Equals
                    | TokenKind::Ampersand
                    | TokenKind::Bar
                    | TokenKind::Question
                    | TokenKind::Colon
                    | TokenKind::ColonColon
                    | TokenKind::At
                    | TokenKind::AtAt
                    | TokenKind::Hash
                    | TokenKind::HashBrace
                    | TokenKind::HashBracket
                    | TokenKind::Star
                    | TokenKind::ForwardSlash
                    | TokenKind::Plus
                    | TokenKind::Hyphen
                    | TokenKind::Exclamation
                    | TokenKind::LessThanEquals
                    | TokenKind::GreaterThanEquals
                    | TokenKind::AmpersandAmpersand
                    | TokenKind::BarBar
                    | TokenKind::EqualsEquals
                    | TokenKind::ExclamationEquals
                    | TokenKind::EqualsGreaterThan
            )
    }

    pub fn is_punctuation(&self) -> bool {
        matches!(
            self,
            TokenKind::OpenBrace
                | TokenKind::CloseBrace
                | TokenKind::OpenParen
                | TokenKind::CloseParen
                | TokenKind::OpenBracket
                | TokenKind::CloseBracket
                | TokenKind::Dot
                | TokenKind::Ellipsis
                | TokenKind::Semicolon
                | TokenKind::Comma
                | TokenKind::LessThan
                | TokenKind::GreaterThan
                | TokenKind::Equals
                | TokenKind::Ampersand
                | TokenKind::Bar
                | TokenKind::Question
                | TokenKind::Colon
                | TokenKind::ColonColon
                | TokenKind::At
                | TokenKind::AtAt
                | TokenKind::Hash
                | TokenKind::HashBrace
                | TokenKind::HashBracket
                | TokenKind::Star
                | TokenKind::ForwardSlash
                | TokenKind::Plus
                | TokenKind::Hyphen
                | TokenKind::Exclamation
                | TokenKind::LessThanEquals
                | TokenKind::GreaterThanEquals
                | TokenKind::AmpersandAmpersand
                | TokenKind::BarBar
                | TokenKind::EqualsEquals
                | TokenKind::ExclamationEquals
                | TokenKind::EqualsGreaterThan
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
        Position {
            line: 1,
            column: 0,
        }
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

/// Token with kind, span, and value
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub value: String,
    pub flags: TokenFlags,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, value: String) -> Self {
        Token {
            kind,
            span,
            value,
            flags: TokenFlags::NONE,
        }
    }

    pub fn with_flags(mut self, flags: TokenFlags) -> Self {
        self.flags = flags;
        self
    }
}

/// Keywords lookup map
fn get_keywords() -> std::collections::HashMap<&'static str, TokenKind> {
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
    keywords.insert("prop", TokenKind::PropKeyword);
    keywords.insert("property", TokenKind::PropertyKeyword);
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
    keywords: std::collections::HashMap<&'static str, TokenKind>,
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
            keywords: get_keywords(),
        }
    }

    /// Get the current position in the source
    pub fn position(&self) -> Position {
        self.position
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
            TokenKind::StringLiteral
            | TokenKind::StringTemplateHead
            | TokenKind::StringTemplateMiddle
            | TokenKind::StringTemplateTail => self.unescape_string(text),
            TokenKind::Identifier => text.to_string(),
            _ => text.to_string(),
        }
    }

    /// Check if we've reached the end of input
    pub fn eof(&mut self) -> bool {
        self.chars.peek().is_none()
    }

    /// Scan the next token
    pub fn scan(&mut self) -> TokenKind {
        self.token_start = self.position;
        self.current_span_start = self.chars_consumed;

        if self.eof() {
            return self.finish_token(TokenKind::EndOfFile);
        }

        let ch = self.chars.next().unwrap();
        self.chars_consumed += ch.len_utf8();

        match ch {
            '\n' | '\r' => {
                if ch == '\r' {
                    if let Some(&'\n') = self.chars.peek() {
                        self.chars.next();
                        self.chars_consumed += 1;
                    }
                }
                self.position.line += 1;
                self.position.column = 0;
                self.finish_token(TokenKind::NewLine)
            }

            ' ' | '\t' | '\u{0b}' | '\u{0c}' => self.scan_whitespace(),

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

            '#' => {
                match self.chars.peek().copied() {
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
                }
            }

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
                // Check for ellipsis (...)
                let mut lookahead = self.chars.clone();
                if lookahead.peek() == Some(&'.') {
                    lookahead.next();
                    if lookahead.peek() == Some(&'.') {
                        // This is an ellipsis
                        self.chars = lookahead;
                        self.chars_consumed = self.current_span_start + 3;
                        return self.finish_token(TokenKind::Ellipsis);
                    }
                }
                self.finish_token(TokenKind::Dot)
            }

            '/' => {
                match self.chars.peek().copied() {
                    Some('/') => self.scan_single_line_comment(),
                    Some('*') => self.scan_multi_line_comment(),
                    _ => self.finish_token(TokenKind::ForwardSlash),
                }
            }

            '0' => {
                match self.chars.peek().copied() {
                    Some('x') | Some('X') => self.scan_hex_number(),
                    Some('b') | Some('B') => self.scan_binary_number(),
                    _ => self.scan_number(),
                }
            }

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
                let cc = char_code(ch);
                if is_lowercase_ascii_letter(ch) {
                    self.scan_identifier_or_keyword()
                } else if is_ascii_identifier_start(ch) {
                    self.scan_identifier()
                } else if cc <= 0x7f {
                    self.scan_invalid_character()
                } else {
                    self.scan_non_ascii_token()
                }
            }
        }
    }

    /// Scan doc comment content
    #[allow(dead_code)]
    pub fn scan_doc(&mut self) -> TokenKind {
        self.token_start = self.position;
        self.current_span_start = self.chars_consumed;

        if self.eof() {
            return self.finish_token(TokenKind::EndOfFile);
        }

        let ch = self.chars.next().unwrap();
        self.chars_consumed += ch.len_utf8();

        match ch {
            '\n' | '\r' => {
                if ch == '\r' {
                    if let Some(&'\n') = self.chars.peek() {
                        self.chars.next();
                        self.chars_consumed += 1;
                    }
                }
                self.position.line += 1;
                self.position.column = 0;
                self.finish_token(TokenKind::NewLine)
            }

            '\\' => {
                if let Some(&'@') = self.chars.peek() {
                    self.chars.next();
                    self.chars_consumed += 1;
                    self.finish_token(TokenKind::DocText)
                } else {
                    self.finish_token(TokenKind::DocText)
                }
            }

            ' ' | '\t' | '\u{0b}' | '\u{0c}' => self.scan_whitespace(),

            '}' => self.finish_token(TokenKind::CloseBrace),
            '@' => self.finish_token(TokenKind::At),
            '*' => self.finish_token(TokenKind::Star),
            '`' => {
                let mut lookahead = self.chars.clone();
                if lookahead.peek() == Some(&'`') {
                    lookahead.next();
                    if lookahead.peek() == Some(&'`') {
                        self.chars = lookahead;
                        self.chars_consumed += 2;
                        return self.finish_token(TokenKind::DocCodeFenceDelimiter);
                    }
                }
                self.scan_doc_code_span()
            }

            '<' | '>' | '=' | '|' => {
                if self.at_conflict_marker() {
                    self.scan_conflict_marker()
                } else {
                    self.finish_token(TokenKind::DocText)
                }
            }

            '-' => self.finish_token(TokenKind::Hyphen),

            _ => {
                if is_ascii_identifier_start(ch) {
                    self.scan_identifier()
                } else {
                    self.finish_token(TokenKind::DocText)
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
            self.scan_required_digits();
        }

        // Check for exponent
        if let Some(&'e') | Some(&'E') = self.chars.peek() {
            self.chars.next();
            self.chars_consumed += 1;
            if let Some(&'+') | Some(&'-') = self.chars.peek() {
                self.chars.next();
                self.chars_consumed += 1;
            }
            self.scan_required_digits();
        }

        self.finish_token(TokenKind::NumericLiteral)
    }

    fn scan_required_digits(&mut self) {
        if let Some(&ch) = self.chars.peek() {
            if is_digit(ch) {
                self.chars.next();
                self.chars_consumed += ch.len_utf8();
                while let Some(&ch) = self.chars.peek() {
                    if is_digit(ch) {
                        self.chars.next();
                        self.chars_consumed += ch.len_utf8();
                    } else {
                        break;
                    }
                }
            }
        }
    }

    fn scan_signed_number(&mut self) -> TokenKind {
        self.chars.next(); // consume +/-
        self.chars_consumed += 1;
        self.scan_number()
    }

    fn scan_hex_number(&mut self) -> TokenKind {
        self.chars.next(); // consume x/X
        self.chars_consumed += 1;

        if let Some(&ch) = self.chars.peek() {
            if !is_hex_digit(ch) {
                // Error: hex digit expected
                return self.finish_token(TokenKind::NumericLiteral);
            }
        }

        while let Some(&ch) = self.chars.peek() {
            if is_hex_digit(ch) {
                self.chars.next();
                self.chars_consumed += ch.len_utf8();
            } else {
                break;
            }
        }

        self.finish_token(TokenKind::NumericLiteral)
    }

    fn scan_binary_number(&mut self) -> TokenKind {
        self.chars.next(); // consume b/B
        self.chars_consumed += 1;

        if let Some(&ch) = self.chars.peek() {
            if !is_binary_digit(ch) {
                // Error: binary digit expected
                return self.finish_token(TokenKind::NumericLiteral);
            }
        }

        while let Some(&ch) = self.chars.peek() {
            if is_binary_digit(ch) {
                self.chars.next();
                self.chars_consumed += ch.len_utf8();
            } else {
                break;
            }
        }

        self.finish_token(TokenKind::NumericLiteral)
    }

    fn scan_string(&mut self, flags: TokenFlags) -> TokenKind {
        let is_triple = flags.contains(TokenFlags::TRIPLE_QUOTED);
        let template_part = if is_triple {
            TokenKind::StringTemplateHead
        } else {
            TokenKind::StringLiteral
        };

        loop {
            if self.eof() {
                return self.finish_token(TokenKind::StringLiteral);
            }

            let ch = self.chars.next().unwrap();
            self.chars_consumed += ch.len_utf8();

            match ch {
                '\\' => {
                    if let Some(esc_ch) = self.chars.next() {
                        self.chars_consumed += esc_ch.len_utf8();
                    }
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
                        return template_part;
                    }
                }
                '\n' | '\r' => {
                    if !is_triple {
                        self.position.line += 1;
                        self.position.column = 0;
                        return self.finish_token(TokenKind::StringLiteral);
                    }
                    if ch == '\r' {
                        if let Some(&'\n') = self.chars.peek() {
                            self.chars.next();
                            self.chars_consumed += 1;
                        }
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

            let ch = self.chars.next().unwrap();
            self.chars_consumed += ch.len_utf8();

            match ch {
                '`' => {
                    return self.finish_token(TokenKind::Identifier);
                }
                '\\' => {
                    if let Some(_) = self.chars.next() {
                        self.chars_consumed += 1;
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
        if count >= 2 && count <= 12 {
            if let Some(keyword) = self.keywords.get(text) {
                return self.finish_token(keyword.clone());
            }
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
        if self.source.as_bytes()[pos - 1] == b'\n'
            || self.source.as_bytes()[pos - 1] == b'\r'
        {
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
            // or ======= alone
            matches!(marker_char, b'<' | b'>' | b'=')
        } else {
            false
        }
    }

    fn scan_conflict_marker(&mut self) -> TokenKind {
        // Consume 7 same characters
        let marker_char = self.chars.next().unwrap();
        for _ in 1..7 {
            self.chars.next();
            self.chars_consumed += 1;
        }

        if marker_char == '<' || marker_char == '>' {
            // Consume to end of line
            while let Some(ch) = self.chars.next() {
                self.chars_consumed += ch.len_utf8();
                if is_line_break(ch) {
                    if ch == '\r' {
                        if let Some(&'\n') = self.chars.peek() {
                            self.chars.next();
                            self.chars_consumed += 1;
                        }
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
                    if ch == '\r' {
                        if let Some(&'\n') = self.chars.peek() {
                            self.chars.next();
                            self.chars_consumed += 1;
                        }
                    }
                    self.position.line += 1;
                    self.position.column = 0;
                }
            }
        }

        self.finish_token(TokenKind::ConflictMarker)
    }

    fn scan_non_ascii_token(&mut self) -> TokenKind {
        if let Some(&ch) = self.chars.peek() {
            if is_non_ascii_whitespace_single_line(ch) {
                return self.scan_whitespace();
            }
        }
        self.scan_identifier()
    }

    fn scan_invalid_character(&mut self) -> TokenKind {
        self.finish_token(TokenKind::Invalid)
    }

    fn scan_doc_code_span(&mut self) -> TokenKind {
        // Consume first backtick already
        while let Some(ch) = self.chars.next() {
            self.chars_consumed += ch.len_utf8();
            if ch == '`' {
                return self.finish_token(TokenKind::DocCodeSpan);
            }
            if is_line_break(ch) {
                return self.finish_token(TokenKind::DocCodeSpan);
            }
        }
        self.finish_token(TokenKind::DocCodeSpan)
    }

    fn finish_token(&mut self, kind: TokenKind) -> TokenKind {
        self.current_token_kind = kind.clone();
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
        if chars_vec.len() > 1 && chars_vec[chars_vec.len() - 1] == '"' && chars_vec[chars_vec.len() - 2] == '"' {
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

    /// Get the full token with all information
    #[allow(dead_code)]
    pub fn get_token(&self) -> Token {
        let text = self.token_text().to_string();
        Token::new(
            self.current_token_kind.clone(),
            Span::new(self.token_start, self.position),
            text,
        )
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
            "model", "enum", "interface", "union", "namespace", "using", "import", "scalar",
            "op", "alias", "is", "extends", "if", "else", "projection", "fn", "valueof",
            "typeof", "const", "init", "dec", "true", "false", "return", "void", "never",
            "unknown", "extern", "internal",
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
            assert!(
                lexer.eof(),
                "Expected EOF after keyword '{}'",
                kw
            );
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

        // After comment, should be NewLine, then skip to model
        let kind = lexer.scan();
        assert_eq!(kind, TokenKind::NewLine);

        let kind = lexer.scan();
        assert_eq!(kind, TokenKind::ModelKeyword);
    }

    #[test]
    fn test_multi_line_comment() {
        let mut lexer = Lexer::new("/* multi\nline\ncomment */model");
        let kind = lexer.scan();
        assert_eq!(kind, TokenKind::MultiLineComment);

        // After comment, skip trivia to model
        let kind = lexer.scan();
        assert_eq!(kind, TokenKind::ModelKeyword);
    }

    #[test]
    fn test_whitespace() {
        let mut lexer = Lexer::new("   model");
        let kind = lexer.scan();
        assert_eq!(kind, TokenKind::Whitespace);

        let kind = lexer.scan();
        assert_eq!(kind, TokenKind::ModelKeyword);
    }
}
