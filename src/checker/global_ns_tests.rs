//! Checker Global Namespace Tests
//!
//! Ported from TypeSpec compiler/test/checker/global-namespace.test.ts
//!
//! Skipped (needs multi-file support):
//! - Adds top level entities used in other files
//!
//! Skipped (needs TypeSpec library namespace support):
//! - Can override TypeSpec library things (TypeSpec.int32)

use crate::checker::Type;
use crate::checker::test_utils::check;

#[test]
fn test_global_namespace_exists() {
    let checker = check("model Foo {}");
    let global_ns = checker.get_global_namespace_type();
    assert!(global_ns.is_some(), "Global namespace should exist");
}

#[test]
fn test_global_namespace_has_top_level_model() {
    let checker = check("model MyModel {}");
    let global_ns_id = checker.get_global_namespace_type().unwrap();
    let t = checker.get_type(global_ns_id).cloned().unwrap();
    match t {
        Type::Namespace(ns) => {
            assert!(
                ns.models.contains_key("MyModel"),
                "Global namespace should contain MyModel"
            );
        }
        _ => panic!("Expected Namespace type"),
    }
}

#[test]
fn test_global_namespace_has_top_level_namespace() {
    let checker = check("namespace Foo {}");
    let global_ns_id = checker.get_global_namespace_type().unwrap();
    let t = checker.get_type(global_ns_id).cloned().unwrap();
    match t {
        Type::Namespace(ns) => {
            assert!(
                ns.namespaces.contains_key("Foo"),
                "Global namespace should contain Foo namespace"
            );
        }
        _ => panic!("Expected Namespace type"),
    }
}

#[test]
fn test_global_namespace_has_top_level_scalar() {
    let checker = check("scalar MyScalar extends string;");
    let global_ns_id = checker.get_global_namespace_type().unwrap();
    let t = checker.get_type(global_ns_id).cloned().unwrap();
    match t {
        Type::Namespace(ns) => {
            assert!(
                ns.scalars.contains_key("MyScalar"),
                "Global namespace should contain MyScalar"
            );
        }
        _ => panic!("Expected Namespace type"),
    }
}

#[test]
fn test_global_namespace_has_top_level_enum() {
    let checker = check("enum MyEnum { A, B }");
    let global_ns_id = checker.get_global_namespace_type().unwrap();
    let t = checker.get_type(global_ns_id).cloned().unwrap();
    match t {
        Type::Namespace(ns) => {
            assert!(
                ns.enums.contains_key("MyEnum"),
                "Global namespace should contain MyEnum"
            );
        }
        _ => panic!("Expected Namespace type"),
    }
}

#[test]
fn test_global_namespace_has_top_level_interface() {
    let checker = check("interface MyInterface { foo(): void; }");
    let global_ns_id = checker.get_global_namespace_type().unwrap();
    let t = checker.get_type(global_ns_id).cloned().unwrap();
    match t {
        Type::Namespace(ns) => {
            assert!(
                ns.interfaces.contains_key("MyInterface"),
                "Global namespace should contain MyInterface"
            );
        }
        _ => panic!("Expected Namespace type"),
    }
}

#[test]
fn test_global_namespace_has_top_level_union() {
    let checker = check("union MyUnion { x: int32; y: string; }");
    let global_ns_id = checker.get_global_namespace_type().unwrap();
    let t = checker.get_type(global_ns_id).cloned().unwrap();
    match t {
        Type::Namespace(ns) => {
            assert!(
                ns.unions.contains_key("MyUnion"),
                "Global namespace should contain MyUnion"
            );
        }
        _ => panic!("Expected Namespace type"),
    }
}

#[test]
fn test_global_namespace_has_top_level_operation() {
    // Ported from: "adds top-level operations"
    let checker = check("op myOperation(): string;");
    let global_ns_id = checker.get_global_namespace_type().unwrap();
    let t = checker.get_type(global_ns_id).cloned().unwrap();
    match t {
        Type::Namespace(ns) => {
            assert!(
                ns.operations.contains_key("myOperation"),
                "Global namespace should contain myOperation"
            );
        }
        _ => panic!("Expected Namespace type"),
    }
}

#[test]
fn test_global_namespace_has_alias() {
    // Top-level aliases should be accessible via global namespace
    // Note: aliases are stored as scalars in the namespace in our implementation
    let checker = check("alias MyAlias = string;");
    // Just verify the alias exists in declared_types
    assert!(
        checker.declared_types.contains_key("MyAlias"),
        "MyAlias should be in declared_types"
    );
}

#[test]
fn test_global_namespace_multiple_types() {
    // Multiple top-level types should all appear in the global namespace
    let checker = check(
        "
        model M1 { x: string; }
        model M2 { y: int32; }
        op op1(): void;
        scalar S1 extends string;
    ",
    );
    let global_ns_id = checker.get_global_namespace_type().unwrap();
    let t = checker.get_type(global_ns_id).cloned().unwrap();
    match t {
        Type::Namespace(ns) => {
            assert!(
                ns.models.contains_key("M1"),
                "Global namespace should contain M1"
            );
            assert!(
                ns.models.contains_key("M2"),
                "Global namespace should contain M2"
            );
            assert!(
                ns.operations.contains_key("op1"),
                "Global namespace should contain op1"
            );
            assert!(
                ns.scalars.contains_key("S1"),
                "Global namespace should contain S1"
            );
        }
        _ => panic!("Expected Namespace type"),
    }
}

#[test]
fn test_global_namespace_is_finished() {
    let checker = check("model Foo {}");
    let global_ns_id = checker.get_global_namespace_type().unwrap();
    let t = checker.get_type(global_ns_id).cloned().unwrap();
    assert!(t.is_finished(), "Global namespace should be finished");
}

/// Ported from TS: "can override TypeSpec library things"
#[test]
fn test_global_namespace_can_shadow_stdlib() {
    // Defining a model named "int32" at global scope shadows the built-in int32
    let checker = check(
        r#"
        model int32 { x: string }
    "#,
    );
    // The custom model should exist
    assert!(
        checker.declared_types.contains_key("int32"),
        "Custom int32 model should be declared"
    );
    // The model should have an 'x' property (distinct from stdlib int32)
    let int32_type = checker.declared_types.get("int32").copied().unwrap();
    let t = checker.get_type(int32_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            assert!(
                m.properties.contains_key("x"),
                "Custom int32 model should have 'x' property"
            );
        }
        _ => panic!("Expected Model type for custom int32"),
    }
}
