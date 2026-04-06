//! AST Node Builder - Helper functions for creating AST nodes

use crate::ast::types::*;
use crate::ast::token::{Span, Position};
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
    DocParamTag(DocParamTag),
    DocPropTag(DocPropTag),
    DocReturnsTag(DocReturnsTag),
    DocErrorsTag(DocErrorsTag),
    DocTemplateTag(DocTemplateTag),
    DocUnknownTag(DocUnknownTag),
    LineComment(LineComment),
    BlockComment(BlockComment),
    EmptyStatement(EmptyStatement),
    InvalidStatement(InvalidStatement),
    JsNamespaceDeclaration(JsNamespaceDeclaration),
    Modifier(Modifier),
}

impl AstBuilder {
    pub fn new(source: String) -> Self {
        AstBuilder {
            node_id_gen: NodeIdGenerator::new(),
            nodes: HashMap::new(),
            source,
        }
    }

    pub fn id_to_node(&self, id: u32) -> Option<&AstNode> {
        self.nodes.get(&id)
    }

    pub fn set_parent(&mut self, child_id: u32, parent_id: u32) {
        if let Some(node) = self.nodes.get_mut(&child_id) {
            match node {
                AstNode::Identifier(n) => n.id = parent_id,
                AstNode::MemberExpression(n) => n.id = parent_id,
                AstNode::StringLiteral(n) => n.id = parent_id,
                AstNode::NumericLiteral(n) => n.id = parent_id,
                AstNode::BooleanLiteral(n) => n.id = parent_id,
                AstNode::ArrayExpression(n) => n.id = parent_id,
                AstNode::TupleExpression(n) => n.id = parent_id,
                AstNode::UnionExpression(n) => n.id = parent_id,
                AstNode::IntersectionExpression(n) => n.id = parent_id,
                AstNode::TypeReference(n) => n.id = parent_id,
                AstNode::CallExpression(n) => n.id = parent_id,
                AstNode::ValueOfExpression(n) => n.id = parent_id,
                AstNode::TypeOfExpression(n) => n.id = parent_id,
                AstNode::StringTemplateExpression(n) => n.id = parent_id,
                AstNode::StringTemplateHead(n) => n.id = parent_id,
                AstNode::StringTemplateMiddle(n) => n.id = parent_id,
                AstNode::StringTemplateTail(n) => n.id = parent_id,
                AstNode::StringTemplateSpan(n) => n.id = parent_id,
                AstNode::ObjectLiteral(n) => n.id = parent_id,
                AstNode::ObjectLiteralProperty(n) => n.id = parent_id,
                AstNode::ObjectLiteralSpreadProperty(n) => n.id = parent_id,
                AstNode::ArrayLiteral(n) => n.id = parent_id,
                AstNode::ExternKeyword(n) => n.id = parent_id,
                AstNode::InternalKeyword(n) => n.id = parent_id,
                AstNode::VoidKeyword(n) => n.id = parent_id,
                AstNode::NeverKeyword(n) => n.id = parent_id,
                AstNode::UnknownKeyword(n) => n.id = parent_id,
                AstNode::ModelDeclaration(n) => n.id = parent_id,
                AstNode::ModelExpression(n) => n.id = parent_id,
                AstNode::ModelProperty(n) => n.id = parent_id,
                AstNode::ModelSpreadProperty(n) => n.id = parent_id,
                AstNode::ScalarDeclaration(n) => n.id = parent_id,
                AstNode::ScalarConstructor(n) => n.id = parent_id,
                AstNode::InterfaceDeclaration(n) => n.id = parent_id,
                AstNode::UnionDeclaration(n) => n.id = parent_id,
                AstNode::UnionVariant(n) => n.id = parent_id,
                AstNode::EnumDeclaration(n) => n.id = parent_id,
                AstNode::EnumMember(n) => n.id = parent_id,
                AstNode::EnumSpreadMember(n) => n.id = parent_id,
                AstNode::NamespaceDeclaration(n) => n.id = parent_id,
                AstNode::UsingDeclaration(n) => n.id = parent_id,
                AstNode::OperationDeclaration(n) => n.id = parent_id,
                AstNode::OperationSignatureDeclaration(n) => n.id = parent_id,
                AstNode::OperationSignatureReference(n) => n.id = parent_id,
                AstNode::AliasStatement(n) => n.id = parent_id,
                AstNode::ConstStatement(n) => n.id = parent_id,
                AstNode::DecoratorDeclaration(n) => n.id = parent_id,
                AstNode::DecoratorExpression(n) => n.id = parent_id,
                AstNode::AugmentDecoratorStatement(n) => n.id = parent_id,
                AstNode::FunctionDeclaration(n) => n.id = parent_id,
                AstNode::FunctionParameter(n) => n.id = parent_id,
                AstNode::FunctionTypeExpression(n) => n.id = parent_id,
                AstNode::TemplateParameterDeclaration(n) => n.id = parent_id,
                AstNode::TemplateArgument(n) => n.id = parent_id,
                AstNode::ImportStatement(n) => n.id = parent_id,
                AstNode::DirectiveExpression(n) => n.id = parent_id,
                AstNode::Doc(n) => n.id = parent_id,
                AstNode::DocText(n) => n.id = parent_id,
                AstNode::DocParamTag(n) => n.id = parent_id,
                AstNode::DocPropTag(n) => n.id = parent_id,
                AstNode::DocReturnsTag(n) => n.id = parent_id,
                AstNode::DocErrorsTag(n) => n.id = parent_id,
                AstNode::DocTemplateTag(n) => n.id = parent_id,
                AstNode::DocUnknownTag(n) => n.id = parent_id,
                AstNode::LineComment(n) => n.id = parent_id,
                AstNode::BlockComment(n) => n.id = parent_id,
                AstNode::EmptyStatement(n) => n.id = parent_id,
                AstNode::InvalidStatement(n) => n.id = parent_id,
                AstNode::JsNamespaceDeclaration(n) => n.id = parent_id,
                AstNode::Modifier(n) => n.id = parent_id,
                AstNode::TypeSpecScript(n) => n.id = parent_id,
            }
        }
    }

