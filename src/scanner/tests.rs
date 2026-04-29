//! Scanner/Lexer Tests
//!
//! Ported from TypeSpec compiler/test/scanner.test.ts
//!
//! These tests verify the tokenization functionality.

#[cfg(test)]
mod tests {
    use crate::scanner::{Lexer, TokenKind};

    /// Helper to scan all tokens from input
    fn scan_all(source: &str) -> Vec<(TokenKind, String)> {
        let mut lexer = Lexer::new(source);
        let mut tokens = Vec::new();
        loop {
            let kind = lexer.scan();
            let text = lexer.token_text().to_string();
            let is_eof = std::mem::discriminant(&kind) == std::mem::discriminant(&TokenKind::EndOfFile);
            tokens.push((kind, text));
            if is_eof {
                break;
            }
        }
        tokens
    }

    /// Helper to scan and get token kinds
    fn token_kinds(source: &str) -> Vec<TokenKind> {
        let mut lexer = Lexer::new(source);
        let mut kinds = Vec::new();
        loop {
            let kind = lexer.scan();
            let is_eof = std::mem::discriminant(&kind) == std::mem::discriminant(&TokenKind::EndOfFile);
            kinds.push(kind);
            if is_eof {
                break;
            }
        }
        kinds
    }

    /// Helper to get token values (processed)
    fn token_values(source: &str) -> Vec<String> {
        let mut lexer = Lexer::new(source);
        let mut values = Vec::new();
        loop {
            let kind = lexer.scan();
            let is_eof = std::mem::discriminant(&kind) == std::mem::discriminant(&TokenKind::EndOfFile);
            values.push(lexer.token_value());
            if is_eof {
                break;
            }
        }
        values
    }

    // ==================== Basic Tokenization Tests ====================

    #[test]
    fn test_scan_model_keyword() {
        let tokens = scan_all("model");
        assert_eq!(tokens.len(), 2); // model + EOF
        assert_eq!(tokens[0].0, TokenKind::ModelKeyword);
        assert_eq!(tokens[0].1, "model");
    }

    #[test]
    fn test_scan_identifier() {
        let tokens = scan_all("foo");
        assert_eq!(tokens.len(), 2); // foo + EOF
        assert_eq!(tokens[0].0, TokenKind::Identifier);
        assert_eq!(tokens[0].1, "foo");
    }

    #[test]
    fn test_scan_string_literal() {
        let tokens = scan_all("\"hello\"");
        assert_eq!(tokens.len(), 2); // string + EOF
        assert_eq!(tokens[0].0, TokenKind::StringLiteral);
        assert_eq!(tokens[0].1, "\"hello\"");
    }

    #[test]
    fn test_scan_numeric_literal() {
        let tokens = scan_all("42");
        assert_eq!(tokens.len(), 2); // number + EOF
        assert_eq!(tokens[0].0, TokenKind::NumericLiteral);
        assert_eq!(tokens[0].1, "42");
    }

    // ==================== Whitespace and Trivia Tests ====================

    #[test]
    fn test_skip_whitespace() {
        // By default, scan() skips whitespace
        let kinds = token_kinds("   foo   ");
        assert_eq!(kinds.len(), 2); // Identifier + EOF (whitespace skipped)
        assert_eq!(kinds[0], TokenKind::Identifier);
    }

    #[test]
    fn test_scan_with_whitespace() {
        let tokens = scan_all("foo bar");
        assert_eq!(tokens.len(), 3); // foo + bar + EOF
        assert_eq!(tokens[0].0, TokenKind::Identifier);
        assert_eq!(tokens[1].0, TokenKind::Identifier);
    }

    // ==================== Punctuation Tests ====================

    #[test]
    fn test_scan_open_brace() {
        let tokens = scan_all("{");
        assert_eq!(tokens.len(), 2); // { + EOF
        assert_eq!(tokens[0].0, TokenKind::OpenBrace);
    }

    #[test]
    fn test_scan_close_brace() {
        let tokens = scan_all("}");
        assert_eq!(tokens.len(), 2); // } + EOF
        assert_eq!(tokens[0].0, TokenKind::CloseBrace);
    }

    #[test]
    fn test_scan_colon() {
        let tokens = scan_all(":");
        assert_eq!(tokens.len(), 2); // : + EOF
        assert_eq!(tokens[0].0, TokenKind::Colon);
    }

    #[test]
    fn test_scan_semicolon() {
        let tokens = scan_all(";");
        assert_eq!(tokens.len(), 2); // ; + EOF
        assert_eq!(tokens[0].0, TokenKind::Semicolon);
    }

