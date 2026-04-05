//! Visitor trait for AST nodes
//! Provides a way to traverse and visit all AST node types

use super::types::*;

/// Visitor trait for traversing AST nodes
/// Implement this trait to perform operations on AST nodes
pub trait Visitor {
    // Default implementations that do nothing

    // Root node
    fn visit_script(&mut self, _node: &TypeSpecScript) {}

    // Identifiers and literals
    fn visit_identifier(&mut self, _node: &Identifier) {}
    fn visit_string_literal(&mut self, _node: &StringLiteral) {}
    fn visit_numeric_literal(&mut self, _node: &NumericLiteral) {}
    fn visit_boolean_literal(&mut self, _node: &BooleanLiteral) {}

    // Member expression
    fn visit_member_expression(&mut self, _node: &MemberExpression) {}

    // Expressions
    fn visit_array_expression(&mut self, _node: &ArrayExpression) {}
    fn visit_tuple_expression(&mut self, _node: &TupleExpression) {}
    fn visit_union_expression(&mut self, _node: &UnionExpression) {}
    fn visit_intersection_expression(&mut self, _node: &IntersectionExpression) {}
    fn visit_type_reference(&mut self, _node: &TypeReference) {}
    fn visit_call_expression(&mut self, _node: &CallExpression) {}
    fn visit_value_of_expression(&mut self, _node: &ValueOfExpression) {}
    fn visit_type_of_expression(&mut self, _node: &TypeOfExpression) {}

    // String template
    fn visit_string_template_expression(&mut self, _node: &StringTemplateExpression) {}
    fn visit_string_template_head(&mut self, _node: &StringTemplateHead) {}
    fn visit_string_template_middle(&mut self, _node: &StringTemplateMiddle) {}
    fn visit_string_template_tail(&mut self, _node: &StringTemplateTail) {}
    fn visit_string_template_span(&mut self, _node: &StringTemplateSpan) {}

    // Object literal
    fn visit_object_literal(&mut self, _node: &ObjectLiteral) {}
    fn visit_object_literal_property(&mut self, _node: &ObjectLiteralProperty) {}
    fn visit_object_literal_spread_property(&mut self, _node: &ObjectLiteralSpreadProperty) {}
    fn visit_array_literal(&mut self, _node: &ArrayLiteral) {}

    // Keywords
    fn visit_extern_keyword(&mut self, _node: &ExternKeyword) {}
    fn visit_internal_keyword(&mut self, _node: &InternalKeyword) {}
    fn visit_void_keyword(&mut self, _node: &VoidKeyword) {}
    fn visit_never_keyword(&mut self, _node: &NeverKeyword) {}
    fn visit_unknown_keyword(&mut self, _node: &UnknownKeyword) {}

    // Model
    fn visit_model_declaration(&mut self, _node: &ModelDeclaration) {}
    fn visit_model_expression(&mut self, _node: &ModelExpression) {}
    fn visit_model_property(&mut self, _node: &ModelProperty) {}
    fn visit_model_spread_property(&mut self, _node: &ModelSpreadProperty) {}

    // Scalar
    fn visit_scalar_declaration(&mut self, _node: &ScalarDeclaration) {}
    fn visit_scalar_constructor(&mut self, _node: &ScalarConstructor) {}

    // Interface
    fn visit_interface_declaration(&mut self, _node: &InterfaceDeclaration) {}

    // Union
    fn visit_union_declaration(&mut self, _node: &UnionDeclaration) {}
    fn visit_union_variant(&mut self, _node: &UnionVariant) {}

    // Enum
    fn visit_enum_declaration(&mut self, _node: &EnumDeclaration) {}
    fn visit_enum_member(&mut self, _node: &EnumMember) {}
    fn visit_enum_spread_member(&mut self, _node: &EnumSpreadMember) {}

    // Namespace
    fn visit_namespace_declaration(&mut self, _node: &NamespaceDeclaration) {}
    fn visit_using_declaration(&mut self, _node: &UsingDeclaration) {}

    // Operation
    fn visit_operation_declaration(&mut self, _node: &OperationDeclaration) {}
    fn visit_operation_signature_declaration(&mut self, _node: &OperationSignatureDeclaration) {}
    fn visit_operation_signature_reference(&mut self, _node: &OperationSignatureReference) {}

    // Alias and Const
    fn visit_alias_statement(&mut self, _node: &AliasStatement) {}
    fn visit_const_statement(&mut self, _node: &ConstStatement) {}

    // Decorator
    fn visit_decorator_declaration(&mut self, _node: &DecoratorDeclaration) {}
    fn visit_decorator_expression(&mut self, _node: &DecoratorExpression) {}
    fn visit_augment_decorator_statement(&mut self, _node: &AugmentDecoratorStatement) {}

    // Function
    fn visit_function_declaration(&mut self, _node: &FunctionDeclaration) {}
    fn visit_function_parameter(&mut self, _node: &FunctionParameter) {}
    fn visit_function_type_expression(&mut self, _node: &FunctionTypeExpression) {}