    // Helper to create a span from positions
    pub fn make_span(&self, _start_pos: usize, _end_pos: usize) -> Span {
        // Simple span using byte offsets - convert to line/column if needed
        Span {
            start: Position { line: 1, column: 0 },
            end: Position { line: 1, column: 0 },
        }
    }

    // ==================== Node Creation Methods ====================

    pub fn create_identifier(&mut self, value: String, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::Identifier(Identifier {
            id,
            span,
            value,
        });
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
        let node = AstNode::StringLiteral(StringLiteral {
            id,
            span,
            value,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_numeric_literal(&mut self, value: f64, value_as_string: String, span: Span) -> u32 {
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
        let node = AstNode::BooleanLiteral(BooleanLiteral {
            id,
            span,
            value,
        });
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

    pub fn create_model_expression(&mut self, properties: Vec<u32>, body_range: Span, span: Span) -> u32 {
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
        let node = AstNode::ModelSpreadProperty(ModelSpreadProperty {
            id,
            span,
            target,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_model_declaration(
        &mut self,
        name: u32,
        properties: Vec<u32>,
        extends: Option<u32>,
        is: Option<u32>,
        decorators: Vec<u32>,
        body_range: Span,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::ModelDeclaration(ModelDeclaration {
            id,
            span,
            name,
            properties,
            extends,
            is,
            decorators,
            body_range,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_union_expression(&mut self, options: Vec<u32>, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::UnionExpression(UnionExpression {
            id,
            span,
            options,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_intersection_expression(&mut self, options: Vec<u32>, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::IntersectionExpression(IntersectionExpression {
            id,
            span,
            options,
        });
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
        let node = AstNode::TupleExpression(TupleExpression {
            id,
            span,
            values,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_value_of_expression(&mut self, target: u32, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::ValueOfExpression(ValueOfExpression {
            id,
            span,
            target,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_type_of_expression(&mut self, target: u32, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::TypeOfExpression(TypeOfExpression {
            id,
            span,
            target,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_import_statement(&mut self, path: u32, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::ImportStatement(ImportStatement {
            id,
            span,
            path,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_namespace_declaration(
        &mut self,
        name: u32,
        statements: Vec<u32>,
        decorators: Vec<u32>,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::NamespaceDeclaration(NamespaceDeclaration {
            id,
            span,
            name,
            statements,
            decorators,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_using_declaration(&mut self, name: u32, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::UsingDeclaration(UsingDeclaration {
            id,
            span,
            name,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_interface_declaration(
        &mut self,
        name: u32,
        operations: Vec<u32>,
        extends: Vec<u32>,
        decorators: Vec<u32>,
        template_parameters: Vec<u32>,
        body_range: Span,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::InterfaceDeclaration(InterfaceDeclaration {
            id,
            span,
            name,
            operations,
            extends,
            decorators,
            template_parameters,
            body_range,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_union_declaration(
        &mut self,
        name: u32,
        variants: Vec<u32>,
        decorators: Vec<u32>,
        template_parameters: Vec<u32>,
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
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::EnumDeclaration(EnumDeclaration {
            id,
            span,
            name,
            members,
            decorators,
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
        let node = AstNode::EnumSpreadMember(EnumSpreadMember {
            id,
            span,
            target,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_alias_statement(
        &mut self,
        name: u32,
        value: u32,
        template_parameters: Vec<u32>,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::AliasStatement(AliasStatement {
            id,
            span,
            name,
            value,
            template_parameters,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_const_statement(
        &mut self,
        name: u32,
        value: u32,
        type_annotation: Option<u32>,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::ConstStatement(ConstStatement {
            id,
            span,
            name,
            value,
            type_annotation,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_scalar_declaration(
        &mut self,
        name: u32,
        extends: Option<u32>,
        decorators: Vec<u32>,
        constructors: Vec<u32>,
        body_range: Span,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::ScalarDeclaration(ScalarDeclaration {
            id,
            span,
            name,
            extends,
            decorators,
            constructors,
            body_range,
        });
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

    pub fn create_operation_signature_reference(
        &mut self,
        base_operation: u32,
        span: Span,
    ) -> u32 {
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
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::DecoratorDeclaration(DecoratorDeclaration {
            id,
            span,
            name,
            target,
            parameters,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_function_declaration(
        &mut self,
        name: u32,
        parameters: Vec<u32>,
        return_type: Option<u32>,
        span: Span,
    ) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::FunctionDeclaration(FunctionDeclaration {
            id,
            span,
            name,
            parameters,
            return_type,
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
        let node = AstNode::ObjectLiteralSpreadProperty(ObjectLiteralSpreadProperty {
            id,
            span,
            target,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_array_literal(&mut self, values: Vec<u32>, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::ArrayLiteral(ArrayLiteral {
            id,
            span,
            values,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_string_template_expression(&mut self, head: u32, spans: Vec<u32>, span: Span) -> u32 {
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

    pub fn create_string_template_head(&mut self, value: String, _flags: TokenFlags, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::StringTemplateHead(StringTemplateHead {
            id,
            span,
            value,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_string_template_middle(&mut self, value: String, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::StringTemplateMiddle(StringTemplateMiddle {
            id,
            span,
            value,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_string_template_tail(&mut self, value: String, span: Span) -> u32 {
        let id = self.node_id_gen.next();
        let node = AstNode::StringTemplateTail(StringTemplateTail {
            id,
            span,
            value,
        });
        self.nodes.insert(id, node);
        id
    }

    pub fn create_string_template_span(&mut self, expression: u32, literal: u32, span: Span) -> u32 {
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