    #[test]
    fn test_scan_comma() {
        let tokens = scan_all(",");
        assert_eq!(tokens.len(), 2); // , + EOF
        assert_eq!(tokens[0].0, TokenKind::Comma);
    }

    #[test]
    fn test_scan_dot() {
        let tokens = scan_all(".");
        assert_eq!(tokens.len(), 2); // . + EOF
        assert_eq!(tokens[0].0, TokenKind::Dot);
    }

    #[test]
    fn test_scan_ellipsis() {
        let tokens = scan_all("...");
        assert_eq!(tokens.len(), 2); // ... + EOF
        assert_eq!(tokens[0].0, TokenKind::Ellipsis);
    }

    // ==================== Operators Tests ====================

    #[test]
    fn test_scan_ampersand() {
        let tokens = scan_all("&");
        assert_eq!(tokens.len(), 2); // & + EOF
        assert_eq!(tokens[0].0, TokenKind::Ampersand);
    }

    #[test]
    fn test_scan_bar() {
        let tokens = scan_all("|");
        assert_eq!(tokens.len(), 2); // | + EOF
        assert_eq!(tokens[0].0, TokenKind::Bar);
    }

    #[test]
    fn test_scan_ampersand_ampersand() {
        let tokens = scan_all("&&");
        assert_eq!(tokens.len(), 2); // && + EOF
        assert_eq!(tokens[0].0, TokenKind::AmpersandAmpersand);
    }

    #[test]
    fn test_scan_bar_bar() {
        let tokens = scan_all("||");
        assert_eq!(tokens.len(), 2); // || + EOF
        assert_eq!(tokens[0].0, TokenKind::BarBar);
    }

    #[test]
    fn test_scan_equals_equals() {
        let tokens = scan_all("==");
        assert_eq!(tokens.len(), 2); // == + EOF
        assert_eq!(tokens[0].0, TokenKind::EqualsEquals);
    }

    #[test]
    fn test_scan_exclamation_equals() {
        let tokens = scan_all("!=");
        assert_eq!(tokens.len(), 2); // != + EOF
        assert_eq!(tokens[0].0, TokenKind::ExclamationEquals);
    }

    #[test]
    fn test_scan_less_than_equals() {
        let tokens = scan_all("<=");
        assert_eq!(tokens.len(), 2); // <= + EOF
        assert_eq!(tokens[0].0, TokenKind::LessThanEquals);
    }

    #[test]
    fn test_scan_greater_than_equals() {
        let tokens = scan_all(">=");
        assert_eq!(tokens.len(), 2); // >= + EOF
        assert_eq!(tokens[0].0, TokenKind::GreaterThanEquals);
    }

    #[test]
    fn test_scan_equals_greater_than() {
        let tokens = scan_all("=>");
        assert_eq!(tokens.len(), 2); // => + EOF
        assert_eq!(tokens[0].0, TokenKind::EqualsGreaterThan);
    }

    // ==================== Keywords Tests ====================

    #[test]
    fn test_scan_model_keyword_token() {
        let kinds = token_kinds("model Foo {}");
        assert_eq!(kinds[0], TokenKind::ModelKeyword);
        assert_eq!(kinds[1], TokenKind::Identifier);
        assert_eq!(kinds[2], TokenKind::OpenBrace);
        assert_eq!(kinds[3], TokenKind::CloseBrace);
    }

    #[test]
    fn test_scan_namespace_keyword() {
        let kinds = token_kinds("namespace Foo");
        assert_eq!(kinds[0], TokenKind::NamespaceKeyword);
        assert_eq!(kinds[1], TokenKind::Identifier);
    }

    #[test]
    fn test_scan_scalar_keyword() {
        let kinds = token_kinds("scalar uuid");
        assert_eq!(kinds[0], TokenKind::ScalarKeyword);
        assert_eq!(kinds[1], TokenKind::Identifier);
    }

    #[test]
    fn test_scan_interface_keyword() {
        let kinds = token_kinds("interface Foo");
        assert_eq!(kinds[0], TokenKind::InterfaceKeyword);
        assert_eq!(kinds[1], TokenKind::Identifier);
    }

    #[test]
    fn test_scan_union_keyword() {
        let kinds = token_kinds("union Foo");
        assert_eq!(kinds[0], TokenKind::UnionKeyword);
        assert_eq!(kinds[1], TokenKind::Identifier);
    }

    #[test]
    fn test_scan_enum_keyword() {
        let kinds = token_kinds("enum Foo");
        assert_eq!(kinds[0], TokenKind::EnumKeyword);
        assert_eq!(kinds[1], TokenKind::Identifier);
    }