    // Template
    fn visit_template_parameter_declaration(&mut self, _node: &TemplateParameterDeclaration) {}
    fn visit_template_argument(&mut self, _node: &TemplateArgument) {}

    // Import
    fn visit_import_statement(&mut self, _node: &ImportStatement) {}

    // Directive
    fn visit_directive_expression(&mut self, _node: &DirectiveExpression) {}

    // Doc
    fn visit_doc(&mut self, _node: &Doc) {}
    fn visit_doc_text(&mut self, _node: &DocText) {}
    fn visit_doc_param_tag(&mut self, _node: &DocParamTag) {}
    fn visit_doc_prop_tag(&mut self, _node: &DocPropTag) {}
    fn visit_doc_returns_tag(&mut self, _node: &DocReturnsTag) {}
    fn visit_doc_errors_tag(&mut self, _node: &DocErrorsTag) {}
    fn visit_doc_template_tag(&mut self, _node: &DocTemplateTag) {}
    fn visit_doc_unknown_tag(&mut self, _node: &DocUnknownTag) {}

    // Comments
    fn visit_line_comment(&mut self, _node: &LineComment) {}
    fn visit_block_comment(&mut self, _node: &BlockComment) {}

    // Miscellaneous
    fn visit_empty_statement(&mut self, _node: &EmptyStatement) {}
    fn visit_invalid_statement(&mut self, _node: &InvalidStatement) {}
    fn visit_js_namespace_declaration(&mut self, _node: &JsNamespaceDeclaration) {}

    // Modifier
    fn visit_modifier(&mut self, _node: &Modifier) {}
}

