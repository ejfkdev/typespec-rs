//! AST Node Builder - Helper functions for creating AST nodes

use crate::ast::token::{Position, Span};
use crate::ast::types::*;
use crate::scanner::TokenFlags;
use std::collections::HashMap;

/// Node ID generator
#[derive(Debug, Clone)]
pub struct NodeIdGenerator {
    next_id: u32,
}

impl NodeIdGenerator {
    pub fn new() -> Self {
        NodeIdGenerator { next_id: 1 }
    }

    pub fn next(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

/// AST node storage for all nodes in a parsed file
#[derive(Debug, Clone)]
pub struct AstBuilder {
    pub node_id_gen: NodeIdGenerator,
    pub nodes: HashMap<u32, AstNode>,
    pub source: String,
    /// Map from declaration node ID to its directive nodes (e.g., #deprecated, #suppress)
    pub directives_map: HashMap<u32, Vec<u32>>,
}

#[derive(Debug, Clone)]
pub enum AstNode {
    TypeSpecScript(TypeSpecScript),
    Identifier(Identifier),
    MemberExpression(MemberExpression),
    StringLiteral(StringLiteral),
    NumericLiteral(NumericLiteral),
    BooleanLiteral(BooleanLiteral),
    ArrayExpression(ArrayExpression),
    TupleExpression(TupleExpression),
    UnionExpression(UnionExpression),
    IntersectionExpression(IntersectionExpression),
    TypeReference(TypeReference),
    CallExpression(CallExpression),
    ValueOfExpression(ValueOfExpression),
    TypeOfExpression(TypeOfExpression),
    StringTemplateExpression(StringTemplateExpression),
    StringTemplateHead(StringTemplateHead),
    StringTemplateMiddle(StringTemplateMiddle),
    StringTemplateTail(StringTemplateTail),
    StringTemplateSpan(StringTemplateSpan),
    ObjectLiteral(ObjectLiteral),
    ObjectLiteralProperty(ObjectLiteralProperty),
    ObjectLiteralSpreadProperty(ObjectLiteralSpreadProperty),
    ArrayLiteral(ArrayLiteral),
    ExternKeyword(ExternKeyword),
    InternalKeyword(InternalKeyword),
    VoidKeyword(VoidKeyword),
    NeverKeyword(NeverKeyword),
    UnknownKeyword(UnknownKeyword),
    ModelDeclaration(ModelDeclaration),
    ModelExpression(ModelExpression),
    ModelProperty(ModelProperty),
    ModelSpreadProperty(ModelSpreadProperty),
    ScalarDeclaration(ScalarDeclaration),
    ScalarConstructor(ScalarConstructor),
    InterfaceDeclaration(InterfaceDeclaration),
    UnionDeclaration(UnionDeclaration),
    UnionVariant(UnionVariant),
    EnumDeclaration(EnumDeclaration),
    EnumMember(EnumMember),
    EnumSpreadMember(EnumSpreadMember),
    NamespaceDeclaration(NamespaceDeclaration),
    UsingDeclaration(UsingDeclaration),
    OperationDeclaration(OperationDeclaration),
    OperationSignatureDeclaration(OperationSignatureDeclaration),
    OperationSignatureReference(OperationSignatureReference),
    AliasStatement(AliasStatement),
    ConstStatement(ConstStatement),
    DecoratorDeclaration(DecoratorDeclaration),
    DecoratorExpression(DecoratorExpression),
    AugmentDecoratorStatement(AugmentDecoratorStatement),
    FunctionDeclaration(FunctionDeclaration),
    FunctionParameter(FunctionParameter),
    FunctionTypeExpression(FunctionTypeExpression),
    TemplateParameterDeclaration(TemplateParameterDeclaration),
    TemplateArgument(TemplateArgument),
    ImportStatement(ImportStatement),
    DirectiveExpression(DirectiveExpression),
    Doc(Doc),
    DocText(DocText),
    LineComment(LineComment),
    BlockComment(BlockComment),
    EmptyStatement(EmptyStatement),
    InvalidStatement(InvalidStatement),
    Modifier(Modifier),
}

impl AstBuilder {
    pub fn new(source: String) -> Self {
        AstBuilder {
            node_id_gen: NodeIdGenerator::new(),
            nodes: HashMap::new(),
            source,
            directives_map: HashMap::new(),
        }
    }

    pub fn id_to_node(&self, id: u32) -> Option<&AstNode> {
        self.nodes.get(&id)
    }

    /// Attach directives to a declaration node
    pub fn attach_directives(&mut self, decl_node_id: u32, directives: Vec<u32>) {
        if !directives.is_empty() {
            self.directives_map.insert(decl_node_id, directives);
        }
    }

    /// Get directives attached to a declaration node
    pub fn get_directives(&self, decl_node_id: u32) -> Option<&Vec<u32>> {
        self.directives_map.get(&decl_node_id)
    }

    // Helper to create a span from byte offset positions
    pub fn make_span(&self, start_pos: usize, end_pos: usize) -> Span {
        let start = self.offset_to_position(start_pos);
        let end = self.offset_to_position(end_pos);
        Span { start, end }
    }

    /// Convert a byte offset in the source to a line:column Position.
    /// Line numbers are 1-based, column numbers are 0-based (matching TS convention).
    fn offset_to_position(&self, offset: usize) -> Position {
        if offset == 0 || self.source.is_empty() {
            return Position { line: 1, column: 0 };
        }
        let bytes = self.source.as_bytes();
        let safe_offset = offset.min(bytes.len());
        let mut line = 1u32;
        let mut last_line_start = 0usize;
        for (i, &byte) in bytes.iter().enumerate().take(safe_offset) {
            if byte == b'\n' {
                line += 1;
                last_line_start = i + 1;
            }
        }
        let column = (safe_offset - last_line_start) as u32;
        Position { line, column }
    }

    // ==================== Node Creation Methods ====================

    pub fn create_identifier(&mut self, value: String, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::Identifier(Identifier { id, span, value });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_member_expression(
        &mut self,
        object: u32,
        property: u32,
        selector: MemberSelector,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::MemberExpression(MemberExpression {
            id,
            span,
            object,
            property,
            selector,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_string_literal(&mut self, value: String, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::StringLiteral(StringLiteral { id, span, value });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_numeric_literal(
        &mut self,
        value: f64,
        value_as_string: String,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::NumericLiteral(NumericLiteral {
            id,
            span,
            value,
            value_as_string,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_boolean_literal(&mut self, value: bool, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::BooleanLiteral(BooleanLiteral { id, span, value });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_type_reference(&mut self, name: u32, arguments: Vec<u32>, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::TypeReference(TypeReference {
            id,
            span,
            name,
            arguments,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_call_expression(&mut self, target: u32, arguments: Vec<u32>, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::CallExpression(CallExpression {
            id,
            span,
            target,
            arguments,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_model_expression(
        &mut self,
        properties: Vec<u32>,
        body_range: Span,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::ModelExpression(ModelExpression {
            id,
            span,
            properties,
            body_range,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_model_property(
        &mut self,
        name: u32,
        value: u32,
        decorators: Vec<u32>,
        optional: bool,
        default: Option<u32>,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::ModelProperty(ModelProperty {
            id,
            span,
            name,
            value,
            decorators,
            optional,
            default,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_model_spread_property(&mut self, target: u32, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::ModelSpreadProperty(ModelSpreadProperty { id, span, target });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_model_declaration(&mut self, mut decl: ModelDeclaration) -> u32 {
        let id = self.node_id_gen.next();
        decl.id = id;
        let node = AstNode::ModelDeclaration(decl);
        self.nodes.insert(id, node);
        id
    }

    pub fn create_union_expression(&mut self, options: Vec<u32>, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::UnionExpression(UnionExpression { id, span, options });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_intersection_expression(&mut self, options: Vec<u32>, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::IntersectionExpression(IntersectionExpression { id, span, options });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_array_expression(&mut self, element_type: u32, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::ArrayExpression(ArrayExpression {
            id,
            span,
            element_type,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_tuple_expression(&mut self, values: Vec<u32>, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::TupleExpression(TupleExpression { id, span, values });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_value_of_expression(&mut self, target: u32, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::ValueOfExpression(ValueOfExpression { id, span, target });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_type_of_expression(&mut self, target: u32, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::TypeOfExpression(TypeOfExpression { id, span, target });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_import_statement(&mut self, path: u32, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::ImportStatement(ImportStatement { id, span, path });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_namespace_declaration(
        &mut self,
        name: u32,
        statements: Vec<u32>,
        decorators: Vec<u32>,
        modifiers: Vec<u32>,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::NamespaceDeclaration(NamespaceDeclaration {
            id,
            span,
            name,
            statements,
            decorators,
            modifiers,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_using_declaration(&mut self, name: u32, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::UsingDeclaration(UsingDeclaration { id, span, name });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_interface_declaration(&mut self, mut decl: InterfaceDeclaration) -> u32 {
        let id = self.node_id_gen.next();
        decl.id = id;
        let node = AstNode::InterfaceDeclaration(decl);
        self.nodes.insert(id, node);
        id
    }

    pub fn create_union_declaration(
        &mut self,
        name: u32,
        variants: Vec<u32>,
        decorators: Vec<u32>,
        template_parameters: Vec<u32>,
        modifiers: Vec<u32>,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::UnionDeclaration(UnionDeclaration {
            id,
            span,
            name,
            variants,
            decorators,
            template_parameters,
            modifiers,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_union_variant(
        &mut self,
        name: Option<u32>,
        value: u32,
        decorators: Vec<u32>,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::UnionVariant(UnionVariant {
            id,
            span,
            name,
            value,
            decorators,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_enum_declaration(
        &mut self,
        name: u32,
        members: Vec<u32>,
        decorators: Vec<u32>,
        modifiers: Vec<u32>,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::EnumDeclaration(EnumDeclaration {
            id,
            span,
            name,
            members,
            decorators,
            modifiers,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_enum_member(
        &mut self,
        name: u32,
        value: Option<u32>,
        decorators: Vec<u32>,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::EnumMember(EnumMember {
            id,
            span,
            name,
            value,
            decorators,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_enum_spread_member(&mut self, target: u32, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::EnumSpreadMember(EnumSpreadMember { id, span, target });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_alias_statement(
        &mut self,
        name: u32,
        value: u32,
        template_parameters: Vec<u32>,
        modifiers: Vec<u32>,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::AliasStatement(AliasStatement {
            id,
            span,
            name,
            value,
            template_parameters,
            modifiers,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_const_statement(
        &mut self,
        name: u32,
        value: u32,
        type_annotation: Option<u32>,
        modifiers: Vec<u32>,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::ConstStatement(ConstStatement {
            id,
            span,
            name,
            value,
            type_annotation,
            modifiers,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_scalar_declaration(&mut self, mut decl: ScalarDeclaration) -> u32 {
        let id = self.node_id_gen.next();
        decl.id = id;
        let node = AstNode::ScalarDeclaration(decl);
        self.nodes.insert(id, node);
        id
    }

    pub fn create_scalar_constructor(
        &mut self,
        name: u32,
        parameters: Vec<u32>,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::ScalarConstructor(ScalarConstructor {
            id,
            span,
            name,
            parameters,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_operation_declaration(
        &mut self,
        name: u32,
        signature: u32,
        decorators: Vec<u32>,
        template_parameters: Vec<u32>,
        modifiers: Vec<u32>,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::OperationDeclaration(OperationDeclaration {
            id,
            span,
            name,
            signature,
            decorators,
            template_parameters,
            modifiers,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_operation_signature_declaration(
        &mut self,
        parameters: u32,
        return_type: u32,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::OperationSignatureDeclaration(OperationSignatureDeclaration {
            id,
            span,
            parameters,
            return_type,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_operation_signature_reference(&mut self, base_operation: u32, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::OperationSignatureReference(OperationSignatureReference {
            id,
            span,
            base_operation,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_decorator_expression(
        &mut self,
        target: u32,
        arguments: Vec<u32>,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::DecoratorExpression(DecoratorExpression {
            id,
            span,
            target,
            arguments,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_augment_decorator_statement(
        &mut self,
        target: u32,
        target_type: u32,
        arguments: Vec<u32>,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::AugmentDecoratorStatement(AugmentDecoratorStatement {
            id,
            span,
            target,
            target_type,
            arguments,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_decorator_declaration(
        &mut self,
        name: u32,
        target: u32,
        parameters: Vec<u32>,
        modifiers: Vec<u32>,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::DecoratorDeclaration(DecoratorDeclaration {
            id,
            span,
            name,
            target,
            parameters,
            modifiers,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_function_declaration(
        &mut self,
        name: u32,
        parameters: Vec<u32>,
        return_type: Option<u32>,
        modifiers: Vec<u32>,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::FunctionDeclaration(FunctionDeclaration {
            id,
            span,
            name,
            parameters,
            return_type,
            modifiers,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_function_parameter(
        &mut self,
        name: u32,
        type_annotation: Option<u32>,
        optional: bool,
        rest: bool,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::FunctionParameter(FunctionParameter {
            id,
            span,
            name,
            type_annotation,
            optional,
            rest,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_function_type_expression(
        &mut self,
        parameters: Vec<u32>,
        return_type: Option<u32>,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::FunctionTypeExpression(FunctionTypeExpression {
            id,
            span,
            parameters,
            return_type,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_template_parameter_declaration(
        &mut self,
        name: u32,
        constraint: Option<u32>,
        default: Option<u32>,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::TemplateParameterDeclaration(TemplateParameterDeclaration {
            id,
            span,
            name,
            constraint,
            default,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_template_argument(
        &mut self,
        name: Option<u32>,
        argument: u32,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::TemplateArgument(TemplateArgument {
            id,
            span,
            name,
            argument,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_directive_expression(
        &mut self,
        target: u32,
        arguments: Vec<u32>,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::DirectiveExpression(DirectiveExpression {
            id,
            span,
            target,
            arguments,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_void_keyword(&mut self, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::VoidKeyword(VoidKeyword { id, span });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_never_keyword(&mut self, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::NeverKeyword(NeverKeyword { id, span });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_unknown_keyword(&mut self, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::UnknownKeyword(UnknownKeyword { id, span });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_extern_keyword(&mut self, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::ExternKeyword(ExternKeyword { id, span });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_internal_keyword(&mut self, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::InternalKeyword(InternalKeyword { id, span });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_empty_statement(&mut self, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::EmptyStatement(EmptyStatement { id, span });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_invalid_statement(&mut self, decorators: Vec<u32>, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::InvalidStatement(InvalidStatement {
            id,
            span,
            decorators,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_typespec_script(
        &mut self,
        statements: Vec<u32>,
        comments: Vec<u32>,
        parse_diagnostics: Vec<u32>,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::TypeSpecScript(TypeSpecScript {
            id,
            span,
            statements,
            comments,
            parse_diagnostics,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_object_literal(&mut self, properties: Vec<u32>, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::ObjectLiteral(ObjectLiteral {
            id,
            span,
            properties,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_object_literal_property(&mut self, key: u32, value: u32, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::ObjectLiteralProperty(ObjectLiteralProperty {
            id,
            span,
            key,
            value,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_object_literal_spread_property(&mut self, target: u32, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node =
            AstNode::ObjectLiteralSpreadProperty(ObjectLiteralSpreadProperty { id, span, target });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_array_literal(&mut self, values: Vec<u32>, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::ArrayLiteral(ArrayLiteral { id, span, values });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_string_template_expression(
        &mut self,
        head: u32,
        spans: Vec<u32>,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::StringTemplateExpression(StringTemplateExpression {
            id,
            span,
            head,
            spans,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_string_template_head(
        &mut self,
        value: String,
        _flags: TokenFlags,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::StringTemplateHead(StringTemplateHead { id, span, value });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_string_template_middle(&mut self, value: String, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::StringTemplateMiddle(StringTemplateMiddle { id, span, value });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_string_template_tail(&mut self, value: String, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::StringTemplateTail(StringTemplateTail { id, span, value });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_string_template_span(
        &mut self,
        expression: u32,
        literal: u32,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::StringTemplateSpan(StringTemplateSpan {
            id,
            span,
            expression,
            literal,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_modifier(&mut self, kind: ModifierKind, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::Modifier(Modifier { id, span, kind });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_line_comment(&mut self, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::LineComment(LineComment { id, span });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_block_comment(&mut self, parsed_as_docs: bool, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::BlockComment(BlockComment {
            id,
            span,
            parsed_as_docs,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_doc(&mut self, content: Vec<u32>, tags: Vec<u32>, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::Doc(Doc {
            id,
            span,
            content,
            tags,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_doc_text(&mut self, text: String, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::DocText(DocText { id, span, text });
        self.nodes.insert(id, node);
        id
    }
}
