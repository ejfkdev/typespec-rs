//! Checker Tests
//!
//! Ported from TypeSpec compiler/test/checker/
//!
//! These tests verify the type checking and semantic analysis functionality.

// ============================================================================
// T5.1: Checker Core Type Inference Tests
// ============================================================================

#[cfg(test)]
mod checker_core_tests {

    use crate::checker::test_utils::check;
    use crate::checker::{Checker, IntrinsicTypeName, Type};

    // ==================== Intrinsic Types ====================

    #[test]
    fn test_intrinsic_types_initialized() {
        let checker = Checker::new();
        // Error type
        let error_type = checker.get_type(checker.error_type);
        assert!(
            matches!(error_type, Some(Type::Intrinsic(t)) if t.name == IntrinsicTypeName::ErrorType)
        );
        // Void type
        let void_type = checker.get_type(checker.void_type);
        assert!(matches!(void_type, Some(Type::Intrinsic(t)) if t.name == IntrinsicTypeName::Void));
        // Never type
        let never_type = checker.get_type(checker.never_type);
        assert!(
            matches!(never_type, Some(Type::Intrinsic(t)) if t.name == IntrinsicTypeName::Never)
        );
        // Unknown type
        let unknown_type = checker.get_type(checker.unknown_type);
        assert!(
            matches!(unknown_type, Some(Type::Intrinsic(t)) if t.name == IntrinsicTypeName::Unknown)
        );
        // Null type
        let null_type = checker.get_type(checker.null_type);
        assert!(matches!(null_type, Some(Type::Intrinsic(t)) if t.name == IntrinsicTypeName::Null));
    }

    // ==================== Model Checking ====================

    #[test]
    fn test_check_empty_model() {
        let checker = check("model Foo {}");
        // Find the model type by name
        let foo_type = checker.declared_types.get("Foo").copied();
        assert!(
            foo_type.is_some(),
            "Model 'Foo' should be in declared_types"
        );

        let t = checker.get_type(foo_type.unwrap());
        assert!(matches!(t, Some(Type::Model(m)) if m.name == "Foo" && m.is_finished));
    }

    #[test]
    fn test_check_model_with_properties() {
        let checker = check("model Foo { x: string; y: int32; }");
        let foo_type = checker.declared_types.get("Foo").copied().unwrap();
        let t = checker.get_type(foo_type).cloned().unwrap();

        match &t {
            Type::Model(m) => {
                assert_eq!(
                    m.property_names.len(),
                    2,
                    "Expected 2 properties, got {}",
                    m.property_names.len()
                );
                assert!(m.properties.contains_key("x"));
                assert!(m.properties.contains_key("y"));
            }
            _ => panic!("Expected Model type"),
        }
    }

    #[test]
    fn test_check_model_with_optional_property() {
        let checker = check("model Foo { x?: string; }");
        let foo_type = checker.declared_types.get("Foo").copied().unwrap();

        // Find the property type
        let t = checker.get_type(foo_type).cloned().unwrap();
        match t {
            Type::Model(m) => {
                assert!(m.properties.contains_key("x"));
                let prop_type_id = m.properties.get("x").copied().unwrap();
                let prop = checker.get_type(prop_type_id).cloned().unwrap();
                match prop {
                    Type::ModelProperty(p) => {
                        assert!(p.optional);
                        assert_eq!(p.name, "x");
                    }
                    _ => panic!("Expected ModelProperty type"),
                }
            }
            _ => panic!("Expected Model type"),
        }
    }

    #[test]
    fn test_check_model_extends() {
        let checker = check("model Base {} model Derived extends Base {}");
        let derived_type = checker.declared_types.get("Derived").copied().unwrap();
        let base_type = checker.declared_types.get("Base").copied().unwrap();

        let t = checker.get_type(derived_type).cloned().unwrap();
        match t {
            Type::Model(m) => {
                assert_eq!(m.base_model, Some(base_type));
            }
            _ => panic!("Expected Model type"),
        }
    }

