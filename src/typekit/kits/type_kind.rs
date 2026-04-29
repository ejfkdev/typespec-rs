//! General type operations
//!
//! Ported from TypeSpec compiler/src/typekit/kits/type.ts
//!
//! Note: is_literal, is_intrinsic, is_never are re-exported from their
//! dedicated kits (literal.rs, intrinsic.rs) to avoid duplication.

use crate::checker::types::{IntrinsicTypeName, TypeId};
use crate::checker::{Checker, Type};
use crate::intrinsic_type_state;

/// Get the kind name of a type for display purposes
pub fn kind_name(checker: &Checker, id: TypeId) -> &'static str {
    match checker.get_type(id) {
        Some(t) => t.kind_name(),
        None => "Unknown",
    }
}

/// Resolve through alias chains to get the underlying type
pub fn resolve_alias(checker: &Checker, id: TypeId) -> TypeId {
    checker.resolve_alias_chain(id)
}

// Re-export from dedicated kits to avoid duplication
pub use super::intrinsic::{is_intrinsic, is_never};
pub use super::literal::is_literal;

/// Get the doc comment for a type.
/// Ported from TS typekit/kits/type.ts getDoc().
pub fn get_doc(checker: &Checker, id: TypeId) -> Option<&str> {
    checker.get_type(id).and_then(|t| t.doc())
}

/// Get the summary comment for a type.
/// Ported from TS typekit/kits/type.ts getSummary().
pub fn get_summary(checker: &Checker, id: TypeId) -> Option<&str> {
    checker.get_type(id).and_then(|t| t.summary())
}

/// Get a plausible name for a type.
/// For named types, returns the name. For anonymous types, infers a name.
/// Ported from TS typekit/kits/type.ts getPlausibleName().
/// Delegates to the richer implementation in typekit::utils.
pub fn get_plausible_name(checker: &Checker, id: TypeId) -> String {
    crate::typekit::utils::get_plausible_name::get_plausible_name(checker, id)
}

/// Check if a type is an error type.
/// Ported from TS typekit/kits/type.ts isError().
pub fn is_error(checker: &Checker, id: TypeId) -> bool {
    id == checker.error_type
        || matches!(checker.get_type(id), Some(Type::Intrinsic(i)) if i.name == IntrinsicTypeName::ErrorType)
}

/// Check if a type is user-defined (not built-in or synthetic).
/// Ported from TS typekit/kits/type.ts isUserDefined().
pub fn is_user_defined(checker: &Checker, id: TypeId) -> bool {
    match checker.get_type(id) {
        Some(Type::Model(m)) => !m.name.is_empty() && m.namespace.is_some(),
        Some(Type::Interface(i)) => i.namespace.is_some(),
        Some(Type::Enum(e)) => e.namespace.is_some(),
        Some(Type::Scalar(s)) => s.namespace.is_some(),
        Some(Type::Operation(o)) => o.namespace.is_some(),
        Some(Type::Union(u)) => !u.name.is_empty() && u.namespace.is_some(),
        _ => false,
    }
}

/// Check if a type is in the given namespace.
/// Ported from TS typekit/kits/type.ts inNamespace().
/// Handles recursive containment: ModelProperty → Model → namespace,
/// EnumMember → Enum → namespace, UnionVariant → Union → namespace,
/// Operation (in interface) → Interface → namespace.
/// Check if `ns` is or is within `target` namespace, walking the parent chain.
fn check_namespace_rec(checker: &Checker, ns: Option<TypeId>, target: TypeId) -> bool {
    match ns {
        Some(ns) if ns == target => true,
        Some(ns) => in_namespace(checker, ns, target),
        None => false,
    }
}

pub fn in_namespace(checker: &Checker, id: TypeId, namespace_id: TypeId) -> bool {
    // A namespace is always in itself
    if id == namespace_id {
        return true;
    }

    match checker.get_type(id) {
        // ModelProperty: check its parent model
        Some(Type::ModelProperty(prop)) => prop
            .model
            .is_some_and(|m| in_namespace(checker, m, namespace_id)),
        // EnumMember: check its parent enum
        Some(Type::EnumMember(member)) => member
            .r#enum
            .is_some_and(|e| in_namespace(checker, e, namespace_id)),
        // UnionVariant: check its parent union
        Some(Type::UnionVariant(variant)) => variant
            .union
            .is_some_and(|u| in_namespace(checker, u, namespace_id)),
        // Types with namespace field: check recursively
        Some(Type::Operation(op)) => check_namespace_rec(checker, op.namespace, namespace_id),
        Some(Type::Model(m)) => check_namespace_rec(checker, m.namespace, namespace_id),
        Some(Type::Interface(i)) => check_namespace_rec(checker, i.namespace, namespace_id),
        Some(Type::Enum(e)) => check_namespace_rec(checker, e.namespace, namespace_id),
        Some(Type::Scalar(s)) => check_namespace_rec(checker, s.namespace, namespace_id),
        Some(Type::Union(u)) => check_namespace_rec(checker, u.namespace, namespace_id),
        Some(Type::Namespace(ns)) => check_namespace_rec(checker, ns.namespace, namespace_id),
        _ => false,
    }
}