    #[test]
    fn test_scan_op_keyword() {
        let kinds = token_kinds("op foo()");
        assert_eq!(kinds[0], TokenKind::OpKeyword);
        assert_eq!(kinds[1], TokenKind::Identifier);
        assert_eq!(kinds[2], TokenKind::OpenParen);
        assert_eq!(kinds[3], TokenKind::CloseParen);
    }

    #[test]
    fn test_scan_alias_keyword() {
        let kinds = token_kinds("alias Foo = string");
        assert_eq!(kinds[0], TokenKind::AliasKeyword);
        assert_eq!(kinds[1], TokenKind::Identifier);
        assert_eq!(kinds[2], TokenKind::Equals);
        assert_eq!(kinds[3], TokenKind::Identifier); // string
    }

    #[test]
    fn test_scan_dec_keyword() {
        let kinds = token_kinds("dec foo");
        assert_eq!(kinds[0], TokenKind::DecKeyword);
        assert_eq!(kinds[1], TokenKind::Identifier);
    }

    #[test]
    fn test_scan_const_keyword() {
        let kinds = token_kinds("const foo = 42");
        assert_eq!(kinds[0], TokenKind::ConstKeyword);
        assert_eq!(kinds[1], TokenKind::Identifier);
        assert_eq!(kinds[2], TokenKind::Equals);
        assert_eq!(kinds[3], TokenKind::NumericLiteral);
    }

    // ==================== Modifier Keywords Tests ====================

    #[test]
    fn test_scan_extern_keyword() {
        let kinds = token_kinds("extern dec foo");
        assert_eq!(kinds[0], TokenKind::ExternKeyword);
        assert_eq!(kinds[1], TokenKind::DecKeyword);
    }

    #[test]
    fn test_scan_internal_keyword() {
        let kinds = token_kinds("internal model Foo");
        assert_eq!(kinds[0], TokenKind::InternalKeyword);
        assert_eq!(kinds[1], TokenKind::ModelKeyword);
    }

    // ==================== Other Keywords Tests ====================

    #[test]
    fn test_scan_extends_keyword() {
        let kinds = token_kinds("model Foo extends Bar {}");
        assert_eq!(kinds[0], TokenKind::ModelKeyword);
        assert_eq!(kinds[1], TokenKind::Identifier); // Foo
        assert_eq!(kinds[2], TokenKind::ExtendsKeyword);
        assert_eq!(kinds[3], TokenKind::Identifier); // Bar
    }

    #[test]
    fn test_scan_is_keyword() {
        let kinds = token_kinds("model Foo is Bar {}");
        assert_eq!(kinds[0], TokenKind::ModelKeyword);
        assert_eq!(kinds[1], TokenKind::Identifier); // Foo
        assert_eq!(kinds[2], TokenKind::IsKeyword);
        assert_eq!(kinds[3], TokenKind::Identifier); // Bar
    }

    #[test]
    fn test_scan_true_keyword() {
        let kinds = token_kinds("true");
        assert_eq!(kinds[0], TokenKind::TrueKeyword);
    }

    #[test]
    fn test_scan_false_keyword() {
        let kinds = token_kinds("false");
        assert_eq!(kinds[0], TokenKind::FalseKeyword);
    }

    #[test]
    fn test_scan_void_keyword() {
        let kinds = token_kinds("void");
        assert_eq!(kinds[0], TokenKind::VoidKeyword);
    }

    #[test]
    fn test_scan_never_keyword() {
        let kinds = token_kinds("never");
        assert_eq!(kinds[0], TokenKind::NeverKeyword);
    }

    #[test]
    fn test_scan_unknown_keyword() {
        let kinds = token_kinds("unknown");
        assert_eq!(kinds[0], TokenKind::UnknownKeyword);
    }

    #[test]
    fn test_scan_valueof_keyword() {
        let kinds = token_kinds("valueof string");
        assert_eq!(kinds[0], TokenKind::ValueOfKeyword);
        assert_eq!(kinds[1], TokenKind::Identifier); // string
    }

    #[test]
    fn test_scan_typeof_keyword() {
        let kinds = token_kinds("typeof foo");
        assert_eq!(kinds[0], TokenKind::TypeOfKeyword);
        assert_eq!(kinds[1], TokenKind::Identifier); // foo
    }

    #[test]
    fn test_scan_return_keyword() {
        let kinds = token_kinds("return foo");
        assert_eq!(kinds[0], TokenKind::ReturnKeyword);
        assert_eq!(kinds[1], TokenKind::Identifier); // foo
    }

    // ==================== Decorator Tests ====================

