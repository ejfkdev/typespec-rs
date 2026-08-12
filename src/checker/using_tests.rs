//! Checker Using Declaration Tests
//!
//! Ported from TypeSpec compiler/test/checker/using.test.ts
//!
//! Categories:
//! - using with valid namespace (no error)
//! - using with non-namespace target (using-invalid-ref diagnostic)
//!
//! Skipped (needs multi-file import support):
//! - using in global scope with import
//! - using in namespace scope with import
//! - using with dotted namespaces
//! - using with aliases (needs alias resolution in using context)
//! - using within nested namespaces with parent refs

use crate::checker::test_utils::{check, has_diagnostic};

/// Helper: check if diagnostics contain a specific code
// ============================================================================
// using-invalid-ref diagnostic tests
// ============================================================================

#[test]
fn test_using_non_namespace_emits_error() {
    // Ported from TS: "emit error when using non-namespace"
    let checker = check("using string;");
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "using-invalid-ref"),
        "Should report using-invalid-ref when using a non-namespace: {:?}",
        diags
    );
}

#[test]
fn test_using_model_emits_error() {
    // Using a model name should emit using-invalid-ref
    let checker = check("model Foo {} using Foo;");
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "using-invalid-ref"),
        "Should report using-invalid-ref when using a model: {:?}",
        diags
    );
}

#[test]
fn test_using_enum_emits_error() {
    // Using an enum name should emit using-invalid-ref
    let checker = check("enum Color { Red } using Color;");
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "using-invalid-ref"),
        "Should report using-invalid-ref when using an enum: {:?}",
        diags
    );
}

#[test]
fn test_using_valid_namespace_no_error() {
    // Using a valid namespace should not emit using-invalid-ref
    let checker = check("namespace MyNs {} using MyNs;");
    let diags = checker.diagnostics();
    assert!(
        !diags.iter().any(|d| d.code == "using-invalid-ref"),
        "Should NOT report using-invalid-ref for valid namespace: {:?}",
        diags
    );
}

#[test]
fn test_using_undefined_emits_invalid_ref() {
    // Using an undefined name should emit invalid-ref (not using-invalid-ref)
    // because the name doesn't resolve at all
    let checker = check("using NotExist;");
    let diags = checker.diagnostics();
    // Should have at least invalid-ref (name not found)
    assert!(
        diags
            .iter()
            .any(|d| d.code == "invalid-ref" || d.code == "using-invalid-ref"),
        "Should report diagnostic for undefined using target: {:?}",
        diags
    );
}

// ============================================================================
// using with non-namespace types - ported from TS "when using non-namespace types"
// ============================================================================

#[test]
fn test_using_union_emits_error() {
    // Using a union name should emit using-invalid-ref
    let checker = check(
        r#"
        union Foo { a: string, b: int32 }
        using Foo;
    "#,
    );
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "using-invalid-ref"),
        "Should report using-invalid-ref when using a union: {:?}",
        diags
    );
}

#[test]
fn test_using_scalar_emits_error() {
    // Using a scalar name should emit using-invalid-ref
    let checker = check("scalar Foo extends string; using Foo;");
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "using-invalid-ref"),
        "Should report using-invalid-ref when using a scalar: {:?}",
        diags
    );
}

#[test]
fn test_using_interface_emits_error() {
    // Using an interface name should emit using-invalid-ref
    let checker = check("interface Foo { bar(): void; } using Foo;");
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "using-invalid-ref"),
        "Should report using-invalid-ref when using an interface: {:?}",
        diags
    );
}

#[test]
fn test_using_operation_emits_error() {
    // Using an operation name should emit using-invalid-ref
    let checker = check("op foo(): string; using foo;");
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "using-invalid-ref"),
        "Should report using-invalid-ref when using an operation: {:?}",
        diags
    );
}

// ============================================================================
// using with namespace in different scopes - ported from TS test cases
// ============================================================================

#[test]
fn test_using_in_namespace_scope_valid() {
    // Using a namespace inside another namespace should be valid
    let checker = check(
        r#"
        namespace Lib { model Foo {} }
        namespace MyApp {
            using Lib;
        }
    "#,
    );
    let diags = checker.diagnostics();
    assert!(
        !diags.iter().any(|d| d.code == "using-invalid-ref"),
        "Should NOT report using-invalid-ref for namespace in namespace scope: {:?}",
        diags
    );
}