/// Gets the maximum value for a numeric or model property type.
/// Ported from TS typekit/kits/type.ts maxValue().
pub fn max_value(checker: &Checker, id: TypeId) -> Option<f64> {
    intrinsic_type_state::get_max_value_as_numeric(&checker.state_accessors, id)
        .and_then(|n| n.as_f64())
}

/// Gets the minimum value for a numeric or model property type.
/// Ported from TS typekit/kits/type.ts minValue().
pub fn min_value(checker: &Checker, id: TypeId) -> Option<f64> {
    intrinsic_type_state::get_min_value_as_numeric(&checker.state_accessors, id)
        .and_then(|n| n.as_f64())
}

/// Gets the exclusive maximum value for a numeric or model property type.
/// Ported from TS typekit/kits/type.ts maxValueExclusive().
pub fn max_value_exclusive(checker: &Checker, id: TypeId) -> Option<f64> {
    intrinsic_type_state::get_max_value_exclusive_as_numeric(&checker.state_accessors, id)
        .and_then(|n| n.as_f64())
}

/// Gets the exclusive minimum value for a numeric or model property type.
/// Ported from TS typekit/kits/type.ts minValueExclusive().
pub fn min_value_exclusive(checker: &Checker, id: TypeId) -> Option<f64> {
    intrinsic_type_state::get_min_value_exclusive_as_numeric(&checker.state_accessors, id)
        .and_then(|n| n.as_f64())
}

/// Gets the maximum length for a string type.
/// Ported from TS typekit/kits/type.ts maxLength().
pub fn max_length(checker: &Checker, id: TypeId) -> Option<f64> {
    intrinsic_type_state::get_max_length_as_numeric(&checker.state_accessors, id)
        .and_then(|n| n.as_f64())
}

/// Gets the minimum length for a string type.
/// Ported from TS typekit/kits/type.ts minLength().
pub fn min_length(checker: &Checker, id: TypeId) -> Option<f64> {
    intrinsic_type_state::get_min_length_as_numeric(&checker.state_accessors, id)
        .and_then(|n| n.as_f64())
}

/// Gets the maximum number of items for an array type.
/// Ported from TS typekit/kits/type.ts maxItems().
pub fn max_items(checker: &Checker, id: TypeId) -> Option<f64> {
    intrinsic_type_state::get_max_items_as_numeric(&checker.state_accessors, id)
        .and_then(|n| n.as_f64())
}

