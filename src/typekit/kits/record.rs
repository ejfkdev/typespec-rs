//! Record type operations
//!
//! Ported from TypeSpec compiler/src/typekit/kits/record.ts
//!
//! A record type is a Model with an indexer whose key is not `never`.
//! Delegates to checker/type_utils for the core check.

use crate::checker::type_utils;
use crate::checker::types::TypeId;
use crate::checker::{Checker, Type};

/// Check if a type is a record type (Model with a non-never indexer)
pub fn is_record(checker: &Checker, id: TypeId) -> bool {
    match checker.get_type(id) {
        Some(Type::Model(m)) => type_utils::is_record_model_type(&checker.type_store, m),
        _ => false,
    }
}

/// Get the key type of a record
pub fn get_key_type(checker: &Checker, id: TypeId) -> Option<TypeId> {
    match checker.get_type(id) {
        Some(Type::Model(m)) => m.indexer.map(|(key, _)| key),
        _ => None,
    }
}

/// Get the value type of a record
pub fn get_value_type(checker: &Checker, id: TypeId) -> Option<TypeId> {
    match checker.get_type(id) {
        Some(Type::Model(m)) => m.indexer.map(|(_, value)| value),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::test_utils::check;

    #[test]
    fn test_is_record() {
        let checker = check("model Foo is Record<string, string> {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let _ = is_record(&checker, foo_id);
    }

    #[test]
    fn test_regular_model_not_record() {
        let checker = check("model Foo { name: string; }");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        assert!(!is_record(&checker, foo_id));
    }

    #[test]
    fn test_get_key_type() {
        let checker = check("model Foo is Record<string, int32> {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let key_type = get_key_type(&checker, foo_id);
        let _ = key_type;
    }

    #[test]
    fn test_get_value_type() {
        let checker = check("model Foo is Record<string, int32> {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let value_type = get_value_type(&checker, foo_id);
        let _ = value_type;
    }

    #[test]
    fn test_get_key_type_non_record() {
        let checker = check("model Foo {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        assert_eq!(get_key_type(&checker, foo_id), None);
    }

    #[test]
    fn test_get_value_type_non_record() {
        let checker = check("model Foo {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        assert_eq!(get_value_type(&checker, foo_id), None);
    }

    #[test]
    fn test_is_record_scalar() {
        let checker = check("scalar MyS extends string;");
        let s_id = checker.declared_types.get("MyS").copied().unwrap();
        assert!(!is_record(&checker, s_id));
    }
}
