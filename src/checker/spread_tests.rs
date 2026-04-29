//! Checker Spread Tests
//!
//! Ported from TypeSpec compiler/test/checker/spread.test.ts
//!
//! Skipped (needs decorator execution):
//! - Clones decorated properties
//!
//! Skipped (needs deep spread resolution with aliases):
//! - Spread in model expression via alias
//! - Multiple spreads in chain (A spreads C, B spreads A)
//!
//! Skipped (needs property cloning for parent reference):
//! - Spread property parent model reference

use crate::checker::Type;
use crate::checker::test_utils::check;

// ============================================================================
// Basic Spread Property Tests
// ============================================================================

#[test]
fn test_spread_copies_properties() {
    // Ported from TS: "can extend one other interfaces" (spread variant)
    // model A { x: string } model B { ...A }
    let checker = check("model A { x: string; } model B { ...A; }");
    let b_type = checker.declared_types.get("B").copied().unwrap();
    let t = checker.get_type(b_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            assert!(
                m.properties.contains_key("x"),
                "B should have property 'x' from spread A"
            );
        }
        _ => panic!("Expected Model type"),
    }
}

#[test]
fn test_spread_multiple_models() {
    // model A { x: string } model B { y: int32 } model C { ...A, ...B }
    let checker = check(
        "
        model A { x: string; }
        model B { y: int32; }
        model C { ...A, ...B; }
    ",
    );
    let c_type = checker.declared_types.get("C").copied().unwrap();
    let t = checker.get_type(c_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            assert!(m.properties.contains_key("x"), "C should have 'x' from A");
            assert!(m.properties.contains_key("y"), "C should have 'y' from B");
            assert_eq!(m.properties.len(), 2, "C should have 2 properties");
        }
        _ => panic!("Expected Model type"),
    }
}

#[test]
fn test_spread_with_own_properties() {
    // model A { x: string } model B { ...A, y: int32 }
    let checker = check("model A { x: string; } model B { ...A; y: int32; }");
    let b_type = checker.declared_types.get("B").copied().unwrap();
    let t = checker.get_type(b_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            assert!(
                m.properties.contains_key("x"),
                "B should have 'x' from spread A"
            );
            assert!(m.properties.contains_key("y"), "B should have own 'y'");
            assert_eq!(m.properties.len(), 2, "B should have 2 properties");
        }
        _ => panic!("Expected Model type"),
    }
}

#[test]
fn test_spread_merges_namespaces() {
    // Ported from TS: "merges like namespaces"
    // namespace N { model X { x: string } } namespace N { model Y { y: string } }
    // namespace N { model Z { ...X, ...Y } }
    let checker = check(
        "
        namespace N {
            model X { x: string; }
            model Y { y: string; }
            model Z { ...X, ...Y; }
        }
    ",
    );
    let z_type = checker.declared_types.get("Z").copied().unwrap();
    let t = checker.get_type(z_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            assert_eq!(
                m.properties.len(),
                2,
                "Z should have 2 properties from spreads"
            );
            assert!(m.properties.contains_key("x"), "Z should have 'x'");
            assert!(m.properties.contains_key("y"), "Z should have 'y'");
        }
        _ => panic!("Expected Model type"),
    }
}

// ============================================================================
// Spread Diagnostic Tests
// ============================================================================

#[test]
fn test_spread_non_model_detected() {
    // Ported from: "emit diagnostic if spreading non model type"
    let checker = check(
        r#"
        alias U = (string | int32);
        model Foo {
            ...U
        }
    "#,
    );
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "spread-model"),
        "Should report spread-model when spreading union: {:?}",
        diags
    );
}

#[test]
fn test_spread_scalar_detected() {
    // Ported from: "emit diagnostic if spreading scalar type"
    let checker = check("model Foo { ...string; }");
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "spread-model"),
        "Should report spread-model when spreading scalar: {:?}",
        diags
    );
}