    #[test]
    fn test_scan_decorator_at() {
        let kinds = token_kinds("@foo");
        assert_eq!(kinds[0], TokenKind::At);
        assert_eq!(kinds[1], TokenKind::Identifier); // foo
    }

    #[test]
    fn test_scan_decorator_at_at() {
        let kinds = token_kinds("@@foo");
        assert_eq!(kinds[0], TokenKind::AtAt);
        assert_eq!(kinds[1], TokenKind::Identifier); // foo
    }

    #[test]
    fn test_scan_decorator_with_args() {
        let kinds = token_kinds("@foo(1, \"hello\")");
        assert_eq!(kinds[0], TokenKind::At);
        assert_eq!(kinds[1], TokenKind::Identifier); // foo
        assert_eq!(kinds[2], TokenKind::OpenParen);
        assert_eq!(kinds[3], TokenKind::NumericLiteral);
        assert_eq!(kinds[4], TokenKind::Comma);
        assert_eq!(kinds[5], TokenKind::StringLiteral);
        assert_eq!(kinds[6], TokenKind::CloseParen);
    }

    // ==================== Directive Tests ====================

    #[test]
    fn test_scan_directive_hash() {
        let kinds = token_kinds("#suppress foo \"bar\"");
        assert_eq!(kinds[0], TokenKind::Hash);
        assert_eq!(kinds[1], TokenKind::Identifier); // suppress
        assert_eq!(kinds[2], TokenKind::Identifier); // foo
        assert_eq!(kinds[3], TokenKind::StringLiteral);
    }

    // ==================== String Tests ====================

    #[test]
    fn test_scan_empty_string() {
        let values = token_values("\"\"");
        // Empty string may produce multiple tokens depending on scanner implementation
        assert!(!values.is_empty());
        assert_eq!(values[0], "");
    }

    #[test]
    fn test_scan_simple_string() {
        let values = token_values("\"hello\"");
        assert_eq!(values.len(), 2); // "hello" + EOF
        assert_eq!(values[0], "hello");
    }

    #[test]
    fn test_scan_string_with_escape() {
        let values = token_values("\"hello\\nworld\"");
        assert_eq!(values.len(), 2);
        assert_eq!(values[0], "hello\nworld");
    }

    // ==================== Numeric Literal Tests ====================

    #[test]
    fn test_scan_decimal() {
        let tokens = scan_all("42");
        assert_eq!(tokens[0].0, TokenKind::NumericLiteral);
        assert_eq!(tokens[0].1, "42");
    }

    #[test]
    fn test_scan_hex() {
        let tokens = scan_all("0xBEEF");
        assert_eq!(tokens[0].0, TokenKind::NumericLiteral);
        assert_eq!(tokens[0].1, "0xBEEF");
    }

    #[test]
    fn test_scan_binary() {
        let tokens = scan_all("0b1010");
        assert_eq!(tokens[0].0, TokenKind::NumericLiteral);
        assert_eq!(tokens[0].1, "0b1010");
    }

    #[test]
    fn test_scan_float() {
        let tokens = scan_all("1.5");
        assert_eq!(tokens[0].0, TokenKind::NumericLiteral);
        assert_eq!(tokens[0].1, "1.5");
    }

    #[test]
    fn test_scan_scientific() {
        let tokens = scan_all("1e10");
        assert_eq!(tokens[0].0, TokenKind::NumericLiteral);
        assert_eq!(tokens[0].1, "1e10");
    }

    // ==================== Model Declaration Tests ====================

    #[test]
    fn test_scan_model_declaration() {
        let kinds = token_kinds("model Foo { x: string }");
        assert_eq!(kinds[0], TokenKind::ModelKeyword);
        assert_eq!(kinds[1], TokenKind::Identifier); // Foo
        assert_eq!(kinds[2], TokenKind::OpenBrace);
        assert_eq!(kinds[3], TokenKind::Identifier); // x
        assert_eq!(kinds[4], TokenKind::Colon);
        assert_eq!(kinds[5], TokenKind::Identifier); // string
        assert_eq!(kinds[6], TokenKind::CloseBrace);
    }

    #[test]
    fn test_scan_model_with_multiple_properties() {
        let kinds = token_kinds("model Foo { x: string; y: int32 }");
        assert_eq!(kinds[0], TokenKind::ModelKeyword);
        assert_eq!(kinds[1], TokenKind::Identifier); // Foo
        assert_eq!(kinds[2], TokenKind::OpenBrace);
        assert_eq!(kinds[3], TokenKind::Identifier); // x
        assert_eq!(kinds[4], TokenKind::Colon);
        assert_eq!(kinds[5], TokenKind::Identifier); // string
        assert_eq!(kinds[6], TokenKind::Semicolon);
        assert_eq!(kinds[7], TokenKind::Identifier); // y
        assert_eq!(kinds[8], TokenKind::Colon);
        assert_eq!(kinds[9], TokenKind::Identifier); // int32
        assert_eq!(kinds[10], TokenKind::CloseBrace);
    }

