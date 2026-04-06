//! TypeSpec Parser implementation
//!
//! This module implements the TypeSpec parser that converts token sequences
//! into AST nodes. It follows the structure of the TypeSpec compiler parser.

mod ast_builder;

pub use ast_builder::AstBuilder;

#[cfg(test)]
mod tests;

use crate::scanner::{Lexer, TokenKind, TokenFlags};
use crate::ast::token::{Span, Position};
use crate::ast::types::*;

/// Parser options
#[derive(Debug, Clone, Default)]
pub struct ParseOptions {
    /// Whether to parse documentation comments
    pub docs: bool,
    /// Whether to include comments in the AST
    pub comments: bool,
}

/// Parse result
#[derive(Debug)]
pub struct ParseResult {
    pub root_id: u32,
    pub builder: AstBuilder,
    pub diagnostics: Vec<ParseDiagnostic>,
}

/// Parse diagnostic
#[derive(Debug, Clone)]
pub struct ParseDiagnostic {
    pub code: String,
    pub message: String,
    pub span: Span,
}

/// Main parser struct
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    source: &'a str,
    current_token: TokenKind,
    token_start: usize,
    previous_token_end: usize,
    options: ParseOptions,
    builder: AstBuilder,
    diagnostics: Vec<ParseDiagnostic>,
    parse_error_in_next_node: bool,
    missing_identifier_counter: usize,
    new_line_is_trivia: bool,
}

impl<'a> Parser<'a> {
    /// Create a new parser for the given source
    pub fn new(source: &'a str, options: ParseOptions) -> Self {
        let mut lexer = Lexer::new(source);
        let current_token = lexer.scan();
        Parser {
            lexer,
            source,
            current_token,
            token_start: 0,
            previous_token_end: 0,
            options,
            builder: AstBuilder::new(source.to_string()),
            diagnostics: Vec::new(),
            parse_error_in_next_node: false,
            missing_identifier_counter: 0,
            new_line_is_trivia: true,
        }
    }

    /// Parse the source and return the result
    pub fn parse(&mut self) -> ParseResult {
        let root_id = self.parse_typespec_script();
        ParseResult {
            root_id,
            builder: self.builder.clone(),
            diagnostics: self.diagnostics.clone(),
        }
    }

    // ==================== Token Navigation ====================

    fn current_token(&self) -> TokenKind {
        self.current_token.clone()
    }

    fn token_position(&self) -> usize {
        self.lexer.position().column as usize
    }

    fn token_start_position(&self) -> usize {
        self.token_start
    }

    fn previous_token_end(&self) -> usize {
        self.previous_token_end
    }

    fn token_value(&self) -> String {
        self.lexer.token_value()
    }

    fn token_text(&self) -> String {
        self.lexer.token_text().to_string()
    }

    fn token_flags(&self) -> TokenFlags {
        TokenFlags::NONE
    }

    fn next_token(&mut self) {
        self.previous_token_end = self.lexer.position().column as usize;
        self.current_token = self.lexer.scan();
    }

    /// Skip trivia tokens (comments, newlines, whitespace) at the current position
    fn skip_trivia(&mut self) {
        while self.current_token().is_trivia() {
            self.next_token();
        }
    }

    fn make_span(&self, start: usize, end: usize) -> Span {
        Span {
            start: Position { line: 1, column: 0 },
            end: Position { line: 1, column: 0 },
        }
    }

    // ==================== Error Handling ====================

    fn error(&mut self, code: &str, message: &str) {
        self.parse_error_in_next_node = true;
        let span = self.make_span(self.token_start, self.previous_token_end);
        self.diagnostics.push(ParseDiagnostic {
            code: code.to_string(),
            message: message.to_string(),
            span,
        });
    }

    fn finish_node(&mut self, pos: usize) -> Span {
        let flags = if self.parse_error_in_next_node {
            NodeFlags::ThisNodeHasError
        } else {
            NodeFlags::None
        };
        self.parse_error_in_next_node = false;
        self.make_span(pos, self.previous_token_end)
    }

    // ==================== Parsing Entry Points ====================

    fn parse_typespec_script(&mut self) -> u32 {
        let statements = self.parse_script_item_list();
        let span = self.make_span(0, self.previous_token_end);
        self.builder.create_typespec_script(statements, vec![], vec![], span)
    }

    fn parse_script_item_list(&mut self) -> Vec<u32> {
        let mut statements = Vec::new();

        while self.current_token() != TokenKind::EndOfFile {
            // Skip trivia (comments, newlines, whitespace) before each statement
            self.skip_trivia();

            if self.current_token() == TokenKind::EndOfFile {
                break;
            }

            let decorators = self.parse_decorator_list();
            let pos = self.token_start_position();

            match self.current_token() {
                TokenKind::AtAt => {
                    // Augment decorator
                    self.next_token();
                    let target = self.parse_identifier_or_member_expression(true, true);
                    let args = self.parse_optional_list(TokenKind::OpenParen, TokenKind::CloseParen, TokenKind::Comma, Self::parse_expression);
                    let span = self.make_span(pos, self.previous_token_end);
                    let stmt_id = self.builder.create_augment_decorator_statement(target, target, args, span);
                    statements.push(stmt_id);
                    self.expect_token(TokenKind::Semicolon);
                }
                TokenKind::ImportKeyword => {
                    let stmt = self.parse_import_statement(pos);
                    statements.push(stmt);
                }
                TokenKind::UsingKeyword => {
                    self.next_token(); // advance past `using` keyword
                    let name = self.parse_identifier_or_member_expression(false, false);
                    self.expect_token(TokenKind::Semicolon);
                    let span = self.make_span(pos, self.previous_token_end);
                    let stmt = self.builder.create_using_declaration(name, span);
                    statements.push(stmt);
                }
                TokenKind::NamespaceKeyword => {
                    let stmt = self.parse_namespace_statement(pos, decorators);
                    statements.push(stmt);
                }
                TokenKind::ModelKeyword => {
                    let stmt = self.parse_model_statement(pos, decorators);
                    statements.push(stmt);
                }
                TokenKind::ScalarKeyword => {
                    let stmt = self.parse_scalar_statement(pos, decorators);
                    statements.push(stmt);
                }
                TokenKind::InterfaceKeyword => {
                    let stmt = self.parse_interface_statement(pos, decorators);
                    statements.push(stmt);
                }
                TokenKind::UnionKeyword => {
                    let stmt = self.parse_union_statement(pos, decorators);
                    statements.push(stmt);
                }
                TokenKind::OpKeyword => {
                    let stmt = self.parse_operation_statement(pos, decorators);
                    statements.push(stmt);
                }
                TokenKind::EnumKeyword => {
                    let stmt = self.parse_enum_statement(pos, decorators);
                    statements.push(stmt);
                }
                TokenKind::AliasKeyword => {
                    let stmt = self.parse_alias_statement(pos);
                    statements.push(stmt);
                }
                TokenKind::ConstKeyword => {
                    let stmt = self.parse_const_statement(pos);
                    statements.push(stmt);
                }
                TokenKind::ExternKeyword | TokenKind::InternalKeyword => {
                    let modifiers = self.parse_modifiers();
                    let stmt = self.parse_declaration(pos, decorators, modifiers);
                    statements.push(stmt);
                }
                TokenKind::FnKeyword => {
                    let stmt = self.parse_function_declaration_statement(pos, vec![]);
                    statements.push(stmt);
                }
                TokenKind::DecKeyword => {
                    let stmt = self.parse_decorator_declaration_statement(pos, vec![]);
                    statements.push(stmt);
                }
                TokenKind::Semicolon => {
                    self.next_token();
                    let span = self.make_span(pos, self.previous_token_end);
                    let stmt = self.builder.create_empty_statement(span);
                    statements.push(stmt);
                }
                _ => {
                    // Invalid statement - skip until we find a statement keyword
                    self.error("unexpected-token", &format!("Unexpected token: {:?}", self.current_token()));
                    while !self.is_statement_keyword(self.current_token())
                        && self.current_token() != TokenKind::EndOfFile
                        && self.current_token() != TokenKind::At
                        && self.current_token() != TokenKind::Semicolon
                    {
                        self.next_token();
                    }
                    let span = self.make_span(pos, self.previous_token_end);
                    let stmt = self.builder.create_invalid_statement(decorators, span);
                    statements.push(stmt);
                }
            }
        }

        statements
    }

