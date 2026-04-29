//! Tuple type operations
//!
//! Ported from TypeSpec compiler/src/typekit/kits/tuple.ts

use crate::checker::types::TypeId;
use crate::checker::{Checker, Type};

define_type_check!(is_tuple, Tuple);

/// Get the element types of a tuple
pub fn get_values(checker: &Checker, id: TypeId) -> Vec<TypeId> {
    match checker.get_type(id) {
        Some(Type::Tuple(t)) => t.values.clone(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::test_utils::check;

    #[test]
    fn test_is_tuple() {
        let checker = check("alias Foo = [string, int32];");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = checker.resolve_alias_chain(foo_id);
        assert!(is_tuple(&checker, resolved));
    }

    #[test]
    fn test_tuple_values() {
        let checker = check("alias Foo = [string, int32];");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = checker.resolve_alias_chain(foo_id);
        let values = get_values(&checker, resolved);
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn test_is_tuple_not_model() {
        let checker = check("model Foo {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        assert!(!is_tuple(&checker, foo_id));
    }

    #[test]
    fn test_tuple_three_values() {
        let checker = check("alias Foo = [string, int32, boolean];");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = checker.resolve_alias_chain(foo_id);
        let values = get_values(&checker, resolved);
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_tuple_single_value() {
        let checker = check("alias Foo = [string];");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = checker.resolve_alias_chain(foo_id);
        assert!(is_tuple(&checker, resolved));
        let values = get_values(&checker, resolved);
        assert_eq!(values.len(), 1);
    }

    #[test]
    fn test_get_values_non_tuple() {
        let checker = check("model Foo {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let values = get_values(&checker, foo_id);
        assert!(values.is_empty());
    }
}