    // ==================== Backtick Identifier Tests ====================

    #[test]
    fn test_scan_backtick_identifier() {
        let kinds = token_kinds("`foo`");
        assert_eq!(kinds[0], TokenKind::Identifier);
    }

    #[test]
    fn test_scan_backtick_identifier_with_special_chars() {
        let kinds = token_kinds("`01-01`");
        assert_eq!(kinds[0], TokenKind::Identifier);
    }

    #[test]
    fn test_scan_backtick_identifier_with_space() {
        let kinds = token_kinds("`aa x`");
        assert_eq!(kinds[0], TokenKind::Identifier);
    }

    #[test]
    fn test_scan_reserved_word_as_identifier() {
        // Reserved words like "import" can be used as identifiers with backticks
        let kinds = token_kinds("`import`");
        assert_eq!(kinds[0], TokenKind::Identifier);
    }

    // ==================== Comments Tests ====================

    #[test]
    fn test_scan_single_line_comment() {
        let kinds = token_kinds("// comment\nfoo");
        // Comment is returned, then identifier, newline, and EOF
        assert_eq!(kinds[0], TokenKind::SingleLineComment);
        assert_eq!(kinds[1], TokenKind::Identifier); // foo
    }

    #[test]
    fn test_scan_multi_line_comment() {
        let kinds = token_kinds("/* comment */foo");
        // Multi-line comment is returned, then identifier and EOF
        assert_eq!(kinds[0], TokenKind::MultiLineComment);
        assert_eq!(kinds[1], TokenKind::Identifier); // foo
    }

    // ==================== Intersection Tests ====================

    #[test]
    fn test_scan_intersection() {
        let kinds = token_kinds("A&B");
        assert_eq!(kinds[0], TokenKind::Identifier); // A
        assert_eq!(kinds[1], TokenKind::Ampersand);
        assert_eq!(kinds[2], TokenKind::Identifier); // B
    }

    // ==================== Projection Keywords Tests ====================

    #[test]
    fn test_scan_projection_keyword() {
        let kinds = token_kinds("projection foo");
        assert_eq!(kinds[0], TokenKind::ProjectionKeyword);
        assert_eq!(kinds[1], TokenKind::Identifier);
    }

    #[test]
    fn test_scan_if_keyword() {
        let kinds = token_kinds("if");
        assert_eq!(kinds[0], TokenKind::IfKeyword);
    }

    // ==================== Template/Generic Tests ====================

    #[test]
    fn test_scan_less_than_for_generics() {
        let kinds = token_kinds("Foo<T>");
        assert_eq!(kinds[0], TokenKind::Identifier); // Foo
        assert_eq!(kinds[1], TokenKind::LessThan); // <
        assert_eq!(kinds[2], TokenKind::Identifier); // T
        assert_eq!(kinds[3], TokenKind::GreaterThan); // >
    }

    #[test]
    fn test_scan_template_parameters() {
        let kinds = token_kinds("model Foo<T, U> { }");
        assert_eq!(kinds[0], TokenKind::ModelKeyword);
        assert_eq!(kinds[1], TokenKind::Identifier); // Foo
        assert_eq!(kinds[2], TokenKind::LessThan); // <
        assert_eq!(kinds[3], TokenKind::Identifier); // T
        assert_eq!(kinds[4], TokenKind::Comma); // ,
        assert_eq!(kinds[5], TokenKind::Identifier); // U
        assert_eq!(kinds[6], TokenKind::GreaterThan); // >
    }

    // ==================== Function Type Tests ====================

    #[test]
    fn test_scan_fn_keyword() {
        let kinds = token_kinds("fn foo()");
        assert_eq!(kinds[0], TokenKind::FnKeyword);
        assert_eq!(kinds[1], TokenKind::Identifier); // foo
        assert_eq!(kinds[2], TokenKind::OpenParen);
        assert_eq!(kinds[3], TokenKind::CloseParen);
    }

    // ==================== Augmented Decorator Tests ====================

    #[test]
    fn test_scan_augmented_decorator() {
        let kinds = token_kinds("@@foo");
        assert_eq!(kinds[0], TokenKind::AtAt);
        assert_eq!(kinds[1], TokenKind::Identifier); // foo
    }