    fn is_statement_keyword(&self, token: TokenKind) -> bool {
        matches!(
            token,
            TokenKind::NamespaceKeyword
                | TokenKind::ModelKeyword
                | TokenKind::ScalarKeyword
                | TokenKind::InterfaceKeyword
                | TokenKind::UnionKeyword
                | TokenKind::OpKeyword
                | TokenKind::EnumKeyword
                | TokenKind::AliasKeyword
                | TokenKind::ConstKeyword
                | TokenKind::ExternKeyword
                | TokenKind::InternalKeyword
                | TokenKind::FnKeyword
                | TokenKind::DecKeyword
                | TokenKind::ImportKeyword
                | TokenKind::UsingKeyword
        )
    }

    // ==================== Statement Parsing ====================

    fn parse_namespace_statement(&mut self, pos: usize, decorators: Vec<u32>) -> u32 {
        self.expect_token(TokenKind::NamespaceKeyword);
        let name = self.parse_identifier_or_member_expression(false, false);

        let statements = if self.check_token(TokenKind::OpenBrace) {
            self.expect_token(TokenKind::OpenBrace);
            let stmts = self.parse_statement_list();
            self.expect_token(TokenKind::CloseBrace);
            stmts
        } else {
            self.expect_token(TokenKind::Semicolon);
            vec![]
        };

        let span = self.make_span(pos, self.previous_token_end);
        self.builder.create_namespace_declaration(name, statements, decorators, span)
    }

    fn parse_statement_list(&mut self) -> Vec<u32> {
        let mut statements = Vec::new();

        while self.current_token() != TokenKind::CloseBrace && self.current_token() != TokenKind::EndOfFile {
            let decorators = self.parse_decorator_list();
            let pos = self.token_start_position();

            match self.current_token() {
                TokenKind::ModelKeyword => {
                    let stmt = self.parse_model_statement(pos, decorators);
                    statements.push(stmt);
                }
                TokenKind::NamespaceKeyword => {
                    let stmt = self.parse_namespace_statement(pos, decorators);
                    statements.push(stmt);
                }
                TokenKind::InterfaceKeyword => {
                    let stmt = self.parse_interface_statement(pos, decorators);
                    statements.push(stmt);
                }
                TokenKind::UnionKeyword => {
                    let stmt = self.parse_union_statement(pos, decorators);
                    statements.push(stmt);
                }
                TokenKind::OpKeyword => {
                    let stmt = self.parse_operation_statement(pos, decorators);
                    statements.push(stmt);
                }
                TokenKind::EnumKeyword => {
                    let stmt = self.parse_enum_statement(pos, decorators);
                    statements.push(stmt);
                }
                TokenKind::ScalarKeyword => {
                    let stmt = self.parse_scalar_statement(pos, decorators);
                    statements.push(stmt);
                }
                TokenKind::Semicolon => {
                    self.next_token();
                    let span = self.make_span(pos, self.previous_token_end);
                    let stmt = self.builder.create_empty_statement(span);
                    statements.push(stmt);
                }
                TokenKind::EndOfFile => {
                    break;
                }
                _ => {
                    self.error("unexpected-token", "Expected a statement");
                    while !self.is_statement_keyword(self.current_token())
                        && self.current_token() != TokenKind::CloseBrace
                        && self.current_token() != TokenKind::EndOfFile
                    {
                        self.next_token();
                    }
                    let span = self.make_span(pos, self.previous_token_end);
                    let stmt = self.builder.create_invalid_statement(decorators, span);
                    statements.push(stmt);
                }
            }
        }

        statements
    }

    fn parse_import_statement(&mut self, pos: usize) -> u32 {
        self.expect_token(TokenKind::ImportKeyword);
        let path = self.parse_string_literal();
        self.expect_token(TokenKind::Semicolon);
        let span = self.make_span(pos, self.previous_token_end);
        self.builder.create_import_statement(path, span)
    }

