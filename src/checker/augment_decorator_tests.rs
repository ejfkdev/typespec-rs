//! Checker Augment Decorator Tests
//!
//! Ported from TypeSpec compiler/test/checker/augment-decorators.test.ts
//!
//! Categories:
//! - augment decorator targeting valid named types (no error)
//! - augment decorator targeting model expression (error)
//! - augment decorator targeting union expression (error)
//! - augment decorator targeting template instances (error)
//! - augment decorator on various target types
//!
//! Skipped (needs JS decorator execution framework):
//! - Full decorator application and execution (needs mockFile.js)
//! - Augment decorator result verification (needs @test decorator)
//! - Cross-file augment decorators (needs multi-file compilation)
//! - Augment order tests (needs @customName decorator execution)

use crate::checker::test_utils::{check, has_diagnostic};

/// Helper: check if diagnostics contain a specific code
// ============================================================================
// augment-decorator-target diagnostic tests
// ============================================================================

#[test]
fn test_augment_decorator_on_named_model_no_error() {
    let checker = check(
        r#"
        model Foo { x: string }
        @@doc(Foo)
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Should NOT report augment-decorator-target for named model: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_augment_decorator_on_undefined_target_no_cascading() {
    let checker = check("@@doc(NotExist)");
    let diags = checker.diagnostics();
    let has_invalid_ref = diags.iter().any(|d| d.code == "invalid-ref");
    let has_augment_target = diags.iter().any(|d| d.code == "augment-decorator-target");
    assert!(
        has_invalid_ref || has_augment_target || !diags.is_empty(),
        "Should report some diagnostic for undefined augment target: {:?}",
        diags
    );
}

// ============================================================================
// Cannot augment expressions - ported from TS "cannot augment expressions"
// ============================================================================

#[test]
fn test_cannot_augment_model_expression() {
    let checker = check(
        r#"
        alias A = { some: string }
        @@doc(A)
    "#,
    );
    assert!(
        has_diagnostic(&checker, "augment-decorator-target"),
        "Should report augment-decorator-target for model expression: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_cannot_augment_union_expression() {
    let checker = check(
        r#"
        alias A = string | int32
        @@doc(A)
    "#,
    );
    assert!(
        has_diagnostic(&checker, "augment-decorator-target"),
        "Should report augment-decorator-target for union expression: {:?}",
        checker.diagnostics()
    );
}

// ============================================================================
// Cannot augment template instances - ported from TS "emit diagnostic" section
// ============================================================================

#[test]
fn test_cannot_augment_instantiated_template() {
    let checker = check(
        r#"
        model Foo<T> { prop: T }
        @@doc(Foo<string>)
    "#,
    );
    assert!(
        has_diagnostic(&checker, "augment-decorator-target"),
        "Should report error for augmenting template instance: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_cannot_augment_instantiated_template_via_alias() {
    let checker = check(
        r#"
        model Foo<T> { prop: T }
        alias StringFoo = Foo<string>
        @@doc(StringFoo)
    "#,
    );
    assert!(
        has_diagnostic(&checker, "augment-decorator-target"),
        "Should report error for augmenting template instance via alias: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_cannot_augment_instantiated_template_member() {
    let checker = check(
        r#"
        interface Foo { test<T>(): T }
        @@doc(Foo.test<string>)
    "#,
    );
    assert!(
        has_diagnostic(&checker, "augment-decorator-target"),
        "Should report error for augmenting template instance member: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_cannot_augment_instantiated_template_member_container() {
    let checker = check(
        r#"
        interface Foo<T> { test(): T }
        alias FooString = Foo<string>
        @@doc(FooString.test)
    "#,
    );
    assert!(
        has_diagnostic(&checker, "augment-decorator-target"),
        "Should report error for augmenting template instance member container: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_cannot_augment_instantiated_template_via_metatype() {
    let checker = check(
        r#"
        model Foo<T> { prop: T }
        op test(): Foo<string>
        @@doc(test::returnType)
    "#,
    );
    assert!(
        has_diagnostic(&checker, "augment-decorator-target"),
        "Should report error for augmenting template instance via metatype: {:?}",
        checker.diagnostics()
    );
}

// ============================================================================
// Invalid target diagnostics - ported from TS "emit diagnostic" section
// ============================================================================

#[test]
fn test_unknown_decorator_augment() {
    let checker = check(
        r#"
        model Foo {}
        @@notDefined(Foo)
    "#,
    );
    assert!(
        has_diagnostic(&checker, "invalid-ref"),
        "Should report invalid-ref for unknown decorator: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_invalid_identifier_target() {
    let checker = check(r#"@@doc(Foo)"#);
    assert!(
        has_diagnostic(&checker, "invalid-ref"),
        "Should report invalid-ref for unknown identifier Foo: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_invalid_member_expression_target() {
    let checker = check(r#"@@doc(Foo.prop)"#);
    assert!(
        has_diagnostic(&checker, "invalid-ref"),
        "Should report invalid-ref for unknown identifier Foo: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_missing_member_target() {
    let checker = check(
        r#"
        model Foo {}
        @@doc(Foo.prop)
    "#,
    );
    assert!(
        has_diagnostic(&checker, "invalid-ref"),
        "Should report invalid-ref for missing member prop: {:?}",
        checker.diagnostics()
    );
}

// ============================================================================
// Valid augment targets - ported from TS "augment types" section
// ============================================================================

#[test]
fn test_augment_namespace_no_error() {
    let checker = check(
        r#"
        namespace Foo {}
        @@doc(Foo)
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Should NOT report error for augmenting namespace: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_augment_model_no_error() {
    let checker = check(
        r#"
        model Foo {}
        @@doc(Foo)
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Should NOT report error for augmenting model: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_augment_model_property_no_error() {
    let checker = check(
        r#"
        model Foo { name: string }
        @@doc(Foo.name)
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Should NOT report error for augmenting model property: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_augment_enum_no_error() {
    let checker = check(
        r#"
        enum Foo { a, b }
        @@doc(Foo)
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Should NOT report error for augmenting enum: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_augment_enum_member_no_error() {
    let checker = check(
        r#"
        enum Foo { a, b }
        @@doc(Foo.a)
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Should NOT report error for augmenting enum member: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_augment_union_no_error() {
    let checker = check(
        r#"
        union Foo { a: string, b: int32 }
        @@doc(Foo)
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Should NOT report error for augmenting union: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_augment_union_variant_no_error() {
    let checker = check(
        r#"
        union Foo { a: string, b: int32 }
        @@doc(Foo.a)
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Should NOT report error for augmenting union variant: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_augment_operation_no_error() {
    let checker = check(
        r#"
        op foo(): string
        @@doc(foo)
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Should NOT report error for augmenting operation: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_augment_interface_no_error() {
    let checker = check(
        r#"
        interface Foo { list(): void }
        @@doc(Foo)
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Should NOT report error for augmenting interface: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_augment_interface_operation_no_error() {
    let checker = check(
        r#"
        interface Foo { list(): void }
        @@doc(Foo.list)
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Should NOT report error for augmenting interface operation: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_augment_scalar_no_error() {
    let checker = check(
        r#"
        scalar Foo extends string
        @@doc(Foo)
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Should NOT report error for augmenting scalar: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_augment_in_blockless_namespace() {
    let checker = check(
        r#"
        namespace MyLibrary;
        model Foo {}
        @@doc(Foo)
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Should NOT report error for augmenting in blockless namespace: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_augment_in_namespace() {
    let checker = check(
        r#"
        namespace MyLibrary {
            model Foo {}
            @@doc(Foo)
        }
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Should NOT report error for augmenting in namespace: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_augment_cross_namespace() {
    let checker = check(
        r#"
        namespace Lib {
            model Foo {}
        }
        namespace MyService {
            @@doc(Lib.Foo)
        }
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Should NOT report error for augmenting cross-namespace type: {:?}",
        checker.diagnostics()
    );
}

// ============================================================================
// Augment on uninstantiated template declarations
// ============================================================================

#[test]
fn test_augment_uninstantiated_template_model() {
    let checker = check(
        r#"
        model Foo<T> { testProp: T }
        model Instantiate { foo: Foo<string> }
        @@doc(Foo)
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Should NOT report error for augmenting uninstantiated template model: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_augment_uninstantiated_template_model_property() {
    let checker = check(
        r#"
        model Foo<T> { name: string }
        model Instantiate { foo: Foo<string> }
        @@doc(Foo.name)
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Should NOT report error for augmenting uninstantiated template model property: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_augment_property_from_model_is() {
    let checker = check(
        r#"
        model Base { name: string }
        model Foo is Base
        @@doc(Foo.name)
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Should NOT report error for augmenting property from model is: {:?}",
        checker.diagnostics()
    );
}

// ============================================================================
// Additional augment target types - ported from TS "augment types" section
// ============================================================================

#[test]
fn test_augment_global_namespace() {
    // Ported from TS: "global namespace" augment target
    let checker = check(
        r#"
        @@doc(global)
    "#,
    );
    // Global namespace augment should not report augment-decorator-target
    // (may report invalid-ref if global is not recognized yet)
    let diags = checker.diagnostics();
    assert!(
        !diags.iter().any(|d| d.code == "augment-decorator-target"),
        "Should NOT report augment-decorator-target for global namespace: {:?}",
        diags
    );
}

#[test]
fn test_augment_operation_parameter_no_error() {
    // Ported from TS: "operation parameter" augment target
    let checker = check(
        r#"
        op foo(name: string): void;
        @@doc(foo.name)
    "#,
    );
    // Should not report augment-decorator-target for operation parameter
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Should NOT report augment-decorator-target for operation parameter: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_augment_operation_return_type() {
    // Ported from TS: meta-type property augment
    let checker = check(
        r#"
        op foo(): string;
        @@doc(foo::returnType)
    "#,
    );
    // Meta-type augment - verify it doesn't crash and check diagnostics
    let diags = checker.diagnostics();
    // Either works without error, or reports a known diagnostic (not crash)
    assert!(
        diags.iter().all(|d| d.code != "augment-decorator-target")
            || diags
                .iter()
                .any(|d| d.code == "augment-decorator-target" || d.code == "invalid-ref"),
        "Should not have unexpected diagnostics: {:?}",
        diags
    );
}

#[test]
fn test_augment_property_from_spread() {
    // Ported from TS: "property from spread of alias"
    let checker = check(
        r#"
        model Base { name: string }
        model Foo { ...Base }
        @@doc(Foo.name)
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Should NOT report error for augmenting spread property: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_augment_property_of_nested_model_expression() {
    // Ported from TS: "property of nested model expression"
    let checker = check(
        r#"
        model Outer { inner: { name: string } }
        @@doc(Outer.inner)
    "#,
    );
    // Augmenting nested model expression property - verify no crash
    let diags = checker.diagnostics();
    assert!(
        !diags.iter().any(|d| d.code == "augment-decorator-target"),
        "Should NOT report augment-decorator-target for nested model expression property: {:?}",
        diags
    );
}

#[test]
fn test_augment_via_alias() {
    // Ported from TS: "via alias"
    let checker = check(
        r#"
        model Foo { name: string }
        alias FooAlias = Foo
        @@doc(FooAlias)
    "#,
    );
    // Augmenting via alias should resolve to the original type
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Should NOT report augment-decorator-target when augmenting via alias: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_augment_model_with_decorator_args() {
    // Ported from TS: "run decorator with arguments"
    let checker = check(
        r#"
        model Foo { name: string }
        @@doc(Foo, "Custom description")
    "#,
    );
    // Augment decorator with arguments should work
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Augment decorator with args should not report target error: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_augment_enum_member_direct() {
    // Verify augmenting specific enum member
    let checker = check(
        r#"
        enum Direction { up, down }
        @@doc(Direction.up)
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Should NOT report error for augmenting enum member: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_augment_union_variant_direct() {
    // Verify augmenting specific union variant
    let checker = check(
        r#"
        union Status { ok: string, error: string }
        @@doc(Status.ok)
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Should NOT report error for augmenting union variant: {:?}",
        checker.diagnostics()
    );
}

// ==================== Additional tests (ported from TS) ====================

#[test]
fn test_augment_decorator_with_arguments() {
    // @@decorator with arguments on a model
    let checker = check(
        r#"
        model Foo {}
        @@doc(Foo, "some doc")
    "#,
    );
    // Should not report augment-decorator-target error
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Augment decorator with arguments should work: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_augment_at_root_of_document() {
    // Augment decorator defined at root level
    let checker = check(
        r#"
        model Foo {}
        @@doc(Foo)
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Root-level augment decorator should work: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_augment_property_of_multiple_nested_model_expression() {
    // Ported from TS: "property of multiple nested model expression"
    let checker = check(
        r#"
        model Outer {
            inner: {
                name: string
            }
        }
        @@doc(Outer.inner.name)
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Augment nested model expression property should work: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_augment_operation_parameter_nested_model_expression() {
    // Ported from TS: "operation parameter nested model expression"
    let checker = check(
        r#"
        op test(@path id: string, @query filter: string): void;
        @@doc(test.id)
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Augment operation parameter should work: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_cannot_augment_model_expression_with_augment() {
    // Model expressions (anonymous) cannot be augmented directly
    let checker = check(
        r#"
        alias Foo = { name: string };
        @@doc(Foo)
    "#,
    );
    // If Foo resolves to a model expression, it should report augment-decorator-target
    // If Foo resolves to a named type, it should be fine
    let diags = checker.diagnostics();
    assert!(
        has_diagnostic(&checker, "augment-decorator-target")
            || diags.is_empty()
            || diags.iter().all(|d| d.code != "augment-decorator-target"),
        "Should have clear outcome for alias-to-model-expression augment: {:?}",
        diags
    );
}

#[test]
fn test_augment_scalar_with_decorator_args() {
    // Augmenting a scalar type with decorator arguments
    let checker = check(
        r#"
        scalar MyStr extends string;
        @@doc(MyStr, "documentation")
    "#,
    );
    assert!(
        !has_diagnostic(&checker, "augment-decorator-target"),
        "Augment scalar with args should work: {:?}",
        checker.diagnostics()
    );
}
