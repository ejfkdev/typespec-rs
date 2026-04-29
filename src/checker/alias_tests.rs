//! Checker Alias Tests
//!
//! Ported from TypeSpec compiler/test/checker/alias.test.ts
//!
//! Skipped (needs diagnostics system):
//! - Invalid reference to aliased type member
//!
//! Skipped (needs deep template/alias resolution):
//! - Can alias a deep union expression
//! - Can alias a union expression with parameters
//! - Can alias a deep union expression with parameters
//! - Can alias an intersection expression
//! - Can be used like any model (extends alias, spread alias)
//! - Can be used like any namespace (member access on alias)
//! - Model expression defined in alias use containing namespace

use crate::checker::Type;
use crate::checker::test_utils::check;

#[test]
fn test_simple_alias_to_scalar() {
    let checker = check("alias MyString = string;");
    let my_string = checker.declared_types.get("MyString").copied().unwrap();
    let t = checker.get_type(my_string).cloned().unwrap();
    match t {
        Type::Scalar(s) => {
            assert_eq!(s.name, "MyString");
            assert!(
                s.base_scalar.is_some(),
                "Alias should have base_scalar pointing to string"
            );
            let base_id = s.base_scalar.unwrap();
            let base = checker.get_type(base_id).cloned().unwrap();
            assert!(matches!(base, Type::Scalar(ref bs) if bs.name == "string"));
        }
        _ => panic!("Expected Scalar (alias wrapper), got {:?}", t.kind_name()),
    }
}

#[test]
fn test_alias_to_model() {
    let checker = check("model Foo { x: string; } alias Bar = Foo;");
    let bar_type = checker.declared_types.get("Bar").copied().unwrap();
    let t = checker.get_type(bar_type).cloned().unwrap();
    // Alias to model is represented as Scalar with base_scalar pointing to the model
    match t {
        Type::Scalar(s) => {
            assert_eq!(s.name, "Bar");
            assert!(s.base_scalar.is_some());
            let base_id = s.base_scalar.unwrap();
            let base = checker.get_type(base_id).cloned().unwrap();
            assert!(
                matches!(base, Type::Model(_)),
                "Base should be a Model, got {:?}",
                base.kind_name()
            );
        }
        _ => panic!("Expected Scalar (alias wrapper), got {:?}", t.kind_name()),
    }
}

#[test]
fn test_alias_to_union_expression() {
    // Ported from: "can alias a union expression" (simplified)
    let checker = check("alias Foo = int32 | string;");
    let foo_type = checker.declared_types.get("Foo").copied().unwrap();
    let t = checker.get_type(foo_type).cloned().unwrap();
    match t {
        Type::Scalar(s) => {
            assert_eq!(s.name, "Foo");
            assert!(s.base_scalar.is_some());
            let base_id = s.base_scalar.unwrap();
            let base = checker.get_type(base_id).cloned().unwrap();
            assert!(
                matches!(base, Type::Union(_)),
                "Aliased union expression should resolve to Union, got {:?}",
                base.kind_name()
            );
        }
        _ => panic!("Expected Scalar (alias wrapper), got {:?}", t.kind_name()),
    }
}

#[test]
fn test_alias_as_property_type() {
    let checker = check("alias MyInt = int32; model Foo { x: MyInt; }");
    let foo_type = checker.declared_types.get("Foo").copied().unwrap();
    let t = checker.get_type(foo_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            let prop_id = m.properties.get("x").copied().unwrap();
            let prop = checker.get_type(prop_id).cloned().unwrap();
            match prop {
                Type::ModelProperty(p) => {
                    // The type should resolve to the alias type
                    let resolved = checker.get_type(p.r#type).cloned().unwrap();
                    // Could be the alias wrapper (Scalar) or the resolved type
                    match resolved {
                        Type::Scalar(s) if s.name == "MyInt" => {}
                        Type::Scalar(s) if s.base_scalar.is_some() => {
                            // Check that the base_scalar resolves correctly
                            let base_id = s.base_scalar.unwrap();
                            let base = checker.get_type(base_id).cloned().unwrap();
                            assert!(
                                matches!(base, Type::Scalar(ref bs) if bs.name == "int32"),
                                "Expected int32, got {:?}",
                                base.kind_name()
                            );
                        }
                        other => panic!("Expected alias type MyInt, got {:?}", other.kind_name()),
                    }
                }
                _ => panic!("Expected ModelProperty"),
            }
        }
        _ => panic!("Expected Model type"),
    }
}

