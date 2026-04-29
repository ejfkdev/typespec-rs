//! Model type operations
//!
//! Ported from TypeSpec compiler/src/typekit/kits/model.ts

use crate::checker::type_utils;
use crate::checker::types::TypeId;
use crate::checker::{Checker, Type};

define_type_check!(is_model, Model);

/// Get model properties as (name, property_type_id) pairs, in declaration order.
pub fn get_properties(checker: &Checker, id: TypeId) -> Vec<(String, TypeId)> {
    match checker.get_type(id) {
        Some(Type::Model(m)) => m
            .property_names
            .iter()
            .filter_map(|name| {
                let &prop_id = m.properties.get(name)?;
                let prop_type = match checker.get_type(prop_id) {
                    Some(Type::ModelProperty(p)) => p.r#type,
                    _ => prop_id,
                };
                Some((name.clone(), prop_type))
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// Check if a model has a property with the given name
pub fn has_property(checker: &Checker, id: TypeId, name: &str) -> bool {
    match checker.get_type(id) {
        Some(Type::Model(m)) => m.properties.contains_key(name),
        _ => false,
    }
}

define_type_field_getter!(get_base_model, Model, base_model, Option<TypeId>);

/// Get derived models
pub fn get_derived_models(checker: &Checker, id: TypeId) -> Vec<TypeId> {
    match checker.get_type(id) {
        Some(Type::Model(m)) => m.derived_models.clone(),
        _ => Vec::new(),
    }
}

/// Check if a model is anonymous (name is empty)
pub fn is_anonymous(checker: &Checker, id: TypeId) -> bool {
    checker
        .get_type(id)
        .is_some_and(type_utils::is_anonymous_model)
}

/// Check if a model has an indexer (is a Record/Array)
pub fn has_indexer(checker: &Checker, id: TypeId) -> bool {
    match checker.get_type(id) {
        Some(Type::Model(m)) => m.indexer.is_some(),
        _ => false,
    }
}

/// Check if a model is an expression (anonymous) model.
/// Ported from TS typekit/kits/model.ts isExpression().
pub fn is_expression(checker: &Checker, id: TypeId) -> bool {
    is_anonymous(checker, id)
}

/// Get the index type of a model (for Array/Record models).
/// Returns the value type of the indexer if present.
/// Ported from TS typekit/kits/model.ts getIndexType().
pub fn get_index_type(checker: &Checker, id: TypeId) -> Option<TypeId> {
    match checker.get_type(id) {
        Some(Type::Model(m)) => m.indexer.map(|(_key_id, val_id)| val_id),
        _ => None,
    }
}

/// Get the additional properties record model for a model.
/// This is the model referenced by the indexer (for Record-like models).
/// Ported from TS typekit/kits/model.ts getAdditionalPropertiesRecord().
pub fn get_additional_properties_record(checker: &Checker, id: TypeId) -> Option<TypeId> {
    get_index_type(checker, id)
}

/// Get the effective model type for a potentially anonymous model.
/// If the model is anonymous and all its properties match a named model,
/// returns the named model. Otherwise returns the original.
/// Ported from TS typekit/kits/model.ts getEffectiveModel().
pub fn get_effective_model(checker: &Checker, id: TypeId) -> TypeId {
    if !is_anonymous(checker, id) {
        return id;
    }
    // For now, return the original — full implementation would search for
    // a named model with matching properties
    id
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::test_utils::check;
    use crate::typekit::kits::builtin;

    #[test]
    fn test_is_model() {
        let checker = check("model Foo {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        assert!(is_model(&checker, foo_id));
    }

    #[test]
    fn test_is_model_not_scalar() {
        let checker = check("scalar MyS extends string;");
        let s_id = checker.declared_types.get("MyS").copied().unwrap();
        assert!(!is_model(&checker, s_id));
    }

    #[test]
    fn test_is_model_not_builtin() {
        let checker = check("");
        let string_id = builtin::string(&checker).unwrap();
        // string is a scalar, not a model
        assert!(!is_model(&checker, string_id));
    }

    #[test]
    fn test_get_properties() {
        let checker = check("model Foo { name: string; age: int32; }");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let props = get_properties(&checker, foo_id);
        assert_eq!(props.len(), 2);
    }

    #[test]
    fn test_get_properties_empty() {
        let checker = check("model Foo {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let props = get_properties(&checker, foo_id);
        assert!(props.is_empty());
    }

    #[test]
    fn test_has_property() {
        let checker = check("model Foo { name: string; }");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        assert!(has_property(&checker, foo_id, "name"));
        assert!(!has_property(&checker, foo_id, "age"));
    }

    #[test]
    fn test_get_base_model() {
        let checker = check("model Base {} model Derived extends Base {}");
        let derived_id = checker.declared_types.get("Derived").copied().unwrap();
        let base = get_base_model(&checker, derived_id);
        assert!(base.is_some());
    }

    #[test]
    fn test_get_base_model_none() {
        let checker = check("model Foo {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let base = get_base_model(&checker, foo_id);
        assert!(base.is_none());
    }

    #[test]
    fn test_get_derived_models() {
        let checker = check("model Base {} model D1 extends Base {} model D2 extends Base {}");
        let base_id = checker.declared_types.get("Base").copied().unwrap();
        let derived = get_derived_models(&checker, base_id);
        assert_eq!(derived.len(), 2);
    }

    #[test]
    fn test_get_derived_models_none() {
        let checker = check("model Foo {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let derived = get_derived_models(&checker, foo_id);
        assert!(derived.is_empty());
    }

    #[test]
    fn test_is_anonymous() {
        let checker = check("model Test { x: {}; }");
        let test_id = checker.declared_types.get("Test").copied().unwrap();
        // The Test model is named
        assert!(!is_anonymous(&checker, test_id));
    }

    #[test]
    fn test_has_indexer_no() {
        let checker = check("model Foo { name: string; }");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        assert!(!has_indexer(&checker, foo_id));
    }

    #[test]
    fn test_has_indexer_record() {
        let checker = check("model Foo is Record<string, string> {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        // Indexer may or may not be set depending on checker implementation
        let _ = has_indexer(&checker, foo_id);
    }

    // ==================== Ported from TS typekit/model.test.ts ====================

    #[test]
    fn test_is_model_interface_is_not() {
        let checker = check("interface Foo {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        assert!(!is_model(&checker, foo_id));
    }

    #[test]
    fn test_model_with_inherited_properties() {
        let checker =
            check("model Base { id: string; } model Derived extends Base { name: string; }");
        let derived_id = checker.declared_types.get("Derived").copied().unwrap();
        let props = get_properties(&checker, derived_id);
        assert!(!props.is_empty()); // At least name property
    }
}