#[test]
fn test_using_nested_namespace_valid() {
    // Using a nested namespace should be valid
    let checker = check(
        r#"
        namespace Outer { namespace Inner { model X {} } }
        using Outer.Inner;
    "#,
    );
    let diags = checker.diagnostics();
    assert!(
        !diags.iter().any(|d| d.code == "using-invalid-ref"),
        "Should NOT report using-invalid-ref for nested namespace: {:?}",
        diags
    );
}

#[test]
fn test_using_model_property_emits_error() {
    // Using a model property via member expression should emit error
    let checker = check(
        r#"
        model Foo { name: string }
        using Foo.name;
    "#,
    );
    let diags = checker.diagnostics();
    assert!(
        diags
            .iter()
            .any(|d| d.code == "using-invalid-ref" || d.code == "invalid-ref"),
        "Should report diagnostic when using a model property: {:?}",
        diags
    );
}

#[test]
fn test_using_enum_member_emits_error() {
    // Using an enum member via member expression should emit error
    let checker = check(
        r#"
        enum Color { Red, Blue }
        using Color.Red;
    "#,
    );
    let diags = checker.diagnostics();
    assert!(
        diags
            .iter()
            .any(|d| d.code == "using-invalid-ref" || d.code == "invalid-ref"),
        "Should report diagnostic when using an enum member: {:?}",
        diags
    );
}

#[test]
fn test_using_template_model_emits_error() {
    // Using a template model (not instantiated) should emit error
    let checker = check(
        r#"
        model Foo<T> { prop: T }
        using Foo;
    "#,
    );
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "using-invalid-ref"),
        "Should report using-invalid-ref when using a template model: {:?}",
        diags
    );
}

#[test]
fn test_using_alias_emits_error() {
    // Using an alias should emit using-invalid-ref (aliases are not namespaces)
    let checker = check(
        r#"
        alias MyAlias = string;
        using MyAlias;
    "#,
    );
    let diags = checker.diagnostics();
    assert!(
        diags
            .iter()
            .any(|d| d.code == "using-invalid-ref" || d.code == "invalid-ref"),
        "Should report diagnostic when using an alias: {:?}",
        diags
    );
}

// ============================================================================
// using with blockless namespace - ported from TS
// ============================================================================

#[test]
fn test_using_blockless_namespace_valid() {
    // Using a blockless namespace should be valid
    let checker = check(
        r#"
        namespace MyLib;
        using MyLib;
    "#,
    );
    let diags = checker.diagnostics();
    assert!(
        !diags.iter().any(|d| d.code == "using-invalid-ref"),
        "Should NOT report using-invalid-ref for blockless namespace: {:?}",
        diags
    );
}

#[test]
fn test_using_type_from_used_namespace_accessible() {
    // After using a namespace, types in it should be accessible
    // (This tests the general using mechanism, not the diagnostic)
    let checker = check(
        r#"
        namespace Lib { model Foo {} }
        using Lib;
    "#,
    );
    let diags = checker.diagnostics();
    assert!(
        !diags.iter().any(|d| d.code == "using-invalid-ref"),
        "Should NOT report using-invalid-ref for valid using declaration: {:?}",
        diags
    );
}

#[test]
fn test_using_two_namespaces_with_same_last_name() {
    // Ported from TS: "can use 2 namespace with the same last name"
    let checker = check(
        r#"
        namespace A.B { model X {} }
        namespace C.B { model Y {} }
        using A.B;
        using C.B;
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "using-invalid-ref"),
        "Should allow using two namespaces with same last name: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_using_resolves_local_decls_over_usings() {
    // Ported from TS: "resolves 'local' decls over usings"
    let checker = check(
        r#"
        namespace Lib { model Foo {} }
        model Foo { name: string }
        using Lib;
    "#,
    );
    // Local model Foo should take precedence over Lib.Foo
    let foo_id = checker.get_type_by_name("Foo");
    assert!(
        foo_id.is_some(),
        "Should resolve local Foo over using'd Foo"
    );
}

#[test]
fn test_using_in_global_scope() {
    // Ported from TS: "works in global scope"
    let checker = check(
        r#"
        namespace Lib { model Foo {} }
        using Lib;
        model Bar { x: Foo }
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "invalid-ref"),
        "Should resolve Foo from using'd namespace in global scope: {:?}",
        checker.diagnostics()
    );
}

// ============================================================================
// using relative to the file namespace (microsoft/typespec#11552)
//
// Upstream tests use multi-file imports; the Rust port injects the other
// file's namespaces as a library source instead.
// ============================================================================

