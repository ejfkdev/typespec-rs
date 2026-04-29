//! TypeSpec Parser implementation
//!
//! This module implements the TypeSpec parser that converts token sequences
//! into AST nodes. It follows the structure of the TypeSpec compiler parser.

mod ast_builder;
pub mod parser_utils;

pub use ast_builder::AstBuilder;
pub use ast_builder::AstNode;

#[cfg(test)]
mod tests;

use crate::ast::token::Span;
use crate::ast::types::*;
use crate::scanner::{Lexer, TokenFlags, TokenKind};

/// Parser options
#[derive(Debug, Clone, Default)]
pub struct ParseOptions {}

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
    pub code: &'static str,
    pub message: String,
    pub span: Span,
}

/// Main parser struct
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: TokenKind,
    token_start: usize,
    previous_token_end: usize,
    builder: AstBuilder,
    diagnostics: Vec<ParseDiagnostic>,
}

impl<'a> Parser<'a> {
    /// Create a new parser for the given source
    pub fn new(source: &'a str, _options: ParseOptions) -> Self {
        let mut lexer = Lexer::new(source);
        let current_token = lexer.scan();
        let token_start = lexer.token_start_offset();
        Parser {
            lexer,
            current_token,
            token_start,
            previous_token_end: 0,
            builder: AstBuilder::new(source.to_string()),
            diagnostics: Vec::new(),
        }
    }

    /// Parse the source and return the result.
    /// Consumes the parser to avoid cloning the AST builder and diagnostics.
    pub fn parse(mut self) -> ParseResult {
        let root_id = self.parse_typespec_script();
        ParseResult {
            root_id,
            builder: self.builder,
            diagnostics: self.diagnostics,
        }
    }

    // ==================== Token Navigation ====================

    fn current_token(&self) -> TokenKind {
        self.current_token
    }

    fn token_start_position(&self) -> usize {
        self.token_start
    }

    fn token_value(&self) -> String {
        self.lexer.token_value()
    }

    fn token_text(&self) -> String {
        self.lexer.token_text().to_string()
    }

    fn next_token(&mut self) {
        self.previous_token_end = self.lexer.offset();
        self.current_token = self.lexer.scan();
        self.token_start = self.lexer.token_start_offset();
    }

    /// Skip trivia tokens (comments, newlines, whitespace) at the current position
    fn skip_trivia(&mut self) {
        while self.current_token().is_trivia() {
            self.next_token();
        }
    }

    fn make_span(&self, start: usize, end: usize) -> Span {
        self.builder.make_span(start, end)
    }

    // ==================== Error Handling ====================

    fn error(&mut self, code: &'static str, message: impl Into<String>) {
        let span = self.make_span(self.token_start, self.previous_token_end);
        self.diagnostics.push(ParseDiagnostic {
            code,
            message: message.into(),
            span,
        });
    }

    // ==================== Parsing Entry Points ====================

    fn parse_typespec_script(&mut self) -> u32 {
        let statements = self.parse_script_item_list();
        let span = self.make_span(0, self.previous_token_end);
        self.builder
            .create_typespec_script(statements, vec![], vec![], span)
    }

    fn parse_script_item_list(&mut self) -> Vec<u32> {
        let mut statements = Vec::new();

        while self.current_token() != TokenKind::EndOfFile {
            self.skip_trivia();
            if self.current_token() == TokenKind::EndOfFile {
                break;
            }

            let directives = self.parse_directive_list();
            let decorators = self.parse_decorator_list();
            let pos = self.token_start_position();

            let stmt_id = self.parse_statement_item(
                pos,
                decorators,
                vec![],
                &directives,
                /* top_level */ true,
            );
            statements.push(stmt_id);
        }

        statements
    }