/// Visitor with context for traversal
pub trait VisitorWithContext<C> {
    fn visit_script(&mut self, _ctx: &mut C, _node: &TypeSpecScript) {}
    fn visit_identifier(&mut self, _ctx: &mut C, _node: &Identifier) {}
    fn visit_string_literal(&mut self, _ctx: &mut C, _node: &StringLiteral) {}
    fn visit_numeric_literal(&mut self, _ctx: &mut C, _node: &NumericLiteral) {}
    fn visit_boolean_literal(&mut self, _ctx: &mut C, _node: &BooleanLiteral) {}
    fn visit_member_expression(&mut self, _ctx: &mut C, _node: &MemberExpression) {}
    fn visit_array_expression(&mut self, _ctx: &mut C, _node: &ArrayExpression) {}
    fn visit_tuple_expression(&mut self, _ctx: &mut C, _node: &TupleExpression) {}
    fn visit_union_expression(&mut self, _ctx: &mut C, _node: &UnionExpression) {}
    fn visit_intersection_expression(&mut self, _ctx: &mut C, _node: &IntersectionExpression) {}
    fn visit_type_reference(&mut self, _ctx: &mut C, _node: &TypeReference) {}
    fn visit_call_expression(&mut self, _ctx: &mut C, _node: &CallExpression) {}
    fn visit_value_of_expression(&mut self, _ctx: &mut C, _node: &ValueOfExpression) {}
    fn visit_type_of_expression(&mut self, _ctx: &mut C, _node: &TypeOfExpression) {}
    fn visit_string_template_expression(&mut self, _ctx: &mut C, _node: &StringTemplateExpression) {}
    fn visit_string_template_head(&mut self, _ctx: &mut C, _node: &StringTemplateHead) {}
    fn visit_string_template_middle(&mut self, _ctx: &mut C, _node: &StringTemplateMiddle) {}
    fn visit_string_template_tail(&mut self, _ctx: &mut C, _node: &StringTemplateTail) {}
    fn visit_string_template_span(&mut self, _ctx: &mut C, _node: &StringTemplateSpan) {}
    fn visit_object_literal(&mut self, _ctx: &mut C, _node: &ObjectLiteral) {}
    fn visit_object_literal_property(&mut self, _ctx: &mut C, _node: &ObjectLiteralProperty) {}
    fn visit_object_literal_spread_property(&mut self, _ctx: &mut C, _node: &ObjectLiteralSpreadProperty) {}
    fn visit_array_literal(&mut self, _ctx: &mut C, _node: &ArrayLiteral) {}
    fn visit_extern_keyword(&mut self, _ctx: &mut C, _node: &ExternKeyword) {}
    fn visit_internal_keyword(&mut self, _ctx: &mut C, _node: &InternalKeyword) {}
    fn visit_void_keyword(&mut self, _ctx: &mut C, _node: &VoidKeyword) {}
    fn visit_never_keyword(&mut self, _ctx: &mut C, _node: &NeverKeyword) {}
    fn visit_unknown_keyword(&mut self, _ctx: &mut C, _node: &UnknownKeyword) {}
    fn visit_model_declaration(&mut self, _ctx: &mut C, _node: &ModelDeclaration) {}
    fn visit_model_expression(&mut self, _ctx: &mut C, _node: &ModelExpression) {}
    fn visit_model_property(&mut self, _ctx: &mut C, _node: &ModelProperty) {}
    fn visit_model_spread_property(&mut self, _ctx: &mut C, _node: &ModelSpreadProperty) {}
    fn visit_scalar_declaration(&mut self, _ctx: &mut C, _node: &ScalarDeclaration) {}
    fn visit_scalar_constructor(&mut self, _ctx: &mut C, _node: &ScalarConstructor) {}
    fn visit_interface_declaration(&mut self, _ctx: &mut C, _node: &InterfaceDeclaration) {}
    fn visit_union_declaration(&mut self, _ctx: &mut C, _node: &UnionDeclaration) {}
    fn visit_union_variant(&mut self, _ctx: &mut C, _node: &UnionVariant) {}
    fn visit_enum_declaration(&mut self, _ctx: &mut C, _node: &EnumDeclaration) {}
    fn visit_enum_member(&mut self, _ctx: &mut C, _node: &EnumMember) {}
    fn visit_enum_spread_member(&mut self, _ctx: &mut C, _node: &EnumSpreadMember) {}
    fn visit_namespace_declaration(&mut self, _ctx: &mut C, _node: &NamespaceDeclaration) {}
    fn visit_using_declaration(&mut self, _ctx: &mut C, _node: &UsingDeclaration) {}
    fn visit_operation_declaration(&mut self, _ctx: &mut C, _node: &OperationDeclaration) {}
    fn visit_operation_signature_declaration(&mut self, _ctx: &mut C, _node: &OperationSignatureDeclaration) {}
    fn visit_operation_signature_reference(&mut self, _ctx: &mut C, _node: &OperationSignatureReference) {}
    fn visit_alias_statement(&mut self, _ctx: &mut C, _node: &AliasStatement) {}
    fn visit_const_statement(&mut self, _ctx: &mut C, _node: &ConstStatement) {}
    fn visit_decorator_declaration(&mut self, _ctx: &mut C, _node: &DecoratorDeclaration) {}
    fn visit_decorator_expression(&mut self, _ctx: &mut C, _node: &DecoratorExpression) {}
    fn visit_augment_decorator_statement(&mut self, _ctx: &mut C, _node: &AugmentDecoratorStatement) {}
    fn visit_function_declaration(&mut self, _ctx: &mut C, _node: &FunctionDeclaration) {}
    fn visit_function_parameter(&mut self, _ctx: &mut C, _node: &FunctionParameter) {}
    fn visit_function_type_expression(&mut self, _ctx: &mut C, _node: &FunctionTypeExpression) {}
    fn visit_template_parameter_declaration(&mut self, _ctx: &mut C, _node: &TemplateParameterDeclaration) {}
    fn visit_template_argument(&mut self, _ctx: &mut C, _node: &TemplateArgument) {}
    fn visit_import_statement(&mut self, _ctx: &mut C, _node: &ImportStatement) {}
    fn visit_directive_expression(&mut self, _ctx: &mut C, _node: &DirectiveExpression) {}
    fn visit_doc(&mut self, _ctx: &mut C, _node: &Doc) {}
    fn visit_doc_text(&mut self, _ctx: &mut C, _node: &DocText) {}
    fn visit_doc_param_tag(&mut self, _ctx: &mut C, _node: &DocParamTag) {}
    fn visit_doc_prop_tag(&mut self, _ctx: &mut C, _node: &DocPropTag) {}
    fn visit_doc_returns_tag(&mut self, _ctx: &mut C, _node: &DocReturnsTag) {}
    fn visit_doc_errors_tag(&mut self, _ctx: &mut C, _node: &DocErrorsTag) {}
    fn visit_doc_template_tag(&mut self, _ctx: &mut C, _node: &DocTemplateTag) {}
    fn visit_doc_unknown_tag(&mut self, _ctx: &mut C, _node: &DocUnknownTag) {}
    fn visit_line_comment(&mut self, _ctx: &mut C, _node: &LineComment) {}
    fn visit_block_comment(&mut self, _ctx: &mut C, _node: &BlockComment) {}
    fn visit_empty_statement(&mut self, _ctx: &mut C, _node: &EmptyStatement) {}
    fn visit_invalid_statement(&mut self, _ctx: &mut C, _node: &InvalidStatement) {}
    fn visit_js_namespace_declaration(&mut self, _ctx: &mut C, _node: &JsNamespaceDeclaration) {}
    fn visit_modifier(&mut self, _ctx: &mut C, _node: &Modifier) {}
}

/// Default visitor that traverses all child nodes
pub struct DefaultVisitor;

impl Visitor for DefaultVisitor {}

impl DefaultVisitor {
    pub fn new() -> Self {
        DefaultVisitor
    }
}

impl Default for DefaultVisitor {
    fn default() -> Self {
        Self::new()
    }
}
