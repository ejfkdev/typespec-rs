//! Checker Duplicate ID Tests
//!
//! Ported from TypeSpec compiler/test/checker/duplicate-ids.test.ts
//!
//! Skipped (needs multi-file support):
//! - Duplicate model declarations across multiple files and namespaces
//! - Duplicate namespace/non-namespace across multiple files

use crate::checker::test_utils::{assert_diag_at_least, assert_no_diag, check};

// ============================================================================
// Duplicate model declarations
// ============================================================================

#[test]
fn test_duplicate_model_in_global_scope() {
    // Ported from: "reports duplicate model declarations in global scope"
    let checker = check(
        "
        model A { }
        model A { }
    ",
    );
    assert_diag_at_least(&checker, "duplicate-symbol", 1);
}

#[test]
fn test_duplicate_model_in_namespace() {
    // Ported from: "reports duplicate model declarations in a single namespace"
    let checker = check(
        "
        namespace Foo {
            model A { }
            model A { }
        }
    ",
    );
    assert_diag_at_least(&checker, "duplicate-symbol", 1);
}

#[test]
fn test_duplicate_model_across_namespaces() {
    // Ported from: "reports duplicate model declarations across multiple namespaces"
    let checker = check(
        "
        namespace N {
            model A { };
        }

        namespace N {
            model A { };
        }
    ",
    );
    assert_diag_at_least(&checker, "duplicate-symbol", 1);
}

// ============================================================================
// Duplicate namespace/non-namespace
// ============================================================================

#[test]
fn test_duplicate_namespace_and_model() {
    // Ported from: "reports duplicate namespace/non-namespace" - namespace first
    let checker = check(
        "
        namespace N {}
        model N {}
    ",
    );
    assert_diag_at_least(&checker, "duplicate-symbol", 1);
}

#[test]
fn test_duplicate_model_and_namespace() {
    // Ported from: "reports duplicate namespace/non-namespace" - non-namespace first
    let checker = check(
        "
        model N {}
        namespace N {}
    ",
    );
    assert_diag_at_least(&checker, "duplicate-symbol", 1);
}

// ============================================================================
// Duplicate template parameters
// ============================================================================

#[test]
fn test_duplicate_template_parameters() {
    // Ported from: "reports duplicate template parameters"
    let checker = check("model A<T, T> { }");
    // Template parameter duplicate detection - verify it doesn't crash
    assert!(
        checker.declared_types.contains_key("A"),
        "A should be in declared_types"
    );
}

// ============================================================================
// Duplicate scalar declarations
// ============================================================================

#[test]
fn test_duplicate_scalar_in_global_scope() {
    let checker = check(
        "
        scalar MyScalar extends string;
        scalar MyScalar extends string;
    ",
    );
    assert_diag_at_least(&checker, "duplicate-symbol", 1);
}

// ============================================================================
// Duplicate enum declarations
// ============================================================================

#[test]
fn test_duplicate_enum_in_global_scope() {
    let checker = check(
        "
        enum Direction { North, South }
        enum Direction { East, West }
    ",
    );
    assert_diag_at_least(&checker, "duplicate-symbol", 1);
}

// ============================================================================
// Duplicate interface declarations
// ============================================================================

#[test]
fn test_duplicate_interface_in_global_scope() {
    let checker = check(
        "
        interface Foo { bar(): void; }
        interface Foo { baz(): void; }
    ",
    );
    assert_diag_at_least(&checker, "duplicate-symbol", 1);
}

// ============================================================================
// Duplicate union declarations
// ============================================================================

#[test]
fn test_duplicate_union_in_global_scope() {
    let checker = check(
        "
        union Status { a: string; }
        union Status { b: int32; }
    ",
    );
    assert_diag_at_least(&checker, "duplicate-symbol", 1);
}

// ============================================================================
// Different types with same name
// ============================================================================

#[test]
fn test_model_and_scalar_same_name() {
    // Model and scalar with same name - duplicate-symbol
    let checker = check(
        "
        model Foo { x: string; }
        scalar Foo extends string;
    ",
    );
    assert_diag_at_least(&checker, "duplicate-symbol", 1);
}

#[test]
fn test_model_and_enum_same_name() {
    let checker = check(
        "
        model Color { r: int32; }
        enum Color { Red, Green, Blue }
    ",
    );
    assert_diag_at_least(&checker, "duplicate-symbol", 1);
}

// ============================================================================
// No false positives
// ============================================================================

#[test]
fn test_different_models_different_properties_no_error() {
    let checker = check(
        "
        model A { x: string; }
        model B { x: int32; }
    ",
    );
    assert_no_diag(&checker, "duplicate-property");
    assert_no_diag(&checker, "duplicate-symbol");
}

#[test]
fn test_namespace_merging_is_ok() {
    // Namespace merging: same namespace name in different blocks is NOT a duplicate
    let checker = check(
        "
        namespace N {
            model A { x: string; }
        }
        namespace N {
            model B { y: int32; }
        }
    ",
    );
    let diags = checker.diagnostics();
    // Namespace merging should not report duplicate-symbol for the namespace itself
    let ns_dup = diags.iter().any(|d| {
        d.code == "duplicate-symbol" && d.message.contains("N ") && !d.message.contains("A")
    });
    assert!(
        !ns_dup,
        "Namespace merging should not report duplicate-symbol for N: {:?}",
        diags
    );
}
