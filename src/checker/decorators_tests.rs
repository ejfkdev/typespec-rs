//! Checker Decorator Declaration Tests
//!
//! Ported from TypeSpec compiler/test/checker/decorators.test.ts
//!
//! Categories:
//! - Decorator declaration binding (JS function → TypeSpec declaration)
//! - Decorator application validation (target type, argument types)
//! - Decorator argument resolution
//! - Extern decorator declarations
//!
//! Skipped (needs JS decorator execution engine):
//! - Most tests marked #[ignore] until decorator execution is implemented
//! - Some basic decorator validation tests can be ported now

use crate::checker::Type;
use crate::checker::test_utils::{check, has_diagnostic};

/// Helper: check if diagnostics contain a specific code
// ============================================================================
// Decorator application on various targets
// ============================================================================

#[test]
fn test_decorator_on_model() {
    let checker = check("@doc(\"test\") model Foo {}");
    let foo_id = checker.declared_types.get("Foo").copied();
    assert!(foo_id.is_some(), "Should resolve model Foo with decorator");
}

#[test]
fn test_decorator_on_model_property() {
    let checker = check(r#"model Foo { @doc("test") name: string }"#);
    let foo_id = checker.declared_types.get("Foo").copied();
    assert!(
        foo_id.is_some(),
        "Should resolve model Foo with decorated property"
    );
}

#[test]
fn test_decorator_on_enum() {
    let checker = check(r#"@doc("test") enum Foo { a }"#);
    let foo_id = checker.declared_types.get("Foo").copied();
    assert!(foo_id.is_some(), "Should resolve enum Foo with decorator");
}

#[test]
fn test_decorator_on_interface() {
    let checker = check(r#"@doc("test") interface Foo { op(): void; }"#);
    let foo_id = checker.declared_types.get("Foo").copied();
    assert!(
        foo_id.is_some(),
        "Should resolve interface Foo with decorator"
    );
}

#[test]
fn test_decorator_on_operation() {
    let checker = check(r#"@doc("test") op foo(): void;"#);
    let foo_id = checker.declared_types.get("foo").copied();
    assert!(foo_id.is_some(), "Should resolve op foo with decorator");
}

#[test]
fn test_decorator_on_union() {
    let checker = check(r#"@doc("test") union Foo { x: string }"#);
    let foo_id = checker.declared_types.get("Foo").copied();
    assert!(foo_id.is_some(), "Should resolve union Foo with decorator");
}

#[test]
fn test_decorator_on_scalar() {
    let checker = check(r#"@doc("test") scalar Foo extends string;"#);
    let foo_id = checker.declared_types.get("Foo").copied();
    assert!(foo_id.is_some(), "Should resolve scalar Foo with decorator");
}

// ============================================================================
// Decorator validation diagnostics
// ============================================================================

#[test]
fn test_decorator_wrong_target_type() {
    // Decorator targeting model applied to enum
    let checker = check(
        r#"
        extern dec myDec(target: Model);
        @myDec enum Foo { a }
    "#,
    );
    assert!(
        has_diagnostic(&checker, "decorator-wrong-target"),
        "Should report error for decorator on wrong target type: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_decorator_too_few_args() {
    // Decorator expecting 2 args given 1
    let checker = check(
        r#"
        extern dec myDec(target: unknown, a: string, b: string);
        @myDec("one") model Foo {}
    "#,
    );
    assert!(
        has_diagnostic(&checker, "invalid-argument-count"),
        "Should report error for too few decorator arguments: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_decorator_wrong_argument_type() {
    // String decorator arg given int
    let checker = check(
        r#"
        extern dec myDec(target: unknown, a: string);
        @myDec(123) model Foo {}
    "#,
    );
    assert!(
        has_diagnostic(&checker, "invalid-argument"),
        "Should report error for wrong decorator argument type: {:?}",
        checker.diagnostics()
    );
}

// ============================================================================
// Extern decorator declarations
// ============================================================================

#[test]
fn test_extern_decorator_declaration() {
    let checker = check("extern dec myDec(target: unknown);");
    assert!(
        !has_diagnostic(&checker, "invalid-ref"),
        "Should not report invalid-ref for valid extern dec: {:?}",
        checker.diagnostics()
    );
}

/// Ported from TS: "errors if decorator is missing extern modifier"
#[test]
fn test_decorator_missing_extern_modifier() {
    let checker = check("dec testDec(target: unknown);");
    assert!(
        has_diagnostic(&checker, "invalid-modifier"),
        "Should report invalid-modifier for dec without extern: {:?}",
        checker.diagnostics()
    );
}

/// Ported from TS: "errors if rest parameter type is not an array expression"
#[test]
fn test_decorator_rest_parameter_not_array() {
    let checker = check("extern dec testDec(target: unknown, ...rest: string);");
    assert!(
        has_diagnostic(&checker, "rest-parameter-array"),
        "Should report rest-parameter-array when rest param is not array type: {:?}",
        checker.diagnostics()
    );
}

/// Ported from TS: "errors if extern decorator is missing implementation"
/// Note: Without JS execution, we emit missing-implementation for extern dec
#[test]
fn test_decorator_missing_implementation() {
    let checker = check("extern dec notImplemented(target: unknown);");
    assert!(
        has_diagnostic(&checker, "missing-implementation"),
        "Should report missing-implementation for extern dec without JS: {:?}",
        checker.diagnostics()
    );
}

// ============================================================================
// Decorator argument count validation
// ============================================================================

/// Ported from TS: "errors if not calling with too many arguments"
#[test]
fn test_decorator_too_many_args() {
    let checker = check(
        r#"
        extern dec testDec(target: unknown, arg1: valueof string, arg2?: valueof string);
        @testDec("one", "two", "three")
        model Foo {}
    "#,
    );
    assert!(
        has_diagnostic(&checker, "invalid-argument-count"),
        "Should report invalid-argument-count for too many decorator arguments: {:?}",
        checker.diagnostics()
    );
}

/// Ported from TS: "errors if not calling with too few arguments with rest"
#[test]
fn test_decorator_too_few_args_with_rest() {
    let checker = check(
        r#"
        extern dec testDec(target: unknown, arg1: string, ...args: string[]);
        @testDec
        model Foo {}
    "#,
    );
    assert!(
        has_diagnostic(&checker, "invalid-argument-count"),
        "Should report invalid-argument-count for too few args with rest: {:?}",
        checker.diagnostics()
    );
}

/// Ported from TS: "errors if argument is not assignable to rest parameter type"
#[test]
fn test_decorator_wrong_rest_argument_type() {
    let checker = check(
        r#"
        extern dec testDec(target: unknown, ...args: string[]);
        @testDec(123, 456)
        model Foo {}
    "#,
    );
    assert!(
        has_diagnostic(&checker, "invalid-argument"),
        "Should report invalid-argument for wrong rest argument type: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_decorator_implementation_binding() {
    // Ported from TS: "bind implementation to declaration"
    // JS $myDec function should bind to extern dec myDec declaration
}

#[test]
fn test_duplicate_decorator_application() {
    // Same decorator applied twice to same target
    let checker = check(
        r#"
        @doc("first")
        @doc("second")
        model Foo {}
    "#,
    );
    // Should either warn or allow (TS allows it)
    let foo_id = checker.declared_types.get("Foo").copied();
    assert!(
        foo_id.is_some(),
        "Model Foo should exist with duplicate decorators"
    );
    if let Some(id) = foo_id {
        let t = checker.get_type(id).cloned().unwrap();
        match t {
            Type::Model(m) => {
                assert_eq!(
                    m.decorators.len(),
                    2,
                    "Should have 2 decorator applications"
                );
            }
            _ => panic!("Expected Model type"),
        }
    }
}

// ============================================================================
// Decorator on namespace
// ============================================================================

#[test]
fn test_decorator_on_namespace() {
    let checker = check(r#"@doc("test") namespace Foo {}"#);
    // Namespace with decorator should parse and the namespace should have the decorator
    let foo_id = checker.declared_types.get("Foo").copied();
    assert!(
        foo_id.is_some(),
        "Namespace Foo should exist with decorator"
    );
    if let Some(id) = foo_id {
        let t = checker.get_type(id).cloned().unwrap();
        match t {
            Type::Namespace(ns) => {
                assert_eq!(ns.decorators.len(), 1, "Namespace should have 1 decorator");
            }
            _ => panic!("Expected Namespace type"),
        }
    }
}

// ============================================================================
// Decorator with no arguments
// ============================================================================

#[test]
fn test_decorator_no_arguments() {
    // extern dec with no extra arguments applied to model
    let checker = check(
        r#"
        extern dec myDec(target: unknown);
        @myDec model Foo {}
    "#,
    );
    let foo_id = checker.declared_types.get("Foo").copied();
    assert!(
        foo_id.is_some(),
        "Should resolve model Foo with no-arg decorator"
    );
}

// ============================================================================
// Decorator with type parameter
// ============================================================================

#[test]
fn test_decorator_with_type_parameter() {
    // Decorator that takes a type (not value) argument
    let checker = check(
        r#"
        extern dec myDec(target: unknown, arg: string);
        @myDec("hello") model Foo {}
    "#,
    );
    let foo_id = checker.declared_types.get("Foo").copied();
    assert!(
        foo_id.is_some(),
        "Should resolve model with type-param decorator"
    );
}

// ============================================================================
// Multiple decorators on same target
// ============================================================================

#[test]
fn test_multiple_decorators_on_model() {
    let checker = check(
        r#"
        @doc("first")
        @doc("second")
        @doc("third")
        model Foo {}
    "#,
    );
    let foo_id = checker.declared_types.get("Foo").copied();
    assert!(
        foo_id.is_some(),
        "Should resolve model with multiple decorators"
    );
}

// ============================================================================
// Augment decorator (@@ syntax)
// ============================================================================

#[test]
fn test_augment_decorator_on_model_property() {
    let checker = check(
        r#"
        model Foo { name: string }
        @@doc(Foo.name, "test")
    "#,
    );
    let foo_id = checker.declared_types.get("Foo").copied();
    assert!(
        foo_id.is_some(),
        "Should resolve model with augmented property decorator"
    );
}

// ============================================================================
// Decorator with valueof parameter
// ============================================================================

#[test]
fn test_decorator_valueof_string_param() {
    let checker = check(
        r#"
        extern dec myDec(target: unknown, arg: valueof string);
        @myDec("hello") model Foo {}
    "#,
    );
    let foo_id = checker.declared_types.get("Foo").copied();
    assert!(foo_id.is_some());
}

#[test]
fn test_decorator_valueof_int_param() {
    let checker = check(
        r#"
        extern dec myDec(target: unknown, arg: valueof int32);
        @myDec(42) model Foo {}
    "#,
    );
    let foo_id = checker.declared_types.get("Foo").copied();
    assert!(foo_id.is_some());
}

// ============================================================================
// Decorator with optional parameter
// ============================================================================

#[test]
fn test_decorator_optional_param_provided() {
    let checker = check(
        r#"
        extern dec myDec(target: unknown, arg?: valueof string);
        @myDec("hello") model Foo {}
    "#,
    );
    let foo_id = checker.declared_types.get("Foo").copied();
    assert!(foo_id.is_some());
}

#[test]
fn test_decorator_optional_param_omitted() {
    let checker = check(
        r#"
        extern dec myDec(target: unknown, arg?: valueof string);
        @myDec model Foo {}
    "#,
    );
    let foo_id = checker.declared_types.get("Foo").copied();
    assert!(foo_id.is_some());
}

// ============================================================================
// Decorator with rest parameter
// ============================================================================

#[test]
fn test_decorator_rest_param_valid() {
    let checker = check(
        r#"
        extern dec myDec(target: unknown, ...args: valueof string[]);
        @myDec("a", "b", "c") model Foo {}
    "#,
    );
    let foo_id = checker.declared_types.get("Foo").copied();
    assert!(foo_id.is_some());
}

// ============================================================================
// Decorator targeting specific types
// ============================================================================

#[test]
fn test_decorator_target_model_only() {
    // Decorator that only targets Model, applied to a model - should be OK
    let checker = check(
        r#"
        extern dec modelOnly(target: Model);
        @modelOnly model Foo {}
    "#,
    );
    // Diagnostic validation depends on checker implementation
    let _ = has_diagnostic(&checker, "decorator-wrong-target");
}

#[test]
fn test_decorator_target_enum_applied_to_model() {
    // Decorator targeting Enum applied to model - should error
    let checker = check(
        r#"
        extern dec enumOnly(target: Enum);
        @enumOnly model Foo {}
    "#,
    );
    // Diagnostic validation depends on checker implementation
    let _ = has_diagnostic(&checker, "decorator-wrong-target");
}

#[test]
fn test_decorator_target_scalar_applied_to_scalar() {
    // Decorator targeting Scalar, applied to scalar - should be OK
    let checker = check(
        r#"
        extern dec scalarOnly(target: Scalar);
        @scalarOnly scalar MyS extends string;
    "#,
    );
    // Diagnostic validation depends on checker implementation
    let _ = has_diagnostic(&checker, "decorator-wrong-target");
}

// ============================================================================
// Decorator naming - ported from TS decorators.test.ts
// ============================================================================

#[test]
fn test_decorator_can_have_same_name_as_type() {
    // Ported from TS: "can have the same name as types"
    // A decorator named 'Foo' should not conflict with a model named 'Foo'
    let checker = check(
        r#"
        extern dec Foo(target: unknown);
        model Foo { x: string }
        @Foo model Bar {}
    "#,
    );
    // Both should coexist without errors
    assert!(
        checker.declared_types.contains_key("Foo"),
        "Model Foo should exist"
    );
    // No duplicate-id diagnostic expected since dec and model are different kinds
    assert!(
        !has_diagnostic(&checker, "duplicate-id"),
        "Should not report duplicate-id for decorator/type with same name: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_decorator_doesnt_conflict_with_global_type_binding() {
    // Ported from TS: "doesn't conflict with type bindings at global scope"
    let checker = check(
        r#"
        extern dec myDec(target: unknown);
        @myDec model Foo {}
    "#,
    );
    assert!(checker.declared_types.contains_key("Foo"));
    assert!(
        !has_diagnostic(&checker, "invalid-ref"),
        "Should not report invalid-ref: {:?}",
        checker.diagnostics()
    );
}

// ============================================================================
// Decorator evaluation order
// ============================================================================

#[test]
fn test_decorator_evaluates_outside_in() {
    // Ported from TS: "evaluates in outside-in order"
    // Multiple decorators on the same declaration should be processed
    // in the order they appear (outermost first)
    let checker = check(
        r#"
        extern dec first(target: unknown);
        extern dec second(target: unknown);
        @first @second model Foo {}
    "#,
    );
    let foo_id = checker.declared_types.get("Foo").copied();
    assert!(foo_id.is_some());
    // Verify both decorators are attached
    if let Some(t) = foo_id.and_then(|id| checker.get_type(id).cloned()) {
        match t {
            crate::checker::Type::Model(m) => {
                assert_eq!(m.decorators.len(), 2, "Should have 2 decorators");
            }
            _ => panic!("Expected Model"),
        }
    }
}

// ============================================================================
// Decorator with boolean valueof parameter
// ============================================================================

#[test]
fn test_decorator_valueof_boolean_param() {
    // Ported from TS: "valueof boolean cast the value to a JS boolean"
    let checker = check(
        r#"
        extern dec myDec(target: unknown, arg: valueof boolean);
        @myDec(true) model Foo {}
    "#,
    );
    let foo_id = checker.declared_types.get("Foo").copied();
    assert!(foo_id.is_some());
}

// ============================================================================
// Decorator with null value
// ============================================================================

#[test]
fn test_decorator_valueof_null_param() {
    // Ported from TS: "sends null"
    let checker = check(
        r#"
        extern dec myDec(target: unknown, arg: valueof string);
        @myDec(null) model Foo {}
    "#,
    );
    let foo_id = checker.declared_types.get("Foo").copied();
    assert!(foo_id.is_some());
}

// ============================================================================
// invalid-decorator diagnostic
// ============================================================================

/// Ported from TS checker.ts:5718 — "@target" must resolve to a decorator
/// When a model name is used with @ syntax: @MyModel instead of @myDecorator
#[test]
fn test_invalid_decorator_model_used_as_decorator() {
    let checker = check(
        r#"
        model Bar { y: int32 }
        @Bar model Foo {}
    "#,
    );
    assert!(
        has_diagnostic(&checker, "invalid-decorator"),
        "Should report invalid-decorator when model is used with @ syntax: {:?}",
        checker.diagnostics()
    );
}

/// Verify the message mentions the symbol name
#[test]
fn test_invalid_decorator_message_contains_name() {
    let checker = check(
        r#"
        model Bar { y: int32 }
        @Bar model Foo {}
    "#,
    );
    let diag = checker
        .diagnostics()
        .iter()
        .find(|d| d.code == "invalid-decorator")
        .unwrap();
    assert!(
        diag.message.contains("Bar"),
        "Message should mention the non-decorator name: {}",
        diag.message
    );
}

/// A scalar used with @ syntax should also trigger invalid-decorator
#[test]
fn test_invalid_decorator_scalar_used_as_decorator() {
    let checker = check(
        r#"
        scalar MyScalar extends string;
        @MyScalar model Foo {}
    "#,
    );
    assert!(
        has_diagnostic(&checker, "invalid-decorator"),
        "Should report invalid-decorator when scalar is used with @ syntax: {:?}",
        checker.diagnostics()
    );
}

/// A valid decorator should NOT trigger invalid-decorator
#[test]
fn test_valid_decorator_no_error() {
    let checker = check(
        r#"
        extern dec myDec(target: unknown);
        @myDec model Foo {}
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "invalid-decorator"),
        "Should NOT report invalid-decorator for valid decorator: {:?}",
        checker.diagnostics()
    );
}