#[test]
fn test_spread_self_detected() {
    // Ported from: "emit diagnostic if model spreads itself"
    let checker = check(
        "
        model Foo {
            ...Foo,
            name: string,
        }
    ",
    );
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "spread-model"),
        "Should report spread-model when spreading self: {:?}",
        diags
    );
    // Verify the message mentions self-spread
    let spread_diag = diags.iter().find(|d| d.code == "spread-model").unwrap();
    assert!(
        spread_diag.message.contains("own declaration") || spread_diag.message.contains("within"),
        "Message should mention self-spread: {}",
        spread_diag.message
    );
}

#[test]
fn test_spread_self_via_alias_detected() {
    // Ported from: "emit diagnostic if model spreads itself through alias"
    let checker = check(
        "
        model Foo {
            ...Bar,
            name: string,
        }
        alias Bar = Foo;
    ",
    );
    let diags = checker.diagnostics();
    // May report spread-model (self) or circular-alias-type
    assert!(
        diags
            .iter()
            .any(|d| d.code == "spread-model" || d.code == "circular-alias-type"),
        "Should report spread-model or circular-alias-type: {:?}",
        diags
    );
}

#[test]
fn test_spread_valid_model_no_error() {
    // No diagnostic when spreading a valid model
    let checker = check("model A { x: string; } model B { ...A; }");
    let diags = checker.diagnostics();
    assert!(
        !diags.iter().any(|d| d.code == "spread-model"),
        "Should NOT report spread-model for valid spread: {:?}",
        diags
    );
}

#[test]
fn test_spread_invalid_ref_no_additional_diagnostic() {
    // Ported from: "doesn't emit additional diagnostic if spread reference is invalid-ref"
    let checker = check("model Foo { ...NotDefined; }");
    let diags = checker.diagnostics();
    // Should only report invalid-ref, not an extra spread-model
    assert!(
        diags.iter().any(|d| d.code == "invalid-ref"),
        "Should report invalid-ref for undefined type: {:?}",
        diags
    );
    // Should NOT report spread-model for the unresolved reference
    assert!(
        !diags.iter().any(|d| d.code == "spread-model"),
        "Should NOT report spread-model for invalid-ref: {:?}",
        diags
    );
}

#[test]
fn test_spread_duplicate_property_detected() {
    // Ported from: "emits duplicate diagnostic at correct location"
    let checker = check(
        "
        model Foo { x: string; }
        model Bar { x: string; ...Foo; }
    ",
    );
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "duplicate-property"),
        "Should report duplicate-property when spread introduces duplicate: {:?}",
        diags
    );
}

// ============================================================================
// Spread with Indexer (Record type)
// ============================================================================

#[test]
fn test_spread_record_type() {
    // Ported from TS: "can spread a Record<T>"
    let checker = check("model Test { ...Record<int32>; }");
    let test_type = checker.declared_types.get("Test").copied().unwrap();
    let t = checker.get_type(test_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            // Record spread should provide an indexer
            assert!(
                m.indexer.is_some(),
                "Test should have indexer from Record spread"
            );
        }
        _ => panic!("Expected Model type"),
    }
}

// ============================================================================
// Spread with Array Model
// ============================================================================

#[test]
fn test_spread_array_model_detected() {
    // Ported from TS: "emit diagnostic if spreading an T[]"
    let checker = check("model Test { ...Array<int32>; }");
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "spread-model"),
        "Should report spread-model when spreading Array: {:?}",
        diags
    );
}

// ============================================================================
// Additional Spread Tests
// ============================================================================

#[test]
fn test_spread_empty_model() {
    // Spreading an empty model should work without errors
    let checker = check("model Empty {} model Foo { ...Empty; }");
    let foo_type = checker.declared_types.get("Foo").copied().unwrap();
    let t = checker.get_type(foo_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            assert!(
                m.properties.is_empty(),
                "Foo should have no properties from empty spread"
            );
        }
        _ => panic!("Expected Model type"),
    }
    let diags = checker.diagnostics();
    assert!(diags.is_empty(), "Should have no diagnostics: {:?}", diags);
}

