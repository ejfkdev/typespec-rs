//! String Template Parser Tests
//!
//! Tests for string template parsing based on TypeSpec's string-template.test.ts
//! TypeSpec uses double-quoted strings with ${...} interpolation: "Start ${123} end"

#[cfg(test)]
mod tests {
    use crate::ast::types::*;
    use crate::parser::ast_builder::AstNode;
    use crate::parser::parse;

    /// Helper to extract root script node
    fn get_root_script(result: &crate::parser::ParseResult) -> Option<&TypeSpecScript> {
        match result.builder.id_to_node(result.root_id) {
            Some(AstNode::TypeSpecScript(script)) => Some(script),
            _ => None,
        }
    }

    /// Helper to check model declaration properties
    fn get_model_properties(
        result: &crate::parser::ParseResult,
        model_index: usize,
    ) -> Option<Vec<String>> {
        let script = get_root_script(result)?;
        let model_id = script.statements.get(model_index)?;
        match result.builder.id_to_node(*model_id) {
            Some(AstNode::ModelDeclaration(m)) => {
                let mut props = Vec::new();
                for prop_id in &m.properties {
                    match result.builder.id_to_node(*prop_id) {
                        Some(AstNode::ModelProperty(prop)) => {
                            if let Some(AstNode::Identifier(id)) =
                                result.builder.id_to_node(prop.name)
                            {
                                props.push(id.value.clone());
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
            Some(AstNode::StringTemplateExpression(_)) => Some(SyntaxKind::StringTemplateExpression),
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
            Some(AstNode::DecoratorDeclaration(_)) => {
                Some(SyntaxKind::DecoratorDeclarationStatement)
            }
            Some(AstNode::AugmentDecoratorStatement(_)) => {
                Some(SyntaxKind::AugmentDecoratorStatement)
            }
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

    // ==================== String Template Tests ====================

    #[test]
    fn test_parse_string_template_simple() {
        // Simple string template: "Start ${123} end"
        let result = parse(r#"model Test { test: "Start ${123} end" }"#);
        if !result.diagnostics.is_empty() {
        }
        assert!(
            result.diagnostics.is_empty(),
            "Expected no diagnostics: {:?}",
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
    fn test_parse_string_template_with_string_interpolation() {
        // String template with string interpolation
        let result = parse(r#"model Test { test: "Start ${"interpolate"} end" }"#);
        if !result.diagnostics.is_empty() {
        }
        assert!(
            result.diagnostics.is_empty(),
            "Expected no diagnostics: {:?}",
            result.diagnostics
        );

        let script = get_root_script(&result).unwrap();
        assert_eq!(script.statements.len(), 1);
    }

    #[test]
    fn test_parse_string_template_with_model_interpolation() {
        // String template with model interpolation
        let result = parse(r#"model TestModel {} model Test { test: "Start ${TestModel} end" }"#);
        if !result.diagnostics.is_empty() {
        }
        assert!(
            result.diagnostics.is_empty(),
            "Expected no diagnostics: {:?}",
            result.diagnostics
        );

        let script = get_root_script(&result).unwrap();
        assert!(
            script.statements.len() >= 2,
            "Should have at least two statements"
        );
    }

    #[test]
    fn test_parse_string_template_empty_interpolation() {
        // Empty string in template - regression test for issue #7401
        let result = parse(r#"model Test<T extends valueof string> {} model B is Test<"">;"#);
        if !result.diagnostics.is_empty() {
        }
        assert!(
            result.diagnostics.is_empty(),
            "Expected no diagnostics: {:?}",
            result.diagnostics
        );

        let script = get_root_script(&result).unwrap();
        assert!(
            script.statements.len() >= 2,
            "Should have at least two statements"
        );
    }

    #[test]
    fn test_parse_string_template_multiple_spans() {
        // Multiple interpolations in template
        let result = parse(r#"model Test { test: "${"a"} and ${"b"} and ${123}" }"#);
        if !result.diagnostics.is_empty() {
        }
        assert!(
            result.diagnostics.is_empty(),
            "Expected no diagnostics: {:?}",
            result.diagnostics
        );

        let script = get_root_script(&result).unwrap();
        assert!(
            script.statements.len() >= 1,
            "Should have at least one statement"
        );
    }

    #[test]
    fn test_parse_string_template_in_alias() {
        // String template in alias
        let result = parse(r#"alias Foo = "hello ${"world"}";"#);
        if !result.diagnostics.is_empty() {
        }
        assert!(
            result.diagnostics.is_empty(),
            "Expected no diagnostics: {:?}",
            result.diagnostics
        );

        let script = get_root_script(&result).unwrap();
        assert_eq!(script.statements.len(), 1);
    }

    #[test]
    fn test_parse_string_template_in_decorator() {
        // String template in decorator argument
        let result = parse(r#"@doc("Hello ${"world"}") model Foo {}"#);
        if !result.diagnostics.is_empty() {
        }
        assert!(
            result.diagnostics.is_empty(),
            "Expected no diagnostics: {:?}",
            result.diagnostics
        );

        let script = get_root_script(&result).unwrap();
        assert_eq!(script.statements.len(), 1);
    }

    #[test]
    fn test_parse_string_template_with_type_interpolation() {
        // String template interpolating type (string keyword)
        let result = parse(r#"alias Foo = "prefix ${string} suffix";"#);
        if !result.diagnostics.is_empty() {
        }
        assert!(
            result.diagnostics.is_empty(),
            "Expected no diagnostics: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_parse_string_template_with_const_interpolation() {
        // String template interpolating const value
        let result = parse(r#"const str1 = "hi"; alias str2 = "${str1} and something";"#);
        if !result.diagnostics.is_empty() {
        }
        assert!(
            result.diagnostics.is_empty(),
            "Expected no diagnostics: {:?}",
            result.diagnostics
        );

        let script = get_root_script(&result).unwrap();
        assert!(
            script.statements.len() >= 2,
            "Should have at least two statements"
        );
    }

    #[test]
    fn test_parse_string_template_with_template_parameter() {
        // String template with template parameter
        let result = parse(r#"model Test<T extends valueof string> { prop: "${T}" }"#);
        if !result.diagnostics.is_empty() {
        }
        assert!(
            result.diagnostics.is_empty(),
            "Expected no diagnostics: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_parse_string_template_empty_head() {
        // String template starting with interpolation
        let result = parse(r#"model Test { test: "${123} end" }"#);
        if !result.diagnostics.is_empty() {
        }
        assert!(
            result.diagnostics.is_empty(),
            "Expected no diagnostics: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_parse_string_template_empty_tail() {
        // String template ending with interpolation
        let result = parse(r#"model Test { test: "start ${123}" }"#);
        if !result.diagnostics.is_empty() {
        }
        assert!(
            result.diagnostics.is_empty(),
            "Expected no diagnostics: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_parse_string_template_no_interpolation() {
        // Regular string without interpolation
        let result = parse(r#"model Test { test: "hello world" }"#);
        assert!(
            result.diagnostics.is_empty(),
            "Expected no diagnostics: {:?}",
            result.diagnostics
        );

        let props = get_model_properties(&result, 0);
        assert_eq!(
            props,
            Some(vec!["test".to_string()]),
            "Should have test property"
        );
    }

    #[test]
    fn test_parse_string_template_triple_quoted_no_interpolation() {
        // Triple-quoted string without interpolation
        let result = parse(r#"model Test { test: """hello world""" }"#);
        assert!(
            result.diagnostics.is_empty(),
            "Expected no diagnostics: {:?}",
            result.diagnostics
        );

        let props = get_model_properties(&result, 0);
        assert_eq!(
            props,
            Some(vec!["test".to_string()]),
            "Should have test property"
        );
    }

    #[test]
    fn test_parse_string_template_triple_quoted_with_newline() {
        // Triple-quoted string with newline
        let result = parse("model Test { prop: \"\"\"hello\nworld\"\"\" }");
        assert!(
            result.diagnostics.is_empty(),
            "Expected no diagnostics: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_parse_string_template_triple_quoted_with_quote_inside() {
        // Triple-quoted string with quotes inside
        let result = parse(r#"model Test { prop: """hello "world" end""" }"#);
        assert!(
            result.diagnostics.is_empty(),
            "Expected no diagnostics: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_parse_string_template_triple_quoted_with_interpolation() {
        // Triple-quoted string with interpolation
        let result = parse(r#"const str1 = "hi"; alias str2 = """${str1} and ${string}""";"#);
        let script = get_root_script(&result).unwrap();
        assert!(
            script.statements.len() >= 2,
            "Should have at least two statements"
        );
    }

    #[test]
    fn test_parse_string_template_value_in_type_context() {
        // String template with value in type context (checker-level error, not parser)
        let result = parse(r#"const str1 = "hi"; alias str2 = "with value ${str1}";"#);
        let script = get_root_script(&result).unwrap();
        assert!(
            script.statements.len() >= 2,
            "Should have at least two statements"
        );
    }

    #[test]
    fn test_parse_string_template_model_property_value_in_type_context() {
        // String template with value in model property type context
        let result = parse(r#"const str1 = "hi"; model Foo { a: "with value ${str1}" }"#);
        let script = get_root_script(&result).unwrap();
        assert!(
            script.statements.len() >= 2,
            "Should have at least two statements"
        );
    }

    #[test]
    fn test_parse_string_template_template_param_ambiguous() {
        // Template parameter that can be type or value (checker-level issue, not parser)
        let result = parse(r#"alias Template<T extends string | (valueof string)> = { a: "${T}" };"#);
        let script = get_root_script(&result).unwrap();
        assert_eq!(script.statements.len(), 1);
    }

    #[test]
    fn test_parse_string_template_template_param_value_used_as_type() {
        // Template parameter that is a value but used as type (checker-level issue)
        let result = parse(r#"alias Template<T extends valueof string> = { a: "${T}" };"#);
        let script = get_root_script(&result).unwrap();
        assert_eq!(script.statements.len(), 1);
    }
}
