//! Tests for the public helper methods in helpers.rs

use super::*;
use crate::checker::test_utils::check;

// ============================================================================
// Decorator lookup helpers
// ============================================================================

#[test]
fn test_find_decorator() {
    let checker = check(
        r#"@doc("A foo model")
model Foo { }"#,
    );
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    let dec = checker.find_decorator(model_id, "doc");
    assert!(dec.is_some(), "Should find @doc decorator on Foo");
}

#[test]
fn test_find_decorator_qualified_name() {
    let checker = check(
        r#"@TypeSpec.doc("A foo model")
model Foo { }"#,
    );
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    let dec = checker.find_decorator(model_id, "doc");
    assert!(
        dec.is_some(),
        "Should find @TypeSpec.doc decorator by short name"
    );
}

#[test]
fn test_find_decorator_missing() {
    let checker = check(r#"model Foo { }"#);
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    let dec = checker.find_decorator(model_id, "nonexistent");
    assert!(dec.is_none(), "Should not find nonexistent decorator");
}

#[test]
fn test_find_decorators_multiple() {
    let checker = check(
        r#"@doc("first") @doc("second")
model Foo { }"#,
    );
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    let decs = checker.find_decorators(model_id, "doc");
    assert_eq!(decs.len(), 2, "Should find both @doc decorators");
}

#[test]
fn test_find_decorator_on_property() {
    let checker = check(r#"model Foo { @doc("prop doc") name: string }"#);
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    let prop_id = checker
        .get_model_property(model_id, "name")
        .expect("name should exist");
    let dec = checker.find_decorator(prop_id, "doc");
    assert!(dec.is_some(), "Should find @doc on ModelProperty");
}

#[test]
fn test_get_decorator_string_arg() {
    let checker = check(
        r#"@doc("hello world")
model Foo { }"#,
    );
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    let val = checker.get_decorator_string_arg(model_id, "doc", 0);
    assert_eq!(val, Some("hello world".to_string()));
}

#[test]
fn test_get_decorator_string_arg_out_of_bounds() {
    let checker = check(
        r#"@doc("hello")
model Foo { }"#,
    );
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    let val = checker.get_decorator_string_arg(model_id, "doc", 5);
    assert_eq!(val, None, "Out of bounds arg index returns None");
}

#[test]
fn test_get_decorator_numeric_arg() {
    let checker = check(
        r#"@minValue(10)
model Foo { }"#,
    );
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    let val = checker.get_decorator_numeric_arg(model_id, "minValue", 0);
    assert_eq!(val, Some(10.0));
}

#[test]
fn test_get_decorator_numeric_arg_float() {
    let checker = check(
        r#"@minValue(2.71)
model Foo { }"#,
    );
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    let val = checker.get_decorator_numeric_arg(model_id, "minValue", 0);
    assert!(
        (val.unwrap() - 2.71).abs() < 0.001,
        "Should parse float arg"
    );
}

#[test]
fn test_get_decorator_bool_arg_missing() {
    let checker = check(r#"model Foo { }"#);
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    let val = checker.get_decorator_bool_arg(model_id, "nonexistent", 0);
    assert_eq!(val, None);
}

// ============================================================================
// Type lookup helpers
// ============================================================================

#[test]
fn test_lookup_type_by_fqn_simple() {
    let checker = check(r#"model Foo { }"#);
    let id = checker.lookup_type_by_fqn("Foo");
    assert!(id.is_some(), "Should find Foo by simple name");
}

#[test]
fn test_lookup_type_by_fqn_qualified() {
    let checker = check(r#"namespace MyNs { model Bar { name: string } }"#);
    let id = checker.lookup_type_by_fqn("MyNs.Bar");
    assert!(id.is_some(), "Should find MyNs.Bar by qualified name");
}

#[test]
fn test_lookup_type_by_fqn_nested_namespace() {
    let checker = check(r#"namespace A { namespace B { model Inner { x: int32 } } }"#);
    let id = checker.lookup_type_by_fqn("A.B.Inner");
    assert!(
        id.is_some(),
        "Should find A.B.Inner by nested qualified name"
    );
}

#[test]
fn test_lookup_type_by_fqn_missing() {
    let checker = check(r#"model Foo { }"#);
    let id = checker.lookup_type_by_fqn("NonExistent");
    assert!(id.is_none(), "Should not find nonexistent type");
}

#[test]
fn test_lookup_type_by_fqn_enum() {
    let checker = check(r#"namespace App { enum Status { Active, Inactive } }"#);
    let id = checker.lookup_type_by_fqn("App.Status");
    assert!(id.is_some(), "Should find enum by FQN");
    assert!(matches!(checker.get_type(id.unwrap()), Some(Type::Enum(_))));
}

#[test]
fn test_lookup_type_by_fqn_interface() {
    let checker = check(r#"namespace App { interface PetOps { list(): string; } }"#);
    let id = checker.lookup_type_by_fqn("App.PetOps");
    assert!(id.is_some(), "Should find interface by FQN");
    assert!(matches!(
        checker.get_type(id.unwrap()),
        Some(Type::Interface(_))
    ));
}

// ============================================================================
// Value extraction helpers
// ============================================================================

#[test]
fn test_extract_default_value_string() {
    let checker = check(r#"model Foo { name: string = "default" }"#);
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    let prop_id = checker
        .get_model_property(model_id, "name")
        .expect("name prop should exist");
    let val = checker.extract_default_value(prop_id);
    assert_eq!(val, Some("default".to_string()));
}

#[test]
fn test_extract_default_value_numeric() {
    let checker = check(r#"model Foo { count: int32 = 42 }"#);
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    let prop_id = checker
        .get_model_property(model_id, "count")
        .expect("count prop should exist");
    let val = checker.extract_default_value(prop_id);
    assert_eq!(val, Some("42".to_string()));
}

#[test]
fn test_extract_default_value_none() {
    let checker = check(r#"model Foo { name: string }"#);
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    let prop_id = checker
        .get_model_property(model_id, "name")
        .expect("name prop should exist");
    let val = checker.extract_default_value(prop_id);
    assert_eq!(val, None, "No default value returns None");
}

#[test]
fn test_extract_enum_member_value_string() {
    let checker = check(r#"enum Color { Red: "red", Blue: "blue" }"#);
    let enum_id = checker
        .lookup_type_by_fqn("Color")
        .expect("Color should exist");
    if let Some(Type::Enum(e)) = checker.get_type(enum_id) {
        let red_id = e.members.get("Red").expect("Red member should exist");
        let val = checker.extract_enum_member_value(*red_id);
        assert_eq!(val, Some("red".to_string()));
    } else {
        panic!("Color should be an enum");
    }
}

#[test]
fn test_extract_enum_member_value_implicit() {
    let checker = check(r#"enum Direction { Up, Down }"#);
    let enum_id = checker
        .lookup_type_by_fqn("Direction")
        .expect("Direction should exist");
    if let Some(Type::Enum(e)) = checker.get_type(enum_id) {
        let up_id = e.members.get("Up").expect("Up member should exist");
        // Implicit enum members have no explicit value
        let val = checker.extract_enum_member_value(*up_id);
        assert_eq!(val, None, "Implicit enum members have no value");
    }
}

// ============================================================================
// Type iteration helpers
// ============================================================================

#[test]
fn test_iter_models() {
    let checker = check(r#"model Foo { } model Bar { }"#);
    let models: Vec<_> = checker.iter_models().collect();
    let names: Vec<String> = models.iter().map(|(_, m)| m.name.clone()).collect();
    assert!(names.contains(&"Foo".to_string()), "Should contain Foo");
    assert!(names.contains(&"Bar".to_string()), "Should contain Bar");
}

#[test]
fn test_iter_enums() {
    let checker = check(r#"enum Dir { Up, Down }"#);
    let enums: Vec<_> = checker.iter_enums().collect();
    let names: Vec<String> = enums.iter().map(|(_, e)| e.name.clone()).collect();
    assert!(names.contains(&"Dir".to_string()), "Should contain Dir");
}

#[test]
fn test_iter_namespaces() {
    let checker = check(r#"namespace MyNs { model X { } }"#);
    let nss: Vec<_> = checker.iter_namespaces().collect();
    let names: Vec<String> = nss.iter().map(|(_, ns)| ns.name.clone()).collect();
    assert!(names.contains(&"MyNs".to_string()), "Should contain MyNs");
}

#[test]
fn test_iter_operations() {
    let checker = check(r#"namespace App { op list(): string; }"#);
    let ops: Vec<_> = checker.iter_operations().collect();
    let names: Vec<String> = ops.iter().map(|(_, o)| o.name.clone()).collect();
    assert!(
        names.contains(&"list".to_string()),
        "Should contain list operation"
    );
}

#[test]
fn test_iter_interfaces() {
    let checker = check(r#"namespace App { interface Svc { op ping(): string; } }"#);
    let ifaces: Vec<_> = checker.iter_interfaces().collect();
    let names: Vec<String> = ifaces.iter().map(|(_, i)| i.name.clone()).collect();
    assert!(
        names.contains(&"Svc".to_string()),
        "Should contain Svc interface"
    );
}

#[test]
fn test_iter_types() {
    let checker = check(r#"model M { } enum E { A }"#);
    let count = checker.iter_types().count();
    assert!(count > 0, "Should have some types");
}

// ============================================================================
// Property access helpers
// ============================================================================

#[test]
fn test_get_model_property() {
    let checker = check(r#"model Foo { name: string, age: int32 }"#);
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    assert!(checker.get_model_property(model_id, "name").is_some());
    assert!(checker.get_model_property(model_id, "age").is_some());
    assert!(checker.get_model_property(model_id, "missing").is_none());
}

#[test]
fn test_walk_model_properties_no_inheritance() {
    let checker = check(r#"model Foo { x: int32, y: string }"#);
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    let props = checker.walk_model_properties(model_id);
    let names: Vec<&str> = props.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, vec!["x", "y"]);
}

#[test]
fn test_walk_model_properties_with_inheritance() {
    let checker = check(r#"model Base { id: string } model Derived extends Base { name: string }"#);
    let derived_id = checker
        .lookup_type_by_fqn("Derived")
        .expect("Derived should exist");
    let props = checker.walk_model_properties(derived_id);
    let names: Vec<&str> = props.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(
        names,
        vec!["id", "name"],
        "Inherited props should come first"
    );
}

#[test]
fn test_walk_model_properties_deep_inheritance() {
    let checker = check(
        r#"model A { a: int32 } model B extends A { b: string } model C extends B { c: boolean }"#,
    );
    let c_id = checker.lookup_type_by_fqn("C").expect("C should exist");
    let props = checker.walk_model_properties(c_id);
    let names: Vec<&str> = props.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(
        names,
        vec!["a", "b", "c"],
        "Deep inheritance: base props first"
    );
}

#[test]
fn test_get_property_type_scalar() {
    let checker = check(r#"model Foo { name: string }"#);
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    let prop_id = checker
        .get_model_property(model_id, "name")
        .expect("name should exist");
    let prop_type_id = checker.get_property_type(prop_id);
    if let Some(t) = checker.get_type(prop_type_id) {
        match t {
            Type::String(_) | Type::Scalar(_) => {}
            other => panic!("Expected string/scalar type, got {:?}", other.kind_name()),
        }
    }
}

#[test]
fn test_get_property_type_model_ref() {
    let checker = check(r#"model Bar { id: int32 } model Foo { bar: Bar }"#);
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    let prop_id = checker
        .get_model_property(model_id, "bar")
        .expect("bar should exist");
    let prop_type_id = checker.get_property_type(prop_id);
    assert!(
        matches!(checker.get_type(prop_type_id), Some(Type::Model(_))),
        "Property type for Bar should be Model"
    );
}

// ============================================================================
// Constraint accessor helpers
// ============================================================================

#[test]
fn test_constraint_accessors() {
    let checker = check(
        r#"@minValue(1) @maxValue(100)
model Foo { @minValue(0) @maxLength(50) val: string }"#,
    );
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");

    // Model-level constraints
    assert_eq!(checker.get_type_min_value(model_id), Some(1.0));
    assert_eq!(checker.get_type_max_value(model_id), Some(100.0));

    // Property-level constraints
    let prop_id = checker
        .get_model_property(model_id, "val")
        .expect("val should exist");
    assert_eq!(checker.get_type_min_value(prop_id), Some(0.0));
    assert_eq!(checker.get_type_max_length(prop_id), Some(50.0));
}

#[test]
fn test_exclusive_constraint_accessors() {
    let checker = check(
        r#"@minValueExclusive(0) @maxValueExclusive(100)
model Foo { }"#,
    );
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    assert_eq!(checker.get_type_min_value_exclusive(model_id), Some(0.0));
    assert_eq!(checker.get_type_max_value_exclusive(model_id), Some(100.0));
}

#[test]
fn test_min_max_length_constraints() {
    let checker = check(r#"model Foo { @minLength(1) @maxLength(255) name: string }"#);
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    let prop_id = checker
        .get_model_property(model_id, "name")
        .expect("name should exist");
    assert_eq!(checker.get_type_min_length(prop_id), Some(1.0));
    assert_eq!(checker.get_type_max_length(prop_id), Some(255.0));
}

#[test]
fn test_pattern_and_format_constraints() {
    let checker = check(r#"model Foo { @pattern("^[a-z]+$") @format("email") val: string }"#);
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    let prop_id = checker
        .get_model_property(model_id, "val")
        .expect("val should exist");
    assert_eq!(
        checker.get_type_pattern(prop_id),
        Some("^[a-z]+$".to_string())
    );
    assert_eq!(checker.get_type_format(prop_id), Some("email".to_string()));
}

#[test]
fn test_min_max_items_constraints() {
    let checker = check(
        r#"@minItems(1) @maxItems(10)
model Foo { }"#,
    );
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    assert_eq!(checker.get_type_min_items(model_id), Some(1.0));
    assert_eq!(checker.get_type_max_items(model_id), Some(10.0));
}

#[test]
fn test_no_constraints_returns_none() {
    let checker = check(r#"model Foo { name: string }"#);
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    assert_eq!(checker.get_type_min_value(model_id), None);
    assert_eq!(checker.get_type_max_value(model_id), None);
    assert_eq!(checker.get_type_min_length(model_id), None);
    assert_eq!(checker.get_type_pattern(model_id), None);
}

#[test]
fn test_is_type_error() {
    let checker = check(
        r#"@error
model ErrorResult { message: string }"#,
    );
    let model_id = checker
        .lookup_type_by_fqn("ErrorResult")
        .expect("ErrorResult should exist");
    assert!(
        checker.is_type_error(model_id),
        "@error model should return true"
    );
}

#[test]
fn test_is_type_error_false() {
    let checker = check(r#"model Foo { }"#);
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    assert!(
        !checker.is_type_error(model_id),
        "Normal model should return false"
    );
}

// ============================================================================
// Doc/Summary field population (evaluate_std_decorators)
// ============================================================================

#[test]
fn test_model_doc_field_populated() {
    let checker = check(
        r#"@doc("A documented model")
model Foo { }"#,
    );
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    if let Some(Type::Model(m)) = checker.get_type(model_id) {
        assert_eq!(m.doc, Some("A documented model".to_string()));
    }
}

#[test]
fn test_model_summary_field_populated() {
    let checker = check(
        r#"@summary("Brief desc")
model Foo { }"#,
    );
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    if let Some(Type::Model(m)) = checker.get_type(model_id) {
        assert_eq!(m.summary, Some("Brief desc".to_string()));
    }
}

#[test]
fn test_property_doc_populated() {
    let checker = check(r#"model Foo { @doc("The name") name: string }"#);
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    let prop_id = checker
        .get_model_property(model_id, "name")
        .expect("name should exist");
    if let Some(Type::ModelProperty(p)) = checker.get_type(prop_id) {
        assert_eq!(p.doc, Some("The name".to_string()));
    }
}

#[test]
fn test_property_summary_populated() {
    let checker = check(r#"model Foo { @summary("prop sum") name: string }"#);
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    let prop_id = checker
        .get_model_property(model_id, "name")
        .expect("name should exist");
    if let Some(Type::ModelProperty(p)) = checker.get_type(prop_id) {
        assert_eq!(p.summary, Some("prop sum".to_string()));
    }
}

#[test]
fn test_enum_member_doc_populated() {
    let checker = check(r#"enum Color { @doc("Red color") Red, Blue }"#);
    let enum_id = checker
        .lookup_type_by_fqn("Color")
        .expect("Color should exist");
    if let Some(Type::Enum(e)) = checker.get_type(enum_id) {
        let red_id = e.members.get("Red").expect("Red member should exist");
        if let Some(Type::EnumMember(m)) = checker.get_type(*red_id) {
            assert_eq!(m.doc, Some("Red color".to_string()));
        }
    }
}

#[test]
fn test_union_variant_doc_populated() {
    let checker = check(r#"union Shape { @doc("A circle") Circle: string, Square: string }"#);
    let union_id = checker
        .lookup_type_by_fqn("Shape")
        .expect("Shape should exist");
    if let Some(Type::Union(u)) = checker.get_type(union_id) {
        let circle_id = u
            .variants
            .get("Circle")
            .expect("Circle variant should exist");
        if let Some(Type::UnionVariant(v)) = checker.get_type(*circle_id) {
            assert_eq!(v.doc, Some("A circle".to_string()));
        }
    }
}

#[test]
fn test_namespace_doc_populated() {
    let checker = check(
        r#"@doc("My namespace")
namespace MyApp { }"#,
    );
    let ns_id = checker
        .lookup_type_by_fqn("MyApp")
        .expect("MyApp should exist");
    if let Some(Type::Namespace(ns)) = checker.get_type(ns_id) {
        assert_eq!(ns.doc, Some("My namespace".to_string()));
    }
}

#[test]
fn test_operation_doc_populated() {
    let checker = check(r#"namespace App { @doc("List items") op list(): string; }"#);
    let ns_id = checker.lookup_type_by_fqn("App").expect("App should exist");
    if let Some(Type::Namespace(ns)) = checker.get_type(ns_id) {
        let op_id = ns.operations.get("list").expect("list op should exist");
        if let Some(Type::Operation(o)) = checker.get_type(*op_id) {
            assert_eq!(o.doc, Some("List items".to_string()));
        }
    }
}

#[test]
fn test_interface_doc_populated() {
    let checker = check(
        r#"namespace App { @doc("Service interface") interface Svc { op ping(): string; } }"#,
    );
    let ns_id = checker.lookup_type_by_fqn("App").expect("App should exist");
    if let Some(Type::Namespace(ns)) = checker.get_type(ns_id) {
        let iface_id = ns.interfaces.get("Svc").expect("Svc should exist");
        if let Some(Type::Interface(i)) = checker.get_type(*iface_id) {
            assert_eq!(i.doc, Some("Service interface".to_string()));
        }
    }
}

#[test]
fn test_no_doc_returns_none() {
    let checker = check(r#"model Foo { name: string }"#);
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    if let Some(Type::Model(m)) = checker.get_type(model_id) {
        assert_eq!(m.doc, None);
        assert_eq!(m.summary, None);
    }
}

// ============================================================================
// Type::doc() / Type::summary() dispatch consistency
// ============================================================================

#[test]
fn test_type_dispatch_doc_and_summary() {
    let checker = check(
        r#"@doc("doc text") @summary("sum text")
model Foo { }"#,
    );
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    if let Some(t) = checker.get_type(model_id) {
        assert_eq!(t.doc(), Some("doc text"));
        assert_eq!(t.summary(), Some("sum text"));
    }
}

#[test]
fn test_type_dispatch_doc_on_property() {
    let checker = check(r#"model Foo { @doc("prop doc") name: string }"#);
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    let prop_id = checker
        .get_model_property(model_id, "name")
        .expect("name should exist");
    if let Some(t) = checker.get_type(prop_id) {
        assert_eq!(t.doc(), Some("prop doc"));
    }
}

#[test]
fn test_type_dispatch_doc_on_enum_member() {
    let checker = check(r#"enum Status { @doc("Active state") Active }"#);
    let enum_id = checker
        .lookup_type_by_fqn("Status")
        .expect("Status should exist");
    if let Some(Type::Enum(e)) = checker.get_type(enum_id) {
        let member_id = e.members.get("Active").expect("Active member should exist");
        if let Some(t) = checker.get_type(*member_id) {
            assert_eq!(t.doc(), Some("Active state"));
        }
    }
}

#[test]
fn test_state_accessor_and_type_field_consistency() {
    // Both get_type_doc() (StateAccessors) and Type.doc() should return the same value
    let checker = check(
        r#"@doc("consistent doc")
model Foo { }"#,
    );
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");

    let state_doc = checker.get_type_doc(model_id);
    let type_doc = checker
        .get_type(model_id)
        .and_then(|t| t.doc().map(|s| s.to_string()));

    assert_eq!(
        state_doc, type_doc,
        "StateAccessors doc and Type.doc() should be consistent"
    );
}

#[test]
fn test_state_accessor_and_summary_consistency() {
    let checker = check(
        r#"@summary("consistent summary")
model Foo { }"#,
    );
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");

    let state_summary = checker.get_type_summary(model_id);
    let type_summary = checker
        .get_type(model_id)
        .and_then(|t| t.summary().map(|s| s.to_string()));

    assert_eq!(
        state_summary, type_summary,
        "StateAccessors summary and Type.summary() should be consistent"
    );
}

#[test]
fn test_doc_with_empty_string() {
    let checker = check(
        r#"@doc("")
model Foo { }"#,
    );
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    // Empty string @doc("") may or may not populate the doc field depending on
    // how the marshaller handles empty strings. Just verify no crash.
    let _ = checker.get_type_doc(model_id);
    if let Some(Type::Model(m)) = checker.get_type(model_id) {
        // Empty string is still a valid doc value
        assert!(m.doc.is_none() || m.doc.as_ref().map(|s| s.is_empty()).unwrap_or(false));
    }
}

#[test]
fn test_doc_with_multiline_text() {
    let checker = check(
        r#"@doc("Line one\nLine two")
model Foo { }"#,
    );
    let model_id = checker.lookup_type_by_fqn("Foo").expect("Foo should exist");
    if let Some(Type::Model(m)) = checker.get_type(model_id) {
        assert!(m.doc.is_some(), "Multiline doc should be populated");
    }
}
