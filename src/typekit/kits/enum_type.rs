//! Enum type operations
//!
//! Ported from TypeSpec compiler/src/typekit/kits/enum.ts

use crate::checker::types::TypeId;
use crate::checker::{Checker, Type};

define_type_check!(is_enum, Enum);

/// Get enum members as (name, member_type_id) pairs
pub fn get_members(checker: &Checker, id: TypeId) -> Vec<(String, TypeId)> {
    match checker.get_type(id) {
        Some(Type::Enum(e)) => e
            .member_names
            .iter()
            .filter_map(|name| e.members.get(name).map(|&m| (name.clone(), m)))
            .collect(),
        _ => Vec::new(),
    }
}

/// Check if an enum has a member with the given name
pub fn has_member(checker: &Checker, id: TypeId, name: &str) -> bool {
    match checker.get_type(id) {
        Some(Type::Enum(e)) => e.members.contains_key(name),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::test_utils::check;
    use crate::typekit::kits::builtin;

    #[test]
    fn test_is_enum() {
        let checker = check("enum Color { red, green, blue }");
        let e_id = checker.declared_types.get("Color").copied().unwrap();
        assert!(is_enum(&checker, e_id));
    }

    #[test]
    fn test_is_enum_not_model() {
        let checker = check("model Foo {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        assert!(!is_enum(&checker, foo_id));
    }

    #[test]
    fn test_get_members() {
        let checker = check("enum Color { red, green, blue }");
        let e_id = checker.declared_types.get("Color").copied().unwrap();
        let members = get_members(&checker, e_id);
        assert_eq!(members.len(), 3);
    }

    #[test]
    fn test_has_member() {
        let checker = check("enum Color { red, green, blue }");
        let e_id = checker.declared_types.get("Color").copied().unwrap();
        assert!(has_member(&checker, e_id, "red"));
        assert!(!has_member(&checker, e_id, "purple"));
    }

    #[test]
    fn test_enum_with_string_values() {
        let checker = check(r#"enum Direction { north: "N", south: "S", east: "E", west: "W" }"#);
        let e_id = checker.declared_types.get("Direction").copied().unwrap();
        assert!(is_enum(&checker, e_id));
        let members = get_members(&checker, e_id);
        assert_eq!(members.len(), 4);
        assert!(has_member(&checker, e_id, "north"));
    }

    #[test]
    fn test_enum_with_numeric_values() {
        let checker = check("enum Priority { low: 1, medium: 2, high: 3 }");
        let e_id = checker.declared_types.get("Priority").copied().unwrap();
        assert!(is_enum(&checker, e_id));
        let members = get_members(&checker, e_id);
        assert_eq!(members.len(), 3);
    }

    #[test]
    fn test_enum_implicit_values() {
        let checker = check("enum Status { active, inactive }");
        let e_id = checker.declared_types.get("Status").copied().unwrap();
        assert!(is_enum(&checker, e_id));
        assert!(has_member(&checker, e_id, "active"));
        assert!(has_member(&checker, e_id, "inactive"));
    }

    #[test]
    fn test_is_enum_not_scalar() {
        let checker = check("");
        let string_id = builtin::string(&checker).unwrap();
        assert!(!is_enum(&checker, string_id));
    }
}
