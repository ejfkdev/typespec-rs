//! Declaration expressions tests
//!
//! Ported from TypeSpec compiler tests for microsoft/typespec#11019
//! ("allow declarations to be used as expressions").

use crate::checker::{Checker, Type};
use crate::diagnostics::CompilerOptions;
use crate::parser::parse;

/// Parse and check with the `declaration-expressions` feature enabled or not.
fn check_decl_expr(source: &str, feature_enabled: bool) -> Checker {
    let result = parse(source);
    let options = CompilerOptions {
        features: if feature_enabled {
            vec![
                "declaration-expressions".to_string(),
                "union-extends".to_string(),
            ]
        } else {
            Vec::new()
        },
        ..CompilerOptions::default()
    };
    let mut checker = Checker::with_options(options);
    checker.set_parse_result(result.root_id, result.builder);
    checker.check_program();
    checker
}

fn has_code(checker: &Checker, code: &str) -> bool {
    checker.diagnostics().iter().any(|d| d.code == code)
}

/// Ported from TS: declaration expressions require the feature.
#[test]
fn test_declaration_expression_requires_feature() {
    let checker = check_decl_expr("alias Foo = model { x: string };", false);
    assert!(
        has_code(&checker, "declaration-expression-disabled"),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
}

/// Ported from TS: "anonymous model declaration expression"
#[test]
fn test_anonymous_model_declaration_expression() {
    let checker = check_decl_expr("alias Foo = model { x: string };", true);
    assert!(
        checker.diagnostics().is_empty(),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
}

/// Ported from TS: "named declaration expression is not registered"
#[test]
fn test_named_declaration_expression_not_registered() {
    let checker = check_decl_expr("alias Foo = model Inner { x: string };", true);
    assert!(
        checker.diagnostics().is_empty(),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
    assert!(
        checker.get_type_by_name("Inner").is_none(),
        "named declaration expression must not be registered in the namespace"
    );
    assert!(checker.get_type_by_name("Foo").is_some());
}

/// Ported from TS: all four declaration kinds work in expression position.
#[test]
fn test_declaration_expressions_in_property_types() {
    let checker = check_decl_expr(
        r#"
        model Bar {
            status: enum { active, inactive };
            unit: scalar extends string;
            inner: model { x: string };
            choice: union { a: "x" };
        }
        "#,
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
    // The property types are the inline declarations.
    let bar = checker.get_type_by_name("Bar").expect("Bar");
    if let Some(Type::Model(m)) = checker.get_type(bar) {
        assert_eq!(m.properties.len(), 4);
        let status = *m.properties.get("status").expect("status");
        let status_type = match checker.get_type(status) {
            Some(Type::ModelProperty(p)) => p.r#type,
            _ => panic!("expected property"),
        };
        assert!(
            matches!(checker.get_type(status_type), Some(Type::Enum(_))),
            "status should be an inline enum"
        );
    }
}

/// Ported from TS: a declaration expression cannot have template parameters.
#[test]
fn test_templated_declaration_expression_rejected() {
    let checker = check_decl_expr("alias Foo = model Box<T> { x: T };", true);
    assert!(
        has_code(&checker, "templated-declaration-in-expression"),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
}

/// Union declaration expressions are marked as expressions (upstream sets
/// `expression: true` on the type).
#[test]
fn test_union_declaration_expression_flag() {
    let checker = check_decl_expr(r#"alias U = union { a: "x" };"#, true);
    assert!(
        checker.diagnostics().is_empty(),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
    let u = checker.get_type_by_name("U").expect("U alias");
    let resolved = checker.resolve_alias_chain(u);
    assert!(
        matches!(checker.get_type(resolved), Some(Type::Union(u)) if u.expression),
        "union declaration expression should have expression = true"
    );
}

/// Scalar declaration expression with `extends`.
#[test]
fn test_scalar_declaration_expression_extends() {
    let checker = check_decl_expr(
        r#"
        model M { s: scalar extends int32; }
        "#,
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
}