    // ==================== Using Statement Tests ====================

    #[test]
    fn test_scan_using_keyword() {
        let kinds = token_kinds("using Foo");
        assert_eq!(kinds[0], TokenKind::UsingKeyword);
        assert_eq!(kinds[1], TokenKind::Identifier); // Foo
    }

    #[test]
    fn test_scan_using_member() {
        let kinds = token_kinds("using Foo.Bar");
        assert_eq!(kinds[0], TokenKind::UsingKeyword);
        assert_eq!(kinds[1], TokenKind::Identifier); // Foo
        assert_eq!(kinds[2], TokenKind::Dot);
        assert_eq!(kinds[3], TokenKind::Identifier); // Bar
    }

    // ==================== Import Tests ====================

    #[test]
    fn test_scan_import_keyword() {
        let kinds = token_kinds("import \"./foo\"");
        assert_eq!(kinds[0], TokenKind::ImportKeyword);
        assert_eq!(kinds[1], TokenKind::StringLiteral); // "./foo"
    }

    // ==================== ColonColon Tests ====================

    #[test]
    fn test_scan_colon_colon() {
        let kinds = token_kinds("Foo::Bar");
        assert_eq!(kinds[0], TokenKind::Identifier); // Foo
        assert_eq!(kinds[1], TokenKind::ColonColon);
        assert_eq!(kinds[2], TokenKind::Identifier); // Bar
    }

    // ==================== Hash Variants Tests ====================

    #[test]
    fn test_scan_hash_brace() {
        let kinds = token_kinds("#{");
        assert_eq!(kinds[0], TokenKind::HashBrace);
    }

    #[test]
    fn test_scan_hash_bracket() {
        let kinds = token_kinds("#[");
        assert_eq!(kinds[0], TokenKind::HashBracket);
    }

    // ==================== Keyword Collision Tests ====================

    #[test]
    fn test_outerface_not_keyword() {
        // "outerface" should be an identifier, not a keyword
        let kinds = token_kinds("outerface");
        assert_eq!(kinds[0], TokenKind::Identifier);
    }

    #[test]
    fn test_famespace_not_keyword() {
        // "famespace" should be an identifier, not a keyword
        let kinds = token_kinds("famespace");
        assert_eq!(kinds[0], TokenKind::Identifier);
    }

    #[test]
    fn test_notanamespace_not_keyword() {
        // "notanamespace" should be an identifier, not a keyword
        let kinds = token_kinds("notanamespace");
        assert_eq!(kinds[0], TokenKind::Identifier);
    }

    #[test]
    fn test_notaninterface_not_keyword() {
        // "notaninterface" should be an identifier, not a keyword
        let kinds = token_kinds("notaninterface");
        assert_eq!(kinds[0], TokenKind::Identifier);
    }

    // ==================== Reserved Keywords Tests ====================

    #[test]
    fn test_scan_sealed_keyword() {
        let kinds = token_kinds("sealed");
        assert_eq!(kinds[0], TokenKind::SealedKeyword);
    }

    #[test]
    fn test_scan_local_keyword() {
        let kinds = token_kinds("local");
        assert_eq!(kinds[0], TokenKind::LocalKeyword);
    }

    #[test]
    fn test_scan_async_keyword() {
        let kinds = token_kinds("async");
        assert_eq!(kinds[0], TokenKind::AsyncKeyword);
    }

    // ==================== Array Type Tests ====================

    #[test]
    fn test_scan_open_bracket() {
        let tokens = scan_all("[");
        assert_eq!(tokens[0].0, TokenKind::OpenBracket);
    }

    #[test]
    fn test_scan_close_bracket() {
        let tokens = scan_all("]");
        assert_eq!(tokens[0].0, TokenKind::CloseBracket);
    }

    #[test]
    fn test_scan_array_type() {
        let kinds = token_kinds("string[]");
        assert_eq!(kinds[0], TokenKind::Identifier); // string
        assert_eq!(kinds[1], TokenKind::OpenBracket);
        assert_eq!(kinds[2], TokenKind::CloseBracket);
    }

    // ==================== Tuple Tests ====================

    #[test]
    fn test_scan_tuple() {
        let kinds = token_kinds("[foo, bar]");
        assert_eq!(kinds[0], TokenKind::OpenBracket);
        assert_eq!(kinds[1], TokenKind::Identifier); // foo
        assert_eq!(kinds[2], TokenKind::Comma);
        assert_eq!(kinds[3], TokenKind::Identifier); // bar
        assert_eq!(kinds[4], TokenKind::CloseBracket);
    }