#[test]
fn test_spread_model_with_optional_properties() {
    // Spreading a model with optional properties
    let checker = check("model A { x?: string; } model B { ...A; }");
    let b_type = checker.declared_types.get("B").copied().unwrap();
    let t = checker.get_type(b_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            assert!(
                m.properties.contains_key("x"),
                "B should have 'x' from spread A"
            );
        }
        _ => panic!("Expected Model type"),
    }
}

#[test]
fn test_spread_in_namespace_model() {
    // Spread within a namespace
    let checker = check(
        "
        namespace N {
            model A { x: string; }
            model B { ...A; y: int32; }
        }
    ",
    );
    let b_type = checker.declared_types.get("B").copied().unwrap();
    let t = checker.get_type(b_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            assert!(
                m.properties.contains_key("x"),
                "B should have 'x' from spread A"
            );
            assert!(m.properties.contains_key("y"), "B should have own 'y'");
        }
        _ => panic!("Expected Model type"),
    }
}

#[test]
fn test_spread_with_decorator_on_source() {
    // Properties from spread should be present (decorators may not be cloned independently)
    let checker = check("model A { @doc x: string; } model B { ...A; }");
    let b_type = checker.declared_types.get("B").copied().unwrap();
    let t = checker.get_type(b_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            assert!(
                m.properties.contains_key("x"),
                "B should have 'x' from spread A"
            );
        }
        _ => panic!("Expected Model type"),
    }
}

#[test]
fn test_spread_enum_detected() {
    // Spreading an enum should report spread-model
    let checker = check("enum E { A, B } model Foo { ...E; }");
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "spread-model"),
        "Should report spread-model when spreading enum: {:?}",
        diags
    );
}

#[test]
fn test_spread_interface_detected() {
    // Spreading an interface should report spread-model
    let checker = check("interface I { foo(): void; } model Foo { ...I; }");
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "spread-model"),
        "Should report spread-model when spreading interface: {:?}",
        diags
    );
}

// ============================================================================
// Spread Diagnostic Message Tests
// ============================================================================

#[test]
fn test_spread_non_model_message() {
    // Ported from: "emit diagnostic if spreading non model type" - message check
    let checker = check(
        r#"
        alias U = (string | int32);
        model Foo {
            ...U
        }
    "#,
    );
    let diags = checker.diagnostics();
    let spread_diag = diags.iter().find(|d| d.code == "spread-model").unwrap();
    assert!(
        spread_diag.message.contains("non-model") || spread_diag.message.contains("Cannot spread"),
        "Message should mention non-model type or cannot spread: {}",
        spread_diag.message
    );
}

#[test]
fn test_spread_self_message() {
    // Ported from: "emit diagnostic if model spreads itself" - message check
    let checker = check(
        "
        model Foo {
            ...Foo,
            name: string,
        }
    ",
    );
    let diags = checker.diagnostics();
    let spread_diag = diags.iter().find(|d| d.code == "spread-model").unwrap();
    assert!(
        spread_diag.message.contains("own declaration") || spread_diag.message.contains("within"),
        "Message should mention self-spread: {}",
        spread_diag.message
    );
}

#[test]
fn test_spread_duplicate_property_with_spread_message() {
    // Ported from: "emits duplicate diagnostic at correct location" - message check
    let checker = check(
        "
        model Foo { x: string; }
        model Bar { x: string; ...Foo; }
    ",
    );
    let diags = checker.diagnostics();
    let dup_diag = diags
        .iter()
        .find(|d| d.code == "duplicate-property")
        .unwrap();
    assert!(
        dup_diag.message.contains("x"),
        "Message should mention property 'x': {}",
        dup_diag.message
    );
}

// ============================================================================
// Spread with Template Instantiation
// ============================================================================