    fn parse_model_statement(&mut self, pos: usize, decorators: Vec<u32>) -> u32 {
        self.expect_token(TokenKind::ModelKeyword);
        let name = self.parse_identifier();

        // Template parameters
        let template_parameters = if self.check_token(TokenKind::LessThan) {
            self.parse_template_parameter_list()
        } else {
            vec![]
        };

        // Extends clause
        let extends = if self.check_token(TokenKind::ExtendsKeyword) {
            self.next_token();
            Some(self.parse_expression())
        } else {
            None
        };

        // `is` clause
        let is = if self.check_token(TokenKind::IsKeyword) {
            self.next_token();
            Some(self.parse_expression())
        } else {
            None
        };

        // Model body
        let (properties, body_range) = if self.check_token(TokenKind::OpenBrace) {
            self.expect_token(TokenKind::OpenBrace);
            let props = self.parse_model_property_list();
            let body_start = self.token_start_position();
            self.expect_token(TokenKind::CloseBrace);
            let body_end = self.previous_token_end;
            (props, self.make_span(body_start, body_end))
        } else if extends.is_some() || is.is_some() {
            // When using extends/is, expect semicolon
            self.expect_token(TokenKind::Semicolon);
            (vec![], self.make_span(0, 0))
        } else {
            (vec![], self.make_span(0, 0))
        };

        let span = self.make_span(pos, self.previous_token_end);
        self.builder.create_model_declaration(name, properties, extends, is, decorators, body_range, span)
    }

    /// Parse operation parameters (typed parameters inside parentheses)
    fn parse_operation_parameters(&mut self) -> Vec<u32> {
        let mut properties = Vec::new();

        while self.current_token() != TokenKind::CloseParen && self.current_token() != TokenKind::EndOfFile {
            if self.check_token(TokenKind::Ellipsis) {
                self.next_token();
                let target = self.parse_expression();
                let span = self.make_span(self.token_start_position(), self.previous_token_end);
                let spread_id = self.builder.create_model_spread_property(target, span);
                properties.push(spread_id);
            } else {
                let decorators = self.parse_decorator_list();
                let pos = self.token_start_position();
                let name = self.parse_model_property_name();

                let optional = self.check_token(TokenKind::Question);
                if optional {
                    self.next_token();
                }

                self.expect_token(TokenKind::Colon);
                let value = self.parse_expression();

                let default = if self.check_token(TokenKind::Equals) {
                    self.next_token();
                    Some(self.parse_expression())
                } else {
                    None
                };

                let span = self.make_span(pos, self.previous_token_end);
                let prop_id = self.builder.create_model_property(name, value, decorators, optional, default, span);
                properties.push(prop_id);
            }

            // Handle delimiter - comma or semicolon in operation parameters
            if self.check_token(TokenKind::Comma) {
                self.next_token();
            } else if self.check_token(TokenKind::Semicolon) {
                self.next_token();
            } else if self.current_token() != TokenKind::CloseParen {
                // Error recovery - assume missing comma
                if self.current_token() != TokenKind::EndOfFile {
                    self.error("expected-comma", "Expected ',' or ')'");
                }
                break;
            }
        }

        properties
    }

    fn parse_model_property_list(&mut self) -> Vec<u32> {
        let mut properties = Vec::new();

        while self.current_token() != TokenKind::CloseBrace && self.current_token() != TokenKind::EndOfFile {
            if self.check_token(TokenKind::Ellipsis) {
                self.next_token();
                let target = self.parse_expression();
                let span = self.make_span(self.token_start_position(), self.previous_token_end);
                let spread_id = self.builder.create_model_spread_property(target, span);
                properties.push(spread_id);
            } else {
                let decorators = self.parse_decorator_list();
                let pos = self.token_start_position();
                let name = self.parse_model_property_name();

                let optional = self.check_token(TokenKind::Question);
                if optional {
                    self.next_token();
                }

                self.expect_token(TokenKind::Colon);
                let value = self.parse_expression();

                let default = if self.check_token(TokenKind::Equals) {
                    self.next_token();
                    Some(self.parse_expression())
                } else {
                    None
                };

                let span = self.make_span(pos, self.previous_token_end);
                let prop_id = self.builder.create_model_property(name, value, decorators, optional, default, span);
                properties.push(prop_id);
            }

            // Handle delimiter
            if self.check_token(TokenKind::Semicolon) {
                self.next_token();
            } else if self.check_token(TokenKind::Comma) {
                self.next_token();
            } else if self.current_token() != TokenKind::CloseBrace {
                // Error recovery - assume missing semicolon
                if self.current_token() != TokenKind::EndOfFile {
                    self.error("expected-semicolon", "Expected ';' or ','");
                }
                break;
            }
        }

        properties
    }

    fn parse_interface_statement(&mut self, pos: usize, decorators: Vec<u32>) -> u32 {
        self.expect_token(TokenKind::InterfaceKeyword);
        let name = self.parse_identifier();

        // Template parameters
        let template_parameters = if self.check_token(TokenKind::LessThan) {
            self.parse_template_parameter_list()
        } else {
            vec![]
        };

        // Extends clause
        let extends = if self.check_token(TokenKind::ExtendsKeyword) {
            self.next_token();
            self.parse_type_reference_list()
        } else {
            vec![]
        };

        // Interface body
        self.expect_token(TokenKind::OpenBrace);
        let operations = self.parse_operation_list();
        let body_start = self.token_start_position();
        self.expect_token(TokenKind::CloseBrace);
        let body_range = self.make_span(body_start, self.previous_token_end);

        let span = self.make_span(pos, self.previous_token_end);
        self.builder.create_interface_declaration(name, operations, extends, decorators, template_parameters, body_range, span)
    }

    fn parse_operation_list(&mut self) -> Vec<u32> {
        let mut operations = Vec::new();

        while self.current_token() != TokenKind::CloseBrace && self.current_token() != TokenKind::EndOfFile {
            let decorators = self.parse_decorator_list();
            let pos = self.token_start_position();

            // OpKeyword is optional inside interface body
            if self.check_token(TokenKind::OpKeyword) {
                self.next_token();
            }
            let name = self.parse_identifier();

            // Template parameters
            let template_parameters = if self.check_token(TokenKind::LessThan) {
                self.parse_template_parameter_list()
            } else {
                vec![]
            };

            // Parse operation signature
            let signature = if self.check_token(TokenKind::OpenParen) {
                // Parameters
                self.next_token();
                let params = self.parse_expression_list(TokenKind::CloseParen);
                self.expect_token(TokenKind::CloseParen);
                self.expect_token(TokenKind::Colon);
                let return_type = self.parse_expression();
                let span = self.make_span(pos, self.previous_token_end);
                let params_model = self.builder.create_model_expression(params, span, span);
                self.builder.create_operation_signature_declaration(params_model, return_type, span)
            } else {
                self.expect_token(TokenKind::IsKeyword);
                let base_op = self.parse_expression();
                let span = self.make_span(pos, self.previous_token_end);
                self.builder.create_operation_signature_reference(base_op, span)
            };

            self.expect_token(TokenKind::Semicolon);

            let span = self.make_span(pos, self.previous_token_end);
            let op_id = self.builder.create_operation_declaration(name, signature, decorators, template_parameters, span);
            operations.push(op_id);
        }

        operations
    }