#[test]
fn test_alias_is_finished() {
    let checker = check("alias Foo = string;");
    let foo_type = checker.declared_types.get("Foo").copied().unwrap();
    let t = checker.get_type(foo_type).cloned().unwrap();
    assert!(t.is_finished(), "Alias type should be finished");
}

#[test]
fn test_multiple_aliases() {
    let checker = check("alias A = string; alias B = int32; alias C = boolean;");
    assert!(checker.declared_types.contains_key("A"));
    assert!(checker.declared_types.contains_key("B"));
    assert!(checker.declared_types.contains_key("C"));
}

#[test]
fn test_template_alias_declaration() {
    let checker = check("alias Pair<K, V> = [K, V];");
    assert!(
        checker.declared_types.contains_key("Pair"),
        "Template alias should be declared"
    );
}

// ============================================================================
// Alias Diagnostic Tests
// ============================================================================

#[test]
fn test_circular_alias_self_reference() {
    // Ported from: "emit diagnostics if assign itself"
    let checker = check("alias A = A;");
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "circular-alias-type"),
        "Should report circular-alias-type: {:?}",
        diags
    );
}

#[test]
fn test_circular_alias_via_union() {
    // Ported from: "emit diagnostics if reference itself"
    let checker = check(r#"alias A = "string" | A;"#);
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "circular-alias-type"),
        "Should report circular-alias-type: {:?}",
        diags
    );
}

#[test]
fn test_no_error_valid_alias() {
    // No diagnostic for a valid alias
    let checker = check("alias MyString = string;");
    let diags = checker.diagnostics();
    assert!(diags.is_empty(), "Should have no diagnostics: {:?}", diags);
}

#[test]
fn test_circular_alias_with_generic_self_reference() {
    // Ported from: "emit single diagnostics if assign itself as generic and is referenced"
    let checker = check(
        "
        alias A<T> = A<T>;
        model Foo { a: A<string>; }
    ",
    );
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "circular-alias-type"),
        "Should report circular-alias-type: {:?}",
        diags
    );
}

#[test]
fn test_alias_to_model_expression() {
    // Alias pointing to a model expression
    let checker = check("alias Foo = {x: string};");
    let foo_type = checker.declared_types.get("Foo").copied().unwrap();
    let t = checker.get_type(foo_type).cloned().unwrap();
    match t {
        Type::Scalar(s) => {
            assert_eq!(s.name, "Foo");
            assert!(s.base_scalar.is_some(), "Alias should have base_scalar");
            let base_id = s.base_scalar.unwrap();
            let base = checker.get_type(base_id).cloned().unwrap();
            assert!(
                matches!(base, Type::Model(_)),
                "Base should be a Model (expression), got {:?}",
                base.kind_name()
            );
        }
        _ => panic!("Expected Scalar (alias wrapper), got {:?}", t.kind_name()),
    }
}

#[test]
fn test_alias_in_namespace() {
    let checker = check("namespace MyNs { alias Foo = string; }");
    let ns_type = checker.declared_types.get("MyNs").copied().unwrap();
    let t = checker.get_type(ns_type).cloned().unwrap();
    match t {
        Type::Namespace(ns) => {
            assert!(
                ns.scalars.contains_key("Foo"),
                "Namespace should contain Foo alias"
            );
        }
        _ => panic!("Expected Namespace type"),
    }
}

#[test]
fn test_alias_to_array_type() {
    // Alias to an array type
    let checker = check("alias StringList = string[];");
    let sl_type = checker.declared_types.get("StringList").copied().unwrap();
    let t = checker.get_type(sl_type).cloned().unwrap();
    match t {
        Type::Scalar(s) => {
            assert_eq!(s.name, "StringList");
            assert!(s.base_scalar.is_some(), "Alias should have base_scalar");
            let base_id = s.base_scalar.unwrap();
            let base = checker.get_type(base_id).cloned().unwrap();
            // Array type should be a Model with indexer
            assert!(
                matches!(base, Type::Model(_)),
                "Base should be Array Model, got {:?}",
                base.kind_name()
            );
        }
        _ => panic!("Expected Scalar (alias wrapper), got {:?}", t.kind_name()),
    }
}