    // ==================== Multi-line String Tests ====================

    #[test]
    fn test_scan_triple_quote_string() {
        let kinds = token_kinds("\"\"\"hello\"\"\"");
        assert_eq!(kinds[0], TokenKind::StringLiteral);
    }

    // ==================== Full Model Parse Tests ====================

    #[test]
    fn test_scan_model_with_decorator() {
        let kinds = token_kinds("@doc(\"test\") model Foo {}");
        assert_eq!(kinds[0], TokenKind::At); // @
        assert_eq!(kinds[1], TokenKind::Identifier); // doc
        assert_eq!(kinds[2], TokenKind::OpenParen);
        assert_eq!(kinds[3], TokenKind::StringLiteral);
        assert_eq!(kinds[4], TokenKind::CloseParen);
        assert_eq!(kinds[5], TokenKind::ModelKeyword);
        assert_eq!(kinds[6], TokenKind::Identifier); // Foo
        assert_eq!(kinds[7], TokenKind::OpenBrace);
        assert_eq!(kinds[8], TokenKind::CloseBrace);
    }

    #[test]
    fn test_scan_model_with_optional_property() {
        let kinds = token_kinds("model Foo { x?: string }");
        assert_eq!(kinds[0], TokenKind::ModelKeyword);
        assert_eq!(kinds[1], TokenKind::Identifier); // Foo
        assert_eq!(kinds[2], TokenKind::OpenBrace);
        assert_eq!(kinds[3], TokenKind::Identifier); // x
        assert_eq!(kinds[4], TokenKind::Question);
        assert_eq!(kinds[5], TokenKind::Colon);
        assert_eq!(kinds[6], TokenKind::Identifier); // string
        assert_eq!(kinds[7], TokenKind::CloseBrace);
    }

    #[test]
    fn test_scan_interface_declaration() {
        let kinds = token_kinds("interface Foo { get(): string }");
        assert_eq!(kinds[0], TokenKind::InterfaceKeyword);
        assert_eq!(kinds[1], TokenKind::Identifier); // Foo
        assert_eq!(kinds[2], TokenKind::OpenBrace);
        assert_eq!(kinds[3], TokenKind::Identifier); // get
        assert_eq!(kinds[4], TokenKind::OpenParen);
        assert_eq!(kinds[5], TokenKind::CloseParen);
        assert_eq!(kinds[6], TokenKind::Colon);
        assert_eq!(kinds[7], TokenKind::Identifier); // string
        assert_eq!(kinds[8], TokenKind::CloseBrace);
    }

    #[test]
    fn test_scan_enum_declaration() {
        let kinds = token_kinds("enum Foo { A, B, C }");
        assert_eq!(kinds[0], TokenKind::EnumKeyword);
        assert_eq!(kinds[1], TokenKind::Identifier); // Foo
        assert_eq!(kinds[2], TokenKind::OpenBrace);
        assert_eq!(kinds[3], TokenKind::Identifier); // A
        assert_eq!(kinds[4], TokenKind::Comma);
        assert_eq!(kinds[5], TokenKind::Identifier); // B
        assert_eq!(kinds[6], TokenKind::Comma);
        assert_eq!(kinds[7], TokenKind::Identifier); // C
        assert_eq!(kinds[8], TokenKind::CloseBrace);
    }

    #[test]
    fn test_scan_union_declaration() {
        let kinds = token_kinds("union Foo { A: string, B: int32 }");
        assert_eq!(kinds[0], TokenKind::UnionKeyword);
        assert_eq!(kinds[1], TokenKind::Identifier); // Foo
        assert_eq!(kinds[2], TokenKind::OpenBrace);
        assert_eq!(kinds[3], TokenKind::Identifier); // A
        assert_eq!(kinds[4], TokenKind::Colon);
        assert_eq!(kinds[5], TokenKind::Identifier); // string
        assert_eq!(kinds[6], TokenKind::Comma);
        assert_eq!(kinds[7], TokenKind::Identifier); // B
        assert_eq!(kinds[8], TokenKind::Colon);
        assert_eq!(kinds[9], TokenKind::Identifier); // int32
        assert_eq!(kinds[10], TokenKind::CloseBrace);
    }

    // ==================== Identifier Character Tests ====================

    #[test]
    fn test_scan_dollar_identifier() {
        let kinds = token_kinds("$foo");
        assert_eq!(kinds[0], TokenKind::Identifier);
    }

    #[test]
    fn test_scan_underscore_identifier() {
        let kinds = token_kinds("_foo");
        assert_eq!(kinds[0], TokenKind::Identifier);
    }