    fn parse_union_statement(&mut self, pos: usize, decorators: Vec<u32>) -> u32 {
        self.expect_token(TokenKind::UnionKeyword);
        let name = self.parse_identifier();

        // Template parameters
        let template_parameters = if self.check_token(TokenKind::LessThan) {
            self.parse_template_parameter_list()
        } else {
            vec![]
        };

        // Union variants
        self.expect_token(TokenKind::OpenBrace);
        let variants = self.parse_union_variant_list();
        let body_start = self.token_start_position();
        self.expect_token(TokenKind::CloseBrace);
        let body_range = self.make_span(body_start, self.previous_token_end);

        let span = self.make_span(pos, self.previous_token_end);
        self.builder.create_union_declaration(name, variants, decorators, template_parameters, span)
    }

    fn parse_union_variant_list(&mut self) -> Vec<u32> {
        let mut variants = Vec::new();

        while self.current_token() != TokenKind::CloseBrace && self.current_token() != TokenKind::EndOfFile {
            let decorators = self.parse_decorator_list();
            let pos = self.token_start_position();

            // Check for identifier or value
            let (name, value) = if self.is_reserved_identifier(self.current_token()) {
                let id = self.parse_identifier();
                if self.check_token(TokenKind::Colon) {
                    self.next_token();
                    let val = self.parse_expression();
                    (Some(id), val)
                } else {
                    // Reserved keyword used as type reference
                    let span = self.make_span(pos, self.previous_token_end);
                    let type_ref = self.builder.create_type_reference(id, vec![], span);
                    (None, type_ref)
                }
            } else {
                let val = self.parse_expression();
                if self.check_token(TokenKind::Colon) {
                    self.next_token();
                    let id = self.parse_identifier();
                    (Some(id), val)
                } else {
                    (None, val)
                }
            };

            let span = self.make_span(pos, self.previous_token_end);
            let variant_id = self.builder.create_union_variant(name, value, decorators, span);
            variants.push(variant_id);

            // Handle delimiter
            if self.check_token(TokenKind::Semicolon) {
                self.next_token();
            } else if self.check_token(TokenKind::Comma) {
                self.next_token();
            } else if self.current_token() != TokenKind::CloseBrace {
                break;
            }
        }

        variants
    }

    fn parse_enum_statement(&mut self, pos: usize, decorators: Vec<u32>) -> u32 {
        self.expect_token(TokenKind::EnumKeyword);
        let name = self.parse_identifier();

        self.expect_token(TokenKind::OpenBrace);
        let members = self.parse_enum_member_list();
        self.expect_token(TokenKind::CloseBrace);

        let span = self.make_span(pos, self.previous_token_end);
        self.builder.create_enum_declaration(name, members, decorators, span)
    }

    fn parse_enum_member_list(&mut self) -> Vec<u32> {
        let mut members = Vec::new();

        while self.current_token() != TokenKind::CloseBrace && self.current_token() != TokenKind::EndOfFile {
            if self.check_token(TokenKind::Ellipsis) {
                self.next_token();
                let target = self.parse_expression();
                let span = self.make_span(self.token_start_position(), self.previous_token_end);
                let spread_id = self.builder.create_enum_spread_member(target, span);
                members.push(spread_id);
            } else {
                let decorators = self.parse_decorator_list();
                let pos = self.token_start_position();
                let name = self.parse_identifier();

                let value = if self.check_token(TokenKind::Colon) {
                    self.next_token();
                    Some(self.parse_expression())
                } else {
                    None
                };

                let span = self.make_span(pos, self.previous_token_end);
                let member_id = self.builder.create_enum_member(name, value, decorators, span);
                members.push(member_id);
            }

            // Handle delimiter
            if self.check_token(TokenKind::Semicolon) {
                self.next_token();
            } else if self.check_token(TokenKind::Comma) {
                self.next_token();
            } else if self.current_token() != TokenKind::CloseBrace {
                break;
            }
        }

        members
    }

    fn parse_scalar_statement(&mut self, pos: usize, decorators: Vec<u32>) -> u32 {
        self.expect_token(TokenKind::ScalarKeyword);
        let name = self.parse_identifier();

        // Template parameters
        let template_parameters = if self.check_token(TokenKind::LessThan) {
            self.parse_template_parameter_list()
        } else {
            vec![]
        };

        // Extends clause
        let extends = if self.check_token(TokenKind::ExtendsKeyword) {
            self.next_token();
            Some(self.parse_expression())
        } else {
            None
        };

        // Scalar body
        let (constructors, body_range) = if self.check_token(TokenKind::OpenBrace) {
            self.expect_token(TokenKind::OpenBrace);
            let ctors = self.parse_scalar_constructor_list();
            let body_start = self.token_start_position();
            self.expect_token(TokenKind::CloseBrace);
            let body_range = self.make_span(body_start, self.previous_token_end);
            (ctors, body_range)
        } else if self.check_token(TokenKind::Semicolon) {
            self.next_token();
            (vec![], self.make_span(0, 0))
        } else {
            (vec![], self.make_span(0, 0))
        };

        let span = self.make_span(pos, self.previous_token_end);
        self.builder.create_scalar_declaration(name, extends, decorators, constructors, body_range, span)
    }

    fn parse_scalar_constructor_list(&mut self) -> Vec<u32> {
        let mut constructors = Vec::new();

        while self.current_token() != TokenKind::CloseBrace && self.current_token() != TokenKind::EndOfFile {
            let pos = self.token_start_position();
            self.expect_token(TokenKind::InitKeyword);
            let name = self.parse_identifier();

            let params = if self.check_token(TokenKind::OpenParen) {
                self.next_token();
                let params = self.parse_operation_parameters();
                self.expect_token(TokenKind::CloseParen);
                params
            } else {
                vec![]
            };

            let span = self.make_span(pos, self.previous_token_end);
            let ctor_id = self.builder.create_scalar_constructor(name, params, span);
            constructors.push(ctor_id);

            if self.check_token(TokenKind::Semicolon) {
                self.next_token();
            } else if self.check_token(TokenKind::Comma) {
                self.next_token();
            }
        }

        constructors
    }

