//! Parser Tests
//!
//! Comprehensive tests for the TypeSpec parser implementation.

use crate::ast::types::*;
use crate::parser::ast_builder::AstNode;
use crate::parser::{ParseResult, parse};

/// Helper function to check if a node is an identifier and get its value
fn get_identifier_value(
    builder: &crate::parser::ast_builder::AstBuilder,
    node_id: u32,
) -> Option<String> {
    match builder.id_to_node(node_id) {
        Some(AstNode::Identifier(id)) => Some(id.value.clone()),
        _ => None,
    }
}

/// Helper function to get node kind
fn get_node_kind(
    builder: &crate::parser::ast_builder::AstBuilder,
    node_id: u32,
) -> Option<SyntaxKind> {
    match builder.id_to_node(node_id) {
        Some(AstNode::Identifier(_)) => Some(SyntaxKind::Identifier),
        Some(AstNode::ModelDeclaration(_)) => Some(SyntaxKind::ModelStatement),
        Some(AstNode::ModelProperty(_)) => Some(SyntaxKind::ModelProperty),
        Some(AstNode::NamespaceDeclaration(_)) => Some(SyntaxKind::NamespaceStatement),
        Some(AstNode::InterfaceDeclaration(_)) => Some(SyntaxKind::InterfaceStatement),
        Some(AstNode::UnionDeclaration(_)) => Some(SyntaxKind::UnionStatement),
        Some(AstNode::EnumDeclaration(_)) => Some(SyntaxKind::EnumStatement),
        Some(AstNode::ScalarDeclaration(_)) => Some(SyntaxKind::ScalarStatement),
        Some(AstNode::AliasStatement(_)) => Some(SyntaxKind::AliasStatement),
        Some(AstNode::ConstStatement(_)) => Some(SyntaxKind::ConstStatement),
        Some(AstNode::ImportStatement(_)) => Some(SyntaxKind::ImportStatement),
        Some(AstNode::UsingDeclaration(_)) => Some(SyntaxKind::UsingStatement),
        Some(AstNode::OperationDeclaration(_)) => Some(SyntaxKind::OperationStatement),
        Some(AstNode::DecoratorExpression(_)) => Some(SyntaxKind::DecoratorExpression),
        Some(AstNode::TypeReference(_)) => Some(SyntaxKind::TypeReference),
        Some(AstNode::CallExpression(_)) => Some(SyntaxKind::CallExpression),
        Some(AstNode::StringLiteral(_)) => Some(SyntaxKind::StringLiteral),
        Some(AstNode::NumericLiteral(_)) => Some(SyntaxKind::NumericLiteral),
        Some(AstNode::BooleanLiteral(_)) => Some(SyntaxKind::BooleanLiteral),
        Some(AstNode::UnionExpression(_)) => Some(SyntaxKind::UnionExpression),
        Some(AstNode::IntersectionExpression(_)) => Some(SyntaxKind::IntersectionExpression),
        Some(AstNode::ArrayExpression(_)) => Some(SyntaxKind::ArrayExpression),
        Some(AstNode::TupleExpression(_)) => Some(SyntaxKind::TupleExpression),
        Some(AstNode::ObjectLiteral(_)) => Some(SyntaxKind::ObjectLiteral),
        Some(AstNode::ArrayLiteral(_)) => Some(SyntaxKind::ArrayLiteral),
        Some(AstNode::ValueOfExpression(_)) => Some(SyntaxKind::ValueOfExpression),
        Some(AstNode::TypeOfExpression(_)) => Some(SyntaxKind::TypeOfExpression),
        Some(AstNode::FunctionTypeExpression(_)) => Some(SyntaxKind::FunctionTypeExpression),
        Some(AstNode::FunctionDeclaration(_)) => Some(SyntaxKind::FunctionDeclarationStatement),
        Some(AstNode::FunctionParameter(_)) => Some(SyntaxKind::FunctionParameter),
        Some(AstNode::TemplateParameterDeclaration(_)) => {
            Some(SyntaxKind::TemplateParameterDeclaration)
        }
        Some(AstNode::DecoratorDeclaration(_)) => Some(SyntaxKind::DecoratorDeclarationStatement),
        Some(AstNode::AugmentDecoratorStatement(_)) => Some(SyntaxKind::AugmentDecoratorStatement),
        Some(AstNode::ModelSpreadProperty(_)) => Some(SyntaxKind::ModelSpreadProperty),
        Some(AstNode::ModelExpression(_)) => Some(SyntaxKind::ModelExpression),
        Some(AstNode::EnumMember(_)) => Some(SyntaxKind::EnumMember),
        Some(AstNode::EnumSpreadMember(_)) => Some(SyntaxKind::EnumSpreadMember),
        Some(AstNode::UnionVariant(_)) => Some(SyntaxKind::UnionVariant),
        Some(AstNode::OperationSignatureDeclaration(_)) => {
            Some(SyntaxKind::OperationSignatureDeclaration)
        }
        Some(AstNode::OperationSignatureReference(_)) => {
            Some(SyntaxKind::OperationSignatureReference)
        }
        Some(AstNode::ScalarConstructor(_)) => Some(SyntaxKind::ScalarConstructor),
        Some(AstNode::MemberExpression(_)) => Some(SyntaxKind::MemberExpression),
        Some(AstNode::VoidKeyword(_)) => Some(SyntaxKind::VoidKeyword),
        Some(AstNode::NeverKeyword(_)) => Some(SyntaxKind::NeverKeyword),
        Some(AstNode::UnknownKeyword(_)) => Some(SyntaxKind::UnknownKeyword),
        Some(AstNode::EmptyStatement(_)) => Some(SyntaxKind::EmptyStatement),
        Some(AstNode::InvalidStatement(_)) => Some(SyntaxKind::InvalidStatement),
        Some(AstNode::TypeSpecScript(_)) => Some(SyntaxKind::TypeSpecScript),
        _ => None,
    }
}

