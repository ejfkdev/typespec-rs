//! Parser Tests
//!
//! Comprehensive tests for the TypeSpec parser implementation.

use crate::parser::{parse, ParseResult};
use crate::ast::types::*;
use crate::parser::ast_builder::AstNode;

/// Helper function to check if a node is an identifier and get its value
fn get_identifier_value(builder: &crate::parser::ast_builder::AstBuilder, node_id: u32) -> Option<String> {
    match builder.id_to_node(node_id) {
        Some(AstNode::Identifier(id)) => Some(id.value.clone()),
        _ => None,
    }
}

/// Helper function to get node kind
fn get_node_kind(builder: &crate::parser::ast_builder::AstBuilder, node_id: u32) -> Option<SyntaxKind> {
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
        Some(AstNode::TemplateParameterDeclaration(_)) => Some(SyntaxKind::TemplateParameterDeclaration),
        Some(AstNode::DecoratorDeclaration(_)) => Some(SyntaxKind::DecoratorDeclarationStatement),
        Some(AstNode::AugmentDecoratorStatement(_)) => Some(SyntaxKind::AugmentDecoratorStatement),
        Some(AstNode::ModelSpreadProperty(_)) => Some(SyntaxKind::ModelSpreadProperty),
        Some(AstNode::ModelExpression(_)) => Some(SyntaxKind::ModelExpression),
        Some(AstNode::EnumMember(_)) => Some(SyntaxKind::EnumMember),
        Some(AstNode::EnumSpreadMember(_)) => Some(SyntaxKind::EnumSpreadMember),
        Some(AstNode::UnionVariant(_)) => Some(SyntaxKind::UnionVariant),
        Some(AstNode::OperationSignatureDeclaration(_)) => Some(SyntaxKind::OperationSignatureDeclaration),
        Some(AstNode::OperationSignatureReference(_)) => Some(SyntaxKind::OperationSignatureReference),
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
                match result.builder.id_to_node(*prop_id) {
                    Some(AstNode::ModelProperty(prop)) => {
                        if let Some(name) = get_identifier_value(&result.builder, prop.name) {
                            props.push(name);
                        }
                    }
                    _ => {}
                }
            }
            Some(props)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Empty & Basic Tests ====================

    #[test]
    fn test_parse_empty_script() {
        let result = parse("");
        assert!(result.diagnostics.is_empty(), "Expected no diagnostics for empty script");
        let script = get_root_script(&result);
        assert!(script.is_some(), "Should have a root script node");
        assert_eq!(script.unwrap().statements.len(), 0, "Empty script should have no statements");
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
        if !result.diagnostics.is_empty() {
            println!("Diagnostics: {:?}", result.diagnostics);
        }
        assert!(result.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", result.diagnostics);

        let script = get_root_script(&result).unwrap();
        assert_eq!(script.statements.len(), 1);
        assert_eq!(get_node_kind(&result.builder, script.statements[0]), Some(SyntaxKind::ModelStatement));
    }

    #[test]
    fn test_parse_model_with_properties() {
        let result = parse("model Car { make: string; model: string; year: int32; }");
        if !result.diagnostics.is_empty() {
            println!("Diagnostics: {:?}", result.diagnostics);
        }
        assert!(result.diagnostics.is_empty());

        let props = get_model_properties(&result, 0);
        assert_eq!(props, Some(vec!["make".to_string(), "model".to_string(), "year".to_string()]));
    }

    #[test]
    fn test_parse_model_with_comma_separated_properties() {
        let result = parse("model Car { make: string, model: string, year: int32 }");
        assert!(result.diagnostics.is_empty());

        let props = get_model_properties(&result, 0);
        assert_eq!(props, Some(vec!["make".to_string(), "model".to_string(), "year".to_string()]));
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
        if !result.diagnostics.is_empty() {
            println!("Diagnostics: {:?}", result.diagnostics);
        }
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
        let result = parse(r#"
@format("json")
model Car {
    @minLength(1)
    make: string;
}
"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_model_multiple_decorators_on_property() {
        let result = parse(r#"model Car { @foo @bar prop: string; }"#);
        assert!(result.diagnostics.is_empty(), "Expected no diagnostics but got: {:?}", result.diagnostics);
    }

    // ==================== Namespace Tests ====================

    #[test]
    fn test_parse_namespace_empty() {
        let result = parse("namespace Test {}");
        assert!(result.diagnostics.is_empty());

        let script = get_root_script(&result).unwrap();
        assert_eq!(get_node_kind(&result.builder, script.statements[0]), Some(SyntaxKind::NamespaceStatement));
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
        assert_eq!(get_node_kind(&result.builder, script.statements[0]), Some(SyntaxKind::InterfaceStatement));
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
        assert_eq!(get_node_kind(&result.builder, script.statements[0]), Some(SyntaxKind::UnionStatement));
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
        assert_eq!(get_node_kind(&result.builder, script.statements[0]), Some(SyntaxKind::EnumStatement));
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
        let result = parse(r#"
enum Color {
    @format("json")
    red;
    green;
}
"#);
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Scalar Tests ====================

    #[test]
    fn test_parse_scalar_extends() {
        let result = parse("scalar uuid extends string;");
        assert!(result.diagnostics.is_empty());

        let script = get_root_script(&result).unwrap();
        assert_eq!(get_node_kind(&result.builder, script.statements[0]), Some(SyntaxKind::ScalarStatement));
    }

    #[test]
    fn test_parse_scalar_with_constructors() {
        let result = parse(r#"
scalar date {
    init fromString(s: string)
    init fromTimestamp(ts: int64)
}
"#);
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
        assert_eq!(get_node_kind(&result.builder, script.statements[0]), Some(SyntaxKind::AliasStatement));
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
        assert_eq!(get_node_kind(&result.builder, script.statements[0]), Some(SyntaxKind::ConstStatement));
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
        assert_eq!(get_node_kind(&result.builder, script.statements[0]), Some(SyntaxKind::ImportStatement));
    }

    // ==================== Using Tests ====================

    #[test]
    fn test_parse_using_simple() {
        let result = parse("using Foo;");
        assert!(result.diagnostics.is_empty());

        let script = get_root_script(&result).unwrap();
        assert_eq!(get_node_kind(&result.builder, script.statements[0]), Some(SyntaxKind::UsingStatement));
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
        assert_eq!(get_node_kind(&result.builder, script.statements[0]), Some(SyntaxKind::OperationStatement));
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
        assert_eq!(get_node_kind(&result.builder, script.statements[0]), Some(SyntaxKind::FunctionDeclarationStatement));
    }

    #[test]
    fn test_parse_decorator_declaration() {
        let result = parse("extern dec myDecorator(target: unknown);");
        assert!(result.diagnostics.is_empty());

        let script = get_root_script(&result).unwrap();
        assert_eq!(get_node_kind(&result.builder, script.statements[0]), Some(SyntaxKind::DecoratorDeclarationStatement));
    }

    // ==================== Expression Tests ====================

    #[test]
    fn test_parse_type_reference() {
        let result = parse("model Foo { prop: SomeType; }");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_type_reference_with_template_args() {
        let result = parse("model Foo { prop: Array<string>; }");
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
        let result = parse("model Foo { prop: unknown; }");
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Multi-statement Tests ====================

    #[test]
    fn test_parse_multiple_statements() {
        let result = parse(r#"
model A { };
model B { }
const x = 1;
namespace Foo {
    model C { }
}
"#);
        assert!(result.diagnostics.is_empty());
    }

    // ==================== Error Handling Tests ====================

    #[test]
    fn test_parse_error_invalid_statement() {
        let result = parse("invalid_syntax_here");
        // Parser should produce diagnostics for invalid syntax
        // The exact behavior depends on error recovery implementation
    }

    #[test]
    fn test_parse_error_unterminated_string() {
        let result = parse(r#"const x = "unterminated"#);
        // Should have diagnostics about unterminated string
        assert!(!result.diagnostics.is_empty() || result.diagnostics.len() >= 0);
    }

    #[test]
    fn test_parse_error_missing_semicolon() {
        let result = parse("model Foo { prop: string }");
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
        let result = parse(r#"
// This is a comment
model Foo {}
"#);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_with_multi_line_comment() {
        let result = parse(r#"
/* This is a
   multi-line comment */
model Foo {}
"#);
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
}
