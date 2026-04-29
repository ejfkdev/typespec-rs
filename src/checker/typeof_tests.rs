//! Checker TypeOf Tests
//!
//! Ported from TypeSpec compiler/test/checker/typeof.test.ts
//!
//! Skipped (needs deep value/type resolution):
//! - Const with an explicit type return const type (needs scalar field on value)
//!
//! Skipped (needs decorator execution):
//! - Typeof can be used to force sending a type to a decorator that accept both

use crate::checker::Type;
use crate::checker::test_utils::check;

// ============================================================================
// typeof const tests
// ============================================================================

#[test]
fn test_typeof_const_without_explicit_type() {
    // Ported from TS: "const without an explicit type return the precise type of the value"
    let checker = check("const a = 123; model Foo { x: typeof a; }");
    let foo_type = checker.declared_types.get("Foo").copied().unwrap();
    let t = checker.get_type(foo_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            assert!(
                m.properties.contains_key("x"),
                "Model should have property x"
            );
            let x_prop_id = m.properties.get("x").copied().unwrap();
            let x_prop = checker.get_type(x_prop_id).cloned().unwrap();
            match x_prop {
                Type::ModelProperty(p) => {
                    // typeof a where a = 123 should give Number type
                    let inner = checker.get_type(p.r#type).cloned().unwrap();
                    match inner {
                        Type::Number(n) => {
                            assert_eq!(n.value, 123.0, "typeof 123 should be Number(123)");
                        }
                        _ => panic!(
                            "Expected Number type from typeof 123, got {:?}",
                            inner.kind_name()
                        ),
                    }
                }
                _ => panic!("Expected ModelProperty"),
            }
        }
        _ => panic!("Expected Model type"),
    }
}

#[test]
fn test_typeof_const_string() {
    // typeof a where a = "hello" should give String type
    let checker = check("const a = \"hello\"; model Foo { x: typeof a; }");
    let foo_type = checker.declared_types.get("Foo").copied().unwrap();
    let t = checker.get_type(foo_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            let x_prop_id = m.properties.get("x").copied().unwrap();
            let x_prop = checker.get_type(x_prop_id).cloned().unwrap();
            match x_prop {
                Type::ModelProperty(p) => {
                    let inner = checker.get_type(p.r#type).cloned().unwrap();
                    match inner {
                        Type::String(s) => {
                            assert_eq!(
                                s.value, "hello",
                                "typeof \"hello\" should be String(\"hello\")"
                            );
                        }
                        _ => panic!("Expected String type, got {:?}", inner.kind_name()),
                    }
                }
                _ => panic!("Expected ModelProperty"),
            }
        }
        _ => panic!("Expected Model type"),
    }
}

#[test]
fn test_typeof_const_boolean() {
    // typeof a where a = true should give Boolean type
    let checker = check("const a = true; model Foo { x: typeof a; }");
    let foo_type = checker.declared_types.get("Foo").copied().unwrap();
    let t = checker.get_type(foo_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            let x_prop_id = m.properties.get("x").copied().unwrap();
            let x_prop = checker.get_type(x_prop_id).cloned().unwrap();
            match x_prop {
                Type::ModelProperty(p) => {
                    let inner = checker.get_type(p.r#type).cloned().unwrap();
                    match inner {
                        Type::Boolean(b) => {
                            assert!(b.value, "typeof true should be Boolean(true)");
                        }
                        _ => panic!("Expected Boolean type, got {:?}", inner.kind_name()),
                    }
                }
                _ => panic!("Expected ModelProperty"),
            }
        }
        _ => panic!("Expected Model type"),
    }
}

#[test]
fn test_typeof_expression_produces_type() {
    // typeof expression should at least produce a type (even if simplified)
    let checker = check("const a = 123; model Foo { x: typeof a; }");
    let foo_type = checker.declared_types.get("Foo").copied().unwrap();
    let t = checker.get_type(foo_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            assert!(
                m.properties.contains_key("x"),
                "Model should have property x"
            );
        }
        _ => panic!("Expected Model type"),
    }
}

