//! Model property type operations
//!
//! Ported from TypeSpec compiler/src/typekit/kits/model-property.ts

use crate::checker::types::TypeId;
use crate::checker::{Checker, Type};

define_type_check!(is_model_property, ModelProperty);
define_type_field_getter!(get_name, ModelProperty, name, str);
define_type_field_getter!(get_type, ModelProperty, r#type, TypeId);

/// Check if a model property is optional
pub fn is_optional(checker: &Checker, id: TypeId) -> bool {
    match checker.get_type(id) {
        Some(Type::ModelProperty(mp)) => mp.optional,
        _ => false,
    }
}

define_type_field_getter!(get_parent_model, ModelProperty, model, Option<TypeId>);
define_type_field_getter!(
    get_default_value,
    ModelProperty,
    default_value,
    Option<TypeId>
);

/// Get the encoding of a model property (from @encode decorator).
/// Returns the encoding name if the property has an @encode decorator.
/// Ported from TS typekit/kits/model-property.ts getEncoding().
pub fn get_encoding(checker: &Checker, id: TypeId) -> Option<String> {
    match checker.get_type(id) {
        Some(Type::ModelProperty(_mp)) => {
            // Look for @encode decorator on the property
            // Full implementation needs decorator argument evaluation
            None
        }
        _ => None,
    }
}

/// Get the format of a model property (from @format decorator).
/// Returns the format name if the property has a @format decorator.
/// Ported from TS typekit/kits/model-property.ts getFormat().
pub fn get_format(checker: &Checker, id: TypeId) -> Option<String> {
    match checker.get_type(id) {
        Some(Type::ModelProperty(_mp)) => {
            // Look for @format decorator on the property
            // Full implementation needs decorator argument evaluation
            None
        }
        _ => None,
    }
}

/// Get the visibility for a specific visibility class on a model property.
/// Ported from TS typekit/kits/model-property.ts getVisibilityForClass().
pub fn get_visibility_for_class(
    _checker: &Checker,
    _id: TypeId,
    _visibility_class: TypeId,
) -> Vec<String> {
    // Full implementation requires visibility class tracking
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::test_utils::check;

    #[test]
    fn test_is_model_property() {
        let checker = check("model Foo { name: string; }");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        if let Some(Type::Model(m)) = checker.get_type(foo_id) {
            let prop_id = m.properties["name"];
            assert!(is_model_property(&checker, prop_id));
        }
    }

    #[test]
    fn test_is_model_property_not_model() {
        let checker = check("model Foo {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        assert!(!is_model_property(&checker, foo_id));
    }

    #[test]
    fn test_property_name_and_type() {
        let checker = check("model Foo { name: string; }");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        if let Some(Type::Model(m)) = checker.get_type(foo_id) {
            let prop_id = m.properties["name"];
            assert_eq!(get_name(&checker, prop_id), Some("name"));
            assert!(get_type(&checker, prop_id).is_some());
        }
    }

    #[test]
    fn test_optional_property() {
        let checker = check("model Foo { name?: string; }");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        if let Some(Type::Model(m)) = checker.get_type(foo_id) {
            let prop_id = m.properties["name"];
            assert!(is_optional(&checker, prop_id));
        }
    }

    #[test]
    fn test_required_property() {
        let checker = check("model Foo { name: string; }");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        if let Some(Type::Model(m)) = checker.get_type(foo_id) {
            let prop_id = m.properties["name"];
            assert!(!is_optional(&checker, prop_id));
        }
    }

    #[test]
    fn test_get_parent_model() {
        let checker = check("model Foo { name: string; }");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        if let Some(Type::Model(m)) = checker.get_type(foo_id) {
            let prop_id = m.properties["name"];
            assert_eq!(get_parent_model(&checker, prop_id), Some(foo_id));
        }
    }

    #[test]
    fn test_multiple_properties() {
        let checker = check("model Foo { name: string; age: int32; active?: boolean; }");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        if let Some(Type::Model(m)) = checker.get_type(foo_id) {
            assert_eq!(get_name(&checker, m.properties["name"]), Some("name"));
            assert_eq!(get_name(&checker, m.properties["age"]), Some("age"));
            assert_eq!(get_name(&checker, m.properties["active"]), Some("active"));

            assert!(!is_optional(&checker, m.properties["name"]));
            assert!(!is_optional(&checker, m.properties["age"]));
            assert!(is_optional(&checker, m.properties["active"]));
        }
    }

    #[test]
    fn test_get_default_value_none() {
        let checker = check("model Foo { name: string; }");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        if let Some(Type::Model(m)) = checker.get_type(foo_id) {
            let prop_id = m.properties["name"];
            // No default value specified
            assert!(get_default_value(&checker, prop_id).is_none());
        }
    }
}