#[test]
fn test_spread_template_model() {
    // Spreading a template model instantiation
    let checker = check(
        "
        model Template<T> { value: T; }
        model Foo { ...Template<string>; }
    ",
    );
    let foo_type = checker.declared_types.get("Foo").copied().unwrap();
    let t = checker.get_type(foo_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            assert!(
                m.properties.contains_key("value"),
                "Foo should have 'value' from spread Template<string>"
            );
        }
        _ => panic!("Expected Model type"),
    }
}

// ============================================================================
// Spread resolution order tests
// Ported from TS: "ensure the target model is completely resolved before spreading"
// ============================================================================

#[test]
fn test_spread_resolved_before_declared_before() {
    // Ported from TS: "declared before" (spread in model statement)
    // model B { ...A } model A { b: B; prop: string; }
    let checker = check(
        "
        model B { ...A }
        model A {
            b: B;
            prop: string;
        }
    ",
    );
    let b_type = checker.declared_types.get("B").copied().unwrap();
    let t = checker.get_type(b_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            assert!(
                m.properties.contains_key("b"),
                "B should have 'b' from spread A"
            );
            assert!(
                m.properties.contains_key("prop"),
                "B should have 'prop' from spread A"
            );
        }
        _ => panic!("Expected Model type"),
    }
}

#[test]
fn test_spread_resolved_before_declared_after() {
    // Ported from TS: "declared after" (spread in model statement)
    // model A { b: B; prop: string; } model B { ...A }
    let checker = check(
        "
        model A {
            b: B;
            prop: string;
        }
        model B { ...A }
    ",
    );
    let b_type = checker.declared_types.get("B").copied().unwrap();
    let t = checker.get_type(b_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            assert!(
                m.properties.contains_key("b"),
                "B should have 'b' from spread A"
            );
            assert!(
                m.properties.contains_key("prop"),
                "B should have 'prop' from spread A"
            );
        }
        _ => panic!("Expected Model type"),
    }
}

#[test]
fn test_spread_via_alias_declared_before() {
    // Ported from TS: "declared before" (spread in model expression via alias)
    // model B { ...Alias } alias Alias = { ...A }; model A { b: B; prop: string; }
    let checker = check(
        "
        model B { ...Alias }
        alias Alias = { ...A };
        model A {
            b: B;
            prop: string;
        }
    ",
    );
    let b_type = checker.declared_types.get("B").copied().unwrap();
    let t = checker.get_type(b_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            assert!(
                m.properties.contains_key("b"),
                "B should have 'b' from spread via alias"
            );
            assert!(
                m.properties.contains_key("prop"),
                "B should have 'prop' from spread via alias"
            );
        }
        _ => panic!("Expected Model type"),
    }
}

#[test]
fn test_spread_via_alias_declared_after() {
    // Ported from TS: "declared after" (spread in model expression via alias)
    // model A { b: B; prop: string; } alias Alias = { ...A }; model B { ...Alias }
    let checker = check(
        "
        model A {
            b: B;
            prop: string;
        }
        alias Alias = { ...A };
        model B { ...Alias }
    ",
    );
    let b_type = checker.declared_types.get("B").copied().unwrap();
    let t = checker.get_type(b_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            assert!(
                m.properties.contains_key("b"),
                "B should have 'b' from spread via alias"
            );
            assert!(
                m.properties.contains_key("prop"),
                "B should have 'prop' from spread via alias"
            );
        }
        _ => panic!("Expected Model type"),
    }
}

#[test]
fn test_spread_chain_resolved() {
    // Ported from TS: "in the middle" (multiple spreads)
    // model A { ...C } model B { ...A } model C { b: B; prop: string; }
    let checker = check(
        "
        model A { ...C }
        model B { ...A }
        model C {
            b: B;
            prop: string;
        }
    ",
    );
    let b_type = checker.declared_types.get("B").copied().unwrap();
    let t = checker.get_type(b_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            assert!(
                m.properties.contains_key("b"),
                "B should have 'b' from chain spread A->C"
            );
            assert!(
                m.properties.contains_key("prop"),
                "B should have 'prop' from chain spread A->C"
            );
        }
        _ => panic!("Expected Model type"),
    }
}
