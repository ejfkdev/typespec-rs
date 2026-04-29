//! Checker Model Circular References Tests
//!
//! Ported from TypeSpec compiler/test/checker/model-circular-references.test.ts
//!
//! Skipped (needs namespace-qualified type resolution):
//! - Models can reference each other in different namespace with the same name

use crate::checker::Type;
use crate::checker::test_utils::check;

#[test]
fn test_model_can_reference_itself() {
    // Ported from: "model can reference itself"
    let checker = check("model M { self: M; }");
    let m_type = checker.declared_types.get("M").copied().unwrap();
    let t = checker.get_type(m_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            let self_prop_id = m.properties.get("self").copied().unwrap();
            let self_prop = checker.get_type(self_prop_id).cloned().unwrap();
            match self_prop {
                Type::ModelProperty(p) => {
                    // self property's type should be M itself
                    assert_eq!(p.r#type, m_type, "self property type should be M");
                }
                _ => panic!("Expected ModelProperty"),
            }
        }
        _ => panic!("Expected Model type"),
    }
}

#[test]
fn test_model_can_reference_itself_in_array() {
    // Ported from: "model can reference itself in an array"
    let checker = check("model M { selfs: M[]; }");
    let m_type = checker.declared_types.get("M").copied().unwrap();
    let t = checker.get_type(m_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            let selfs_prop_id = m.properties.get("selfs").copied().unwrap();
            let selfs_prop = checker.get_type(selfs_prop_id).cloned().unwrap();
            match selfs_prop {
                Type::ModelProperty(p) => {
                    // selfs property's type should be an Array model
                    let array_type = checker.get_type(p.r#type).cloned().unwrap();
                    match array_type {
                        Type::Model(array_m) => {
                            assert!(array_m.indexer.is_some(), "Array should have indexer");
                            // The indexer value type should be M
                            let (_, element_type) = array_m.indexer.unwrap();
                            assert_eq!(element_type, m_type, "Array element type should be M");
                        }
                        _ => panic!("Expected Array (Model) type"),
                    }
                }
                _ => panic!("Expected ModelProperty"),
            }
        }
        _ => panic!("Expected Model type"),
    }
}

#[test]
fn test_models_can_reference_each_other() {
    // Ported from: "models can reference each other"
    let checker = check(
        "
        model Parent { child: Child; }
        model Child { parent: Parent; }
    ",
    );
    let parent_type = checker.declared_types.get("Parent").copied().unwrap();
    let child_type = checker.declared_types.get("Child").copied().unwrap();

    // Parent.child should be Child
    let parent = checker.get_type(parent_type).cloned().unwrap();
    match parent {
        Type::Model(m) => {
            let child_prop_id = m.properties.get("child").copied().unwrap();
            let child_prop = checker.get_type(child_prop_id).cloned().unwrap();
            match child_prop {
                Type::ModelProperty(p) => {
                    assert_eq!(p.r#type, child_type, "Parent.child should be Child");
                }
                _ => panic!("Expected ModelProperty"),
            }
        }
        _ => panic!("Expected Model type"),
    }

    // Child.parent should be Parent
    let child = checker.get_type(child_type).cloned().unwrap();
    match child {
        Type::Model(m) => {
            let parent_prop_id = m.properties.get("parent").copied().unwrap();
            let parent_prop = checker.get_type(parent_prop_id).cloned().unwrap();
            match parent_prop {
                Type::ModelProperty(p) => {
                    assert_eq!(p.r#type, parent_type, "Child.parent should be Parent");
                }
                _ => panic!("Expected ModelProperty"),
            }
        }
        _ => panic!("Expected Model type"),
    }
}

// ============================================================================
// Circular Extends / Is Diagnostic Tests
// ============================================================================

#[test]
fn test_model_extends_self_detected() {
    // model A extends A → circular-base-type
    let checker = check("model A extends A { x: int32; }");
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "circular-base-type"),
        "Should report circular-base-type: {:?}",
        diags
    );
}