    fn parse_operation_statement(&mut self, pos: usize, decorators: Vec<u32>) -> u32 {
        self.expect_token(TokenKind::OpKeyword);
        let name = self.parse_identifier();

        // Template parameters
        let template_parameters = if self.check_token(TokenKind::LessThan) {
            self.parse_template_parameter_list()
        } else {
            vec![]
        };

        // Parse operation signature
        let signature = if self.check_token(TokenKind::OpenParen) {
            self.next_token();
            let params = self.parse_operation_parameters();
            self.expect_token(TokenKind::CloseParen);
            self.expect_token(TokenKind::Colon);
            let return_type = self.parse_expression();
            let span = self.make_span(pos, self.previous_token_end);
            let params_model = self.builder.create_model_expression(params, span, span);
            self.builder.create_operation_signature_declaration(params_model, return_type, span)
        } else {
            self.expect_token(TokenKind::IsKeyword);
            let base_op = self.parse_expression();
            let span = self.make_span(pos, self.previous_token_end);
            self.builder.create_operation_signature_reference(base_op, span)
        };

        self.expect_token(TokenKind::Semicolon);

        let span = self.make_span(pos, self.previous_token_end);
        self.builder.create_operation_declaration(name, signature, decorators, template_parameters, span)
    }

    fn parse_alias_statement(&mut self, pos: usize) -> u32 {
        self.expect_token(TokenKind::AliasKeyword);
        let name = self.parse_identifier();

        // Template parameters
        let template_parameters = if self.check_token(TokenKind::LessThan) {
            self.parse_template_parameter_list()
        } else {
            vec![]
        };

        self.expect_token(TokenKind::Equals);
        let value = self.parse_expression();
        self.expect_token(TokenKind::Semicolon);

        let span = self.make_span(pos, self.previous_token_end);
        self.builder.create_alias_statement(name, value, template_parameters, span)
    }

    fn parse_const_statement(&mut self, pos: usize) -> u32 {
        self.expect_token(TokenKind::ConstKeyword);
        let name = self.parse_identifier();

        // Type annotation
        let type_annotation = if self.check_token(TokenKind::Colon) {
            self.next_token();
            Some(self.parse_expression())
        } else {
            None
        };

        self.expect_token(TokenKind::Equals);
        let value = self.parse_expression();
        self.expect_token(TokenKind::Semicolon);

        let span = self.make_span(pos, self.previous_token_end);
        self.builder.create_const_statement(name, value, type_annotation, span)
    }

    fn parse_function_declaration_statement(&mut self, pos: usize, modifiers: Vec<u32>) -> u32 {
        self.expect_token(TokenKind::FnKeyword);
        let name = self.parse_identifier();

        // Parameters
        self.expect_token(TokenKind::OpenParen);
        let params = self.parse_function_parameter_list();
        self.expect_token(TokenKind::CloseParen);

        // Return type
        let return_type = if self.check_token(TokenKind::Colon) {
            self.next_token();
            Some(self.parse_expression())
        } else {
            None
        };

        self.expect_token(TokenKind::Semicolon);

        let span = self.make_span(pos, self.previous_token_end);
        self.builder.create_function_declaration(name, params, return_type, span)
    }

    fn parse_function_parameter_list(&mut self) -> Vec<u32> {
        let mut params = Vec::new();

        while self.current_token() != TokenKind::CloseParen && self.current_token() != TokenKind::EndOfFile {
            let pos = self.token_start_position();

            let rest = self.check_token(TokenKind::Ellipsis);
            if rest {
                self.next_token();
            }

            let name = self.parse_identifier();
            let optional = self.check_token(TokenKind::Question);
            if optional {
                self.next_token();
            }

            let type_annotation = if self.check_token(TokenKind::Colon) {
                self.next_token();
                Some(self.parse_expression())
            } else {
                None
            };

            let span = self.make_span(pos, self.previous_token_end);
            let param_id = self.builder.create_function_parameter(name, type_annotation, optional, rest, span);
            params.push(param_id);

            if self.check_token(TokenKind::Comma) {
                self.next_token();
            } else {
                break;
            }
        }

        params
    }

    fn parse_decorator_declaration_statement(&mut self, pos: usize, modifiers: Vec<u32>) -> u32 {
        self.expect_token(TokenKind::DecKeyword);
        let name = self.parse_identifier();

        // Parameters
        self.expect_token(TokenKind::OpenParen);
        let params = self.parse_function_parameter_list();
        self.expect_token(TokenKind::CloseParen);

        // First parameter is the target
        let target = params.first().copied().unwrap_or_else(|| {
            let span = self.make_span(0, 0);
            self.builder.create_function_parameter(0, None, false, false, span)
        });

        self.expect_token(TokenKind::Semicolon);

        let span = self.make_span(pos, self.previous_token_end);
        self.builder.create_decorator_declaration(name, target, params[1..].to_vec(), span)
    }

    fn parse_declaration(&mut self, pos: usize, decorators: Vec<u32>, modifiers: Vec<u32>) -> u32 {
        match self.current_token() {
            TokenKind::ModelKeyword => self.parse_model_statement(pos, decorators),
            TokenKind::ScalarKeyword => self.parse_scalar_statement(pos, decorators),
            TokenKind::NamespaceKeyword => self.parse_namespace_statement(pos, decorators),
            TokenKind::InterfaceKeyword => self.parse_interface_statement(pos, decorators),
            TokenKind::UnionKeyword => self.parse_union_statement(pos, decorators),
            TokenKind::OpKeyword => self.parse_operation_statement(pos, decorators),
            TokenKind::EnumKeyword => self.parse_enum_statement(pos, decorators),
            TokenKind::AliasKeyword => self.parse_alias_statement(pos),
            TokenKind::ConstKeyword => self.parse_const_statement(pos),
            TokenKind::FnKeyword => self.parse_function_declaration_statement(pos, modifiers),
            TokenKind::DecKeyword => self.parse_decorator_declaration_statement(pos, modifiers),
            _ => {
                self.error("invalid-declaration", "Invalid declaration");
                let span = self.make_span(pos, self.previous_token_end);
                self.builder.create_invalid_statement(decorators, span)
            }
        }
    }

    // ==================== Expression Parsing ====================