    /// Parse a single statement item, shared by both script-level and namespace-body parsing.
    /// `top_level`: true for script items (supports @@augment, import), false for namespace body.
    fn parse_statement_item(
        &mut self,
        pos: usize,
        decorators: Vec<u32>,
        modifiers: Vec<u32>,
        directives: &[u32],
        top_level: bool,
    ) -> u32 {
        let stmt_id = match self.current_token() {
            TokenKind::AtAt if top_level => self.parse_augment_decorator_statement(pos),
            TokenKind::ImportKeyword if top_level => self.parse_import_statement(pos),
            TokenKind::UsingKeyword => {
                self.next_token();
                let name = self.parse_identifier_or_member_expression(false, false);
                self.expect_token(TokenKind::Semicolon);
                let span = self.make_span(pos, self.previous_token_end);
                self.builder.create_using_declaration(name, span)
            }
            TokenKind::NamespaceKeyword => {
                self.parse_namespace_statement(pos, decorators, modifiers)
            }
            TokenKind::ModelKeyword => self.parse_model_statement(pos, decorators, modifiers),
            TokenKind::ScalarKeyword => self.parse_scalar_statement(pos, decorators, modifiers),
            TokenKind::InterfaceKeyword => {
                self.parse_interface_statement(pos, decorators, modifiers)
            }
            TokenKind::UnionKeyword => self.parse_union_statement(pos, decorators, modifiers),
            TokenKind::OpKeyword => self.parse_operation_statement(pos, decorators, modifiers),
            TokenKind::EnumKeyword => self.parse_enum_statement(pos, decorators, modifiers),
            TokenKind::AliasKeyword => self.parse_alias_statement(pos, modifiers),
            TokenKind::ConstKeyword => self.parse_const_statement(pos, modifiers),
            TokenKind::ExternKeyword | TokenKind::InternalKeyword => {
                let mods = self.parse_modifiers();
                self.parse_declaration(pos, decorators, mods)
            }
            TokenKind::FnKeyword => self.parse_function_declaration_statement(pos, modifiers),
            TokenKind::DecKeyword => self.parse_decorator_declaration_statement(pos, modifiers),
            TokenKind::Semicolon => {
                self.next_token();
                let span = self.make_span(pos, self.previous_token_end);
                self.builder.create_empty_statement(span)
            }
            _ => {
                let msg = if top_level {
                    format!("Unexpected token: {:?}", self.current_token())
                } else {
                    "Expected a statement".to_string()
                };
                self.error("unexpected-token", msg);
                // Error recovery: skip until we find a statement keyword or boundary
                self.next_token();
                let boundary_tokens = if top_level {
                    vec![TokenKind::EndOfFile, TokenKind::At, TokenKind::Semicolon]
                } else {
                    vec![TokenKind::CloseBrace, TokenKind::EndOfFile]
                };
                while !self.current_token().is_statement_keyword()
                    && !boundary_tokens.contains(&self.current_token())
                {
                    self.next_token();
                }
                let span = self.make_span(pos, self.previous_token_end);
                self.builder.create_invalid_statement(decorators, span)
            }
        };

        if !directives.is_empty() {
            self.builder.attach_directives(stmt_id, directives.to_vec());
        }
        stmt_id
    }

    fn parse_augment_decorator_statement(&mut self, pos: usize) -> u32 {
        self.next_token(); // consume @@
        let target = self.parse_identifier_or_member_expression(true, true);
        let args = self.parse_optional_list(
            TokenKind::OpenParen,
            TokenKind::CloseParen,
            TokenKind::Comma,
            Self::parse_expression,
        );
        let span = self.make_span(pos, self.previous_token_end);

        let (target_type, decorator_args) = if args.is_empty() {
            let err_id = self
                .builder
                .create_identifier(String::new(), self.make_span(pos, pos));
            (err_id, args)
        } else {
            let mut args = args;
            let target_entity = args.remove(0);
            (target_entity, args)
        };

        let id = self.builder.create_augment_decorator_statement(
            target,
            target_type,
            decorator_args,
            span,
        );
        self.expect_token(TokenKind::Semicolon);
        id
    }

    // ==================== Statement Parsing ====================

    fn parse_namespace_statement(
        &mut self,
        pos: usize,
        decorators: Vec<u32>,
        modifiers: Vec<u32>,
    ) -> u32 {
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
        self.builder
            .create_namespace_declaration(name, statements, decorators, modifiers, span)
    }

