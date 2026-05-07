//! Integration tests for tspc compilation pipeline

use typespec_rs::checker::{Checker, CustomDecoratorDef};
use typespec_rs::checker::types::Type;
use typespec_rs::diagnostics::DiagnosticSeverity;
use typespec_rs::parser;

fn parse_and_check(source: &str) -> Checker {
    let parse_result = parser::parse(source);
    let mut checker = Checker::new();
    checker.set_parse_result(parse_result.root_id, parse_result.builder.clone());
    checker.check_program();
    checker
}

fn parse_and_check_with_decorators(
    source: &str,
    decorators: Vec<(&str, &str, &str)>,
) -> Checker {
    let parse_result = parser::parse(source);
    let mut checker = Checker::new();
    checker.set_parse_result(parse_result.root_id, parse_result.builder.clone());
    checker.register_decorators(decorators);
    checker.check_program();
    checker
}

// ============================================================================
// Basic pipeline tests
// ============================================================================

#[test]
fn test_pipeline_simple_model() {
    let checker = parse_and_check("model Pet { name: string; id: int32; }");
    let errors: Vec<_> = checker
        .diagnostics()
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect();
    assert!(errors.is_empty(), "should have no errors: {:?}", errors);
}

#[test]
fn test_pipeline_enum() {
    let checker = parse_and_check("enum Direction { north, south, east, west }");
    let errors: Vec<_> = checker
        .diagnostics()
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect();
    assert!(errors.is_empty(), "should have no errors: {:?}", errors);
}

#[test]
fn test_pipeline_union() {
    let checker = parse_and_check("union Shape { circle: string, square: int32 }");
    let errors: Vec<_> = checker
        .diagnostics()
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect();
    assert!(errors.is_empty(), "should have no errors: {:?}", errors);
}

#[test]
fn test_pipeline_namespace() {
    let checker = parse_and_check(
        r#"
        namespace MyService {
            model Item { id: string; }
        }
    "#,
    );
    let errors: Vec<_> = checker
        .diagnostics()
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect();
    assert!(errors.is_empty(), "should have no errors: {:?}", errors);
    assert!(checker.declared_types.contains_key("MyService"));
}

#[test]
fn test_pipeline_interface() {
    let checker = parse_and_check(
        r#"
        interface MyApi {
            op get(): string;
        }
    "#,
    );
    let errors: Vec<_> = checker
        .diagnostics()
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect();
    assert!(errors.is_empty(), "should have no errors: {:?}", errors);
}

#[test]
fn test_pipeline_scalar() {
    let checker = parse_and_check(
        r#"
        scalar myUrl extends url;
    "#,
    );
    let errors: Vec<_> = checker
        .diagnostics()
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect();
    assert!(errors.is_empty(), "should have no errors: {:?}", errors);
}

// ============================================================================
// Decorator registration tests
// ============================================================================

#[test]
fn test_register_decorator_creates_namespace() {
    let checker = parse_and_check_with_decorators(
        "model Config { name: string; }",
        vec![("command", "CLI", "unknown")],
    );

    assert!(checker.declared_types.contains_key("CLI"));

    let cli_id = checker.declared_types["CLI"];
    if let Some(Type::Namespace(ns)) = checker.get_type(cli_id) {
        assert!(ns.decorator_declarations.contains_key("command"));
    } else {
        panic!("CLI should be a namespace");
    }
}

#[test]
fn test_register_multiple_namespaces() {
    let checker = parse_and_check_with_decorators(
        "model Config { name: string; }",
        vec![
            ("command", "CLI", "unknown"),
            ("route", "HTTP", "Operation"),
        ],
    );

    assert!(checker.declared_types.contains_key("CLI"));
    assert!(checker.declared_types.contains_key("HTTP"));
}

#[test]
fn test_register_decorator_keyword_names() {
    // These names are TypeSpec reserved keywords — can't use with `extern dec`
    let checker = parse_and_check_with_decorators(
        "model Config { name: string; }",
        vec![
            ("flag", "CLI", "unknown"),
            ("arg", "CLI", "unknown"),
            ("env", "CLI", "unknown"),
        ],
    );

    let cli_id = checker.declared_types["CLI"];
    if let Some(Type::Namespace(ns)) = checker.get_type(cli_id) {
        assert!(ns.decorator_declarations.contains_key("flag"));
        assert!(ns.decorator_declarations.contains_key("arg"));
        assert!(ns.decorator_declarations.contains_key("env"));
    }
}

#[test]
fn test_register_decorator_target_type_preserved() {
    let checker = parse_and_check_with_decorators(
        "model Pet { name: string; }",
        vec![("route", "HTTP", "Operation")],
    );

    let http_id = checker.declared_types["HTTP"];
    if let Some(Type::Namespace(ns)) = checker.get_type(http_id) {
        let dec_id = ns.decorator_declarations["route"];
        if let Some(Type::Decorator(dt)) = checker.get_type(dec_id) {
            assert_eq!(dt.target_type, "Operation");
        }
    }
}

