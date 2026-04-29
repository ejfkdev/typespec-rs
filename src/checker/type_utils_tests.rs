//! Checker Type-Utils Tests
//!
//! Ported from TypeSpec compiler/test/checker/type-utils.test.ts

use crate::checker::Type;
use crate::checker::test_utils::check;
use crate::checker::type_utils::{
    is_declared_in_namespace, is_template_declaration, is_template_declaration_or_instance,
    is_template_instance,
};

// ============================================================================
// Template utils tests
// ============================================================================

#[test]
fn test_model_is_template_declaration() {
    // Ported from TS: "check model is a template declaration"
    let checker = check("model Foo<T> { t: T }; model Bar { foo: Foo<string> }");
    let foo_id = checker.declared_types.get("Foo").copied().unwrap();
    let foo_type = checker.get_type(foo_id).cloned().unwrap();

    assert!(
        is_template_declaration_or_instance(&checker, foo_id),
        "Foo<T> should be a template declaration or instance"
    );
    assert!(
        is_template_declaration(&checker, foo_id),
        "Foo<T> should BE a template declaration"
    );
    assert!(
        !is_template_instance(&foo_type),
        "Foo<T> should NOT be a template instance"
    );
}

#[test]
fn test_model_reference_is_template_instance() {
    // Ported from TS: "check model reference is a template instance"
    // Foo<string> should be a template instance, not a template declaration
    let checker = check("model Foo<T> { t: T }; model Bar { foo: Foo<string> }");
    let bar_id = checker.declared_types.get("Bar").copied().unwrap();
    let bar_type = checker.get_type(bar_id).cloned().unwrap();

    match bar_type {
        Type::Model(m) => {
            let foo_prop_id = m.properties.get("foo").copied().unwrap();
            let foo_prop = checker.get_type(foo_prop_id).cloned().unwrap();
            match foo_prop {
                Type::ModelProperty(p) => {
                    let foo_instance_id = p.r#type;
                    let foo_instance = checker.get_type(foo_instance_id).cloned().unwrap();

                    assert!(
                        is_template_declaration_or_instance(&checker, foo_instance_id),
                        "Foo<string> should be a template declaration or instance"
                    );
                    assert!(
                        is_template_instance(&foo_instance),
                        "Foo<string> should BE a template instance"
                    );
                    assert!(
                        !is_template_declaration(&checker, foo_instance_id),
                        "Foo<string> should NOT be a template declaration"
                    );
                }
                _ => panic!("Expected ModelProperty for foo"),
            }
        }
        _ => panic!("Expected Model type for Bar"),
    }
}

#[test]
fn test_model_expression_in_template_instance_is_instance() {
    // Ported from TS: "check model expression inside a template instance is also a template instance"
    // model Foo<T> { a: { b: T } } — the inner { b: T } should be a template instance
    let checker = check("model Foo<T> { a: { b: T } }; model Bar { foo: Foo<string> }");
    let bar_id = checker.declared_types.get("Bar").copied().unwrap();
    let bar_type = checker.get_type(bar_id).cloned().unwrap();

    match bar_type {
        Type::Model(m) => {
            let foo_prop_id = m.properties.get("foo").copied().unwrap();
            let foo_prop = checker.get_type(foo_prop_id).cloned().unwrap();
            match foo_prop {
                Type::ModelProperty(p) => {
                    let foo_instance_id = p.r#type;
                    let foo_instance = checker.get_type(foo_instance_id).cloned().unwrap();
                    match foo_instance {
                        Type::Model(foo_model) => {
                            let a_prop_id = foo_model.properties.get("a").copied().unwrap();
                            let a_prop = checker.get_type(a_prop_id).cloned().unwrap();
                            match a_prop {
                                Type::ModelProperty(ap) => {
                                    let inner_id = ap.r#type;
                                    let inner = checker.get_type(inner_id).cloned().unwrap();
                                    assert!(
                                        is_template_instance(&inner),
                                        "Inner model expression should BE a template instance"
                                    );
                                    assert!(
                                        !is_template_declaration(&checker, inner_id),
                                        "Inner model expression should NOT be a template declaration"
                                    );
                                }
                                _ => panic!("Expected ModelProperty for a"),
                            }
                        }
                        _ => panic!("Expected Model for Foo<string>"),
                    }
                }
                _ => panic!("Expected ModelProperty for foo"),
            }
        }
        _ => panic!("Expected Model type for Bar"),
    }
}