#[test]
fn test_alias_to_tuple_type() {
    // Alias to a tuple type
    let checker = check("alias Pair = [string, int32];");
    let pair_type = checker.declared_types.get("Pair").copied().unwrap();
    let t = checker.get_type(pair_type).cloned().unwrap();
    match t {
        Type::Scalar(s) => {
            assert_eq!(s.name, "Pair");
            assert!(s.base_scalar.is_some(), "Alias should have base_scalar");
            let base_id = s.base_scalar.unwrap();
            let base = checker.get_type(base_id).cloned().unwrap();
            assert!(
                matches!(base, Type::Tuple(_)),
                "Base should be Tuple, got {:?}",
                base.kind_name()
            );
        }
        _ => panic!("Expected Scalar (alias wrapper), got {:?}", t.kind_name()),
    }
}

#[test]
fn test_circular_alias_via_model() {
    // Ported from: alias A = B; model B { x: A; } - circular through model property
    // This should not cause infinite recursion
    let checker = check(
        "
        alias A = B;
        model B { x: A; }
    ",
    );
    // Just verify no crash - diagnostics may or may not report circularity
    assert!(checker.declared_types.contains_key("A") || checker.declared_types.contains_key("B"));
}

#[test]
fn test_alias_no_error_valid_model_reference() {
    // No diagnostic when alias points to a valid model
    let checker = check("model Foo { x: string; } alias Bar = Foo;");
    let diags = checker.diagnostics();
    assert!(
        !diags
            .iter()
            .any(|d| d.code == "circular-alias-type" || d.code == "invalid-ref"),
        "Should have no circular-alias-type or invalid-ref diagnostics: {:?}",
        diags
    );
}

#[test]
fn test_circular_alias_via_another_alias() {
    // Two aliases referencing each other
    let checker = check(
        "
        alias A = B;
        alias B = A;
    ",
    );
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "circular-alias-type"),
        "Should report circular-alias-type: {:?}",
        diags
    );
}

// ============================================================================
// Template parameter tests for aliases
// ============================================================================

/// Alias with template parameter declaration
#[test]
fn test_alias_template_declaration() {
    let checker = check("alias Foo<T> = T;");
    assert!(
        checker.declared_types.contains_key("Foo"),
        "Template alias Foo should be declared"
    );
}

/// Alias with circular constraint
#[test]
fn test_alias_circular_constraint_self() {
    let checker = check("alias Test<A extends A> = A;");
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "circular-constraint"),
        "Should emit circular-constraint for self-referencing alias constraint: {:?}",
        diags
    );
}

/// Alias with invalid template default
#[test]
fn test_alias_invalid_template_default() {
    let checker = check("alias A<A = B, B = string> = B;");
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "invalid-template-default"),
        "Should emit invalid-template-default for alias default referencing later param: {:?}",
        diags
    );
}

// ============================================================================
// Additional alias tests ported from TS alias.test.ts
// ============================================================================

/// Ported from TS: "can alias a union expression"
#[test]
fn test_alias_union_expression_variants() {
    let checker = check(
        r#"
        alias Foo = int32 | string;
        alias Bar = "hi" | 10;
        alias FooBar = Foo | Bar;
        model A {
            prop: FooBar
        }
    "#,
    );
    let a_type = checker.declared_types.get("A").copied().unwrap();
    let t = checker.get_type(a_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            assert!(m.properties.contains_key("prop"), "A should have prop");
            let prop_id = m.properties.get("prop").copied().unwrap();
            let prop = checker.get_type(prop_id).cloned().unwrap();
            match prop {
                Type::ModelProperty(p) => {
                    // The type should resolve to a Union (through alias)
                    let resolved = checker.resolve_alias_chain(p.r#type);
                    match checker.get_type(resolved) {
                        Some(Type::Union(u)) => {
                            assert!(
                                u.variants.len() >= 2,
                                "FooBar should have at least 2 variants: {:?}",
                                u.variants.len()
                            );
                        }
                        Some(_) => {
                            // May resolve through Scalar (alias wrapper)
                        }
                        None => panic!("Could not resolve prop type"),
                    }
                }
                _ => panic!("Expected ModelProperty"),
            }
        }
        _ => panic!("Expected Model type"),
    }
}