/// Helper to extract root script node
fn get_root_script(result: &ParseResult) -> Option<&TypeSpecScript> {
    match result.builder.id_to_node(result.root_id) {
        Some(AstNode::TypeSpecScript(script)) => Some(script),
        _ => None,
    }
}

/// Helper to check model declaration properties
fn get_model_properties(result: &ParseResult, model_index: usize) -> Option<Vec<String>> {
    let script = get_root_script(result)?;
    let model_id = script.statements.get(model_index)?;
    match result.builder.id_to_node(*model_id) {
        Some(AstNode::ModelDeclaration(m)) => {
            let mut props = Vec::new();
            for prop_id in &m.properties {
                if let Some(AstNode::ModelProperty(prop)) = result.builder.id_to_node(*prop_id)
                    && let Some(name) = get_identifier_value(&result.builder, prop.name)
                {
                    props.push(name);
                }
            }
            Some(props)
        }
        _ => None,
    }
}

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use super::*;

    // ==================== Empty & Basic Tests ====================

    #[test]
    fn test_parse_empty_script() {
        let result = parse("");
        assert!(
            result.diagnostics.is_empty(),
            "Expected no diagnostics for empty script"
        );
        let script = get_root_script(&result);
        assert!(script.is_some(), "Should have a root script node");
        assert_eq!(
            script.unwrap().statements.len(),
            0,
            "Empty script should have no statements"
        );
    }

    #[test]
    fn test_parse_whitespace_only() {
        let result = parse("   \n\t  \n   ");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_empty_statements() {
        let result = parse(";;;");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Model Statement Tests ====================

    #[test]
    fn test_parse_model_empty() {
        let result = parse("model Foo {}");
        assert!(
            result.diagnostics.is_empty(),
            "Unexpected diagnostics: {:?}",
            result.diagnostics
        );

        let script = get_root_script(&result).unwrap();
        assert_eq!(script.statements.len(), 1);
        assert_eq!(
            get_node_kind(&result.builder, script.statements[0]),
            Some(SyntaxKind::ModelStatement)
        );
    }

    #[test]
    fn test_parse_model_with_properties() {
        let result = parse("model Car { make: string; model: string; year: int32; }");
        assert!(result.diagnostics.is_empty());

        let props = get_model_properties(&result, 0);
        assert_eq!(
            props,
            Some(vec![
                "make".to_string(),
                "model".to_string(),
                "year".to_string()
            ])
        );
    }

    #[test]
    fn test_parse_model_with_comma_separated_properties() {
        let result = parse("model Car { make: string, model: string, year: int32 }");
        assert!(result.diagnostics.is_empty());

        let props = get_model_properties(&result, 0);
        assert_eq!(
            props,
            Some(vec![
                "make".to_string(),
                "model".to_string(),
                "year".to_string()
            ])
        );
    }

    #[test]
    fn test_parse_model_extends() {
        let result = parse("model Dog extends Animal { breed: string; }");
        assert!(result.diagnostics.is_empty());

        let script = get_root_script(&result).unwrap();
        match result.builder.id_to_node(script.statements[0]) {
            Some(AstNode::ModelDeclaration(m)) => {
                assert!(m.extends.is_some(), "Should have extends clause");
            }
            _ => panic!("Expected model declaration"),
        }
    }

    #[test]
    fn test_parse_model_is() {
        let result = parse("model Car is Vehicle {}");
        assert!(result.diagnostics.is_empty());

        let script = get_root_script(&result).unwrap();
        match result.builder.id_to_node(script.statements[0]) {
            Some(AstNode::ModelDeclaration(m)) => {
                assert!(m.is.is_some(), "Should have is clause");
            }
            _ => panic!("Expected model declaration"),
        }
    }

    #[test]
    fn test_parse_model_template_parameters() {
        let result = parse("model Pair<T, V> { key: T; value: V; }");
        assert!(result.diagnostics.is_empty());

        let script = get_root_script(&result).unwrap();
        match result.builder.id_to_node(script.statements[0]) {
            Some(AstNode::ModelDeclaration(_)) => {
                // Template parameters are parsed
            }
            _ => panic!("Expected model declaration"),
        }
    }

    #[test]
    fn test_parse_model_optional_property() {
        let result = parse("model Person { name?: string; age: int32; }");
        assert!(result.diagnostics.is_empty());

        let script = get_root_script(&result).unwrap();
        match result.builder.id_to_node(script.statements[0]) {
            Some(AstNode::ModelDeclaration(m)) => {
                assert_eq!(m.properties.len(), 2);
            }
            _ => panic!("Expected model declaration"),
        }
    }

    #[test]
    fn test_parse_model_default_property() {
        let result = parse(r#"model Config { name: string = "default"; }"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_model_spread_property() {
        let result = parse("model Car { ...BaseCar; make: string; }");
        assert!(result.diagnostics.is_empty());

        let script = get_root_script(&result).unwrap();
        match result.builder.id_to_node(script.statements[0]) {
            Some(AstNode::ModelDeclaration(m)) => {
                assert_eq!(m.properties.len(), 2);
            }
            _ => panic!("Expected model declaration"),
        }
    }

    #[test]
    fn test_parse_model_with_decorators() {
        let result = parse(
            r#"
@format("json")
model Car {
    @minLength(1)
    make: string;
}
"#,
        );
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_model_multiple_decorators_on_property() {
        let result = parse(r#"model Car { @foo @bar name: string; }"#);
        assert!(
            result.diagnostics.is_empty(),
            "Expected no diagnostics but got: {:?}",
            result.diagnostics
        );
    }

    // ==================== Namespace Tests ====================

    #[test]
    fn test_parse_namespace_empty() {
        let result = parse("namespace Test {}");
        assert!(result.diagnostics.is_empty());

        let script = get_root_script(&result).unwrap();
        assert_eq!(
            get_node_kind(&result.builder, script.statements[0]),
            Some(SyntaxKind::NamespaceStatement)
        );
    }

    #[test]
    fn test_parse_namespace_blockless() {
        let result = parse("namespace Test;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_namespace_nested() {
        let result = parse("namespace Store.Read {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_namespace_with_statements() {
        let result = parse("namespace Store { model Car {}; op read(): int32; }");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Interface Tests ====================

    #[test]
    fn test_parse_interface_empty() {
        let result = parse("interface Foo {}");
        assert!(result.diagnostics.is_empty());

        let script = get_root_script(&result).unwrap();
        assert_eq!(
            get_node_kind(&result.builder, script.statements[0]),
            Some(SyntaxKind::InterfaceStatement)
        );
    }

    #[test]
    fn test_parse_interface_with_operations() {
        let result = parse("interface Foo { bar(): int32; baz(): string; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_interface_with_template_parameters() {
        let result = parse("interface Foo<T> {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_interface_extends() {
        let result = parse("interface Foo extends Bar {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_interface_multiple_extends() {
        let result = parse("interface Foo extends Bar, Baz {}");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Union Tests ====================

    #[test]
    fn test_parse_union_with_variants() {
        let result = parse("union Color { red: string; green: string; blue: string; }");
        assert!(result.diagnostics.is_empty());

        let script = get_root_script(&result).unwrap();
        assert_eq!(
            get_node_kind(&result.builder, script.statements[0]),
            Some(SyntaxKind::UnionStatement)
        );
    }

    #[test]
    fn test_parse_union_with_string_variants() {
        let result = parse(r#"union Status { "active": string; "inactive": string; }"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_union_anonymous_variants() {
        let result = parse("union StringOrNull { string; null; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_union_template_parameters() {
        let result = parse("union Result<T, E> { value: T; error: E; }");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Enum Tests ====================

    #[test]
    fn test_parse_enum_simple() {
        let result = parse("enum Color { red; green; blue; }");
        assert!(result.diagnostics.is_empty());

        let script = get_root_script(&result).unwrap();
        assert_eq!(
            get_node_kind(&result.builder, script.statements[0]),
            Some(SyntaxKind::EnumStatement)
        );
    }

    #[test]
    fn test_parse_enum_with_values() {
        let result = parse(r#"enum Status { active: 1; inactive: 2; }"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_enum_with_string_values() {
        let result = parse(r#"enum Direction { north: "n"; south: "s"; }"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_enum_with_decorators() {
        let result = parse(
            r#"
enum Color {
    @format("json")
    red;
    green;
}
"#,
        );
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Scalar Tests ====================

    #[test]
    fn test_parse_scalar_extends() {
        let result = parse("scalar uuid extends string;");
        assert!(result.diagnostics.is_empty());

        let script = get_root_script(&result).unwrap();
        assert_eq!(
            get_node_kind(&result.builder, script.statements[0]),
            Some(SyntaxKind::ScalarStatement)
        );
    }

    #[test]
    fn test_parse_scalar_with_constructors() {
        let result = parse(
            r#"
scalar date {
    init fromString(s: string)
    init fromTimestamp(ts: int64)
}
"#,
        );
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_scalar_with_trailing_comma() {
        let result = parse("scalar date { init fromString(s: string,); }");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Alias Tests ====================

    #[test]
    fn test_parse_alias_simple() {
        let result = parse("alias MyAlias = SomeType;");
        assert!(result.diagnostics.is_empty());

        let script = get_root_script(&result).unwrap();
        assert_eq!(
            get_node_kind(&result.builder, script.statements[0]),
            Some(SyntaxKind::AliasStatement)
        );
    }

    #[test]
    fn test_parse_alias_with_template_parameters() {
        let result = parse("alias Optional<T> = T | null;");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Const Tests ====================

    #[test]
    fn test_parse_const_simple() {
        let result = parse("const pi = 3.14159;");
        assert!(result.diagnostics.is_empty());

        let script = get_root_script(&result).unwrap();
        assert_eq!(
            get_node_kind(&result.builder, script.statements[0]),
            Some(SyntaxKind::ConstStatement)
        );
    }

    #[test]
    fn test_parse_const_with_type_annotation() {
        let result = parse("const pi: float = 3.14159;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_const_boolean() {
        let result = parse("const enabled = true;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_const_string() {
        let result = parse(r#"const name = "TypeSpec";"#);
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Import Tests ====================

    #[test]
    fn test_parse_import_simple() {
        let result = parse(r#"import "./test";"#);
        assert!(result.diagnostics.is_empty());

        let script = get_root_script(&result).unwrap();
        assert_eq!(
            get_node_kind(&result.builder, script.statements[0]),
            Some(SyntaxKind::ImportStatement)
        );
    }

    // ==================== Using Tests ====================

    #[test]
    fn test_parse_using_simple() {
        let result = parse("using Foo;");
        assert!(result.diagnostics.is_empty());

        let script = get_root_script(&result).unwrap();
        assert_eq!(
            get_node_kind(&result.builder, script.statements[0]),
            Some(SyntaxKind::UsingStatement)
        );
    }

    #[test]
    fn test_parse_using_member_expression() {
        let result = parse("using Foo.Bar.Baz;");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Operation Tests ====================

    #[test]
    fn test_parse_operation_simple() {
        let result = parse("op read(): int32;");
        assert!(result.diagnostics.is_empty());

        let script = get_root_script(&result).unwrap();
        assert_eq!(
            get_node_kind(&result.builder, script.statements[0]),
            Some(SyntaxKind::OperationStatement)
        );
    }

    #[test]
    fn test_parse_operation_with_parameters() {
        let result = parse("op write(data: string): {};");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_operation_with_template_parameters() {
        let result = parse("op convert<T, V>(value: T): V;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_operation_reference() {
        let result = parse("op myOp is BaseOp;");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Decorator Tests ====================

    #[test]
    fn test_parse_decorator_simple() {
        let result = parse(r#"@format("json") model Foo {}"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_decorator_with_args() {
        let result = parse(r#"@doc("description") model Foo {}"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_decorator_member_expression() {
        let result = parse(r#"@foo.bar model Foo {}"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_augment_decorator() {
        let result = parse("@@format(DateTime);");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Function Declaration Tests ====================

    #[test]
    fn test_parse_function_declaration() {
        let result = parse("extern fn print(message: string): void;");
        assert!(result.diagnostics.is_empty());

        let script = get_root_script(&result).unwrap();
        assert_eq!(
            get_node_kind(&result.builder, script.statements[0]),
            Some(SyntaxKind::FunctionDeclarationStatement)
        );
    }

    #[test]
    fn test_parse_decorator_declaration() {
        let result = parse("extern dec myDecorator(target: unknown);");
        assert!(result.diagnostics.is_empty());

        let script = get_root_script(&result).unwrap();
        assert_eq!(
            get_node_kind(&result.builder, script.statements[0]),
            Some(SyntaxKind::DecoratorDeclarationStatement)
        );
    }

    // ==================== Expression Tests ====================

    #[test]
    fn test_parse_type_reference() {
        let result = parse("model Foo { value: SomeType; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_type_reference_with_template_args() {
        let result = parse("model Foo { value: Array<string>; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_call_expression() {
        let result = parse("const x = int8(123);");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_member_expression() {
        let result = parse("const x = foo.bar;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_string_literal() {
        let result = parse(r#"const str = "hello";"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_numeric_literal() {
        let result = parse("const num = 42;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_boolean_literals() {
        let result = parse("const t = true; const f = false;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_array_type() {
        let result = parse("model Foo { arr: string[]; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_nested_array_type() {
        let result = parse("model Foo { arr: string[][]; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_union_type() {
        let result = parse("model Foo { val: string | null; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_intersection_type() {
        let result = parse("model Foo { val: Foo & Bar; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_tuple_expression() {
        let result = parse("const tuple = [1, 2, 3];");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_object_literal() {
        let result = parse(r#"const obj = #{name: "test", value: 42};"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_object_literal_with_spread() {
        let result = parse(r#"const obj = #{name: "test", ...other};"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_array_literal() {
        let result = parse(r#"const arr = #["a", "b", "c"];"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_value_of_expression() {
        let result = parse("model Foo<T extends valueof string> {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_type_of_expression() {
        let result = parse(r#"const x: typeof "123" = 123;"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_function_type() {
        // Test operation returning void
        let result = parse("op foo(): void;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_void_keyword() {
        // Test that void keyword can be parsed in various contexts
        let result = parse("op foo(): void;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_never_keyword() {
        let result = parse("op bar(): never;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_unknown_keyword() {
        let result = parse("model Foo { value: unknown; }");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Multi-statement Tests ====================

    #[test]
    fn test_parse_multiple_statements() {
        let result = parse(
            r#"
model A { };
model B { }
const x = 1;
namespace Foo {
    model C { }
}
"#,
        );
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Error Handling Tests ====================

    #[test]
    fn test_parse_error_invalid_statement() {
        let _result = parse("invalid_syntax_here");
        // Parser should produce diagnostics for invalid syntax
        // The exact behavior depends on error recovery implementation
    }

    #[test]
    fn test_parse_error_unterminated_string() {
        let result = parse(r#"const x = "unterminated"#);
        // Should have diagnostics about unterminated string
        assert!(!result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_error_missing_semicolon() {
        let _result = parse("model Foo { prop: string }");
        // Some parsers require semicolons, check behavior
    }

    // ==================== Template Parameter Tests ====================

    #[test]
    fn test_parse_template_parameter_with_constraint() {
        let result = parse("model Foo<T extends object> {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_parameter_with_default() {
        let result = parse("model Foo<T = string> {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_parameter_with_constraint_and_default() {
        let result = parse("model Foo<T extends object = {}> {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_multiple_template_parameters() {
        let result = parse("model Foo<A, B, C> {}");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Comments Tests ====================

    #[test]
    fn test_parse_with_single_line_comment() {
        let result = parse(
            r#"
// This is a comment
model Foo {}
"#,
        );
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_with_multi_line_comment() {
        let result = parse(
            r#"
/* This is a
   multi-line comment */
model Foo {}
"#,
        );
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Reserved Keywords Tests ====================

    #[test]
    fn test_parse_model_keyword_as_property_name() {
        let result = parse("model Foo { model: string; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_interface_keyword_as_property_name() {
        let result = parse("model Foo { interface: string; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_enum_keyword_as_property_name() {
        let result = parse("model Foo { enum: string; }");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Special Keywords as Identifiers Tests ====================

    #[test]
    fn test_parse_using_keywords_as_identifiers_in_member_position() {
        // These keywords should work when used as property/member names
        let result = parse(r#"const x = foo.model; const y = bar.enum;"#);
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_parse_model_with_quoted_property_name() {
        let result = parse(r#"model Foo { "prop-name": string; }"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_model_multiple_spread() {
        let result = parse("model Car { ...Base, ...Features, make: string; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_model_with_underscore_property_name() {
        let result = parse("model Foo { _private: string; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_model_with_dollar_property_name() {
        let result = parse("model Foo { $special: string; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_trailing_comma_in_model() {
        let result = parse("model Foo { a: string, b: int32, }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_operation_trailing_comma() {
        let result = parse("op foo(a: string, b: int32,): int32;");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Additional Edge Case Tests ====================

    #[test]
    fn test_parse_union_with_string_keys() {
        // Union with string keys for variants
        let result = parse(r#"union StringKeys { "hi there": string, "bye": int32 }"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_union_with_mixed_variants() {
        // Union with named and unnamed variants
        let result = parse(r#"union Mixed { string, int32, named: boolean }"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_const_with_object_type() {
        // Const with inline object type annotation
        let result = parse(r#"const a: {inline: string} = #{inline: "abc"};"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_object_literal_with_multiple_spreads() {
        // Object literal with multiple spreads
        let result = parse(r#"const A = #{a: "abc", ...B, c: "ghi"};"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_array_literal_with_mixed_types() {
        // Array literal with mixed types
        let result = parse(r#"const A = #["abc", 123, #{nested: true}];"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_valueof_with_complex_type() {
        // Valueof with complex type
        let result = parse("model Foo<T extends valueof {a: string, b: int32}> {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_valueof_with_array_type() {
        // Valueof with array type
        let result = parse("model Foo<T extends valueof int8[]> {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_parenthesized_union() {
        // Parenthesized expressions
        let result = parse("model A { x: ((B | C) & D)[]; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_const_with_template_reference() {
        // Const with type reference using template
        let result = parse("const a: string | int32 = int32;");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Namespace Tests ====================

    #[test]
    fn test_parse_namespace_nested_with_operations() {
        // Nested namespace with operations
        let result = parse("namespace Store { op read(): int32; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_namespace_with_nested_namespaces() {
        // Namespace with nested namespaces
        let result = parse("namespace Store { namespace Read { op read(): int32; } }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_namespace_dotted_name() {
        // Dotted namespace name
        let result = parse("namespace Store.Read;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_using_before_blockless_namespace() {
        // Using before blockless namespace
        let result = parse("using A.B; namespace Foo;");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Template Instantiation Tests ====================

    #[test]
    fn test_parse_template_instantiation_simple() {
        // Template instantiation
        let result = parse("model A { x: Foo<number, string>; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_instantiation_with_array() {
        // Template instantiation with array
        let result = parse("model B { x: Foo<number, string>[]; }");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Array Type Tests ====================

    #[test]
    fn test_parse_model_with_single_array_type() {
        // Model with single array type
        let result = parse("model A { foo: B[] }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_model_with_nested_array_type() {
        // Model with nested array type
        let result = parse("model A { foo: B[][] }");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Union Expression Tests ====================

    #[test]
    fn test_parse_model_with_union_type() {
        // Model with union type
        let result = parse("model A { foo: B | C }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_model_with_complex_union_type() {
        // Model with complex union type
        let result = parse("model A { foo: B | C & D }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_model_with_empty_union_prefix() {
        // Model with empty union prefix
        let result = parse("model A { foo: | B | C }");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Intersection Expression Tests ====================

    #[test]
    fn test_parse_model_with_intersection_type() {
        // Model with intersection type
        let result = parse("model A { foo: B & C }");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Multiple Statements Tests ====================

    #[test]
    fn test_parse_multiple_model_statements() {
        // Multiple model statements
        let result = parse("model A { }; model B { }");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Enum Spread Member Tests ====================

    #[test]
    fn test_parse_enum_with_spread_member() {
        // Enum with spread member
        let result = parse("enum Foo { ...Bar, Three: \"3\" }");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Empty Statement Tests ====================

    #[test]
    fn test_parse_empty_statements_multiple() {
        // Multiple empty statements
        let result = parse(";;;;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_empty_statements_with_namespace() {
        // Empty statements with namespace
        let result = parse("namespace Foo { model Car { }; };");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_empty_statements_after_model() {
        // Empty statements after model
        let result = parse("model Car { };;;;");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Numeric Literal Tests ====================

    #[test]
    fn test_parse_numeric_hex() {
        // Hexadecimal numeric literal
        let result = parse("const x = 0xABCD;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_numeric_binary() {
        // Binary numeric literal
        let result = parse("const x = 0b1010;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_numeric_decimal() {
        // Decimal numeric literal
        let result = parse("const x = 123;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_numeric_with_exponent() {
        // Numeric literal with exponent
        let result = parse("const x = 123e42;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_numeric_negative() {
        // Negative numeric literal
        let result = parse("const x = -123;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_numeric_positive() {
        // Positive numeric literal
        let result = parse("const x = +123;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_numeric_float() {
        // Float numeric literal
        let result = parse("const x = 123.456;");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Identifier Tests ====================

    #[test]
    fn test_parse_identifier_with_underscore() {
        // Identifier with underscore
        let result = parse("model has_underscore {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_identifier_with_dollar() {
        // Identifier with dollar sign
        let result = parse("model has_$dollar {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_identifier_starting_with_underscore() {
        // Identifier starting with underscore
        let result = parse("model _startsWithUnderscore {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_identifier_starting_with_dollar() {
        // Identifier starting with dollar sign
        let result = parse("model $startsWithDollar {}");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Alias with Expression Tests ====================

    #[test]
    fn test_parse_alias_with_numeric() {
        // Alias with numeric value
        let result = parse("alias M = 123;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_alias_with_hex() {
        // Alias with hex value
        let result = parse("alias M = 0xABCD;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_alias_with_binary() {
        // Alias with binary value
        let result = parse("alias M = 0b1010;");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Model Expression Tests ====================

    #[test]
    fn test_parse_model_expression_inline() {
        // Model with inline expression
        let result = parse(r#"model Car { engine: { type: "v8" } }"#);
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Tuple Model Expression Tests ====================

    #[test]
    fn test_parse_tuple_model_expression() {
        // Tuple model expression in operation
        let result = parse("alias EmptyTuple = [];");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_with_default_tuple() {
        // Template with default tuple value
        let result = parse("model Template<T=[]> { }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_alias_with_trailing_comma() {
        // Alias with trailing comma in tuple
        let result = parse("alias TrailingComma = [1, 2,];");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Multi-line Comments Tests ====================

    #[test]
    fn test_parse_model_with_emoji_in_comment() {
        // Model with emoji in comment (parsing only, not semantic)
        let result = parse(r#"model Car { /* 👀 */ value: int32; }"#);
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Enum Statement Tests ====================

    #[test]
    fn test_parse_enum_empty() {
        // Empty enum
        let result = parse("enum Foo { }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_enum_with_members() {
        // Enum with members
        let result = parse("enum Foo { a, b }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_alias_with_union() {
        // Alias with union
        let result = parse("alias X = A | B;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_alias_with_template() {
        // Alias with template
        let result = parse("alias MaybeUndefined<T> = T | undefined;");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Decorator Declaration Tests ====================

    #[test]
    fn test_parse_decorator_declaration_simple() {
        // Simple decorator declaration
        let result = parse("dec myDec(target: Type);");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_decorator_declaration_extern() {
        // Extern decorator declaration
        let result = parse("extern dec myDec(target: Type);");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_decorator_declaration_with_args() {
        // Decorator declaration with arguments
        let result = parse("extern dec myDec(target: Type, arg1: StringLiteral);");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_decorator_declaration_with_optional() {
        // Decorator declaration with optional parameter
        let result = parse("extern dec myDec(target: Type, optional?: StringLiteral);");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_decorator_declaration_with_rest() {
        // Decorator declaration with rest parameter
        let result = parse("extern dec myDec(target: Type, ...rest: StringLiteral[]);");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_decorator_declaration_trailing_comma() {
        // Decorator declaration with trailing comma
        let result = parse("extern dec trailingComma(target, arg1: other, arg2: string,);");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Function Declaration Tests ====================

    #[test]
    fn test_parse_function_declaration_simple() {
        // Simple function declaration
        let result = parse("fn myDec(): void;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_function_declaration_extern() {
        // Extern function declaration
        let result = parse("extern fn myDec(): StringLiteral;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_function_declaration_with_arg() {
        // Function declaration with argument
        let result = parse("extern fn myDec(arg1: StringLiteral): void;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_function_declaration_with_optional() {
        // Function declaration with optional parameter
        let result = parse("extern fn myDec(optional?: StringLiteral): void;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_function_declaration_with_rest() {
        // Function declaration with rest parameter
        let result = parse("extern fn myDec(...rest: StringLiteral[]): void;");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Augment Decorator Tests ====================

    #[test]
    fn test_parse_augment_decorator_simple() {
        // Simple augment decorator
        let result = parse("@@tag(Foo);");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_augment_decorator_with_args() {
        // Augment decorator with arguments
        let result = parse(r#"@@doc(Foo, "x");"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_augment_decorator_with_member() {
        // Augment decorator on member
        let result = parse(r#"@@doc(Foo.prop1, "x");"#);
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Import Statement Tests ====================

    #[test]
    fn test_parse_import_statement() {
        // Simple import
        let result = parse(r#"import "x";"#);
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Scalar Statement Tests ====================

    #[test]
    fn test_parse_scalar_with_decorator() {
        // Scalar with decorator
        let result = parse("@foo() scalar uuid extends string;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_scalar_with_init() {
        // Scalar with init
        let result = parse("scalar uuid {\n        init fromString(def: string)\n      }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_scalar_with_init_and_extends() {
        // Scalar extends and has init
        let result =
            parse("scalar bar extends uuid {\n        init fromOther(abc: string)\n      }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_scalar_with_init_trailing_comma() {
        // Scalar with init and trailing comma
        let result =
            parse("scalar bar {\n        init trailingComma(abc: string, def: string,)\n      }");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Operation Statement Tests ====================

    #[test]
    fn test_parse_operation_with_trailing_comma() {
        // Operation with trailing comma
        let result = parse("op trailingCommas(a: string,b: other,): int32;");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Interface Statement Tests ====================

    #[test]
    fn test_parse_interface_with_template() {
        // Interface with template
        let result = parse("interface Foo<T> { }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_interface_multi_extends() {
        // Interface with multiple extends
        let result = parse("interface Foo extends Bar, Baz<T> { }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_interface_with_multiple_operations() {
        // Interface with multiple operations
        let result = parse("interface Foo { foo(): int32; bar(): int32; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_interface_with_op_keyword() {
        // Interface with op keyword
        let result = parse("interface Foo { op foo(): int32; op bar(): int32; baz(): int32; }");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Union Declaration Tests ====================

    #[test]
    fn test_parse_union_declaration() {
        // Simple union
        let result = parse("union A { x: number, y: number } ");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_union_declaration_with_decorator() {
        // Union with decorator
        let result = parse("@myDec union A { @myDec a: string }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_union_declaration_string_key() {
        // Union with string key
        let result = parse(r#"union A { "hi there": string }"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_union_declaration_with_variants() {
        // Union with variant types
        let result = parse("union A { string, int32 }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_union_declaration_with_template_variants() {
        // Union with template variants
        let result = parse("union A { B<T>, C<T> }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_union_declaration_with_string_variants() {
        // Union with string variants
        let result = parse(r#"union A { "hi", `bye` }"#);
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Const Statement Tests ====================

    #[test]
    fn test_parse_const_with_type() {
        // Const with type annotation
        let result = parse("const a: Info = 123;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_const_with_object() {
        // Const with object literal
        let result = parse(r#"const a: {inline: string} = #{inline: "abc"};"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_const_with_union_type() {
        // Const with union type
        let result = parse("const a: string | int32 = int32;");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Call Expression Tests ====================

    #[test]
    fn test_parse_call_expression_member() {
        // Member call expression
        let result = parse("const a = utcDateTime.fromISO(\"abc\");");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_call_expression_multiple_args() {
        // Call with multiple args
        let result = parse(r#"const a = utcDateTime.fromISO("abc", "def");"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_call_expression_trailing_comma() {
        // Call with trailing comma
        let result = parse(r#"const trailingComma = utcDateTime.fromISO("abc", "def",);"#);
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Object Literal Tests ====================

    // ==================== Array Literal Tests ====================

    #[test]
    fn test_parse_array_literal_single() {
        // Simple array literal
        let result = parse(r#"const A = #["abc"];"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_array_literal_multiple() {
        // Array literal with multiple elements
        let result = parse(r#"const A = #["abc", 123];"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_array_literal_nested() {
        // Array literal with nested objects
        let result = parse(r#"const A = #["abc", 123, #{nested: true}];"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_array_literal_trailing_comma() {
        // Array literal with trailing comma
        let result = parse(r#"const A = #["abc", 123,];"#);
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Using Statement Tests ====================

    #[test]
    fn test_parse_using_in_namespace() {
        // Using inside namespace
        let result = parse("namespace Foo { using A; }");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Valueof Expression Tests ====================

    #[test]
    fn test_parse_valueof_string() {
        // Valueof string expression
        let result = parse("model Foo<T extends valueof string> {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_valueof_int32() {
        // Valueof int32 expression
        let result = parse("model Foo<T extends valueof int32> {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_valueof_object() {
        // Valueof object expression
        let result = parse("model Foo<T extends valueof {a: string, b: int32}> {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_valueof_array() {
        // Valueof array expression
        let result = parse("model Foo<T extends valueof int8[]> {}");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Intersection Expression Tests ====================

    #[test]
    fn test_parse_intersection_simple() {
        // Simple intersection
        let result = parse("model A { foo: B & C }");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Namespace Statement Tests ====================

    #[test]
    fn test_parse_namespace_with_operation() {
        // Namespace with operation
        let result = parse("namespace Store { op read(): int32; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_namespace_with_multiple_operations() {
        // Namespace with multiple operations
        let result = parse("namespace Store { op read(): int32; op write(v: int32): {}; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_namespace_with_decorator() {
        // Namespace with decorator
        let result = parse(
            "@foo namespace Store { @myDec op read(): number; @myDec op write(n: number): {}; }",
        );
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_namespace_multiple_decorators() {
        // Namespace with multiple decorators
        let result = parse("@foo @bar namespace Store { @foo @bar op read(): number; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_namespace_nested_block() {
        // Nested block namespaces
        let result = parse(
            "namespace Store { namespace Read { op read(): int32; } namespace Write { op write(v: int32): {}; } }",
        );
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_namespace_blockless_nested() {
        // Blockless nested namespace
        let result = parse("namespace Store.Read;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_namespace_blockless_with_decorator() {
        // Blockless namespace with decorator
        let result = parse("@foo namespace Store.Read;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_namespace_blockless_with_braces_and_decorator() {
        // Blockless namespace with braces and decorator
        let result = parse("@foo namespace Store.Read { };");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Tuple Model Expression Tests ====================

    #[test]
    fn test_parse_tuple_empty() {
        // Empty tuple
        let result = parse("alias EmptyTuple = [];");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_tuple_with_trailing_comma() {
        // Tuple with trailing comma
        let result = parse("alias TrailingComma = [1, 2,];");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_tuple_in_operation() {
        // Tuple in operation parameter
        let result = parse("namespace A { op b(param: [number, string]): [1, \"hi\"]; }");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Model Expression Tests ====================

    #[test]
    fn test_parse_model_expression() {
        // Model expression (inline)
        let result = parse(r#"model Car { engine: { type: "v8" } }"#);
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Model Extends Tests ====================

    #[test]
    fn test_parse_model_extends_simple() {
        // Simple model extends
        let result = parse("model foo extends bar { }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_model_extends_member() {
        // Model extends with member expression
        let result = parse("model foo extends bar.baz { }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_model_extends_template() {
        // Model extends with template
        let result = parse("model foo extends bar<T> { }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_model_extends_template_with_params() {
        // Model extends with template and parameters
        let result = parse("model foo<T> extends bar<T> { }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_model_extends_template_member() {
        // Model extends with template and member
        let result = parse("model foo<T> extends bar.baz<T> { }");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Array Type Tests ====================

    #[test]
    fn test_parse_array_type_simple() {
        // Simple array type
        let result = parse("model A { foo: B[] }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_array_type_nested() {
        // Nested array type
        let result = parse("model A { foo: B[][] }");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Model is Tests ====================

    #[test]
    fn test_parse_model_is_simple() {
        // Simple model is
        let result = parse("model Car is Vehicle;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_model_is_array() {
        // Model is array
        let result = parse("model Names is string[];");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Template Arguments Tests ====================

    #[test]
    fn test_parse_template_argument_simple() {
        // Simple template argument
        let result = parse("alias Test = Foo<T>;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_argument_trailing_comma() {
        // Template argument with trailing comma
        let result = parse("alias TrailingComma = Foo<A, B,>;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_argument_with_function() {
        // Template argument with function type
        let result = parse("alias Test = Foo<fn()>;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_argument_with_function_return_type() {
        // Template argument with function return type
        let result = parse("alias Test = Foo<fn() => string>;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_argument_with_function_params() {
        // Template argument with function parameters
        let result = parse("alias Test = Foo<fn(a: string, b: int) => string>;");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_template_argument_with_function_trailing_comma() {
        // Template argument with function and trailing comma
        let result = parse("alias TrailingComma = Foo<fn(a: string, b: int) => string,>;");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Model Properties with Keywords Tests ====================

    #[test]
    fn test_parse_model_with_keyword_property_name() {
        // Model with keyword as property name
        let result = parse("model Foo { interface: string }");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Model Multiple Decorators Tests ====================

    #[test]
    fn test_parse_model_multiple_decorators() {
        // Model with multiple decorators
        let result = parse("@foo @bar model Foo {}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_model_property_with_multiple_decorators() {
        // Model property with multiple decorators
        let result = parse("model Foo { @foo @bar name: string }");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Union Variants Tests ====================

    // ==================== Scalar Multiple Constructors Tests ====================

    #[test]
    fn test_parse_scalar_multiple_constructors() {
        // Scalar with multiple constructors
        let result =
            parse("scalar MyScalar {\ninit fromString(s: string)\ninit fromInt(i: int32)\n}");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Operation Multiple Parameters Tests ====================

    #[test]
    fn test_parse_operation_multiple_parameters() {
        // Operation with multiple parameters
        let result = parse("op foo(a: string, b: int32, c: boolean): void;");
        assert!(result.diagnostics.is_empty());
    }
}