#[test]
fn test_union_expression_in_template_instance_is_instance() {
    // Ported from TS: "check union expression inside a template instance is also a template instance"
    // model Foo<T> { a: int32 | T } — the inner int32 | T should be a template instance
    let checker = check("model Foo<T> { a: int32 | T }; model Bar { foo: Foo<string> }");
    let bar_id = checker.declared_types.get("Bar").copied().unwrap();
    let bar_type = checker.get_type(bar_id).cloned().unwrap();

    match bar_type {
        Type::Model(m) => {
            let foo_prop_id = m.properties.get("foo").copied().unwrap();
            let foo_prop = checker.get_type(foo_prop_id).cloned().unwrap();
            match foo_prop {
                Type::ModelProperty(p) => {
                    let foo_instance_id = p.r#type;
                    let foo_instance = checker.get_type(foo_instance_id).cloned().unwrap();
                    match foo_instance {
                        Type::Model(foo_model) => {
                            let a_prop_id = foo_model.properties.get("a").copied().unwrap();
                            let a_prop = checker.get_type(a_prop_id).cloned().unwrap();
                            match a_prop {
                                Type::ModelProperty(ap) => {
                                    let inner_id = ap.r#type;
                                    let inner = checker.get_type(inner_id).cloned().unwrap();
                                    assert!(
                                        is_template_instance(&inner),
                                        "Inner union expression should BE a template instance"
                                    );
                                    assert!(
                                        !is_template_declaration(&checker, inner_id),
                                        "Inner union expression should NOT be a template declaration"
                                    );
                                }
                                _ => panic!("Expected ModelProperty for a"),
                            }
                        }
                        _ => panic!("Expected Model for Foo<string>"),
                    }
                }
                _ => panic!("Expected ModelProperty for foo"),
            }
        }
        _ => panic!("Expected Model type for Bar"),
    }
}

// ============================================================================
// Definition utils tests — isDeclaredInNamespace
// ============================================================================

#[test]
fn test_is_declared_in_namespace_recursive() {
    // Ported from TS: "checks if a type is defined in a particular namespace or its child namespaces"
    let checker = check(
        r#"
        namespace Alpha {
            namespace SubAlpha {
                model FooModel {}
                enum FooEnum { A }
                op fooOp(): unknown;
                namespace FooNamespace {}
                interface FooInterface {
                    op barOp(): unknown;
                }
            }
        }

        namespace Beta {}
    "#,
    );

    let alpha_id = checker.declared_types.get("Alpha").copied().unwrap();
    let sub_alpha_id = checker.declared_types.get("SubAlpha").copied().unwrap();
    let beta_id = checker.declared_types.get("Beta").copied().unwrap();
    let foo_model_id = checker.declared_types.get("FooModel").copied().unwrap();
    let foo_enum_id = checker.declared_types.get("FooEnum").copied().unwrap();
    let foo_namespace_id = checker.declared_types.get("FooNamespace").copied().unwrap();

    // All types in SubAlpha should be found under Alpha (recursive) and SubAlpha
    let candidates = [
        ("FooModel", foo_model_id),
        ("FooEnum", foo_enum_id),
        ("FooNamespace", foo_namespace_id),
    ];

    for (name, type_id) in candidates {
        assert!(
            is_declared_in_namespace(&checker.type_store, type_id, alpha_id, true),
            "{} should be found recursively under Alpha",
            name
        );
        assert!(
            is_declared_in_namespace(&checker.type_store, type_id, sub_alpha_id, true),
            "{} should be found under SubAlpha",
            name
        );
        assert!(
            !is_declared_in_namespace(&checker.type_store, type_id, alpha_id, false),
            "{} should NOT be found when recursive: false",
            name
        );
        assert!(
            !is_declared_in_namespace(&checker.type_store, type_id, beta_id, false),
            "{} should NOT be found in namespace Beta",
            name
        );
    }
}