    fn parse_statement_list(&mut self) -> Vec<u32> {
        let mut statements = Vec::new();

        while self.current_token() != TokenKind::CloseBrace
            && self.current_token() != TokenKind::EndOfFile
        {
            let directives = self.parse_directive_list();
            let decorators = self.parse_decorator_list();
            let modifiers = self.parse_modifiers();
            let pos = self.token_start_position();

            let stmt_id = self.parse_statement_item(
                pos,
                decorators,
                modifiers,
                &directives,
                /* top_level */ false,
            );
            statements.push(stmt_id);
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

    fn parse_model_statement(
        &mut self,
        pos: usize,
        decorators: Vec<u32>,
        modifiers: Vec<u32>,
    ) -> u32 {
        self.expect_token(TokenKind::ModelKeyword);
        let name = self.parse_identifier();

        let template_parameters = self.parse_optional_template_parameters();

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
        self.builder.create_model_declaration(ModelDeclaration {
            id: 0, // auto-generated
            span,
            name,
            properties,
            extends,
            is,
            decorators,
            template_parameters,
            body_range,
            modifiers,
        })
    }

    /// Parse a property list (used by both model bodies and operation parameters).
    /// `end_token` — the token that ends this block (CloseBrace for models, CloseParen for ops)
    /// `error_code` / `error_msg` — diagnostic for missing delimiter
    fn parse_property_list(
        &mut self,
        end_token: TokenKind,
        error_code: &'static str,
        error_msg: &'static str,
    ) -> Vec<u32> {
        let mut properties = Vec::new();

        while self.current_token() != end_token && self.current_token() != TokenKind::EndOfFile {
            if self.check_token(TokenKind::Ellipsis) {
                let pos = self.token_start_position();
                self.next_token();
                let target = self.parse_expression();
                let span = self.make_span(pos, self.previous_token_end);
                let spread_id = self.builder.create_model_spread_property(target, span);
                properties.push(spread_id);
            } else {
                let directives = self.parse_directive_list();
                let decorators = self.parse_decorator_list();
                let pos = self.token_start_position();
                let name = self.parse_identifier_allow_string();

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
                let prop_id = self
                    .builder
                    .create_model_property(name, value, decorators, optional, default, span);
                if !directives.is_empty() {
                    self.builder.attach_directives(prop_id, directives);
                }
                properties.push(prop_id);
            }

            // Handle delimiter - both comma and semicolon are valid
            if self.check_token(TokenKind::Comma) || self.check_token(TokenKind::Semicolon) {
                self.next_token();
            } else if self.current_token() != end_token {
                // Error recovery - assume missing delimiter
                if self.current_token() != TokenKind::EndOfFile {
                    self.error(error_code, error_msg);
                }
                break;
            }
        }

        properties
    }

    /// Parse operation parameters (typed parameters inside parentheses)
    fn parse_operation_parameters(&mut self) -> Vec<u32> {
        self.parse_property_list(
            TokenKind::CloseParen,
            "expected-comma",
            "Expected ',' or ')'",
        )
    }

    fn parse_model_property_list(&mut self) -> Vec<u32> {
        self.parse_property_list(
            TokenKind::CloseBrace,
            "expected-semicolon",
            "Expected ';' or ','",
        )
    }

    fn parse_interface_statement(
        &mut self,
        pos: usize,
        decorators: Vec<u32>,
        modifiers: Vec<u32>,
    ) -> u32 {
        self.expect_token(TokenKind::InterfaceKeyword);
        let name = self.parse_identifier();

        let template_parameters = self.parse_optional_template_parameters();

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
        self.builder
            .create_interface_declaration(InterfaceDeclaration {
                id: 0, // auto-generated
                span,
                name,
                operations,
                extends,
                decorators,
                template_parameters,
                body_range,
                modifiers,
            })
    }

    /// Parse template parameters and operation signature (shared by parse_operation_list
    /// and parse_operation_statement).
    fn parse_operation_signature(&mut self, pos: usize) -> (Vec<u32>, u32) {
        let template_parameters = self.parse_optional_template_parameters();

        let signature = if self.check_token(TokenKind::OpenParen) {
            self.next_token();
            let params = self.parse_operation_parameters();
            self.expect_token(TokenKind::CloseParen);
            self.expect_token(TokenKind::Colon);
            let return_type = self.parse_expression();
            let span = self.make_span(pos, self.previous_token_end);
            let params_model = self.builder.create_model_expression(params, span, span);
            self.builder
                .create_operation_signature_declaration(params_model, return_type, span)
        } else {
            self.expect_token(TokenKind::IsKeyword);
            let base_op = self.parse_expression();
            let span = self.make_span(pos, self.previous_token_end);
            self.builder
                .create_operation_signature_reference(base_op, span)
        };

        (template_parameters, signature)
    }

    fn parse_operation_list(&mut self) -> Vec<u32> {
        let mut operations = Vec::new();

        while self.current_token() != TokenKind::CloseBrace
            && self.current_token() != TokenKind::EndOfFile
        {
            let directives = self.parse_directive_list();
            let decorators = self.parse_decorator_list();
            let pos = self.token_start_position();

            // OpKeyword is optional inside interface body
            if self.check_token(TokenKind::OpKeyword) {
                self.next_token();
            }
            let name = self.parse_identifier();

            let (template_parameters, signature) = self.parse_operation_signature(pos);

            // Interface operations can be delimited by semicolons or commas
            if self.check_token(TokenKind::Semicolon) || self.check_token(TokenKind::Comma) {
                self.next_token();
            }

            let span = self.make_span(pos, self.previous_token_end);
            let op_id = self.builder.create_operation_declaration(
                name,
                signature,
                decorators,
                template_parameters,
                vec![],
                span,
            );
            if !directives.is_empty() {
                self.builder.attach_directives(op_id, directives);
            }
            operations.push(op_id);
        }

        operations
    }

    fn parse_union_statement(
        &mut self,
        pos: usize,
        decorators: Vec<u32>,
        modifiers: Vec<u32>,
    ) -> u32 {
        self.expect_token(TokenKind::UnionKeyword);
        let name = self.parse_identifier();

        let template_parameters = self.parse_optional_template_parameters();

        // Union variants
        self.expect_token(TokenKind::OpenBrace);
        let variants = self.parse_union_variant_list();
        self.expect_token(TokenKind::CloseBrace);

        let span = self.make_span(pos, self.previous_token_end);
        self.builder.create_union_declaration(
            name,
            variants,
            decorators,
            template_parameters,
            modifiers,
            span,
        )
    }

    fn parse_union_variant_list(&mut self) -> Vec<u32> {
        let mut variants = Vec::new();

        while self.current_token() != TokenKind::CloseBrace
            && self.current_token() != TokenKind::EndOfFile
        {
            let directives = self.parse_directive_list();
            let decorators = self.parse_decorator_list();
            let pos = self.token_start_position();

            // Check for identifier or value
            // TypeSpec syntax: variantName: type  OR  just type (unnamed variant)
            // String literals can also serve as variant names: "hi there": type
            // This matches TS parseIdOrValueForVariant + parseOptional(Colon)
            let (name, value) = if self.current_token() == TokenKind::Identifier
                || self.is_reserved_identifier(self.current_token())
            {
                // Identifier or reserved keyword used as identifier
                let id = self.parse_identifier();
                if self.check_token(TokenKind::Colon) {
                    self.next_token();
                    let val = self.parse_expression();
                    (Some(id), val)
                } else {
                    // No colon - this is a type reference (unnamed variant)
                    let template_args = if self.check_token(TokenKind::LessThan) {
                        self.parse_template_argument_list()
                    } else {
                        vec![]
                    };
                    let span = self.make_span(pos, self.previous_token_end);
                    let type_ref = self.builder.create_type_reference(id, template_args, span);
                    (None, type_ref)
                }
            } else if self.current_token() == TokenKind::StringLiteral {
                // String literal: could be "name": type or just a string value
                // Parse as identifier (allows string literal) first, then check for colon
                let id = self.parse_identifier_allow_string();
                if self.check_token(TokenKind::Colon) {
                    // "name": type pattern - string literal is the variant name
                    self.next_token();
                    let val = self.parse_expression();
                    (Some(id), val)
                } else {
                    // Just a string value (unnamed variant)
                    // The identifier was created from the string literal - use it as a string literal value
                    (None, id)
                }
            } else {
                let val = self.parse_expression();
                (None, val)
            };

            let span = self.make_span(pos, self.previous_token_end);
            let variant_id = self
                .builder
                .create_union_variant(name, value, decorators, span);
            if !directives.is_empty() {
                self.builder.attach_directives(variant_id, directives);
            }
            variants.push(variant_id);

            // Handle delimiter
            if self.check_token(TokenKind::Semicolon) || self.check_token(TokenKind::Comma) {
                self.next_token();
            } else if self.current_token() != TokenKind::CloseBrace {
                break;
            }
        }

        variants
    }

    fn parse_enum_statement(
        &mut self,
        pos: usize,
        decorators: Vec<u32>,
        modifiers: Vec<u32>,
    ) -> u32 {
        self.expect_token(TokenKind::EnumKeyword);
        let name = self.parse_identifier();

        self.expect_token(TokenKind::OpenBrace);
        let members = self.parse_enum_member_list();
        self.expect_token(TokenKind::CloseBrace);

        let span = self.make_span(pos, self.previous_token_end);
        self.builder
            .create_enum_declaration(name, members, decorators, modifiers, span)
    }

    fn parse_enum_member_list(&mut self) -> Vec<u32> {
        let mut members = Vec::new();

        while self.current_token() != TokenKind::CloseBrace
            && self.current_token() != TokenKind::EndOfFile
        {
            if self.check_token(TokenKind::Ellipsis) {
                let pos = self.token_start_position();
                self.next_token();
                let target = self.parse_expression();
                let span = self.make_span(pos, self.previous_token_end);
                let spread_id = self.builder.create_enum_spread_member(target, span);
                members.push(spread_id);
            } else {
                let directives = self.parse_directive_list();
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
                let member_id = self
                    .builder
                    .create_enum_member(name, value, decorators, span);
                if !directives.is_empty() {
                    self.builder.attach_directives(member_id, directives);
                }
                members.push(member_id);
            }

            // Handle delimiter
            if self.check_token(TokenKind::Semicolon) || self.check_token(TokenKind::Comma) {
                self.next_token();
            } else if self.current_token() != TokenKind::CloseBrace {
                break;
            }
        }

        members
    }

    fn parse_scalar_statement(
        &mut self,
        pos: usize,
        decorators: Vec<u32>,
        modifiers: Vec<u32>,
    ) -> u32 {
        self.expect_token(TokenKind::ScalarKeyword);
        let name = self.parse_identifier();

        let template_parameters = self.parse_optional_template_parameters();

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
        self.builder.create_scalar_declaration(ScalarDeclaration {
            id: 0, // auto-generated
            span,
            name,
            template_parameters,
            extends,
            decorators,
            constructors,
            body_range,
            modifiers,
        })
    }

    fn parse_scalar_constructor_list(&mut self) -> Vec<u32> {
        let mut constructors = Vec::new();

        while self.current_token() != TokenKind::CloseBrace
            && self.current_token() != TokenKind::EndOfFile
        {
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

            if self.check_token(TokenKind::Semicolon) || self.check_token(TokenKind::Comma) {
                self.next_token();
            }
        }

        constructors
    }

    fn parse_operation_statement(
        &mut self,
        pos: usize,
        decorators: Vec<u32>,
        modifiers: Vec<u32>,
    ) -> u32 {
        self.expect_token(TokenKind::OpKeyword);
        let name = self.parse_identifier();

        let (template_parameters, signature) = self.parse_operation_signature(pos);

        self.expect_token(TokenKind::Semicolon);

        let span = self.make_span(pos, self.previous_token_end);
        self.builder.create_operation_declaration(
            name,
            signature,
            decorators,
            template_parameters,
            modifiers,
            span,
        )
    }

    fn parse_alias_statement(&mut self, pos: usize, modifiers: Vec<u32>) -> u32 {
        self.expect_token(TokenKind::AliasKeyword);
        let name = self.parse_identifier();

        let template_parameters = self.parse_optional_template_parameters();

        self.expect_token(TokenKind::Equals);
        let value = self.parse_expression();
        self.expect_token(TokenKind::Semicolon);

        let span = self.make_span(pos, self.previous_token_end);
        self.builder
            .create_alias_statement(name, value, template_parameters, modifiers, span)
    }

    fn parse_const_statement(&mut self, pos: usize, modifiers: Vec<u32>) -> u32 {
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
        self.builder
            .create_const_statement(name, value, type_annotation, modifiers, span)
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
            Some(self.parse_mixed_constraint())
        } else {
            None
        };

        self.expect_token(TokenKind::Semicolon);

        let span = self.make_span(pos, self.previous_token_end);
        self.builder
            .create_function_declaration(name, params, return_type, modifiers, span)
    }

    fn parse_function_parameter_list(&mut self) -> Vec<u32> {
        let mut params = Vec::new();

        while self.current_token() != TokenKind::CloseParen
            && self.current_token() != TokenKind::EndOfFile
        {
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
                Some(self.parse_mixed_constraint())
            } else {
                None
            };

            let span = self.make_span(pos, self.previous_token_end);
            let param_id =
                self.builder
                    .create_function_parameter(name, type_annotation, optional, rest, span);
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
            self.builder
                .create_function_parameter(0, None, false, false, span)
        });

