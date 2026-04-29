//! Literal type operations
//!
//! Ported from TypeSpec compiler/src/typekit/kits/literal.ts

use crate::checker::type_utils;
use crate::checker::types::TypeId;
use crate::checker::{Checker, Type};

/// Check if a type is any literal type (string, number, or boolean)
pub fn is_literal(checker: &Checker, id: TypeId) -> bool {
    checker.get_type(id).is_some_and(|t| {
        type_utils::is_string_literal_type(t)
            || type_utils::is_numeric_literal_type(t)
            || type_utils::is_boolean_literal_type(t)
    })
}

/// Check if a type is a string literal
pub fn is_string_literal(checker: &Checker, id: TypeId) -> bool {
    checker
        .get_type(id)
        .is_some_and(type_utils::is_string_literal_type)
}

/// Check if a type is a numeric literal
pub fn is_numeric_literal(checker: &Checker, id: TypeId) -> bool {
    checker
        .get_type(id)
        .is_some_and(type_utils::is_numeric_literal_type)
}

/// Check if a type is a boolean literal
pub fn is_boolean_literal(checker: &Checker, id: TypeId) -> bool {
    checker
        .get_type(id)
        .is_some_and(type_utils::is_boolean_literal_type)
}

/// Get the string value of a string literal type
pub fn get_string_value(checker: &Checker, id: TypeId) -> Option<&str> {
    match checker.get_type(id) {
        Some(Type::String(s)) => Some(&s.value),
        _ => None,
    }
}

/// Get the numeric value of a numeric literal type
pub fn get_numeric_value(checker: &Checker, id: TypeId) -> Option<f64> {
    match checker.get_type(id) {
        Some(Type::Number(n)) => Some(n.value),
        _ => None,
    }
}

/// Get the boolean value of a boolean literal type
pub fn get_boolean_value(checker: &Checker, id: TypeId) -> Option<bool> {
    match checker.get_type(id) {
        Some(Type::Boolean(b)) => Some(b.value),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::test_utils::check;
    use crate::typekit::kits::builtin;

    #[test]
    fn test_is_literal_string() {
        let checker = check(r#"alias Foo = "hello";"#);
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = checker.resolve_alias_chain(foo_id);
        assert!(is_literal(&checker, resolved));
    }

    #[test]
    fn test_is_literal_numeric() {
        let checker = check("alias Foo = 42;");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = checker.resolve_alias_chain(foo_id);
        assert!(is_literal(&checker, resolved));
    }

    #[test]
    fn test_is_literal_boolean() {
        let checker = check("alias Foo = true;");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = checker.resolve_alias_chain(foo_id);
        assert!(is_literal(&checker, resolved));
    }

    #[test]
    fn test_is_literal_not_model() {
        let checker = check("model Foo {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        assert!(!is_literal(&checker, foo_id));
    }

    #[test]
    fn test_is_literal_not_scalar() {
        let checker = check("");
        let string_id = builtin::string(&checker).unwrap();
        assert!(!is_literal(&checker, string_id));
    }

    #[test]
    fn test_string_literal() {
        let checker = check(r#"alias Foo = "hello";"#);
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = checker.resolve_alias_chain(foo_id);
        assert!(is_string_literal(&checker, resolved));
        assert_eq!(get_string_value(&checker, resolved), Some("hello"));
    }

    #[test]
    fn test_string_literal_empty() {
        let checker = check(r#"alias Foo = "";"#);
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = checker.resolve_alias_chain(foo_id);
        assert!(is_string_literal(&checker, resolved));
        assert_eq!(get_string_value(&checker, resolved), Some(""));
    }

    #[test]
    fn test_string_literal_not_numeric() {
        let checker = check("alias Foo = 42;");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = checker.resolve_alias_chain(foo_id);
        assert!(!is_string_literal(&checker, resolved));
    }

    #[test]
    fn test_numeric_literal() {
        let checker = check("alias Foo = 42;");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = checker.resolve_alias_chain(foo_id);
        assert!(is_numeric_literal(&checker, resolved));
        assert_eq!(get_numeric_value(&checker, resolved), Some(42.0));
    }

    #[test]
    fn test_numeric_literal_zero() {
        let checker = check("alias Foo = 0;");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = checker.resolve_alias_chain(foo_id);
        assert!(is_numeric_literal(&checker, resolved));
        assert_eq!(get_numeric_value(&checker, resolved), Some(0.0));
    }

    #[test]
    fn test_numeric_literal_not_string() {
        let checker = check(r#"alias Foo = "hello";"#);
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = checker.resolve_alias_chain(foo_id);
        assert!(!is_numeric_literal(&checker, resolved));
    }

    #[test]
    fn test_boolean_literal_true() {
        let checker = check("alias Foo = true;");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = checker.resolve_alias_chain(foo_id);
        assert!(is_boolean_literal(&checker, resolved));
        assert_eq!(get_boolean_value(&checker, resolved), Some(true));
    }

    #[test]
    fn test_boolean_literal_false() {
        let checker = check("alias Foo = false;");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = checker.resolve_alias_chain(foo_id);
        assert!(is_boolean_literal(&checker, resolved));
        assert_eq!(get_boolean_value(&checker, resolved), Some(false));
    }

    #[test]
    fn test_boolean_literal_not_numeric() {
        let checker = check("alias Foo = 42;");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = checker.resolve_alias_chain(foo_id);
        assert!(!is_boolean_literal(&checker, resolved));
    }

    #[test]
    fn test_get_string_value_non_string() {
        let checker = check("alias Foo = 42;");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = checker.resolve_alias_chain(foo_id);
        assert_eq!(get_string_value(&checker, resolved), None);
    }

    #[test]
    fn test_get_numeric_value_non_numeric() {
        let checker = check(r#"alias Foo = "hello";"#);
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = checker.resolve_alias_chain(foo_id);
        assert_eq!(get_numeric_value(&checker, resolved), None);
    }

    #[test]
    fn test_get_boolean_value_non_boolean() {
        let checker = check(r#"alias Foo = "hello";"#);
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = checker.resolve_alias_chain(foo_id);
        assert_eq!(get_boolean_value(&checker, resolved), None);
    }
}