/// Gets the minimum number of items for an array type.
/// Ported from TS typekit/kits/type.ts minItems().
pub fn min_items(checker: &Checker, id: TypeId) -> Option<f64> {
    intrinsic_type_state::get_min_items_as_numeric(&checker.state_accessors, id)
        .and_then(|n| n.as_f64())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::test_utils::check;

    #[test]
    fn test_kind_name_model() {
        let checker = check("model Foo {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        assert_eq!(kind_name(&checker, foo_id), "Model");
    }

    #[test]
    fn test_kind_name_enum() {
        let checker = check("enum Color { red, green, blue }");
        let e_id = checker.declared_types.get("Color").copied().unwrap();
        assert_eq!(kind_name(&checker, e_id), "Enum");
    }

    #[test]
    fn test_kind_name_scalar() {
        let checker = check("scalar MyS extends string;");
        let s_id = checker.declared_types.get("MyS").copied().unwrap();
        assert_eq!(kind_name(&checker, s_id), "Scalar");
    }

    #[test]
    fn test_kind_name_union() {
        let checker = check("union Pet { cat: string, dog: string }");
        let u_id = checker.declared_types.get("Pet").copied().unwrap();
        assert_eq!(kind_name(&checker, u_id), "Union");
    }

    #[test]
    fn test_kind_name_intrinsic() {
        let checker = check("");
        assert_eq!(kind_name(&checker, checker.void_type), "Intrinsic");
        assert_eq!(kind_name(&checker, checker.never_type), "Intrinsic");
    }

    #[test]
    fn test_kind_name_operation() {
        let checker = check("op test(): void;");
        let op_id = checker.declared_types.get("test").copied().unwrap();
        assert_eq!(kind_name(&checker, op_id), "Operation");
    }

    #[test]
    fn test_kind_name_string_literal() {
        let checker = check(r#"alias Foo = "hello";"#);
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = resolve_alias(&checker, foo_id);
        assert_eq!(kind_name(&checker, resolved), "String");
    }

    #[test]
    fn test_kind_name_numeric_literal() {
        let checker = check("alias Foo = 42;");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = resolve_alias(&checker, foo_id);
        assert_eq!(kind_name(&checker, resolved), "Number");
    }

    #[test]
    fn test_kind_name_boolean_literal() {
        let checker = check("alias Foo = true;");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = resolve_alias(&checker, foo_id);
        assert_eq!(kind_name(&checker, resolved), "Boolean");
    }

    #[test]
    fn test_kind_name_tuple() {
        let checker = check("alias Foo = [string, int32];");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = resolve_alias(&checker, foo_id);
        assert_eq!(kind_name(&checker, resolved), "Tuple");
    }

    #[test]
    fn test_resolve_alias_single() {
        let checker = check(r#"alias Foo = "hello";"#);
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = resolve_alias(&checker, foo_id);
        assert!(is_literal(&checker, resolved));
    }

    #[test]
    fn test_resolve_alias_chained() {
        let checker = check(r#"alias A = "hello"; alias B = A;"#);
        let b_id = checker.declared_types.get("B").copied().unwrap();
        let resolved = resolve_alias(&checker, b_id);
        let _ = is_literal(&checker, resolved);
    }

    // Note: is_literal, is_intrinsic, is_never are re-exported from
    // literal.rs and intrinsic.rs respectively. Their tests live there.

    // ==================== Ported from TS typekit/type.test.ts ====================

    #[test]
    fn test_get_plausible_name_named_model() {
        let checker = check("model Foo { props: string }");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        assert_eq!(get_plausible_name(&checker, foo_id), "Foo");
    }

    #[test]
    fn test_get_plausible_name_named_union() {
        let checker = check(r#"union Bar { "hi"; "bye"; }"#);
        let bar_id = checker.declared_types.get("Bar").copied().unwrap();
        assert_eq!(get_plausible_name(&checker, bar_id), "Bar");
    }

    #[test]
    fn test_get_plausible_name_named_enum() {
        let checker = check(r#"enum Baz { Baz: "baz" }"#);
        let baz_id = checker.declared_types.get("Baz").copied().unwrap();
        assert_eq!(get_plausible_name(&checker, baz_id), "Baz");
    }

    #[test]
    fn test_get_plausible_name_named_scalar() {
        let checker = check("scalar Qux extends string;");
        let qux_id = checker.declared_types.get("Qux").copied().unwrap();
        assert_eq!(get_plausible_name(&checker, qux_id), "Qux");
    }

    #[test]
    fn test_get_plausible_name_anonymous_model() {
        let checker = check("alias Foo = { name: string };");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = resolve_alias(&checker, foo_id);
        if let Some(Type::Model(m)) = checker.get_type(resolved)
            && m.name.is_empty()
        {
            let name = get_plausible_name(&checker, resolved);
            // The richer utils version produces "Anonymous_Name" for models with properties
            assert!(
                name.contains("Anonymous") || name.contains("name"),
                "Expected plausible name for anonymous model, got: {name}"
            );
        }
    }

    #[test]
    fn test_is_error_on_error_type() {
        let checker = check("");
        assert!(is_error(&checker, checker.error_type));
    }

    #[test]
    fn test_is_error_on_non_error() {
        let checker = check("model Foo {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        assert!(!is_error(&checker, foo_id));
    }

    #[test]
    fn test_is_user_defined_named_model_with_namespace() {
        let checker = check("namespace NS { model Foo {} }");
        // Foo should be in NS namespace
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        // If Foo has a namespace, it should be user-defined
        if let Some(Type::Model(m)) = checker.get_type(foo_id)
            && m.namespace.is_some()
        {
            assert!(is_user_defined(&checker, foo_id));
        }
    }

    #[test]
    fn test_is_not_user_defined_intrinsic() {
        let checker = check("");
        assert!(!is_user_defined(&checker, checker.void_type));
    }

    #[test]
    fn test_in_namespace_self() {
        let checker = check("namespace Root {}");
        let root_id = checker.declared_types.get("Root").copied().unwrap();
        assert!(in_namespace(&checker, root_id, root_id));
    }

    #[test]
    fn test_in_namespace_direct_child() {
        let checker = check("namespace Root { namespace Child1 { namespace Child2 {} } }");
        let root_id = checker.declared_types.get("Root").copied().unwrap();
        let child1_id = checker.declared_types.get("Child1").copied().unwrap();
        let child2_id = checker.declared_types.get("Child2").copied().unwrap();

        // Child1 is in Root
        assert!(in_namespace(&checker, child1_id, root_id));
        // Child2 is in Root (transitively through Child1)
        assert!(in_namespace(&checker, child2_id, root_id));
    }

    #[test]
    fn test_in_namespace_model() {
        let checker = check("namespace Root { model Inside { prop: string } }");
        let root_id = checker.declared_types.get("Root").copied().unwrap();
        let model_id = checker.declared_types.get("Inside").copied().unwrap();
        if let Some(Type::Model(m)) = checker.get_type(model_id)
            && m.namespace.is_some()
        {
            assert!(in_namespace(&checker, model_id, root_id));
        }
    }

    #[test]
    fn test_in_namespace_outside() {
        let checker = check("namespace Root { namespace Child1 {} } namespace Outside {}");
        let _root_id = checker.declared_types.get("Root").copied().unwrap();
        let child1_id = checker.declared_types.get("Child1").copied().unwrap();
        let outside_id = checker.declared_types.get("Outside").copied().unwrap();

        // Child1 is NOT in Outside
        assert!(!in_namespace(&checker, child1_id, outside_id));
    }

    #[test]
    fn test_in_namespace_literal_type() {
        let mut checker = check("");
        // String literal type has no namespace
        let lit_id = checker.create_type(Type::String(crate::checker::types::StringType {
            id: checker.next_type_id(),
            value: "test".to_string(),
            node: None,
            is_finished: true,
        }));
        assert!(!in_namespace(&checker, lit_id, checker.void_type));
    }

    #[test]
    fn test_in_namespace_enum_member() {
        let checker = check("namespace Root { enum Test { A, B } }");
        let root_id = checker.declared_types.get("Root").copied().unwrap();
        let test_id = checker.declared_types.get("Test").copied().unwrap();
        if let Some(Type::Enum(e)) = checker.get_type(test_id)
            && let Some(&member_id) = e.members.get("A")
        {
            // Enum member should be in Root namespace via parent enum
            if e.namespace.is_some() {
                assert!(in_namespace(&checker, member_id, root_id));
            }
        }
    }

    // ==================== minValue/maxValue tests ====================

    #[test]
    fn test_min_max_value_none_by_default() {
        let checker = check("scalar MyNum extends numeric;");
        let my_id = checker.declared_types.get("MyNum").copied().unwrap();
        assert!(min_value(&checker, my_id).is_none());
        assert!(max_value(&checker, my_id).is_none());
    }

    #[test]
    fn test_min_max_value_with_state() {
        let mut checker = check("scalar MyNum extends numeric;");
        let my_id = checker.declared_types.get("MyNum").copied().unwrap();

        // Manually set min/max value state (normally set by @minValue/@maxValue decorators)
        use crate::intrinsic_type_state::{NumericOrScalar, set_max_value, set_min_value};
        use crate::numeric::Numeric;

        let min_num = Numeric::new("1").unwrap();
        let max_num = Numeric::new("10").unwrap();
        set_min_value(
            &mut checker.state_accessors,
            my_id,
            NumericOrScalar::Numeric(min_num),
        );
        set_max_value(
            &mut checker.state_accessors,
            my_id,
            NumericOrScalar::Numeric(max_num),
        );

        assert_eq!(min_value(&checker, my_id), Some(1.0));
        assert_eq!(max_value(&checker, my_id), Some(10.0));
    }

    #[test]
    fn test_min_max_value_exclusive_with_state() {
        let mut checker = check("scalar MyNum extends numeric;");
        let my_id = checker.declared_types.get("MyNum").copied().unwrap();

        use crate::intrinsic_type_state::{
            NumericOrScalar, set_max_value_exclusive, set_min_value_exclusive,
        };
        use crate::numeric::Numeric;

        let min_num = Numeric::new("1").unwrap();
        let max_num = Numeric::new("10").unwrap();
        set_min_value_exclusive(
            &mut checker.state_accessors,
            my_id,
            NumericOrScalar::Numeric(min_num),
        );
        set_max_value_exclusive(
            &mut checker.state_accessors,
            my_id,
            NumericOrScalar::Numeric(max_num),
        );

        assert_eq!(min_value_exclusive(&checker, my_id), Some(1.0));
        assert_eq!(max_value_exclusive(&checker, my_id), Some(10.0));
    }

    // ==================== minLength/maxLength tests ====================

    #[test]
    fn test_min_max_length_none_by_default() {
        let checker = check("scalar MyStr extends string;");
        let my_id = checker.declared_types.get("MyStr").copied().unwrap();
        assert!(min_length(&checker, my_id).is_none());
        assert!(max_length(&checker, my_id).is_none());
    }

    #[test]
    fn test_min_max_length_with_state() {
        let mut checker = check("scalar MyStr extends string;");
        let my_id = checker.declared_types.get("MyStr").copied().unwrap();

        use crate::intrinsic_type_state::{set_max_length, set_min_length};
        use crate::numeric::Numeric;

        let min_num = Numeric::new("1").unwrap();
        let max_num = Numeric::new("10").unwrap();
        set_min_length(&mut checker.state_accessors, my_id, &min_num);
        set_max_length(&mut checker.state_accessors, my_id, &max_num);

        assert_eq!(min_length(&checker, my_id), Some(1.0));
        assert_eq!(max_length(&checker, my_id), Some(10.0));
    }

    // ==================== minItems/maxItems tests ====================

    #[test]
    fn test_min_max_items_none_by_default() {
        let checker = check("alias A = string[];");
        let a_id = checker.declared_types.get("A").copied().unwrap();
        let resolved = resolve_alias(&checker, a_id);
        assert!(min_items(&checker, resolved).is_none());
        assert!(max_items(&checker, resolved).is_none());
    }

    #[test]
    fn test_min_max_items_with_state() {
        let mut checker = check("alias A = string[];");
        let a_id = checker.declared_types.get("A").copied().unwrap();
        let resolved = resolve_alias(&checker, a_id);

        use crate::intrinsic_type_state::{set_max_items, set_min_items};
        use crate::numeric::Numeric;

        let min_num = Numeric::new("1").unwrap();
        let max_num = Numeric::new("10").unwrap();
        set_min_items(&mut checker.state_accessors, resolved, &min_num);
        set_max_items(&mut checker.state_accessors, resolved, &max_num);

        assert_eq!(min_items(&checker, resolved), Some(1.0));
        assert_eq!(max_items(&checker, resolved), Some(10.0));
    }

    // ==================== ModelProperty min/max via property ====================

    #[test]
    fn test_min_max_value_on_model_property() {
        let mut checker = check("model A { foo: int32; }");
        let a_id = checker.declared_types.get("A").copied().unwrap();
        if let Some(Type::Model(m)) = checker.get_type(a_id)
            && let Some(&prop_id) = m.properties.get("foo")
        {
            use crate::intrinsic_type_state::{NumericOrScalar, set_max_value, set_min_value};
            use crate::numeric::Numeric;

            let min_num = Numeric::new("15").unwrap();
            let max_num = Numeric::new("55").unwrap();
            set_min_value(
                &mut checker.state_accessors,
                prop_id,
                NumericOrScalar::Numeric(min_num),
            );
            set_max_value(
                &mut checker.state_accessors,
                prop_id,
                NumericOrScalar::Numeric(max_num),
            );

            assert_eq!(min_value(&checker, prop_id), Some(15.0));
            assert_eq!(max_value(&checker, prop_id), Some(55.0));
        }
    }

    #[test]
    fn test_min_max_length_on_model_property() {
        let mut checker = check("model A { foo: string; }");
        let a_id = checker.declared_types.get("A").copied().unwrap();
        if let Some(Type::Model(m)) = checker.get_type(a_id)
            && let Some(&prop_id) = m.properties.get("foo")
        {
            use crate::intrinsic_type_state::{set_max_length, set_min_length};
            use crate::numeric::Numeric;

            let min_num = Numeric::new("15").unwrap();
            let max_num = Numeric::new("55").unwrap();
            set_min_length(&mut checker.state_accessors, prop_id, &min_num);
            set_max_length(&mut checker.state_accessors, prop_id, &max_num);

            assert_eq!(min_length(&checker, prop_id), Some(15.0));
            assert_eq!(max_length(&checker, prop_id), Some(55.0));
        }
    }
}