    #[test]
    fn test_scan_mixed_case_identifier() {
        let kinds = token_kinds("fooBarBaz");
        assert_eq!(kinds[0], TokenKind::Identifier);
        assert_eq!(kinds[0], TokenKind::Identifier);
    }

    #[test]
    fn test_scan_identifier_with_numbers() {
        let kinds = token_kinds("foo123");
        assert_eq!(kinds[0], TokenKind::Identifier);
    }

    // ==================== TokenKind Utility Method Tests ====================

    #[test]
    fn test_is_comment() {
        assert!(TokenKind::SingleLineComment.is_comment());
        assert!(TokenKind::MultiLineComment.is_comment());
        assert!(!TokenKind::NewLine.is_comment());
        assert!(!TokenKind::Identifier.is_comment());
        assert!(!TokenKind::ModelKeyword.is_comment());
    }

    #[test]
    fn test_is_modifier() {
        assert!(TokenKind::ExternKeyword.is_modifier());
        assert!(TokenKind::InternalKeyword.is_modifier());
        assert!(!TokenKind::ModelKeyword.is_modifier());
        assert!(!TokenKind::ImportKeyword.is_modifier());
        assert!(!TokenKind::Identifier.is_modifier());
    }

    #[test]
    fn test_is_statement_keyword() {
        assert!(TokenKind::ImportKeyword.is_statement_keyword());
        assert!(TokenKind::ModelKeyword.is_statement_keyword());
        assert!(TokenKind::ScalarKeyword.is_statement_keyword());
        assert!(TokenKind::NamespaceKeyword.is_statement_keyword());
        assert!(TokenKind::UsingKeyword.is_statement_keyword());
        assert!(TokenKind::OpKeyword.is_statement_keyword());
        assert!(TokenKind::EnumKeyword.is_statement_keyword());
        assert!(TokenKind::AliasKeyword.is_statement_keyword());
        assert!(TokenKind::InterfaceKeyword.is_statement_keyword());
        assert!(TokenKind::UnionKeyword.is_statement_keyword());
        assert!(TokenKind::DecKeyword.is_statement_keyword());
        assert!(TokenKind::ConstKeyword.is_statement_keyword());
        // Not statement keywords
        assert!(!TokenKind::ExtendsKeyword.is_statement_keyword());
        assert!(!TokenKind::VoidKeyword.is_statement_keyword());
        assert!(!TokenKind::ExternKeyword.is_statement_keyword());
    }

    #[test]
    fn test_is_reserved_keyword() {
        assert!(TokenKind::SealedKeyword.is_reserved_keyword());
        assert!(TokenKind::LocalKeyword.is_reserved_keyword());
        assert!(TokenKind::AsyncKeyword.is_reserved_keyword());
        assert!(TokenKind::MacroKeyword.is_reserved_keyword());
        assert!(TokenKind::PackageKeyword.is_reserved_keyword());
        assert!(TokenKind::DeclareKeyword.is_reserved_keyword());
        assert!(TokenKind::ThisKeyword.is_reserved_keyword());
        assert!(TokenKind::SelfKeyword.is_reserved_keyword());
        assert!(TokenKind::SuperKeyword.is_reserved_keyword());
        assert!(TokenKind::ImplKeyword.is_reserved_keyword());
        assert!(TokenKind::PrivateKeyword.is_reserved_keyword());
        assert!(TokenKind::PublicKeyword.is_reserved_keyword());
        // Not reserved keywords
        assert!(!TokenKind::ModelKeyword.is_reserved_keyword());
        assert!(!TokenKind::ExternKeyword.is_reserved_keyword());
        assert!(!TokenKind::Identifier.is_reserved_keyword());
    }

    #[test]
    fn test_is_trivia_includes_comments_and_whitespace() {
        assert!(TokenKind::SingleLineComment.is_trivia());
        assert!(TokenKind::MultiLineComment.is_trivia());
        assert!(TokenKind::NewLine.is_trivia());
        assert!(TokenKind::Whitespace.is_trivia());
        assert!(!TokenKind::Identifier.is_trivia());
    }

    #[test]
    fn test_is_keyword_excludes_punctuation_and_trivia() {
        assert!(TokenKind::ModelKeyword.is_keyword());
        assert!(TokenKind::VoidKeyword.is_keyword());
        assert!(!TokenKind::OpenBrace.is_keyword());
        assert!(!TokenKind::Identifier.is_keyword());
        assert!(!TokenKind::NumericLiteral.is_keyword());
        assert!(!TokenKind::SingleLineComment.is_keyword());
    }
}