    fn parse_expression(&mut self) -> u32 {
        self.parse_union_expression_or_higher()
    }

    fn parse_union_expression_or_higher(&mut self) -> u32 {
        let pos = self.token_start_position();
        if self.check_token(TokenKind::Bar) {
            self.next_token();
        }
        let mut node = self.parse_intersection_expression_or_higher();

        if self.check_token(TokenKind::Bar) {
            let mut options = vec![node];
            while self.check_token(TokenKind::Bar) {
                self.next_token();
                let expr = self.parse_intersection_expression_or_higher();
                options.push(expr);
            }
            let span = self.make_span(pos, self.previous_token_end);
            node = self.builder.create_union_expression(options, span);
        }

        node
    }

    fn parse_intersection_expression_or_higher(&mut self) -> u32 {
        let pos = self.token_start_position();
        let mut node = self.parse_array_expression_or_higher();

        if self.check_token(TokenKind::Ampersand) {
            let mut options = vec![node];
            while self.check_token(TokenKind::Ampersand) {
                self.next_token();
                let expr = self.parse_array_expression_or_higher();
                options.push(expr);
            }
            let span = self.make_span(pos, self.previous_token_end);
            node = self.builder.create_intersection_expression(options, span);
        }

        node
    }

    fn parse_array_expression_or_higher(&mut self) -> u32 {
        let pos = self.token_start_position();
        let mut node = self.parse_primary_expression();

        while self.check_token(TokenKind::OpenBracket) {
            self.next_token();
            self.expect_token(TokenKind::CloseBracket);
            let span = self.make_span(pos, self.previous_token_end);
            node = self.builder.create_array_expression(node, span);
        }

        node
    }

    fn parse_primary_expression(&mut self) -> u32 {
        let pos = self.token_start_position();

        match self.current_token() {
            TokenKind::Identifier => {
                self.parse_call_or_reference_expression()
            }
            TokenKind::StringLiteral => {
                let value = self.token_value();
                let span = self.make_span(pos, self.previous_token_end);
                self.next_token();
                self.builder.create_string_literal(value, span)
            }
            TokenKind::NumericLiteral => {
                let value_str = self.token_text();
                let value = value_str.parse::<f64>().unwrap_or(0.0);
                let span = self.make_span(pos, self.previous_token_end);
                self.next_token();
                self.builder.create_numeric_literal(value, value_str, span)
            }
            TokenKind::TrueKeyword | TokenKind::FalseKeyword => {
                let value = self.current_token() == TokenKind::TrueKeyword;
                let span = self.make_span(pos, self.previous_token_end);
                self.next_token();
                self.builder.create_boolean_literal(value, span)
            }
            TokenKind::OpenBrace => {
                self.next_token();
                let props = self.parse_model_property_list();
                let body_range = self.make_span(pos, self.previous_token_end);
                self.expect_token(TokenKind::CloseBrace);
                let span = self.make_span(pos, self.previous_token_end);
                self.builder.create_model_expression(props, body_range, span)
            }
            TokenKind::OpenBracket => {
                self.next_token();
                let values = self.parse_expression_list(TokenKind::CloseBracket);
                let span = self.make_span(pos, self.previous_token_end);
                self.expect_token(TokenKind::CloseBracket);
                self.builder.create_tuple_expression(values, span)
            }
            TokenKind::OpenParen => {
                self.next_token();
                let expr = self.parse_expression();
                self.expect_token(TokenKind::CloseParen);
                expr
            }
            TokenKind::HashBrace => {
                self.parse_object_literal()
            }
            TokenKind::HashBracket => {
                self.parse_array_literal()
            }
            TokenKind::ValueOfKeyword => {
                self.next_token();
                let target = self.parse_expression();
                let span = self.make_span(pos, self.previous_token_end);
                self.builder.create_value_of_expression(target, span)
            }
            TokenKind::TypeOfKeyword => {
                self.next_token();
                let target = self.parse_type_of_target();
                let span = self.make_span(pos, self.previous_token_end);
                self.builder.create_type_of_expression(target, span)
            }
            TokenKind::FnKeyword => {
                self.parse_function_type_expression()
            }
            TokenKind::VoidKeyword => {
                let span = self.make_span(pos, self.previous_token_end);
                self.next_token();
                self.builder.create_void_keyword(span)
            }
            TokenKind::NeverKeyword => {
                let span = self.make_span(pos, self.previous_token_end);
                self.next_token();
                self.builder.create_never_keyword(span)
            }
            TokenKind::UnknownKeyword => {
                let span = self.make_span(pos, self.previous_token_end);
                self.next_token();
                self.builder.create_unknown_keyword(span)
            }
            _ => {
                // Error recovery - return missing identifier
                let span = self.make_span(pos, self.previous_token_end);
                self.error("expected-expression", "Expected an expression");
                self.builder.create_identifier("<missing>".to_string(), span)
            }
        }
    }

    fn parse_call_or_reference_expression(&mut self) -> u32 {
        let pos = self.token_start_position();
        let target = self.parse_identifier_or_member_expression(false, false);

        if self.check_token(TokenKind::OpenParen) {
            self.next_token();
            let args = self.parse_expression_list(TokenKind::CloseParen);
            self.expect_token(TokenKind::CloseParen);
            let span = self.make_span(pos, self.previous_token_end);
            self.builder.create_call_expression(target, args, span)
        } else {
            // Check for template arguments
            if self.check_token(TokenKind::LessThan) {
                let args = self.parse_template_argument_list();
                let span = self.make_span(pos, self.previous_token_end);
                self.builder.create_type_reference(target, args, span)
            } else {
                target
            }
        }
    }

    fn parse_identifier_or_member_expression(&mut self, allow_reserved: bool, allow_reserved_in_member: bool) -> u32 {
        let pos = self.token_start_position();
        let mut base = self.parse_identifier();

        while self.check_token(TokenKind::Dot) || self.check_token(TokenKind::ColonColon) {
            let selector = if self.check_token(TokenKind::Dot) {
                MemberSelector::Dot
            } else {
                MemberSelector::DoubleColon
            };
            self.next_token();
            let property = self.parse_identifier();
            let span = self.make_span(pos, self.previous_token_end);
            base = self.builder.create_member_expression(base, property, selector, span);
        }

        base
    }

