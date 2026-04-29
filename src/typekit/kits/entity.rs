//! Entity type operations
//!
//! Ported from TypeSpec compiler/src/typekit/kits/entity.ts
//!
//! Provides type assignability checking and type resolution.

use crate::checker::Checker;
use crate::checker::types::TypeId;

/// Check if source type can be assigned to target type
pub fn is_assignable_to(checker: &mut Checker, source: TypeId, target: TypeId) -> bool {
    let (result, _) = checker.is_type_assignable_to(source, target, 0);
    result
}

/// Resolve a type by name from declared types
pub fn resolve(checker: &Checker, name: &str) -> Option<TypeId> {
    checker.declared_types.get(name).copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::test_utils::check;
    use crate::typekit::kits::builtin;

    #[test]
    fn test_resolve_model() {
        let checker = check("model Foo {}");
        assert!(resolve(&checker, "Foo").is_some());
        assert!(resolve(&checker, "Bar").is_none());
    }

    #[test]
    fn test_resolve_scalar() {
        let checker = check("scalar MyS extends string;");
        assert!(resolve(&checker, "MyS").is_some());
    }

    #[test]
    fn test_resolve_enum() {
        let checker = check("enum Color { red, green, blue }");
        assert!(resolve(&checker, "Color").is_some());
    }

    #[test]
    fn test_resolve_union() {
        let checker = check("union Pet { cat: string, dog: string }");
        assert!(resolve(&checker, "Pet").is_some());
    }

    #[test]
    fn test_resolve_alias() {
        let checker = check(r#"alias Foo = "hello";"#);
        assert!(resolve(&checker, "Foo").is_some());
    }

    #[test]
    fn test_resolve_operation() {
        let checker = check("op test(): void;");
        assert!(resolve(&checker, "test").is_some());
    }

    #[test]
    fn test_is_assignable_same_type() {
        let mut checker = check("scalar A extends string; scalar B extends string;");
        let a_id = checker.declared_types.get("A").copied().unwrap();
        let b_id = checker.declared_types.get("B").copied().unwrap();
        // Both extend string - just verify no panic
        let _ = is_assignable_to(&mut checker, a_id, b_id);
    }

    #[test]
    fn test_is_assignable_incompatible() {
        let mut checker = check("scalar A extends string; scalar B extends int32;");
        let a_id = checker.declared_types.get("A").copied().unwrap();
        let b_id = checker.declared_types.get("B").copied().unwrap();
        // string-based scalar should not be assignable to int32-based scalar
        assert!(!is_assignable_to(&mut checker, a_id, b_id));
    }

    #[test]
    fn test_is_assignable_scalar_extends_string_to_string() {
        let mut checker = check("scalar MyS extends string;");
        let s_id = checker.declared_types.get("MyS").copied().unwrap();
        let string_id = builtin::string(&checker).unwrap();
        // MyS extends string, so should be assignable to string
        assert!(is_assignable_to(&mut checker, s_id, string_id));
    }

    #[test]
    fn test_is_assignable_int32_to_numeric() {
        let mut checker = check("");
        let int32_id = builtin::int32(&checker).unwrap();
        let numeric_id = builtin::numeric(&checker).unwrap();
        // int32 extends numeric
        assert!(is_assignable_to(&mut checker, int32_id, numeric_id));
    }

    #[test]
    fn test_is_assignable_string_literal_to_string() {
        let mut checker = check(r#"alias Foo = "hello";"#);
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = checker.resolve_alias_chain(foo_id);
        let string_id = builtin::string(&checker).unwrap();
        // string literal "hello" is assignable to string
        assert!(is_assignable_to(&mut checker, resolved, string_id));
    }

    #[test]
    fn test_is_assignable_string_literal_to_int32_is_not() {
        let mut checker = check(r#"alias Foo = "hello";"#);
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = checker.resolve_alias_chain(foo_id);
        let int32_id = builtin::int32(&checker).unwrap();
        // string literal is not assignable to int32
        assert!(!is_assignable_to(&mut checker, resolved, int32_id));
    }

    #[test]
    fn test_is_assignable_void_to_void() {
        let mut checker = check("");
        let void_id = checker.void_type;
        assert!(is_assignable_to(&mut checker, void_id, void_id));
    }

    #[test]
    fn test_is_assignable_never_to_anything() {
        let mut checker = check("");
        let string_id = builtin::string(&checker).unwrap();
        let never_id = checker.never_type;
        // never is assignable to anything
        assert!(is_assignable_to(&mut checker, never_id, string_id));
    }
}