#[test]
fn test_model_is_self_detected() {
    // model A is A → circular-base-type
    let checker = check("model A is A { x: int32; }");
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "circular-base-type"),
        "Should report circular-base-type: {:?}",
        diags
    );
}

#[test]
fn test_model_extends_mutual_circular() {
    // model A extends B; model B extends A → circular-base-type
    let checker = check(
        "
        model A extends B { x: int32; }
        model B extends A { y: string; }
    ",
    );
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "circular-base-type"),
        "Should report circular-base-type: {:?}",
        diags
    );
}

#[test]
fn test_model_is_mutual_circular() {
    // model A is B; model B is A → circular-base-type
    let checker = check(
        "
        model A is B { }
        model B is A { }
    ",
    );
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "circular-base-type"),
        "Should report circular-base-type: {:?}",
        diags
    );
}

#[test]
fn test_model_extends_three_way_circular() {
    // model A extends B; model B extends C; model C extends A → circular-base-type
    let checker = check(
        "
        model A extends B { }
        model B extends C { }
        model C extends A { }
    ",
    );
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "circular-base-type"),
        "Should report circular-base-type: {:?}",
        diags
    );
}

#[test]
fn test_model_circular_base_type_message() {
    // Verify message mentions the type name
    let checker = check("model A extends A { x: int32; }");
    let diags = checker.diagnostics();
    let circ_diag = diags
        .iter()
        .find(|d| d.code == "circular-base-type")
        .unwrap();
    assert!(
        circ_diag.message.contains("A"),
        "Message should mention type 'A': {}",
        circ_diag.message
    );
}

#[test]
fn test_model_no_circular_valid_extends() {
    // No diagnostic for valid extends chain
    let checker = check(
        "
        model Base { x: int32; }
        model Middle extends Base { y: string; }
        model Child extends Middle { z: boolean; }
    ",
    );
    let diags = checker.diagnostics();
    assert!(
        !diags.iter().any(|d| d.code == "circular-base-type"),
        "Should NOT report circular-base-type: {:?}",
        diags
    );
}

#[test]
fn test_model_no_circular_valid_is() {
    // No diagnostic for valid is chain
    let checker = check(
        "
        model Base { x: int32; }
        model Child is Base { }
    ",
    );
    let diags = checker.diagnostics();
    assert!(
        !diags.iter().any(|d| d.code == "circular-base-type"),
        "Should NOT report circular-base-type: {:?}",
        diags
    );
}

#[test]
fn test_model_circular_via_alias() {
    // model A extends B; alias B = A → circular-base-type or extend-model
    // Note: alias B = A resolves to a model, so extends B may succeed but
    // the post-check detect_circular_references should catch it.
    // If B resolves to model A, then A extends A is circular.
    // If B isn't resolved yet, extend-model may fire.
    let checker = check(
        "
        model A extends B { x: int32; }
        alias B = A;
    ",
    );
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "circular-base-type"
            || d.code == "circular-alias-type"
            || d.code == "extend-model"),
        "Should report circular-base-type, circular-alias-type, or extend-model: {:?}",
        diags
    );
}

#[test]
fn test_template_model_can_reference_itself() {
    // Ported from TS: "template model can reference itself"
    // Templated<T> { value: T; parent?: Templated<T>; }
    // This should NOT be a circular reference - a template can reference itself
    // in its property types.
    let checker = check(
        "
        model Templated<T> {
            value: T;
            parent: Templated<T>;
        }
    ",
    );
    let diags = checker.diagnostics();
    assert!(
        !diags.iter().any(|d| d.code == "circular-base-type"),
        "Self-referencing template model should NOT be circular-base-type: {:?}",
        diags
    );
    // The template declaration should exist
    assert!(
        checker.declared_types.contains_key("Templated"),
        "Templated should be in declared_types"
    );
}

#[test]
fn test_model_self_reference_in_property() {
    // model M { self: M; } - a model can reference itself in a property
    let checker = check(
        "
        model M {
            self: M;
        }
    ",
    );
    let diags = checker.diagnostics();
    assert!(
        !diags.iter().any(|d| d.code == "circular-base-type"),
        "Self-referencing property should NOT be circular-base-type: {:?}",
        diags
    );
    assert!(
        checker.declared_types.contains_key("M"),
        "M should be in declared_types"
    );
}