        self.expect_token(TokenKind::Semicolon);

        let span = self.make_span(pos, self.previous_token_end);
        self.builder.create_decorator_declaration(
            name,
            target,
            params[1..].to_vec(),
            modifiers,
            span,
        )
    }

    fn parse_declaration(&mut self, pos: usize, decorators: Vec<u32>, modifiers: Vec<u32>) -> u32 {
        match self.current_token() {
            TokenKind::ModelKeyword => self.parse_model_statement(pos, decorators, modifiers),
            TokenKind::ScalarKeyword => self.parse_scalar_statement(pos, decorators, modifiers),
            TokenKind::NamespaceKeyword => {
                self.parse_namespace_statement(pos, decorators, modifiers)
            }
            TokenKind::InterfaceKeyword => {
                self.parse_interface_statement(pos, decorators, modifiers)
            }
            TokenKind::UnionKeyword => self.parse_union_statement(pos, decorators, modifiers),
            TokenKind::OpKeyword => self.parse_operation_statement(pos, decorators, modifiers),
            TokenKind::EnumKeyword => self.parse_enum_statement(pos, decorators, modifiers),
            TokenKind::AliasKeyword => self.parse_alias_statement(pos, modifiers),
            TokenKind::ConstKeyword => self.parse_const_statement(pos, modifiers),
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
            TokenKind::Identifier => self.parse_call_or_reference_expression(),
            TokenKind::StringLiteral => {
                let value = self.token_value();
                let span = self.make_span(pos, self.previous_token_end);
                self.next_token();
                self.builder.create_string_literal(value, span)
            }
            TokenKind::StringTemplateHead => self.parse_string_template_expression(),
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
                self.builder
                    .create_model_expression(props, body_range, span)
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
            TokenKind::HashBrace => self.parse_object_literal(),
            TokenKind::HashBracket => self.parse_array_literal(),
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
            TokenKind::FnKeyword => self.parse_function_type_expression(),
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
                self.builder
                    .create_identifier("<missing>".to_string(), span)
            }
        }
    }

    /// Parse a string template expression (e.g., `"Start ${expr} end"`)
    /// Called when the scanner produces a StringTemplateHead token.
    fn parse_string_template_expression(&mut self) -> u32 {
        let pos = self.token_start_position();

        let head_value = self.token_value();
        let head_span = self.make_span(pos, self.previous_token_end);
        let head_id =
            self.builder
                .create_string_template_head(head_value, TokenFlags::NONE, head_span);
        self.next_token();

        let mut spans = Vec::new();
        let mut done = false;

        while !done {
            // Skip trivia before the interpolated expression
            self.skip_trivia();

            let expr = self.parse_expression();

            // After parsing the expression, we should be at a CloseBrace (the `}` that closes ${...})
            // When we encounter it, re-scan to get the next template token (Middle or Tail)
            if self.current_token() == TokenKind::CloseBrace {
                self.current_token = self.lexer.re_scan_string_template(TokenFlags::NONE);
            }

            let literal_value = self.token_value();
            let literal_span = self.make_span(self.token_start_position(), self.previous_token_end);

            let literal_id = match self.current_token() {
                TokenKind::StringTemplateMiddle => {
                    let id = self
                        .builder
                        .create_string_template_middle(literal_value, literal_span);
                    self.next_token();
                    id
                }
                TokenKind::StringTemplateTail => {
                    let id = self
                        .builder
                        .create_string_template_tail(literal_value, literal_span);
                    self.next_token();
                    done = true;
                    id
                }
                _ => {
                    let id = self
                        .builder
                        .create_string_template_tail(String::new(), literal_span);
                    done = true;
                    id
                }
            };

            let span_span = self.make_span(pos, self.previous_token_end);
            let span_id = self
                .builder
                .create_string_template_span(expr, literal_id, span_span);
            spans.push(span_id);
        }

        let span = self.make_span(pos, self.previous_token_end);
        self.builder
            .create_string_template_expression(head_id, spans, span)
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
            // Always create a TypeReference node for type references,
            // even when there are no template arguments.
            // This allows the checker to distinguish type references from identifiers.
            let args = if self.check_token(TokenKind::LessThan) {
                self.parse_template_argument_list()
            } else {
                vec![]
            };
            let span = self.make_span(pos, self.previous_token_end);
            self.builder.create_type_reference(target, args, span)
        }
    }

    fn parse_identifier_or_member_expression(
        &mut self,
        _allow_reserved: bool,
        _allow_reserved_in_member: bool,
    ) -> u32 {
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
            base = self
                .builder
                .create_member_expression(base, property, selector, span);
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
            Some(self.parse_mixed_constraint())
        } else {
            None
        };

        let span = self.make_span(pos, self.previous_token_end);
        self.builder
            .create_function_type_expression(params, return_type, span)
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
        let mut args = Vec::new();

        while self.current_token() != TokenKind::GreaterThan
            && self.current_token() != TokenKind::EndOfFile
        {
            let pos = self.token_start_position();

            // Check for named argument: Identifier = Expression
            // In TypeSpec: A<T = string> or A<U = int32, T = string>
            let arg = if self.current_token() == TokenKind::Identifier {
                let name_id = self.parse_identifier();

                if self.check_token(TokenKind::Equals) {
                    // Named argument: T = string
                    self.next_token();
                    let value = self.parse_expression();
                    let span = self.make_span(pos, self.previous_token_end);
                    self.builder
                        .create_template_argument(Some(name_id), value, span)
                } else if self.check_token(TokenKind::LessThan) {
                    // The identifier is followed by `<...>` — this is a TypeReference
                    // with template arguments (e.g., T<string>). If this later turns
                    // out to be a named argument (T<string> = value), it's invalid
                    // because parameter names must be bare identifiers.
                    // TS: parseTemplateArgument → if eq && !isBareIdentifier → error
                    let template_args = self.parse_template_argument_list();
                    if self.check_token(TokenKind::Equals) {
                        // T<string> = value — invalid: parameter name must be bare identifier
                        // Report diagnostic and use a missing identifier as name
                        self.error(
                            "invalid-template-argument-name",
                            "Template argument name cannot have template arguments.",
                        );
                        self.next_token(); // consume =
                        let value = self.parse_expression();
                        let span = self.make_span(pos, self.previous_token_end);
                        self.builder.create_template_argument(None, value, span)
                    } else {
                        // Just a type reference with args: T<string>
                        let value = {
                            let span = self.make_span(pos, self.previous_token_end);
                            self.builder
                                .create_type_reference(name_id, template_args, span)
                        };
                        let span = self.make_span(pos, self.previous_token_end);
                        self.builder.create_template_argument(None, value, span)
                    }
                } else {
                    // The identifier is a simple type reference (no template args)
                    let value = {
                        let span = self.make_span(pos, self.previous_token_end);
                        self.builder.create_type_reference(name_id, vec![], span)
                    };
                    let span = self.make_span(pos, self.previous_token_end);
                    self.builder.create_template_argument(None, value, span)
                }
            } else {
                // Positional argument (non-identifier start, e.g., string literal, model expression)
                let value = self.parse_expression();
                let span = self.make_span(pos, self.previous_token_end);
                self.builder.create_template_argument(None, value, span)
            };

            args.push(arg);

            if self.check_token(TokenKind::Comma) {
                self.next_token();
            } else {
                break;
            }
        }

        self.expect_token(TokenKind::GreaterThan);
        args
    }

    /// Parse optional template parameters (`<T, U>`), returning empty vec if none.
    fn parse_optional_template_parameters(&mut self) -> Vec<u32> {
        if self.check_token(TokenKind::LessThan) {
            self.parse_template_parameter_list()
        } else {
            vec![]
        }
    }

    fn parse_template_parameter_list(&mut self) -> Vec<u32> {
        self.expect_token(TokenKind::LessThan);
        let mut params = Vec::new();

        while self.current_token() != TokenKind::GreaterThan
            && self.current_token() != TokenKind::EndOfFile
        {
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
            let param_id = self
                .builder
                .create_template_parameter_declaration(name, constraint, default, span);
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

        while self.current_token() != TokenKind::CloseBrace
            && self.current_token() != TokenKind::EndOfFile
        {
            let prop_pos = self.token_start_position();
            if self.check_token(TokenKind::Ellipsis) {
                self.next_token();
                let target = self.parse_expression();
                let span = self.make_span(prop_pos, self.previous_token_end);
                let spread_id = self
                    .builder
                    .create_object_literal_spread_property(target, span);
                properties.push(spread_id);
            } else {
                let key = self.parse_identifier();
                self.expect_token(TokenKind::Colon);
                let value = self.parse_expression();
                let span = self.make_span(prop_pos, self.previous_token_end);
                let prop_id = self
                    .builder
                    .create_object_literal_property(key, value, span);
                properties.push(prop_id);
            }

            if self.check_token(TokenKind::Comma) {
                self.next_token();
            } else {
                break;
            }
        }

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

            if self.check_token(delimiter) {
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
            return self
                .builder
                .create_identifier("<missing>".to_string(), span);
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
                | TokenKind::SelfKeyword
                | TokenKind::PropKeyword
                | TokenKind::PropertyKeyword
        )
    }

    fn parse_string_literal(&mut self) -> u32 {
        let pos = self.token_start_position();
        let value = self.token_value();
        let span = self.make_span(pos, self.previous_token_end);
        self.next_token();
        self.builder.create_string_literal(value, span)
    }

    // ==================== Directive Parsing ====================

    /// Parse directive expressions (#deprecated, #suppress) that precede declarations.
    /// Ported from TS parser.ts parseDirectiveList()
    fn parse_directive_list(&mut self) -> Vec<u32> {
        let mut directives = Vec::new();

        while self.check_token(TokenKind::Hash) {
            // Only parse if the next token after # is an identifier (not { or [)
            // Peek: save state, check next token
            let pos = self.token_start_position();
            self.next_token(); // consume #

            // Check that next token is an identifier (not { or [)
            if !self.is_identifier_token(self.current_token(), true)
                && self.current_token() != TokenKind::Identifier
            {
                // Not a directive, put back the hash
                // We can't un-get, so just skip this (it's #{ or #[ which are handled elsewhere)
                // Actually, since we already consumed #, we need to handle this differently.
                // The Hash token should only be consumed if followed by an identifier.
                // For now, just create a missing identifier and report error.
                break;
            }

            let target = self.parse_identifier();
            let target_name = match self.builder.id_to_node(target) {
                Some(AstNode::Identifier(id)) => id.value.clone(),
                _ => String::new(),
            };

            // Only "deprecated" and "suppress" are valid directives
            if target_name != "deprecated" && target_name != "suppress" {
                self.error(
                    "unknown-directive",
                    format!(
                        "Unknown directive '#{}'. Only #deprecated and #suppress are allowed.",
                        target_name
                    ),
                );
            }

            // Parse arguments until newline or EOF
            let mut args = Vec::new();
            while self.current_token() != TokenKind::EndOfFile && !self.current_token().is_trivia()
            {
                // Stop at newline or semicolon or statement keywords
                if self.current_token().is_statement_keyword()
                    || self.check_token(TokenKind::Semicolon)
                    || self.check_token(TokenKind::CloseBrace)
                {
                    break;
                }

                if self.check_token(TokenKind::StringLiteral) {
                    args.push(self.parse_string_literal());
                } else if self.check_token(TokenKind::Identifier) {
                    args.push(self.parse_identifier());
                } else {
                    break;
                }
            }

            // Skip to end of line
            while self.current_token().is_trivia() {
                self.next_token();
            }

            let span = self.make_span(pos, self.previous_token_end);
            let dir_id = self.builder.create_directive_expression(target, args, span);
            directives.push(dir_id);
        }

        directives
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
                format!("Expected {:?}, got {:?}", token, self.current_token()),
            );
        }
    }
}

/// Parse TypeSpec source code into an AST
pub fn parse(source: &str) -> ParseResult {
    Parser::new(source, ParseOptions::default()).parse()
}