#[test]
fn test_register_decorator_in_existing_typespec_namespace() {
    let checker = parse_and_check_with_decorators(
        "model Pet { name: string; }",
        vec![("myCustom", "TypeSpec", "unknown")],
    );

    let ts_id = checker.declared_types["TypeSpec"];
    if let Some(Type::Namespace(ns)) = checker.get_type(ts_id) {
        assert!(ns.decorator_declarations.contains_key("myCustom"));
        // Standard decorators should still be present
        assert!(ns.decorator_declarations.contains_key("doc"));
        assert!(ns.decorator_declarations.contains_key("tag"));
    }
}

// ============================================================================
// Model feature tests
// ============================================================================

#[test]
fn test_model_inheritance() {
    let checker = parse_and_check(
        r#"
        model Animal { name: string; }
        model Pet extends Animal { owner: string; }
    "#,
    );
    let errors: Vec<_> = checker
        .diagnostics()
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect();
    assert!(errors.is_empty(), "should have no errors: {:?}", errors);
}

#[test]
fn test_model_spread() {
    let checker = parse_and_check(
        r#"
        model Base { id: string; }
        model Extended { ...Base; name: string; }
    "#,
    );
    let errors: Vec<_> = checker
        .diagnostics()
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect();
    assert!(errors.is_empty(), "should have no errors: {:?}", errors);
}

#[test]
fn test_discriminated_union() {
    let checker = parse_and_check(
        r#"
        @discriminator("kind")
        model Shape { kind: string; }
        model Circle extends Shape { kind: "circle"; radius: float64; }
        model Square extends Shape { kind: "square"; side: float64; }
    "#,
    );
    // May have warnings but should not have fatal errors
    let errors: Vec<_> = checker
        .diagnostics()
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect();
    assert!(errors.is_empty(), "should have no errors: {:?}", errors);
}

#[test]
fn test_record_type() {
    let checker = parse_and_check(
        r#"
        model Config extends Record<string> { version: string; }
    "#,
    );
    let errors: Vec<_> = checker
        .diagnostics()
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect();
    assert!(errors.is_empty(), "should have no errors: {:?}", errors);
}

// ============================================================================
// Enum feature tests
// ============================================================================

#[test]
fn test_string_enum_with_values() {
    let checker = parse_and_check(
        r#"
        enum Color { red: "red", green: "green", blue: "blue" }
    "#,
    );
    let errors: Vec<_> = checker
        .diagnostics()
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect();
    assert!(errors.is_empty(), "should have no errors: {:?}", errors);
}

#[test]
fn test_numeric_enum() {
    let checker = parse_and_check(
        r#"
        enum Status { active: 1, inactive: 0 }
    "#,
    );
    let errors: Vec<_> = checker
        .diagnostics()
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect();
    assert!(errors.is_empty(), "should have no errors: {:?}", errors);
}

// ============================================================================
// Complex TypeSpec patterns (inspired by typespec-rust test suite)
// ============================================================================

#[test]
fn test_recursive_model() {
    let checker = parse_and_check(
        r#"
        model TreeNode {
            value: string;
            left?: TreeNode;
            right?: TreeNode;
        }
    "#,
    );
    let errors: Vec<_> = checker
        .diagnostics()
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect();
    assert!(errors.is_empty(), "should have no errors: {:?}", errors);
}

#[test]
fn test_extensible_union() {
    let checker = parse_and_check(
        r#"
        union Colors {
            string,
            blue: "blue",
            green: "green",
        }
    "#,
    );
    let errors: Vec<_> = checker
        .diagnostics()
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect();
    assert!(errors.is_empty(), "should have no errors: {:?}", errors);
}

#[test]
fn test_multiple_operations() {
    let checker = parse_and_check(
        r#"
        model Widget { name: string; }
        op getWidget(): Widget;
        op listWidgets(): Widget[];
    "#,
    );
    let errors: Vec<_> = checker
        .diagnostics()
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect();
    assert!(errors.is_empty(), "should have no errors: {:?}", errors);
}

#[test]
fn test_nested_namespaces() {
    let checker = parse_and_check(
        r#"
        namespace Outer {
            model Inner { id: string; }
        }
    "#,
    );
    assert!(checker.declared_types.contains_key("Outer"));
}

#[test]
fn test_scalar_with_encode() {
    let checker = parse_and_check(
        r#"
        @encode("base64url")
        scalar base64urlBytes extends bytes;
    "#,
    );
    // @encode is a built-in decorator, should work
    let errors: Vec<_> = checker
        .diagnostics()
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect();
    assert!(errors.is_empty(), "should have no errors: {:?}", errors);
}

#[test]
fn test_model_with_additional_properties() {
    let checker = parse_and_check(
        r#"
        model AddlProps extends Record<string> {
            name: string;
        }
    "#,
    );
    let errors: Vec<_> = checker
        .diagnostics()
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect();
    assert!(errors.is_empty(), "should have no errors: {:?}", errors);
}
