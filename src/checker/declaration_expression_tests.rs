//! Declaration expressions tests
//!
//! Ported from TypeSpec compiler tests for microsoft/typespec#11019
//! ("allow declarations to be used as expressions").

use crate::checker::{Checker, Type, TypeId};
use crate::diagnostics::CompilerOptions;
use crate::parser::parse;

/// Parse and check with the `declaration-expressions` feature enabled or not.
fn check_decl_expr(source: &str, feature_enabled: bool) -> Checker {
    let result = parse(source);
    let options = CompilerOptions {
        features: if feature_enabled {
            vec![
                "declaration-expressions".to_string(),
                "union-extends".to_string(),
            ]
        } else {
            Vec::new()
        },
        ..CompilerOptions::default()
    };
    let mut checker = Checker::with_options(options);
    checker.set_parse_result(result.root_id, result.builder);
    checker.check_program();
    checker
}

fn has_code(checker: &Checker, code: &str) -> bool {
    checker.diagnostics().iter().any(|d| d.code == code)
}

/// Ported from TS: declaration expressions require the feature.
#[test]
fn test_declaration_expression_requires_feature() {
    let checker = check_decl_expr("alias Foo = model { x: string };", false);
    assert!(
        has_code(&checker, "declaration-expression-disabled"),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
}

/// Ported from TS: "anonymous model declaration expression"
#[test]
fn test_anonymous_model_declaration_expression() {
    let checker = check_decl_expr("alias Foo = model { x: string };", true);
    assert!(
        checker.diagnostics().is_empty(),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
}

/// Ported from TS: "named declaration expression is not registered"
#[test]
fn test_named_declaration_expression_not_registered() {
    let checker = check_decl_expr("alias Foo = model Inner { x: string };", true);
    assert!(
        checker.diagnostics().is_empty(),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
    assert!(
        checker.get_type_by_name("Inner").is_none(),
        "named declaration expression must not be registered in the namespace"
    );
    assert!(checker.get_type_by_name("Foo").is_some());
}

/// Ported from TS: all four declaration kinds work in expression position.
#[test]
fn test_declaration_expressions_in_property_types() {
    let checker = check_decl_expr(
        r#"
        model Bar {
            status: enum { active, inactive };
            unit: scalar extends string;
            inner: model { x: string };
            choice: union { a: "x" };
        }
        "#,
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
    // The property types are the inline declarations.
    let bar = checker.get_type_by_name("Bar").expect("Bar");
    if let Some(Type::Model(m)) = checker.get_type(bar) {
        assert_eq!(m.properties.len(), 4);
        let status = *m.properties.get("status").expect("status");
        let status_type = match checker.get_type(status) {
            Some(Type::ModelProperty(p)) => p.r#type,
            _ => panic!("expected property"),
        };
        assert!(
            matches!(checker.get_type(status_type), Some(Type::Enum(_))),
            "status should be an inline enum"
        );
    }
}

/// Ported from TS: a declaration expression cannot have template parameters.
#[test]
fn test_templated_declaration_expression_rejected() {
    let checker = check_decl_expr("alias Foo = model Box<T> { x: T };", true);
    assert!(
        has_code(&checker, "templated-declaration-in-expression"),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
}

/// Union declaration expressions are marked as expressions (upstream sets
/// `expression: true` on the type).
#[test]
fn test_union_declaration_expression_flag() {
    let checker = check_decl_expr(r#"alias U = union { a: "x" };"#, true);
    assert!(
        checker.diagnostics().is_empty(),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
    let u = checker.get_type_by_name("U").expect("U alias");
    let resolved = checker.resolve_alias_chain(u);
    assert!(
        matches!(checker.get_type(resolved), Some(Type::Union(u)) if u.expression),
        "union declaration expression should have expression = true"
    );
}

/// Scalar declaration expression with `extends`.
#[test]
fn test_scalar_declaration_expression_extends() {
    let checker = check_decl_expr(
        r#"
        model M { s: scalar extends int32; }
        "#,
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
}

// ============================================================================
// Ported from upstream declaration-expressions.test.ts (full matrix)
// ============================================================================

fn property_type(checker: &Checker, model_name: &str, prop: &str) -> TypeId {
    use crate::checker::types::Type;
    let model_id = checker
        .get_type_by_name(model_name)
        .unwrap_or_else(|| panic!("model {model_name} should exist"));
    match checker.get_type(model_id) {
        Some(Type::Model(m)) => {
            let prop_id = *m
                .properties
                .get(prop)
                .unwrap_or_else(|| panic!("property {prop} should exist"));
            match checker.get_type(prop_id) {
                Some(Type::ModelProperty(p)) => p.r#type,
                other => panic!("expected property, got {other:?}"),
            }
        }
        other => panic!("expected model, got {other:?}"),
    }
}

fn operation_param_type(checker: &Checker, op_name: &str, param: &str) -> TypeId {
    use crate::checker::types::Type;
    let op_id = checker
        .get_type_by_name(op_name)
        .unwrap_or_else(|| panic!("operation {op_name} should exist"));
    let params_id = match checker.get_type(op_id) {
        Some(Type::Operation(o)) => o.parameters.expect("operation parameters"),
        other => panic!("expected operation, got {other:?}"),
    };
    let Some(Type::Model(params)) = checker.get_type(params_id) else {
        panic!("expected parameters model");
    };
    let prop_id = params.properties[param];
    match checker.get_type(prop_id) {
        Some(Type::ModelProperty(p)) => p.r#type,
        other => panic!("expected property, got {other:?}"),
    }
}

fn namespace_of<'a>(checker: &'a Checker, name: &str) -> &'a crate::checker::types::NamespaceType {
    use crate::checker::types::Type;
    let id = checker
        .lookup_type_by_fqn(name)
        .unwrap_or_else(|| panic!("namespace {name} should exist"));
    match checker.get_type(id) {
        Some(Type::Namespace(ns)) => ns,
        other => panic!("expected namespace, got {other:?}"),
    }
}

fn count_code(checker: &Checker, code: &str) -> usize {
    checker
        .diagnostics()
        .iter()
        .filter(|d| d.code == code)
        .count()
}

fn doc_of(checker: &Checker, id: TypeId) -> Option<String> {
    checker.get_type_doc(id)
}

#[test]
fn test_enum_expression_as_property_type() {
    use crate::checker::types::Type;
    let checker = check_decl_expr("model Foo { status: enum { active, inactive }; }", true);
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let ty = property_type(&checker, "Foo", "status");
    match checker.get_type(ty) {
        Some(Type::Enum(e)) => {
            assert_eq!(e.name, "");
            assert!(e.expression);
            assert_eq!(e.members.len(), 2);
            assert!(e.members.contains_key("active"));
            assert!(e.members.contains_key("inactive"));
        }
        other => panic!("expected enum, got {other:?}"),
    }
}

#[test]
fn test_enum_expression_explicit_member_values() {
    use crate::checker::types::Type;
    let checker = check_decl_expr(
        r#"model Foo { status: enum { active: "a", inactive: "i" }; }"#,
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let ty = property_type(&checker, "Foo", "status");
    let Some(Type::Enum(e)) = checker.get_type(ty) else {
        panic!("expected enum");
    };
    assert!(e.expression);
    let member_id = e.members["active"];
    match checker.get_type(member_id) {
        Some(Type::EnumMember(m)) => {
            let value = m.value.expect("member should have a value");
            match checker.get_type(value) {
                Some(Type::String(s)) => assert_eq!(s.value, "a"),
                other => panic!("expected string value, got {other:?}"),
            }
        }
        other => panic!("expected enum member, got {other:?}"),
    }
}

#[test]
fn test_enum_expression_not_registered_in_namespace() {
    let checker = check_decl_expr("namespace Ns; model Foo { status: enum { a, b }; }", true);
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    assert_eq!(namespace_of(&checker, "Ns").enums.len(), 0);
}

#[test]
fn test_union_expression_keyword_form() {
    use crate::checker::types::Type;
    let checker = check_decl_expr("model Foo { value: union { string, int32 }; }", true);
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let ty = property_type(&checker, "Foo", "value");
    match checker.get_type(ty) {
        Some(Type::Union(u)) => {
            assert!(u.expression);
            assert_eq!(u.variants.len(), 2);
        }
        other => panic!("expected union, got {other:?}"),
    }
}

#[test]
fn test_union_expression_named_variants() {
    use crate::checker::types::Type;
    let checker = check_decl_expr(
        r#"model Foo { value: union { foo: string, bar: int32 }; }"#,
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let ty = property_type(&checker, "Foo", "value");
    match checker.get_type(ty) {
        Some(Type::Union(u)) => {
            assert!(u.expression);
            assert!(u.variants.contains_key("foo"));
            assert!(u.variants.contains_key("bar"));
        }
        other => panic!("expected union, got {other:?}"),
    }
}

#[test]
fn test_union_expression_keeps_members_as_pipe_operand() {
    use crate::checker::types::Type;
    let checker = check_decl_expr(
        r#"model Foo {
            value: union { a: "a1", b: "b1" } | union { a: "a2", c: "c1" };
        }"#,
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let ty = property_type(&checker, "Foo", "value");
    let Some(Type::Union(outer)) = checker.get_type(ty) else {
        panic!("expected outer union");
    };
    assert_eq!(
        outer.variants.len(),
        2,
        "nested declaration-expression unions must not be flattened"
    );
    for &variant_id in outer.variants.values() {
        let Some(Type::UnionVariant(v)) = checker.get_type(variant_id) else {
            panic!("expected variant");
        };
        match checker.get_type(v.r#type) {
            Some(Type::Union(inner)) => assert_eq!(inner.variants.len(), 2),
            other => panic!("expected inner union, got {other:?}"),
        }
    }
}

#[test]
fn test_union_expression_not_registered_in_namespace() {
    let checker = check_decl_expr(
        "namespace Ns; model Foo { value: union { string, int32 }; }",
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    assert_eq!(namespace_of(&checker, "Ns").unions.len(), 0);
}

#[test]
fn test_union_extends_anonymous_expression() {
    use crate::checker::types::Type;
    let checker = check_decl_expr(
        r#"
        model PetBase { name: string }
        model Cat extends PetBase { meow: boolean }
        model Foo { pet: union extends PetBase { cat: Cat }; }
        "#,
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let ty = property_type(&checker, "Foo", "pet");
    let Some(Type::Union(u)) = checker.get_type(ty) else {
        panic!("expected union");
    };
    assert!(u.expression);
    let base = u.base_type.expect("base type");
    match checker.get_type(base) {
        Some(Type::Model(m)) => assert_eq!(m.name, "PetBase"),
        other => panic!("expected model base, got {other:?}"),
    }
}

#[test]
fn test_union_extends_named_expression() {
    use crate::checker::types::Type;
    let checker = check_decl_expr(
        r#"model Foo { status: union Status extends string { ok: "ok", ko: "ko" }; }"#,
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let ty = property_type(&checker, "Foo", "status");
    let Some(Type::Union(u)) = checker.get_type(ty) else {
        panic!("expected union");
    };
    assert_eq!(u.name, "Status");
    assert!(u.expression);
    let base = u.base_type.expect("base type");
    match checker.get_type(base) {
        Some(Type::Scalar(s)) => assert_eq!(s.name, "string"),
        other => panic!("expected scalar base, got {other:?}"),
    }
}

#[test]
fn test_union_extends_rejects_unassignable_variant() {
    let checker = check_decl_expr(
        r#"model Foo { value: union extends string { ok: "ok", ko: 123 }; }"#,
        true,
    );
    assert!(
        has_code(&checker, "unassignable"),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_union_extends_requires_feature() {
    let checker = check_decl_expr(
        r#"model Foo { value: union extends string { ok: "ok" }; }"#,
        false,
    );
    assert!(
        has_code(&checker, "union-extends-disabled"),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_scalar_expression_as_property_type() {
    use crate::checker::types::Type;
    let checker = check_decl_expr("model Foo { unit: scalar extends string; }", true);
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let ty = property_type(&checker, "Foo", "unit");
    match checker.get_type(ty) {
        Some(Type::Scalar(s)) => {
            assert_eq!(s.name, "");
            assert!(s.expression);
            let base = s.base_scalar.expect("base scalar");
            match checker.get_type(base) {
                Some(Type::Scalar(b)) => assert_eq!(b.name, "string"),
                other => panic!("expected scalar base, got {other:?}"),
            }
        }
        other => panic!("expected scalar, got {other:?}"),
    }
}

#[test]
fn test_scalar_expression_supports_constructors() {
    use crate::checker::types::Type;
    let checker = check_decl_expr(
        r#"model Foo {
            unit: scalar extends string {
                init fromValue(value: string);
            };
        }"#,
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let ty = property_type(&checker, "Foo", "unit");
    match checker.get_type(ty) {
        Some(Type::Scalar(s)) => {
            assert!(s.expression);
            assert!(
                s.constructors.iter().any(|&id| matches!(
                    checker.get_type(id),
                    Some(Type::ScalarConstructor(c)) if c.name == "fromValue"
                )),
                "constructors: {:?}",
                s.constructors
            );
        }
        other => panic!("expected scalar, got {other:?}"),
    }
}

#[test]
fn test_scalar_expression_not_registered_in_namespace() {
    let checker = check_decl_expr(
        "namespace Ns; model Foo { unit: scalar extends string; }",
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    assert_eq!(namespace_of(&checker, "Ns").scalars.len(), 0);
}

#[test]
fn test_model_expression_keyword_form() {
    use crate::checker::types::Type;
    let checker = check_decl_expr("model Foo { value: model { x: string }; }", true);
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let ty = property_type(&checker, "Foo", "value");
    match checker.get_type(ty) {
        Some(Type::Model(m)) => {
            assert!(m.expression);
            assert_eq!(m.properties.len(), 1);
            assert!(m.properties.contains_key("x"));
        }
        other => panic!("expected model, got {other:?}"),
    }
}

#[test]
fn test_model_expression_supports_spread() {
    use crate::checker::types::Type;
    let checker = check_decl_expr(
        r#"
        model Base { b: string }
        model Foo { value: model { ...Base, x: string }; }
        "#,
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let ty = property_type(&checker, "Foo", "value");
    match checker.get_type(ty) {
        Some(Type::Model(m)) => {
            assert!(m.expression);
            assert!(m.properties.contains_key("b"));
            assert!(m.properties.contains_key("x"));
        }
        other => panic!("expected model, got {other:?}"),
    }
}

#[test]
fn test_model_expression_not_registered_in_namespace() {
    let checker = check_decl_expr(
        "namespace Ns; model Foo { value: model { x: string }; }",
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let ns = namespace_of(&checker, "Ns");
    assert_eq!(ns.models.len(), 1);
    assert!(ns.models.contains_key("Foo"));
}

#[test]
fn test_model_expression_is_clause_without_body() {
    use crate::checker::types::Type;
    let checker = check_decl_expr(
        r#"
        model Base { a: string; b: int32 }
        model Foo { value: model is Base; }
        "#,
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let ty = property_type(&checker, "Foo", "value");
    match checker.get_type(ty) {
        Some(Type::Model(m)) => {
            assert!(m.expression);
            assert!(m.properties.contains_key("a"));
            assert!(m.properties.contains_key("b"));
        }
        other => panic!("expected model, got {other:?}"),
    }
}

#[test]
fn test_model_expression_is_clause_followed_by_properties() {
    use crate::checker::types::Type;
    let checker = check_decl_expr(
        r#"
        model Base { a: string }
        model Foo {
            value: model is Base;
            other: string;
        }
        "#,
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let value_ty = property_type(&checker, "Foo", "value");
    match checker.get_type(value_ty) {
        Some(Type::Model(m)) => assert!(m.properties.contains_key("a")),
        other => panic!("expected model, got {other:?}"),
    }
    assert!(matches!(
        checker.get_type(property_type(&checker, "Foo", "other")),
        Some(Type::Scalar(_))
    ));
}

#[test]
fn test_model_expression_is_clause_with_body() {
    use crate::checker::types::Type;
    let checker = check_decl_expr(
        r#"
        model Base { a: string }
        model Foo { value: model is Base { extra: int32 }; }
        "#,
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let ty = property_type(&checker, "Foo", "value");
    match checker.get_type(ty) {
        Some(Type::Model(m)) => {
            assert!(m.properties.contains_key("a"));
            assert!(m.properties.contains_key("extra"));
        }
        other => panic!("expected model, got {other:?}"),
    }
}

#[test]
fn test_named_declaration_expressions_keep_names() {
    use crate::checker::types::Type;
    let checker = check_decl_expr(
        r#"model Foo {
            m: model Inner { x: string };
            e: enum Color { red };
            s: scalar Celsius extends int32;
            u: union Choice { string, int32 };
        }"#,
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let m = property_type(&checker, "Foo", "m");
    assert!(
        matches!(checker.get_type(m), Some(Type::Model(m)) if m.name == "Inner" && m.expression)
    );
    let e = property_type(&checker, "Foo", "e");
    assert!(
        matches!(checker.get_type(e), Some(Type::Enum(e)) if e.name == "Color" && e.expression)
    );
    let s = property_type(&checker, "Foo", "s");
    assert!(
        matches!(checker.get_type(s), Some(Type::Scalar(s)) if s.name == "Celsius" && s.expression)
    );
    let u = property_type(&checker, "Foo", "u");
    assert!(
        matches!(checker.get_type(u), Some(Type::Union(u)) if u.name == "Choice" && u.expression)
    );
}

#[test]
fn test_named_declaration_expression_not_in_namespace_models() {
    let checker = check_decl_expr(
        "namespace Ns; model Foo { m: model Inner { x: string }; }",
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let ns = namespace_of(&checker, "Ns");
    assert_eq!(ns.models.len(), 1);
    assert!(ns.models.contains_key("Foo"));
    assert!(!ns.models.contains_key("Inner"));
}

#[test]
fn test_named_declaration_expression_cannot_be_referenced() {
    let checker = check_decl_expr(
        r#"
        alias M = model Inner { x: string };
        model Use { y: Inner }
        "#,
        true,
    );
    assert!(
        has_code(&checker, "invalid-ref"),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_statement_declarations_are_not_expressions() {
    use crate::checker::types::Type;
    let checker = check_decl_expr(
        r#"
        model M {}
        enum E { a }
        scalar S extends string;
        union U { string }
        "#,
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let m = checker.get_type_by_name("M").unwrap();
    assert!(matches!(checker.get_type(m), Some(Type::Model(m)) if !m.expression));
    let e = checker.get_type_by_name("E").unwrap();
    assert!(matches!(checker.get_type(e), Some(Type::Enum(e)) if !e.expression));
    let s = checker.get_type_by_name("S").unwrap();
    assert!(matches!(checker.get_type(s), Some(Type::Scalar(s)) if !s.expression));
    let u = checker.get_type_by_name("U").unwrap();
    assert!(matches!(checker.get_type(u), Some(Type::Union(u)) if !u.expression));
}

#[test]
fn test_declaration_expression_through_alias() {
    use crate::checker::types::Type;
    let checker = check_decl_expr(
        r#"
        alias E = enum { a, b };
        alias U = union { string, int32 };
        alias S = scalar extends string;
        alias M = model { x: string };
        model Foo { e: E; u: U; s: S; m: M; }
        "#,
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let resolve = |id| checker.resolve_alias_chain(id);
    assert!(
        matches!(checker.get_type(resolve(property_type(&checker, "Foo", "e"))), Some(Type::Enum(e)) if e.expression)
    );
    assert!(
        matches!(checker.get_type(resolve(property_type(&checker, "Foo", "u"))), Some(Type::Union(u)) if u.expression)
    );
    assert!(
        matches!(checker.get_type(resolve(property_type(&checker, "Foo", "s"))), Some(Type::Scalar(s)) if s.expression)
    );
    assert!(
        matches!(checker.get_type(resolve(property_type(&checker, "Foo", "m"))), Some(Type::Model(m)) if m.expression)
    );
}

#[test]
fn test_declaration_expression_references_template_parameter() {
    use crate::checker::types::Type;
    let checker = check_decl_expr(
        r#"
        model Wrapper<T> { nested: model { item: T }; }
        model Bar { w: Wrapper<int32>; }
        "#,
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let wrapper_ty = property_type(&checker, "Bar", "w");
    let Some(Type::Model(wrapper)) = checker.get_type(wrapper_ty) else {
        panic!("expected wrapper model");
    };
    let nested_prop = wrapper.properties["nested"];
    let Some(Type::ModelProperty(nested_prop)) = checker.get_type(nested_prop) else {
        panic!("expected nested property");
    };
    let Some(Type::Model(nested)) = checker.get_type(nested_prop.r#type) else {
        panic!("expected nested model");
    };
    assert!(nested.expression);
    let item_prop = nested.properties["item"];
    let Some(Type::ModelProperty(item)) = checker.get_type(item_prop) else {
        panic!("expected item property");
    };
    match checker.get_type(item.r#type) {
        Some(Type::Scalar(s)) => assert_eq!(s.name, "int32"),
        other => panic!("expected int32, got {other:?}"),
    }
}

#[test]
fn test_declaration_expression_as_operation_return_type() {
    use crate::checker::types::Type;
    let checker = check_decl_expr("op test(): enum { a, b };", true);
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let op_id = checker.get_type_by_name("test").unwrap();
    let ret = match checker.get_type(op_id) {
        Some(Type::Operation(o)) => o.return_type.expect("return type"),
        other => panic!("expected operation, got {other:?}"),
    };
    assert!(matches!(checker.get_type(ret), Some(Type::Enum(e)) if e.expression));
}

#[test]
fn test_declaration_expression_as_operation_parameter_type() {
    use crate::checker::types::Type;
    let checker = check_decl_expr("op test(value: model { x: string }): void;", true);
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let ty = operation_param_type(&checker, "test", "value");
    assert!(matches!(checker.get_type(ty), Some(Type::Model(m)) if m.expression));
}

#[test]
fn test_declaration_expression_as_union_variant() {
    use crate::checker::types::Type;
    let checker = check_decl_expr("model Foo { value: string | enum { a, b }; }", true);
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let ty = property_type(&checker, "Foo", "value");
    let Some(Type::Union(u)) = checker.get_type(ty) else {
        panic!("expected union");
    };
    let mut found = false;
    for &variant_id in u.variants.values() {
        let Some(Type::UnionVariant(v)) = checker.get_type(variant_id) else {
            continue;
        };
        if let Some(Type::Enum(e)) = checker.get_type(v.r#type) {
            assert!(e.expression);
            found = true;
        }
    }
    assert!(
        found,
        "enum declaration expression should be a union variant"
    );
}

#[test]
fn test_declaration_expression_nested_inside_another() {
    use crate::checker::types::Type;
    let checker = check_decl_expr("model Foo { value: model { inner: enum { a, b } }; }", true);
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let ty = property_type(&checker, "Foo", "value");
    let Some(Type::Model(m)) = checker.get_type(ty) else {
        panic!("expected model");
    };
    assert!(m.expression);
    let inner_prop = m.properties["inner"];
    let Some(Type::ModelProperty(inner)) = checker.get_type(inner_prop) else {
        panic!("expected inner property");
    };
    assert!(matches!(checker.get_type(inner.r#type), Some(Type::Enum(e)) if e.expression));
}

#[test]
fn test_declaration_expression_member_access_through_alias() {
    use crate::checker::types::Type;
    let checker = check_decl_expr(
        r#"
        alias E = enum { a, b };
        alias A = E.a;
        model Foo { value: A; }
        "#,
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let resolved = checker.resolve_alias_chain(property_type(&checker, "Foo", "value"));
    assert!(matches!(
        checker.get_type(resolved),
        Some(Type::EnumMember(_))
    ));
}

#[test]
fn test_declaration_expression_in_alias_position_compiles() {
    let checker = check_decl_expr(
        r#"
        alias E = enum { a, b };
        alias U = union { string, int32 };
        alias S = scalar extends string;
        alias M = model { x: string };
        "#,
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
}

fn type_name_of(checker: &Checker, id: TypeId) -> String {
    crate::checker::type_utils::type_to_string(&checker.type_store, id)
}

#[test]
fn test_declaration_expression_type_names() {
    let checker = check_decl_expr(
        r#"
        namespace Ns;
        model Foo {
            modelProp: model { x: string };
            enumProp: enum { a, b };
            scalarProp: scalar extends string;
            unionProp: union { string, int32 };
        }
        "#,
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    assert_eq!(
        type_name_of(&checker, property_type(&checker, "Foo", "modelProp")),
        "{ x: string }"
    );
    assert_eq!(
        type_name_of(&checker, property_type(&checker, "Foo", "enumProp")),
        "{ a, b }"
    );
    assert_eq!(
        type_name_of(&checker, property_type(&checker, "Foo", "scalarProp")),
        "scalar extends string"
    );
    assert_eq!(
        type_name_of(&checker, property_type(&checker, "Foo", "unionProp")),
        "string | int32"
    );
}

#[test]
fn test_named_declaration_expression_type_name() {
    let checker = check_decl_expr(
        r#"
        namespace Ns;
        model Foo { named: enum Color { red }; }
        "#,
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    assert_eq!(
        type_name_of(&checker, property_type(&checker, "Foo", "named")),
        "Color"
    );
}

#[test]
fn test_declaration_expression_feature_disabled_error() {
    let checker = check_decl_expr("model Foo { status: enum { active, inactive }; }", false);
    assert_eq!(count_code(&checker, "declaration-expression-disabled"), 1);
}

#[test]
fn test_declaration_expression_feature_disabled_for_each_kind() {
    let checker = check_decl_expr(
        r#"
        model Foo {
            a: model { x: string };
            b: enum { active, inactive };
            c: union { string, int32 };
            d: scalar myStr extends string;
        }
        "#,
        false,
    );
    assert_eq!(
        count_code(&checker, "declaration-expression-disabled"),
        4,
        "diagnostics: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_declaration_expression_feature_error_once_for_template_instances() {
    let checker = check_decl_expr(
        r#"
        model Foo<T> {
            x: model { y: T };
        }
        model A is Foo<string>;
        model B is Foo<int32>;
        "#,
        false,
    );
    assert_eq!(
        count_code(&checker, "declaration-expression-disabled"),
        1,
        "diagnostics: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_declaration_expression_feature_enabled_no_error() {
    let checker = check_decl_expr("model Foo { status: enum { active, inactive }; }", true);
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_templated_declaration_expressions_rejected_for_all_kinds() {
    for source in [
        "alias M = model Foo<T> { x: T };",
        "alias U = union Foo<T> { x: T };",
        "alias S = scalar Foo<T> extends string;",
    ] {
        let checker = check_decl_expr(source, true);
        assert!(
            has_code(&checker, "templated-declaration-in-expression"),
            "source {source:?} diagnostics: {:?}",
            checker.diagnostics()
        );
    }
}

#[test]
fn test_template_parameters_allowed_in_statement_position() {
    let checker = check_decl_expr("model Foo<T> { x: T }", true);
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_decorator_on_anonymous_enum_expression() {
    let checker = check_decl_expr(
        r#"model Foo { status: @doc("the status") enum { active, inactive }; }"#,
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let ty = property_type(&checker, "Foo", "status");
    assert!(matches!(checker.get_type(ty), Some(Type::Enum(e)) if e.expression));
    assert_eq!(doc_of(&checker, ty).as_deref(), Some("the status"));
}

#[test]
fn test_decorator_on_named_model_expression() {
    let checker = check_decl_expr(
        r#"model Foo { inner: @doc("the inner") model Inner { x: string }; }"#,
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let ty = property_type(&checker, "Foo", "inner");
    assert!(
        matches!(checker.get_type(ty), Some(Type::Model(m)) if m.name == "Inner" && m.expression)
    );
    assert_eq!(doc_of(&checker, ty).as_deref(), Some("the inner"));
}

#[test]
fn test_decorator_on_union_expression() {
    let checker = check_decl_expr(
        r#"model Foo { value: @doc("the value") union { string, int32 }; }"#,
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let ty = property_type(&checker, "Foo", "value");
    assert!(matches!(checker.get_type(ty), Some(Type::Union(u)) if u.expression));
    assert_eq!(doc_of(&checker, ty).as_deref(), Some("the value"));
}

#[test]
fn test_decorator_on_scalar_expression() {
    let checker = check_decl_expr(
        r#"model Foo { unit: @doc("the unit") scalar extends string; }"#,
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let ty = property_type(&checker, "Foo", "unit");
    assert!(matches!(checker.get_type(ty), Some(Type::Scalar(s)) if s.expression));
    assert_eq!(doc_of(&checker, ty).as_deref(), Some("the unit"));
}

#[test]
fn test_decorator_before_non_declaration_expression_rejected() {
    let result = parse(r#"model Foo { prop: @doc("nope") string; }"#);
    assert!(
        result.diagnostics.iter().any(|d| {
            d.code == "invalid-decorator-location" && d.message == "Cannot decorate expression."
        }),
        "parse diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_decorators_on_members_inside_expression() {
    let checker = check_decl_expr(r#"model Foo { value: enum { @doc("first") a, b }; }"#, true);
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let ty = property_type(&checker, "Foo", "value");
    assert!(
        matches!(checker.get_type(ty), Some(Type::Enum(e)) if e.expression && e.members.contains_key("a"))
    );
}

fn check_with_use_type(source: &str) -> Checker {
    let result = parse(source);
    let options = CompilerOptions {
        features: vec![
            "declaration-expressions".to_string(),
            "union-extends".to_string(),
        ],
        ..CompilerOptions::default()
    };
    let mut checker = Checker::with_options(options);
    checker.register_decorator("useType", "Test", "unknown");
    checker.set_parse_result(result.root_id, result.builder);
    checker.check_program();
    checker
}

fn use_type_argument(checker: &Checker, model_name: &str) -> TypeId {
    use crate::checker::types::DecoratorMarshalledValue;
    let model_id = checker.get_type_by_name(model_name).unwrap();
    let decorator = checker
        .find_decorator(model_id, "useType")
        .unwrap_or_else(|| panic!("useType should be applied to {model_name}"));
    match decorator.args.first().and_then(|a| a.js_value.as_ref()) {
        Some(DecoratorMarshalledValue::Type(id)) => *id,
        other => panic!("expected type argument, got {other:?}"),
    }
}

#[test]
fn test_declaration_expression_as_decorator_argument() {
    use crate::checker::types::Type;
    let checker = check_with_use_type(
        r#"
        @useType(enum { red, green })
        model Foo {}
        "#,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let arg = use_type_argument(&checker, "Foo");
    match checker.get_type(arg) {
        Some(Type::Enum(e)) => {
            assert!(e.expression);
            assert!(e.members.contains_key("red"));
            assert!(e.members.contains_key("green"));
        }
        other => panic!("expected enum argument, got {other:?}"),
    }
}

#[test]
fn test_named_declaration_expression_as_decorator_argument() {
    use crate::checker::types::Type;
    let checker = check_with_use_type(
        r#"
        @useType(model Inner { x: string })
        model Foo {}
        "#,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let arg = use_type_argument(&checker, "Foo");
    match checker.get_type(arg) {
        Some(Type::Model(m)) => {
            assert_eq!(m.name, "Inner");
            assert!(m.expression);
            assert!(m.properties.contains_key("x"));
        }
        other => panic!("expected model argument, got {other:?}"),
    }
}

#[test]
fn test_decorated_declaration_expression_as_decorator_argument() {
    use crate::checker::types::Type;
    let checker = check_with_use_type(
        r#"
        @useType(@doc("the versions") enum Versions { v1, v2 })
        model Foo {}
        "#,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let arg = use_type_argument(&checker, "Foo");
    assert!(matches!(checker.get_type(arg), Some(Type::Enum(_))));
    assert_eq!(doc_of(&checker, arg).as_deref(), Some("the versions"));
}

#[test]
fn test_declaration_expression_decorator_per_template_instantiation() {
    use crate::checker::types::Type;
    let checker = check_decl_expr(
        r#"
        model Box<T extends valueof string> {
            e: @doc(T) enum { a };
        }
        model A is Box<"alpha"> {}
        model B is Box<"beta"> {}
        "#,
        true,
    );
    assert!(
        checker.diagnostics().is_empty(),
        "{:?}",
        checker.diagnostics()
    );
    let a_ty = property_type(&checker, "A", "e");
    let b_ty = property_type(&checker, "B", "e");
    assert_ne!(a_ty, b_ty, "each instantiation gets its own enum type");
    match (checker.get_type(a_ty), checker.get_type(b_ty)) {
        (Some(Type::Enum(a)), Some(Type::Enum(b))) => {
            assert!(a.expression && b.expression);
        }
        other => panic!("expected enums, got {other:?}"),
    }
    assert_eq!(doc_of(&checker, a_ty).as_deref(), Some("alpha"));
    assert_eq!(doc_of(&checker, b_ty).as_deref(), Some("beta"));
}

#[test]
fn test_augment_anonymous_model_expression_rejected() {
    let checker = check_decl_expr(
        r#"
        alias M = { x: string };
        @@doc(M, "nope");
        "#,
        true,
    );
    assert!(
        has_code(&checker, "augment-decorator-target"),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
}

#[test]
fn test_augment_union_expression_rejected() {
    let checker = check_decl_expr(
        r#"
        alias U = string | int32;
        @@doc(U, "nope");
        "#,
        true,
    );
    assert!(
        has_code(&checker, "augment-decorator-target"),
        "diagnostics: {:?}",
        checker.diagnostics()
    );
}