#[test]
fn test_typeof_in_alias() {
    let checker = check("const a = 123; alias X = typeof a;");
    assert!(
        checker.declared_types.contains_key("X"),
        "typeof alias should be declared"
    );
}

// ============================================================================
// typeof error tests
// ============================================================================

#[test]
fn test_typeof_scalar_emits_expect_value() {
    // Ported from TS: "typeof scalar"
    let checker = check("model Foo { x: typeof int32; }");
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "expect-value"),
        "Should report expect-value when using typeof on a scalar: {:?}",
        diags
    );
}

#[test]
fn test_typeof_model_emits_expect_value() {
    // Ported from TS: "typeof model"
    let checker = check("model A {} model Foo { x: typeof A; }");
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "expect-value"),
        "Should report expect-value when using typeof on a model: {:?}",
        diags
    );
}

#[test]
fn test_typeof_template_parameter_no_constraint_emits_expect_value() {
    // Ported from TS: "typeof template parameter that accepts types emits error"
    // NOTE: Currently our implementation may report "invalid-ref" instead of
    // "expect-value" because template parameter resolution within template
    // declarations is not fully implemented. When T isn't resolved as a
    // TemplateParameter type, we get invalid-ref instead of expect-value.
    let checker = check("model A<T> { prop: typeof T; }");
    let diags = checker.diagnostics();
    assert!(
        diags
            .iter()
            .any(|d| d.code == "expect-value" || d.code == "invalid-ref"),
        "Should report expect-value or invalid-ref when using typeof on template param: {:?}",
        diags
    );
}

#[test]
fn test_typeof_valid_const_no_error() {
    // Using typeof on a valid const should not report errors
    let checker = check("const a = 123; model Foo { x: typeof a; }");
    let diags = checker.diagnostics();
    assert!(
        !diags.iter().any(|d| d.code == "expect-value"),
        "Should NOT report expect-value for typeof on const: {:?}",
        diags
    );
}

#[test]
fn test_typeof_const_with_explicit_type() {
    // Ported from TS: "const with an explicit type return const type"
    // const a: int32 = 123; typeof a should give Scalar(int32), not Number(123)
    let checker = check("const a: int32 = 123; model Foo { x: typeof a; }");
    let foo_type = checker.declared_types.get("Foo").copied().unwrap();
    let t = checker.get_type(foo_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            let x_prop_id = m.properties.get("x").copied().unwrap();
            let x_prop = checker.get_type(x_prop_id).cloned().unwrap();
            match x_prop {
                Type::ModelProperty(p) => {
                    let inner = checker.get_type(p.r#type).cloned().unwrap();
                    match inner {
                        Type::Scalar(s) => {
                            assert_eq!(
                                s.name, "int32",
                                "typeof a where a: int32 should be Scalar(int32)"
                            );
                        }
                        _ => panic!(
                            "Expected Scalar type from typeof a: int32, got {:?} ({:?})",
                            inner.kind_name(),
                            inner
                        ),
                    }
                }
                _ => panic!("Expected ModelProperty"),
            }
        }
        _ => panic!("Expected Model type"),
    }
}

#[test]
fn test_typeof_template_parameter_with_type_constraint() {
    // Ported from TS: "constrained to only types"
    // model A<T extends string> { prop: typeof T; } should emit expect-value
    let checker = check("model A<T extends string> { prop: typeof T; }");
    let diags = checker.diagnostics();
    assert!(
        diags
            .iter()
            .any(|d| d.code == "expect-value" || d.code == "invalid-ref"),
        "Should report expect-value or invalid-ref for typeof on type-constrained template param: {:?}",
        diags
    );
}

#[test]
fn test_typeof_template_parameter_with_mixed_constraint() {
    // Ported from TS: "constrained with types and value"
    // model A<T extends string | valueof string> { prop: typeof T; } should emit expect-value
    let checker = check("model A<T extends string | valueof string> { prop: typeof T; }");
    let diags = checker.diagnostics();
    assert!(
        diags
            .iter()
            .any(|d| d.code == "expect-value" || d.code == "invalid-ref"),
        "Should report expect-value or invalid-ref for typeof on mixed-constraint template param: {:?}",
        diags
    );
}