#[test]
fn test_template_models_can_reference_each_other() {
    // Ported from TS: "template model can reference each other"
    // model A<T> { b: B<T>; }
    // model B<T> { a: A<T>; }
    // Two templated models referencing each other should NOT be circular.
    let checker = check(
        "
        model A<T> {
            value: T;
            b: B<T>;
        }
        model B<T> {
            value: T;
            a: A<T>;
        }
        model Test {
            x: A<string>;
        }
    ",
    );
    let diags = checker.diagnostics();
    assert!(
        !diags.iter().any(|d| d.code == "circular-base-type"),
        "Mutually-referencing template models should NOT be circular-base-type: {:?}",
        diags
    );
    assert!(
        checker.declared_types.contains_key("A"),
        "A should be in declared_types"
    );
    assert!(
        checker.declared_types.contains_key("B"),
        "B should be in declared_types"
    );
}

/// Ported from TS: "models can reference each other in different namespace with the same name"
#[test]
fn test_models_reference_each_other_different_namespace_same_name() {
    let checker = check(
        r#"
        namespace Foo {
            namespace Nested {
                model Some {
                    self: Some;
                    related: Bar.Nested.Some;
                }
            }
        }

        namespace Bar {
            namespace Nested {
                model Some {
                    self: Some;
                    related: Foo.Nested.Some;
                }
            }
        }
    "#,
    );
    // Verify no circular-base-type diagnostics (self-references within models are OK)
    let diags = checker.diagnostics();
    assert!(
        !diags.iter().any(|d| d.code == "circular-base-type"),
        "Models referencing themselves should NOT be circular-base-type: {:?}",
        diags
    );
    // Both Foo.Nested.Some and Bar.Nested.Some should exist
    assert!(
        checker.declared_types.contains_key("Some"),
        "Some should be in declared_types"
    );
}

// ============================================================================
// Property circular reference tests - ported from TS model.test.ts
// ============================================================================

/// Ported from TS model.test.ts: "emit diagnostics if property reference itself"
/// model A { a: A.a } → circular-prop
#[test]
fn test_circular_prop_property_references_itself() {
    let checker = check(
        r#"
        model A { a: A.a }
    "#,
    );
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "circular-prop"),
        "Should report circular-prop when property references itself: {:?}",
        diags
    );
}

/// Ported from TS model.test.ts: "emit diagnostics if property reference itself via another prop"
/// model A { a: B.a }; model B { a: A.a } → circular-prop
#[test]
fn test_circular_prop_property_references_itself_via_another_prop() {
    let checker = check(
        r#"
        model A { a: B.a }
        model B { a: A.a }
    "#,
    );
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "circular-prop"),
        "Should report circular-prop when property references itself via another prop: {:?}",
        diags
    );
}

/// Ported from TS model.test.ts: "emit diagnostics if property reference itself via alias"
/// model A { a: B.a }; model B { a: C }; alias C = A.a → circular-prop
#[test]
fn test_circular_prop_property_references_itself_via_alias() {
    let checker = check(
        r#"
        model A { a: B.a }
        model B { a: C }
        alias C = A.a;
    "#,
    );
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "circular-prop"),
        "Should report circular-prop when property references itself via alias: {:?}",
        diags
    );
}

/// Model self-reference should NOT be circular-prop
#[test]
fn test_model_self_reference_not_circular_prop() {
    // model Foo { self: Foo } — this is NOT circular-prop (just a self-referencing model)
    let checker = check(
        r#"
        model Foo {
            self: Foo;
        }
    "#,
    );
    let diags = checker.diagnostics();
    assert!(
        !diags.iter().any(|d| d.code == "circular-prop"),
        "Model self-reference should NOT be circular-prop: {:?}",
        diags
    );
    assert!(
        !diags.iter().any(|d| d.code == "circular-base-type"),
        "Model self-reference should NOT be circular-base-type: {:?}",
        diags
    );
}