    fn parse_type_of_target(&mut self) -> u32 {
        match self.current_token() {
            TokenKind::TypeOfKeyword => {
                self.next_token();
                self.parse_type_of_target()
            }
            TokenKind::Identifier => self.parse_call_or_reference_expression(),
            TokenKind::StringLiteral => {
                let value = self.token_value();
                let span = self.make_span(self.token_start_position(), self.previous_token_end);
                self.next_token();
                self.builder.create_string_literal(value, span)
            }
            TokenKind::TrueKeyword | TokenKind::FalseKeyword => {
                let value = self.current_token() == TokenKind::TrueKeyword;
                let span = self.make_span(self.token_start_position(), self.previous_token_end);
                self.next_token();
                self.builder.create_boolean_literal(value, span)
            }
            TokenKind::NumericLiteral => {
                let value_str = self.token_text();
                let value = value_str.parse::<f64>().unwrap_or(0.0);
                let span = self.make_span(self.token_start_position(), self.previous_token_end);
                self.next_token();
                self.builder.create_numeric_literal(value, value_str, span)
            }
            TokenKind::OpenParen => {
                self.next_token();
                let target = self.parse_type_of_target();
                self.expect_token(TokenKind::CloseParen);
                target
            }
            _ => self.parse_reference_expression(),
        }
    }

    fn parse_function_type_expression(&mut self) -> u32 {
        let pos = self.token_start_position();
        self.expect_token(TokenKind::FnKeyword);

        self.expect_token(TokenKind::OpenParen);
        let params = self.parse_function_parameter_list();
        self.expect_token(TokenKind::CloseParen);

        let return_type = if self.check_token(TokenKind::EqualsGreaterThan) {
            self.next_token();
            Some(self.parse_expression())
        } else {
            None
        };

        let span = self.make_span(pos, self.previous_token_end);
        self.builder.create_function_type_expression(params, return_type, span)
    }

    fn parse_reference_expression(&mut self) -> u32 {
        let pos = self.token_start_position();
        let name = self.parse_identifier_or_member_expression(false, false);

        // Check for template arguments
        if self.check_token(TokenKind::LessThan) {
            let args = self.parse_template_argument_list();
            let span = self.make_span(pos, self.previous_token_end);
            self.builder.create_type_reference(name, args, span)
        } else {
            let span = self.make_span(pos, self.previous_token_end);
            self.builder.create_type_reference(name, vec![], span)
        }
    }

    fn parse_type_reference_list(&mut self) -> Vec<u32> {
        let mut refs = Vec::new();
        refs.push(self.parse_expression());

        while self.check_token(TokenKind::Comma) {
            self.next_token();
            refs.push(self.parse_expression());
        }
        refs
    }

    fn parse_template_argument_list(&mut self) -> Vec<u32> {
        self.expect_token(TokenKind::LessThan);
        let args = self.parse_expression_list(TokenKind::GreaterThan);
        self.expect_token(TokenKind::GreaterThan);
        args
    }

    fn parse_template_parameter_list(&mut self) -> Vec<u32> {
        self.expect_token(TokenKind::LessThan);
        let mut params = Vec::new();

        while self.current_token() != TokenKind::GreaterThan && self.current_token() != TokenKind::EndOfFile {
            let pos = self.token_start_position();
            let name = self.parse_identifier();

            let constraint = if self.check_token(TokenKind::ExtendsKeyword) {
                self.next_token();
                Some(self.parse_mixed_constraint())
            } else {
                None
            };

            let default = if self.check_token(TokenKind::Equals) {
                self.next_token();
                Some(self.parse_expression())
            } else {
                None
            };

            let span = self.make_span(pos, self.previous_token_end);
            let param_id = self.builder.create_template_parameter_declaration(name, constraint, default, span);
            params.push(param_id);

            if self.check_token(TokenKind::Comma) {
                self.next_token();
            } else {
                break;
            }
        }

        self.expect_token(TokenKind::GreaterThan);
        params
    }

    fn parse_mixed_constraint(&mut self) -> u32 {
        let pos = self.token_start_position();
        if self.check_token(TokenKind::Bar) {
            self.next_token();
        }
        let node = if self.check_token(TokenKind::ValueOfKeyword) {
            self.next_token();
            let target = self.parse_expression();
            let span = self.make_span(pos, self.previous_token_end);
            self.builder.create_value_of_expression(target, span)
        } else if self.check_token(TokenKind::OpenParen) {
            self.next_token();
            let expr = self.parse_mixed_constraint();
            self.expect_token(TokenKind::CloseParen);
            expr
        } else {
            self.parse_array_expression_or_higher()
        };

        if self.check_token(TokenKind::Bar) {
            let mut options = vec![node];
            while self.check_token(TokenKind::Bar) {
                self.next_token();
                let expr = if self.check_token(TokenKind::ValueOfKeyword) {
                    self.next_token();
                    let target = self.parse_expression();
                    let span = self.make_span(pos, self.previous_token_end);
                    self.builder.create_value_of_expression(target, span)
                } else {
                    self.parse_array_expression_or_higher()
                };
                options.push(expr);
            }
            let span = self.make_span(pos, self.previous_token_end);
            return self.builder.create_union_expression(options, span);
        }

        node
    }

    fn parse_object_literal(&mut self) -> u32 {
        let pos = self.token_start_position();
        self.expect_token(TokenKind::HashBrace);
        let mut properties = Vec::new();

        while self.current_token() != TokenKind::CloseBrace && self.current_token() != TokenKind::EndOfFile {
            let prop_pos = self.token_start_position();
            if self.check_token(TokenKind::Ellipsis) {
                self.next_token();
                let target = self.parse_expression();
                let span = self.make_span(prop_pos, self.previous_token_end);
                let spread_id = self.builder.create_object_literal_spread_property(target, span);
                properties.push(spread_id);
            } else {
                let key = self.parse_identifier();
                self.expect_token(TokenKind::Colon);
                let value = self.parse_expression();
                let span = self.make_span(prop_pos, self.previous_token_end);
                let prop_id = self.builder.create_object_literal_property(key, value, span);
                properties.push(prop_id);
            }

            if self.check_token(TokenKind::Comma) {
                self.next_token();
            } else {
                break;
            }
        }

        let body_range = self.make_span(pos, self.previous_token_end);
        self.expect_token(TokenKind::CloseBrace);
        let span = self.make_span(pos, self.previous_token_end);
        self.builder.create_object_literal(properties, span)
    }

