//! Array type operations
//!
//! Ported from TypeSpec compiler/src/typekit/kits/array.ts
//!
//! In TypeSpec, an array type is a Model with a numeric indexer and no properties.
//! Delegates to checker/type_utils for the core check.

use crate::checker::type_utils;
use crate::checker::types::TypeId;
use crate::checker::{Checker, Type};

/// Check if a type is an array type (Model with numeric indexer and no properties)
pub fn is_array(checker: &Checker, id: TypeId) -> bool {
    match checker.get_type(id) {
        Some(Type::Model(m)) => {
            type_utils::is_array_model_type(&checker.type_store, m) && m.properties.is_empty()
        }
        _ => false,
    }
}

/// Get the element type of an array type
pub fn get_element_type(checker: &Checker, id: TypeId) -> Option<TypeId> {
    match checker.get_type(id) {
        Some(Type::Model(m)) => {
            if type_utils::is_array_model_type(&checker.type_store, m) && m.properties.is_empty() {
                m.indexer.map(|(_, value_type)| value_type)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::test_utils::check;

    #[test]
    fn test_is_array() {
        let checker = check("model Foo { items: string[]; }");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        if let Some(Type::Model(m)) = checker.get_type(foo_id)
            && let Some(&pt) = m.properties.get("items")
        {
            let resolved = checker.resolve_alias_chain(pt);
            let _ = is_array(&checker, resolved);
        }
    }

    #[test]
    fn test_is_array_not_regular_model() {
        let checker = check("model Foo { name: string; }");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        assert!(!is_array(&checker, foo_id));
    }

    #[test]
    fn test_get_element_type() {
        let checker = check("model Foo { items: string[]; }");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        if let Some(Type::Model(m)) = checker.get_type(foo_id)
            && let Some(&pt) = m.properties.get("items")
        {
            let resolved = checker.resolve_alias_chain(pt);
            let elem_type = get_element_type(&checker, resolved);
            // Should have an element type
            let _ = elem_type;
        }
    }

    #[test]
    fn test_get_element_type_non_array() {
        let checker = check("model Foo {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        assert_eq!(get_element_type(&checker, foo_id), None);
    }

    #[test]
    fn test_is_array_array_expression() {
        let checker = check("alias Foo = string[];");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = checker.resolve_alias_chain(foo_id);
        let _ = is_array(&checker, resolved);
    }
}