    // ==================== Literal Types ====================

    #[test]
    fn test_check_numeric_literal() {
        let checker = check("model Foo { x: 42; }");
        let foo_type = checker.declared_types.get("Foo").copied().unwrap();
        let t = checker.get_type(foo_type).cloned().unwrap();

        match t {
            Type::Model(m) => {
                let prop_type_id = m.properties.get("x").copied().unwrap();
                let prop = checker.get_type(prop_type_id).cloned().unwrap();
                match prop {
                    Type::ModelProperty(p) => {
                        let value_type = checker.get_type(p.r#type).cloned().unwrap();
                        match value_type {
                            Type::Number(n) => assert_eq!(n.value, 42.0),
                            other => panic!("Expected Number type, got {:?}", other.kind_name()),
                        }
                    }
                    _ => panic!("Expected ModelProperty"),
                }
            }
            _ => panic!("Expected Model type"),
        }
    }

    #[test]
    fn test_check_string_literal() {
        let checker = check("model Foo { x: \"hello\"; }");
        let foo_type = checker.declared_types.get("Foo").copied().unwrap();
        let t = checker.get_type(foo_type).cloned().unwrap();

        match t {
            Type::Model(m) => {
                let prop_type_id = m.properties.get("x").copied().unwrap();
                let prop = checker.get_type(prop_type_id).cloned().unwrap();
                match prop {
                    Type::ModelProperty(p) => {
                        let value_type = checker.get_type(p.r#type).cloned().unwrap();
                        match value_type {
                            Type::String(s) => assert_eq!(s.value, "hello"),
                            other => panic!("Expected String type, got {:?}", other.kind_name()),
                        }
                    }
                    _ => panic!("Expected ModelProperty"),
                }
            }
            _ => panic!("Expected Model type"),
        }
    }

    #[test]
    fn test_check_boolean_literal() {
        let checker = check("model Foo { x: true; }");
        let foo_type = checker.declared_types.get("Foo").copied().unwrap();
        let t = checker.get_type(foo_type).cloned().unwrap();

        match t {
            Type::Model(m) => {
                let prop_type_id = m.properties.get("x").copied().unwrap();
                let prop = checker.get_type(prop_type_id).cloned().unwrap();
                match prop {
                    Type::ModelProperty(p) => {
                        let value_type = checker.get_type(p.r#type).cloned().unwrap();
                        match value_type {
                            Type::Boolean(b) => assert!(b.value),
                            other => panic!("Expected Boolean type, got {:?}", other.kind_name()),
                        }
                    }
                    _ => panic!("Expected ModelProperty"),
                }
            }
            _ => panic!("Expected Model type"),
        }
    }

    // ==================== Enum Checking ====================

    #[test]
    fn test_check_enum() {
        let checker = check("enum Status { active; inactive; }");
        let enum_type = checker.declared_types.get("Status").copied().unwrap();
        let t = checker.get_type(enum_type).cloned().unwrap();

        match t {
            Type::Enum(e) => {
                assert_eq!(e.name, "Status");
                assert!(e.members.contains_key("active"));
                assert!(e.members.contains_key("inactive"));
                assert_eq!(e.member_names.len(), 2);
            }
            _ => panic!("Expected Enum type"),
        }
    }

    // ==================== Scalar Checking ====================

    #[test]
    fn test_check_scalar() {
        let checker = check("scalar uuid extends string;");
        let scalar_type = checker.declared_types.get("uuid").copied().unwrap();
        let t = checker.get_type(scalar_type).cloned().unwrap();

        match t {
            Type::Scalar(s) => {
                assert_eq!(s.name, "uuid");
                assert!(s.base_scalar.is_some());
            }
            _ => panic!("Expected Scalar type"),
        }
    }

    // ==================== Union Checking ====================

    #[test]
    fn test_check_union() {
        let checker = check("union Color { red: string; blue: string; }");
        let union_type = checker.declared_types.get("Color").copied().unwrap();
        let t = checker.get_type(union_type).cloned().unwrap();

        match t {
            Type::Union(u) => {
                assert_eq!(u.name, "Color");
                assert_eq!(u.variant_names.len(), 2);
            }
            _ => panic!("Expected Union type"),
        }
    }

    // ==================== Namespace Checking ====================

    #[test]
    fn test_check_namespace() {
        let checker = check("namespace MyNs { model Foo {} }");
        let ns_type = checker.declared_types.get("MyNs").copied().unwrap();
        let t = checker.get_type(ns_type).cloned().unwrap();

        match t {
            Type::Namespace(ns) => {
                assert_eq!(ns.name, "MyNs");
                assert!(ns.models.contains_key("Foo"));
            }
            _ => panic!("Expected Namespace type"),
        }
    }

    // ==================== Interface Checking ====================

    #[test]
    fn test_check_interface() {
        let checker = check("interface Foo { bar(): void; }");
        let iface_type = checker.declared_types.get("Foo").copied().unwrap();
        let t = checker.get_type(iface_type).cloned().unwrap();

        match t {
            Type::Interface(iface) => {
                assert_eq!(iface.name, "Foo");
                assert!(iface.operations.contains_key("bar"));
            }
            _ => panic!("Expected Interface type"),
        }
    }

    // ==================== Type Reference Resolution ====================

    #[test]
    fn test_type_reference_to_model() {
        let checker = check("model Foo {} model Bar { x: Foo; }");
        let foo_type = checker.declared_types.get("Foo").copied().unwrap();
        let bar_type = checker.declared_types.get("Bar").copied().unwrap();

        // Check that the property x has type Foo
        let t = checker.get_type(bar_type).cloned().unwrap();
        match t {
            Type::Model(m) => {
                let prop_type_id = m.properties.get("x").copied().unwrap();
                let prop = checker.get_type(prop_type_id).cloned().unwrap();
                match prop {
                    Type::ModelProperty(p) => {
                        assert_eq!(
                            p.r#type, foo_type,
                            "Property x should resolve to Foo's TypeId"
                        );
                    }
                    _ => panic!("Expected ModelProperty"),
                }
            }
            _ => panic!("Expected Model type"),
        }
    }

    #[test]
    fn test_type_reference_to_std_scalar() {
        let checker = check("model Foo { x: string; }");
        let bar_type = checker.declared_types.get("Foo").copied().unwrap();

        let t = checker.get_type(bar_type).cloned().unwrap();
        match t {
            Type::Model(m) => {
                let prop_type_id = m.properties.get("x").copied().unwrap();
                let prop = checker.get_type(prop_type_id).cloned().unwrap();
                match prop {
                    Type::ModelProperty(p) => {
                        // string should resolve to a std type
                        let resolved = checker.get_type(p.r#type).cloned().unwrap();
                        assert!(
                            matches!(resolved, Type::Scalar(ref s) if s.name == "string"),
                            "Expected string scalar, got {:?}",
                            resolved.kind_name()
                        );
                    }
                    _ => panic!("Expected ModelProperty"),
                }
            }
            _ => panic!("Expected Model type"),
        }
    }

    // ==================== Tuple and Array ====================

    #[test]
    fn test_check_tuple_expression() {
        let checker = check("model Foo { x: [string, int32]; }");
        let foo_type = checker.declared_types.get("Foo").copied().unwrap();
        let t = checker.get_type(foo_type).cloned().unwrap();

        match t {
            Type::Model(m) => {
                let prop_type_id = m.properties.get("x").copied().unwrap();
                let prop = checker.get_type(prop_type_id).cloned().unwrap();
                match prop {
                    Type::ModelProperty(p) => {
                        let value_type = checker.get_type(p.r#type).cloned().unwrap();
                        assert!(
                            matches!(value_type, Type::Tuple(_)),
                            "Expected Tuple type, got {:?}",
                            value_type.kind_name()
                        );
                    }
                    _ => panic!("Expected ModelProperty"),
                }
            }
            _ => panic!("Expected Model type"),
        }
    }

    #[test]
    fn test_check_array_expression() {
        let checker = check("model Foo { x: string[]; }");
        let foo_type = checker.declared_types.get("Foo").copied().unwrap();
        let t = checker.get_type(foo_type).cloned().unwrap();

        match t {
            Type::Model(m) => {
                let prop_type_id = m.properties.get("x").copied().unwrap();
                let prop = checker.get_type(prop_type_id).cloned().unwrap();
                match prop {
                    Type::ModelProperty(p) => {
                        let value_type = checker.get_type(p.r#type).cloned().unwrap();
                        // Array is modeled as Model with indexer
                        assert!(
                            matches!(value_type, Type::Model(ref arr) if arr.name == "Array"),
                            "Expected Array model, got {:?}",
                            value_type.kind_name()
                        );
                    }
                    _ => panic!("Expected ModelProperty"),
                }
            }
            _ => panic!("Expected Model type"),
        }
    }

    // ==================== Void/Never/Unknown Keywords ====================

    #[test]
    fn test_void_keyword() {
        let checker = check("model Foo { x: void; }");
        let foo_type = checker.declared_types.get("Foo").copied().unwrap();
        let t = checker.get_type(foo_type).cloned().unwrap();

        match t {
            Type::Model(m) => {
                let prop_type_id = m.properties.get("x").copied().unwrap();
                let prop = checker.get_type(prop_type_id).cloned().unwrap();
                match prop {
                    Type::ModelProperty(p) => {
                        assert_eq!(p.r#type, checker.void_type);
                    }
                    _ => panic!("Expected ModelProperty"),
                }
            }
            _ => panic!("Expected Model type"),
        }
    }

    // ==================== Alias ====================

    #[test]
    fn test_check_alias() {
        let checker = check("alias MyStr = string;");
        let alias_type = checker.declared_types.get("MyStr").copied().unwrap();
        let t = checker.get_type(alias_type).cloned().unwrap();

        match t {
            Type::Scalar(s) => {
                assert_eq!(s.name, "MyStr");
                assert!(s.base_scalar.is_some());
            }
            other => panic!("Expected Scalar type (alias), got {:?}", other.kind_name()),
        }
    }
}

// ============================================================================
// T5.3: Template Instantiation Tests
// ============================================================================

#[cfg(test)]
mod template_instantiation_tests {

    use crate::checker::Type;
    use crate::checker::test_utils::check;

    // ==================== Model Template Instantiation ====================

    #[test]
    fn test_template_model_single_param() {
        let checker = check("model Pair<K, V> { key: K; value: V; }");
        // The template declaration itself should be registered
        let pair_type = checker.declared_types.get("Pair").copied();
        assert!(pair_type.is_some(), "Pair should be in declared_types");
    }

    #[test]
    fn test_template_model_instantiation() {
        let checker = check(
            "model Pair<K, V> { key: K; value: V; } model StringPair extends Pair<string, int32> {}",
        );
        // Check that StringPair was created
        let string_pair_type = checker.declared_types.get("StringPair").copied();
        assert!(
            string_pair_type.is_some(),
            "StringPair should be in declared_types"
        );

        // Check that Pair was created
        let pair_type = checker.declared_types.get("Pair").copied();
        assert!(pair_type.is_some(), "Pair should be in declared_types");
    }

    #[test]
    fn test_template_alias_instantiation() {
        let checker =
            check("alias MaybeUndefined<T> = T; model Foo { x: MaybeUndefined<string>; }");
        let foo_type = checker.declared_types.get("Foo").copied();
        assert!(foo_type.is_some(), "Foo should be in declared_types");
    }

    #[test]
    fn test_template_model_with_default() {
        // Template parameter with default value
        let checker = check("model Container<T, Wrapper = T> { value: T; wrapper: Wrapper; }");
        let container_type = checker.declared_types.get("Container").copied();
        assert!(
            container_type.is_some(),
            "Container should be in declared_types"
        );
    }

    // ==================== Template Parameter Resolution ====================

    #[test]
    fn test_template_param_resolved_in_model() {
        let checker = check("model Box<T> { content: T; }");
        let box_type = checker.declared_types.get("Box").copied().unwrap();
        let t = checker.get_type(box_type).cloned().unwrap();

        // The model should have one property 'content'
        match t {
            Type::Model(m) => {
                assert!(
                    m.properties.contains_key("content"),
                    "Box should have 'content' property"
                );
            }
            _ => panic!("Expected Model type, got {:?}", t.kind_name()),
        }
    }

    // ==================== Interface Template ====================

    #[test]
    fn test_template_interface() {
        let checker = check("interface Repository<T> { get(id: int32): T; set(item: T): void; }");
        let repo_type = checker.declared_types.get("Repository").copied();
        assert!(
            repo_type.is_some(),
            "Repository should be in declared_types"
        );
    }

    // ==================== Union Template ====================

    #[test]
    fn test_template_union() {
        let checker = check("union Result<T> { ok: T; error: string; }");
        let result_type = checker.declared_types.get("Result").copied();
        assert!(result_type.is_some(), "Result should be in declared_types");
    }

    // ==================== Operation Template ====================

    #[test]
    fn test_template_operation() {
        let _checker = check("op getItem<T>(id: int32): T;");
        // Operations are not in declared_types by default, just verify no errors
    }

    // ==================== Scalar Template ====================

    #[test]
    fn test_template_scalar() {
        let checker = check("scalar constrained<T> extends T;");
        let scalar_type = checker.declared_types.get("constrained").copied();
        assert!(
            scalar_type.is_some(),
            "constrained should be in declared_types"
        );
    }

    // ==================== Instantiation with Type Verification ====================

    #[test]
    fn test_template_model_instantiation_resolves_params() {
        // Model Pair<K, V> { key: K; value: V; }
        // model StringIntPair is Pair<string, int32> {}
        // Verify that when we reference Pair<string, int32>, a new type is instantiated
        let checker =
            check("model Pair<K, V> { key: K; value: V; } model MyPair is Pair<string, int32> {}");

        // MyPair should exist
        let my_pair_type = checker.declared_types.get("MyPair").copied().unwrap();
        let t = checker.get_type(my_pair_type).cloned().unwrap();
        match t {
            Type::Model(m) => {
                // The 'is' heritage should copy properties from Pair
                assert!(
                    m.source_model.is_some(),
                    "MyPair should have source_model from 'is'"
                );
            }
            other => panic!("Expected Model type, got {:?}", other.kind_name()),
        }
    }

    #[test]
    fn test_template_model_spread_instantiation() {
        // model Foo<T> { value: T; }
        // model Bar { ... Foo<string>; extra: int32; }
        let checker =
            check("model Foo<T> { value: T; } model Bar { ... Foo<string>; extra: int32; }");

        // Bar should exist
        let bar_type = checker.declared_types.get("Bar").copied().unwrap();
        let t = checker.get_type(bar_type).cloned().unwrap();
        match t {
            Type::Model(m) => {
                // Bar should have at least the 'extra' property
                assert!(
                    m.properties.contains_key("extra"),
                    "Bar should have 'extra' property"
                );
            }
            other => panic!("Expected Model type, got {:?}", other.kind_name()),
        }
    }

    #[test]
    fn test_nested_template_reference() {
        // Alias wrapping a template model
        let checker = check("model Box<T> { content: T; } model MyBox { box: Box<string>; }");

        let my_box_type = checker.declared_types.get("MyBox").copied().unwrap();
        let t = checker.get_type(my_box_type).cloned().unwrap();
        match t {
            Type::Model(m) => {
                assert!(
                    m.properties.contains_key("box"),
                    "MyBox should have 'box' property"
                );
            }
            other => panic!("Expected Model type, got {:?}", other.kind_name()),
        }
    }

    #[test]
    fn test_multiple_instantiations_same_template() {
        // Multiple instantiations of the same template with different args
        let checker = check(
            "model Container<T> { value: T; } model A { x: Container<string>; } model B { y: Container<int32>; }",
        );

        // Both A and B should be created
        assert!(checker.declared_types.contains_key("A"), "A should exist");
        assert!(checker.declared_types.contains_key("B"), "B should exist");
        assert!(
            checker.declared_types.contains_key("Container"),
            "Container should exist"
        );
    }
}

// ============================================================================
// T5.4: Decorator Checking Tests
// ============================================================================

#[cfg(test)]
mod decorator_checking_tests {

    use crate::checker::Type;
    use crate::checker::test_utils::check;

    // ==================== Decorator Application on Model ====================

    #[test]
    fn test_model_with_decorator_application() {
        let checker = check("@doc model Foo {}");
        let foo_type = checker.declared_types.get("Foo").copied().unwrap();
        let t = checker.get_type(foo_type).cloned().unwrap();
        match t {
            Type::Model(m) => {
                assert_eq!(m.decorators.len(), 1, "Foo should have 1 decorator");
            }
            other => panic!("Expected Model type, got {:?}", other.kind_name()),
        }
    }

    // ==================== Decorator on Other Types ====================

    // ==================== Decorator Declaration ====================

    #[test]
    fn test_decorator_declaration() {
        let checker = check("extern dec myDec(target: Type);");
        let dec_type = checker.declared_types.get("myDec").copied();
        assert!(dec_type.is_some(), "myDec should be in declared_types");

        let t = checker.get_type(dec_type.unwrap()).cloned().unwrap();
        match t {
            Type::Decorator(d) => {
                assert_eq!(d.name, "myDec");
            }
            other => panic!("Expected Decorator type, got {:?}", other.kind_name()),
        }
    }

    #[test]
    fn test_decorator_with_args() {
        let checker = check("@doc model Foo {}");
        let foo_type = checker.declared_types.get("Foo").copied().unwrap();
        let t = checker.get_type(foo_type).cloned().unwrap();
        match t {
            Type::Model(m) => {
                assert_eq!(m.decorators.len(), 1);
                // The decorator application should have the decorator node
                assert!(m.decorators[0].node.is_some());
            }
            other => panic!("Expected Model type, got {:?}", other.kind_name()),
        }
    }
}

// ============================================================================
// T5.2: Value vs Type Distinction Tests
// ============================================================================

#[cfg(test)]
mod value_tests {

    use crate::checker::test_utils::check;
    use crate::checker::{Checker, Value};
    use crate::parser::parse;

    // ==================== Entity Checking ====================

    #[test]
    fn test_string_literal_is_indeterminate() {
        // String literals in type context are types, but can be used as values
        let result = parse(r#"const x = "hello";"#);
        let mut checker = Checker::new();
        checker.set_parse_result(result.root_id, result.builder);
        checker.check_program();

        // Find the string literal node inside the const
        let const_value_id = *checker
            .declared_values
            .get("x")
            .expect("const x should have a value");
        let value = checker
            .get_value(const_value_id)
            .expect("should have value")
            .clone();
        match value {
            Value::StringValue(sv) => {
                assert_eq!(sv.value, "hello");
            }
            other => panic!("Expected StringValue, got {:?}", other.value_kind_name()),
        }
    }

    #[test]
    fn test_numeric_literal_value() {
        let result = parse("const x = 42;");
        let mut checker = Checker::new();
        checker.set_parse_result(result.root_id, result.builder);
        checker.check_program();

        let value_id = checker
            .declared_values
            .get("x")
            .copied()
            .expect("const x should have a value");
        let value = checker
            .get_value(value_id)
            .expect("should have value")
            .clone();
        match value {
            Value::NumericValue(nv) => {
                assert_eq!(nv.value, 42.0);
            }
            other => panic!("Expected NumericValue, got {:?}", other.value_kind_name()),
        }
    }

    #[test]
    fn test_boolean_literal_value() {
        let result = parse("const x = true;");
        let mut checker = Checker::new();
        checker.set_parse_result(result.root_id, result.builder);
        checker.check_program();

        let value_id = checker
            .declared_values
            .get("x")
            .copied()
            .expect("const x should have a value");
        let value = checker
            .get_value(value_id)
            .expect("should have value")
            .clone();
        match value {
            Value::BooleanValue(bv) => {
                assert!(bv.value);
            }
            other => panic!("Expected BooleanValue, got {:?}", other.value_kind_name()),
        }
    }

    #[test]
    fn test_const_with_string_value() {
        let checker = check(r#"const greeting = "hello";"#);
        let value_id = checker.declared_values.get("greeting").copied();
        assert!(value_id.is_some(), "const greeting should have a value");

        let value = checker.get_value(value_id.unwrap()).cloned().unwrap();
        match value {
            Value::StringValue(sv) => {
                assert_eq!(sv.value, "hello");
            }
            other => panic!("Expected StringValue, got {:?}", other.value_kind_name()),
        }
    }

    #[test]
    fn test_const_with_numeric_value() {
        let checker = check("const count = 100;");
        let value_id = checker.declared_values.get("count").copied();
        assert!(value_id.is_some(), "const count should have a value");

        let value = checker.get_value(value_id.unwrap()).cloned().unwrap();
        match value {
            Value::NumericValue(nv) => {
                assert_eq!(nv.value, 100.0);
            }
            other => panic!("Expected NumericValue, got {:?}", other.value_kind_name()),
        }
    }

    #[test]
    fn test_const_with_boolean_value() {
        let checker = check("const isActive = false;");
        let value_id = checker.declared_values.get("isActive").copied();
        assert!(value_id.is_some(), "const isActive should have a value");

        let value = checker.get_value(value_id.unwrap()).cloned().unwrap();
        match value {
            Value::BooleanValue(bv) => {
                assert!(!bv.value);
            }
            other => panic!("Expected BooleanValue, got {:?}", other.value_kind_name()),
        }
    }

    #[test]
    fn test_object_literal_value() {
        let checker = check("const obj = #{ name: \"test\", count: 1 };");
        let value_id = checker.declared_values.get("obj").copied();
        assert!(value_id.is_some(), "const obj should have a value");

        let value = checker.get_value(value_id.unwrap()).cloned().unwrap();
        match value {
            Value::ObjectValue(ov) => {
                assert_eq!(ov.properties.len(), 2);
                assert_eq!(ov.properties[0].name, "name");
                assert_eq!(ov.properties[1].name, "count");
            }
            other => panic!("Expected ObjectValue, got {:?}", other.value_kind_name()),
        }
    }

    #[test]
    fn test_array_literal_value() {
        let checker = check("const arr = #[1, 2, 3];");
        let value_id = checker.declared_values.get("arr").copied();
        assert!(value_id.is_some(), "const arr should have a value");

        let value = checker.get_value(value_id.unwrap()).cloned().unwrap();
        match value {
            Value::ArrayValue(av) => {
                assert_eq!(av.values.len(), 3);
            }
            other => panic!("Expected ArrayValue, got {:?}", other.value_kind_name()),
        }
    }

    #[test]
    fn test_get_value_for_node() {
        let result = parse(r#"const x = "hello";"#);
        let mut checker = Checker::new();
        checker.set_parse_result(result.root_id, result.builder);
        checker.check_program();

        // Verify the value is stored
        assert!(checker.declared_values.contains_key("x"));
    }

    #[test]
    fn test_entity_type_from_model() {
        // Model declarations produce Type entities
        let result = parse("model Foo {}");
        let mut checker = Checker::new();
        checker.set_parse_result(result.root_id, result.builder);
        checker.check_program();

        let foo_type = checker.declared_types.get("Foo").copied();
        assert!(foo_type.is_some(), "model Foo should have a type");
    }

    #[test]
    fn test_value_vs_type_distinction() {
        // Verify that type context returns TypeId and value context returns ValueId
        let checker = check(r#"const x = "hello"; model Foo { name: string; }"#);

        // Type context: Foo is a type
        assert!(checker.declared_types.contains_key("Foo"));

        // Value context: x is a value
        assert!(checker.declared_values.contains_key("x"));
    }

    #[test]
    fn test_multiple_consts() {
        let checker = check(r#"const a = "hello"; const b = 42; const c = true;"#);

        assert!(checker.declared_values.contains_key("a"));
        assert!(checker.declared_values.contains_key("b"));
        assert!(checker.declared_values.contains_key("c"));

        let av = checker
            .get_value(checker.declared_values.get("a").copied().unwrap())
            .cloned()
            .unwrap();
        match av {
            Value::StringValue(sv) => assert_eq!(sv.value, "hello"),
            _ => panic!("Expected StringValue"),
        }

        let bv = checker
            .get_value(checker.declared_values.get("b").copied().unwrap())
            .cloned()
            .unwrap();
        match bv {
            Value::NumericValue(nv) => assert_eq!(nv.value, 42.0),
            _ => panic!("Expected NumericValue"),
        }

        let cv = checker
            .get_value(checker.declared_values.get("c").copied().unwrap())
            .cloned()
            .unwrap();
        match cv {
            Value::BooleanValue(bv) => assert!(bv.value),
            _ => panic!("Expected BooleanValue"),
        }
    }
}

// ============================================================================
// T5.5: getEffectiveModelType Tests
// ============================================================================

#[cfg(test)]
mod effective_model_type_tests {

    use crate::checker::Type;
    use crate::checker::test_utils::check;

    #[test]
    fn test_named_model_returns_self() {
        // Named models are their own effective type
        let checker = check("model Foo { x: string; }");
        let foo_type = checker.declared_types.get("Foo").copied().unwrap();
        let effective = checker.get_effective_model_type(foo_type);
        assert_eq!(effective, foo_type, "Named model should return itself");
    }

    #[test]
    fn test_model_is_source_model() {
        // model Dog is Pet → Dog has source_model = Pet's instantiated type
        // getEffectiveModelType on the anonymous model from 'is' should return the source
        let checker = check("model Pet { name: string; } model Dog is Pet { breed: string; }");
        let dog_type = checker.declared_types.get("Dog").copied().unwrap();

        // Dog is a named model, so its effective type is itself
        let effective = checker.get_effective_model_type(dog_type);
        assert_eq!(effective, dog_type);

        // But the source_model should be set
        let t = checker.get_type(dog_type).cloned().unwrap();
        match t {
            Type::Model(m) => {
                assert!(
                    m.source_model.is_some(),
                    "Dog should have source_model from 'is'"
                );
            }
            _ => panic!("Expected Model type"),
        }
    }

    #[test]
    fn test_named_model_always_self() {
        // Even models with 'extends' should return themselves
        let checker = check("model Base {} model Derived extends Base {}");
        let derived_type = checker.declared_types.get("Derived").copied().unwrap();
        let effective = checker.get_effective_model_type(derived_type);
        assert_eq!(effective, derived_type);
    }

    #[test]
    fn test_empty_named_model_returns_self() {
        let checker = check("model Empty {}");
        let empty_type = checker.declared_types.get("Empty").copied().unwrap();
        let effective = checker.get_effective_model_type(empty_type);
        assert_eq!(effective, empty_type);
    }

    #[test]
    fn test_non_model_type_returns_self() {
        // getEffectiveModelType on a non-model type returns the input
        let checker = check("enum Status { active; }");
        let enum_type = checker.declared_types.get("Status").copied().unwrap();
        let effective = checker.get_effective_model_type(enum_type);
        assert_eq!(effective, enum_type);
    }
}