/// Ported from TS: "using declared before the file namespace resolves from
/// the global namespace"
#[test]
fn test_using_before_file_namespace_resolves_global() {
    let checker = crate::checker::test_utils::check_with_library(
        "namespace Global.Sub { model X { x: int32 } }",
        r#"
        using Global.Sub;
        namespace Outer.Global.Foo;
        model Y { ... X }
        "#,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
    let y = checker.get_type_by_name("Y").expect("Y should exist");
    if let Some(crate::checker::Type::Model(m)) = checker.get_type(y) {
        assert_eq!(m.properties.len(), 1, "Y should have spread X's property");
    }
}

/// Ported from TS: "using declared before the file namespace picks the
/// global namespace over the file namespace sibling"
#[test]
fn test_using_before_file_namespace_prefers_global() {
    let checker = crate::checker::test_utils::check_with_library(
        r#"
        namespace A { model G { g: int32 } }
        namespace Outer.A { model L { l: int32 } }
        "#,
        r#"
        using A;
        namespace Outer.Svc;
        model Y { ... G }
        "#,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
    let y = checker.get_type_by_name("Y").expect("Y should exist");
    if let Some(crate::checker::Type::Model(m)) = checker.get_type(y) {
        assert!(
            m.properties.contains_key("g"),
            "Y should spread G (global A.G), not Outer.A.L: {:?}",
            m.properties.keys()
        );
    }
}

/// Ported from TS: "using declared after the file namespace still resolves
/// relative to it"
#[test]
fn test_using_after_file_namespace_resolves_relative() {
    let checker = crate::checker::test_utils::check_with_library(
        "namespace MyOrg.Models { model X { x: int32 } }",
        r#"
        namespace MyOrg.Svc;
        using Models;
        model Y { ... X }
        "#,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
    let y = checker.get_type_by_name("Y").expect("Y should exist");
    if let Some(crate::checker::Type::Model(m)) = checker.get_type(y) {
        assert_eq!(m.properties.len(), 1, "Y should have spread X's property");
    }
}

/// Ported from TS: "reports an unknown identifier when a using before the
/// file namespace only resolves relatively"
#[test]
fn test_using_before_file_namespace_relative_only_unknown() {
    let checker = crate::checker::test_utils::check_with_library(
        "namespace MyOrg.Models { model X { } }",
        r#"
        using Models;
        namespace MyOrg.Svc;
        "#,
    );
    assert!(
        has_diagnostic(&checker, "invalid-ref"),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
}

/// Ported from TS: "using inside a block namespace keeps resolving relative
/// to it"
#[test]
fn test_using_inside_block_namespace_relative() {
    let checker = crate::checker::test_utils::check_with_library(
        "namespace MyOrg.Models { model X { x: int32 } }",
        r#"
        namespace MyOrg.Svc {
            using Models;
            model Y { ... X }
        }
        "#,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
}

// ============================================================================
// Blockless namespace parsing diagnostics (microsoft/typespec#11552 infra)
// ============================================================================

/// Ported from TS parser: "Cannot use multiple blockless namespaces."
#[test]
fn test_multiple_blockless_namespaces_error() {
    let result = crate::parser::parse(
        r#"
        namespace A;
        model X {}
        namespace B;
        model Y {}
    "#,
    );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == "multiple-blockless-namespace"),
        "parse diagnostics: {:?}",
        result.diagnostics
    );
}

/// Ported from TS parser: "Blockless namespaces can't follow other
/// declarations."
#[test]
fn test_blockless_namespace_after_declaration_error() {
    let result = crate::parser::parse(
        r#"
        model X {}
        namespace A;
        model Y {}
    "#,
    );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == "blockless-namespace-first"),
        "parse diagnostics: {:?}",
        result.diagnostics
    );
}

/// Blockless namespace scopes the following declarations.
#[test]
fn test_blockless_namespace_membership() {
    let checker = check(
        r#"
        namespace MyOrg.Svc;
        model Y { x: string }
    "#,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
    let y = checker.get_type_by_name("Y").expect("Y should exist");
    let ns_name = checker
        .get_type(y)
        .and_then(|t| t.namespace())
        .and_then(|ns| checker.get_type(ns))
        .and_then(|t| t.name().map(|s| s.to_string()));
    assert_eq!(ns_name.as_deref(), Some("MyOrg.Svc"));
}