    fn parse_array_literal(&mut self) -> u32 {
        let pos = self.token_start_position();
        self.expect_token(TokenKind::HashBracket);
        let values = self.parse_expression_list(TokenKind::CloseBracket);
        let span = self.make_span(pos, self.previous_token_end);
        self.expect_token(TokenKind::CloseBracket);
        self.builder.create_array_literal(values, span)
    }

    fn parse_expression_list(&mut self, end_token: TokenKind) -> Vec<u32> {
        let mut expressions = Vec::new();

        while self.current_token() != end_token && self.current_token() != TokenKind::EndOfFile {
            expressions.push(self.parse_expression());

            if self.check_token(TokenKind::Comma) {
                self.next_token();
            } else {
                break;
            }
        }

        expressions
    }

    fn parse_optional_list(
        &mut self,
        open: TokenKind,
        close: TokenKind,
        delimiter: TokenKind,
        parse_item: fn(&mut Self) -> u32,
    ) -> Vec<u32> {
        if !self.check_token(open) {
            return vec![];
        }

        self.next_token();
        let mut items = Vec::new();

        while self.current_token() != close && self.current_token() != TokenKind::EndOfFile {
            let item = parse_item(self);
            items.push(item);

            if self.check_token(delimiter.clone()) {
                self.next_token();
            } else {
                break;
            }
        }

        self.expect_token(close);
        items
    }

    // ==================== Identifier Parsing ====================

    /// Parse an identifier - allows reserved keywords (like `fn`) which is valid TypeSpec
    fn parse_identifier(&mut self) -> u32 {
        self.parse_identifier_impl(true, false)
    }

    /// Parse an identifier, optionally allowing string literals (for model property names)
    fn parse_identifier_allow_string(&mut self) -> u32 {
        self.parse_identifier_impl(true, true)
    }

    /// Parse model property name - allows identifiers and string literals, and reserved keywords
    fn parse_model_property_name(&mut self) -> u32 {
        self.parse_identifier_impl(true, true)
    }

    fn parse_identifier_impl(&mut self, allow_reserved: bool, allow_string_literal: bool) -> u32 {
        let pos = self.token_start_position();

        // Handle string literal as identifier (for quoted property names like "prop-name")
        if allow_string_literal && self.current_token() == TokenKind::StringLiteral {
            let value = self.token_value();
            let span = self.make_span(pos, self.previous_token_end);
            self.next_token();
            return self.builder.create_identifier(value, span);
        }

        if !self.is_identifier_token(self.current_token(), allow_reserved) {
            self.error("expected-identifier", "Expected an identifier");
            let span = self.make_span(pos, self.previous_token_end);
            return self.builder.create_identifier("<missing>".to_string(), span);
        }

        let value = self.token_value();
        let span = self.make_span(pos, self.previous_token_end);
        self.next_token();
        self.builder.create_identifier(value, span)
    }

    fn is_identifier_token(&self, token: TokenKind, allow_reserved: bool) -> bool {
        if token == TokenKind::Identifier {
            return true;
        }
        if allow_reserved {
            return self.is_reserved_identifier(token);
        }
        false
    }

    fn is_reserved_identifier(&self, token: TokenKind) -> bool {
        matches!(
            token,
            TokenKind::ModelKeyword
                | TokenKind::EnumKeyword
                | TokenKind::InterfaceKeyword
                | TokenKind::UnionKeyword
                | TokenKind::NamespaceKeyword
                | TokenKind::UsingKeyword
                | TokenKind::OpKeyword
                | TokenKind::ScalarKeyword
                | TokenKind::AliasKeyword
                | TokenKind::ConstKeyword
                | TokenKind::ImportKeyword
                | TokenKind::ExtendsKeyword
                | TokenKind::IsKeyword
                | TokenKind::FnKeyword
                | TokenKind::DecKeyword
                | TokenKind::ExternKeyword
                | TokenKind::InternalKeyword
                | TokenKind::InitKeyword
                | TokenKind::ValueOfKeyword
                | TokenKind::TypeOfKeyword
                | TokenKind::TrueKeyword
                | TokenKind::FalseKeyword
                | TokenKind::VoidKeyword
                | TokenKind::NeverKeyword
                | TokenKind::UnknownKeyword
        )
    }

    fn parse_string_literal(&mut self) -> u32 {
        let pos = self.token_start_position();
        let value = self.token_value();
        let span = self.make_span(pos, self.previous_token_end);
        self.next_token();
        self.builder.create_string_literal(value, span)
    }

    // ==================== Decorator Parsing ====================

    fn parse_decorator_list(&mut self) -> Vec<u32> {
        let mut decorators = Vec::new();

        while self.check_token(TokenKind::At) {
            self.next_token();
            let pos = self.token_start_position();
            let target = self.parse_identifier_or_member_expression(true, true);
            let args = if self.check_token(TokenKind::OpenParen) {
                self.next_token();
                let args = self.parse_expression_list(TokenKind::CloseParen);
                self.expect_token(TokenKind::CloseParen);
                args
            } else {
                vec![]
            };

            let span = self.make_span(pos, self.previous_token_end);
            let dec_id = self.builder.create_decorator_expression(target, args, span);
            decorators.push(dec_id);
        }

        decorators
    }

    fn parse_modifiers(&mut self) -> Vec<u32> {
        let mut modifiers = Vec::new();

        loop {
            match self.current_token() {
                TokenKind::ExternKeyword => {
                    let span = self.make_span(self.token_start_position(), self.previous_token_end);
                    self.next_token();
                    modifiers.push(self.builder.create_modifier(ModifierKind::Extern, span));
                }
                TokenKind::InternalKeyword => {
                    let span = self.make_span(self.token_start_position(), self.previous_token_end);
                    self.next_token();
                    modifiers.push(self.builder.create_modifier(ModifierKind::Internal, span));
                }
                _ => break,
            }
        }

        modifiers
    }

    // ==================== Token Expectation Helpers ====================

    fn check_token(&self, token: TokenKind) -> bool {
        self.current_token() == token
    }

    fn expect_token(&mut self, token: TokenKind) {
        if self.current_token() == token {
            self.next_token();
        } else {
            self.error(
                "expected-token",
                &format!("Expected {:?}, got {:?}", token, self.current_token()),
            );
        }
    }
}

/// Parse TypeSpec source code into an AST
pub fn parse(source: &str) -> ParseResult {
    Parser::new(source, ParseOptions::default()).parse()
}