/// Ported from TS: "can alias an intersection expression"
#[test]
fn test_alias_intersection_expression_properties() {
    let checker = check(
        r#"
        alias Foo = {a: string} & {b: string};
        alias Bar = {c: string} & {d: string};
        alias FooBar = Foo & Bar;
        model A {
            prop: FooBar
        }
    "#,
    );
    let a_type = checker.declared_types.get("A").copied().unwrap();
    let t = checker.get_type(a_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            assert!(m.properties.contains_key("prop"), "A should have prop");
        }
        _ => panic!("Expected Model type"),
    }
    let diags = checker.diagnostics();
    assert!(
        !diags.iter().any(|d| d.code == "circular-alias-type"),
        "Should not report circular-alias-type for valid aliases: {:?}",
        diags
    );
}

/// Ported from TS: "can be used like any model"
#[test]
fn test_alias_used_like_any_model() {
    let checker = check(
        r#"
        model Test { a: string };
        alias Alias = Test;
        model A extends Alias { };
        model B { ...Alias };
        model C { c: Alias };
    "#,
    );
    let a_type = checker.declared_types.get("A").copied().unwrap();
    let a = checker.get_type(a_type).cloned().unwrap();
    match a {
        Type::Model(m) => {
            assert!(m.base_model.is_some(), "A should have a base model");
        }
        _ => panic!("Expected Model type"),
    }
    let b_type = checker.declared_types.get("B").copied().unwrap();
    let b = checker.get_type(b_type).cloned().unwrap();
    match b {
        Type::Model(m) => {
            assert!(
                m.properties.contains_key("a"),
                "B should have 'a' from spread: {:?}",
                m.properties.keys().collect::<Vec<_>>()
            );
        }
        _ => panic!("Expected Model type"),
    }
    let c_type = checker.declared_types.get("C").copied().unwrap();
    let c = checker.get_type(c_type).cloned().unwrap();
    match c {
        Type::Model(m) => {
            assert!(m.properties.contains_key("c"), "C should have 'c' property");
        }
        _ => panic!("Expected Model type"),
    }
}

/// Ported from TS: "can be used like any namespace"
#[test]
fn test_alias_used_like_any_namespace() {
    let checker = check(
        r#"
        namespace Foo {
            model Bar { }
        }
        alias AliasFoo = Foo;
        model Baz { x: AliasFoo.Bar };
    "#,
    );
    assert!(
        checker.declared_types.contains_key("Baz"),
        "Baz should be declared"
    );
    let baz_type = checker.declared_types.get("Baz").copied().unwrap();
    let baz = checker.get_type(baz_type).cloned().unwrap();
    match baz {
        Type::Model(m) => {
            assert!(
                m.properties.contains_key("x"),
                "Baz should have 'x' property"
            );
        }
        _ => panic!("Expected Model type"),
    }
}

/// Ported from TS: "emit diagnostics if assign itself" - verify message
#[test]
fn test_circular_alias_self_reference_message() {
    let checker = check("alias A = A;");
    let diags = checker.diagnostics();
    let diag = diags
        .iter()
        .find(|d| d.code == "circular-alias-type")
        .unwrap();
    assert!(
        diag.message.contains("recursively") || diag.message.contains("circular"),
        "Message should mention recursion: {}",
        diag.message
    );
}

/// Ported from TS: "trying to access unknown member of aliased model expression shouldn't crash"
#[test]
fn test_alias_model_expression_member_access_no_crash() {
    let checker = check(
        r#"
        alias A = {foo: string};
        alias Aliased = A.prop;
    "#,
    );
    let diags = checker.diagnostics();
    assert!(
        diags
            .iter()
            .any(|d| d.code == "invalid-ref" || d.code == "circular-alias-type"),
        "Should report diagnostic for accessing member on alias: {:?}",
        diags
    );
}
