//! Union variant type operations
//!
//! Ported from TypeSpec compiler/src/typekit/kits/union-variant.ts

#[cfg(test)]
use crate::checker::Type;
use crate::checker::types::TypeId;

define_type_check!(is_union_variant, UnionVariant);
define_type_field_getter!(get_name, UnionVariant, name, str);
define_type_field_getter!(get_type, UnionVariant, r#type, TypeId);
define_type_field_getter!(get_union, UnionVariant, union, Option<TypeId>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::test_utils::check;

    #[test]
    fn test_is_union_variant() {
        let checker = check("union Shape { circle: string, square: string }");
        let u_id = checker.declared_types.get("Shape").copied().unwrap();
        if let Some(Type::Union(u)) = checker.get_type(u_id) {
            let variant_id = u.variants["circle"];
            assert!(is_union_variant(&checker, variant_id));
        }
    }

    #[test]
    fn test_is_union_variant_not_model() {
        let checker = check("model Foo {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        assert!(!is_union_variant(&checker, foo_id));
    }

    #[test]
    fn test_variant_name_and_type() {
        let checker = check("union Shape { circle: string, square: string }");
        let u_id = checker.declared_types.get("Shape").copied().unwrap();
        if let Some(Type::Union(u)) = checker.get_type(u_id) {
            let variant_id = u.variants["circle"];
            assert_eq!(get_name(&checker, variant_id), Some("circle"));
            assert!(get_type(&checker, variant_id).is_some());
        }
    }

    #[test]
    fn test_get_union_parent() {
        let checker = check("union Shape { circle: string, square: string }");
        let u_id = checker.declared_types.get("Shape").copied().unwrap();
        if let Some(Type::Union(u)) = checker.get_type(u_id) {
            let variant_id = u.variants["circle"];
            assert_eq!(get_union(&checker, variant_id), Some(u_id));
        }
    }

    #[test]
    fn test_all_variants() {
        let checker = check("union Shape { circle: string, square: int32 }");
        let u_id = checker.declared_types.get("Shape").copied().unwrap();
        if let Some(Type::Union(u)) = checker.get_type(u_id) {
            let circle_id = u.variants["circle"];
            let square_id = u.variants["square"];
            assert_eq!(get_name(&checker, circle_id), Some("circle"));
            assert_eq!(get_name(&checker, square_id), Some("square"));
            // Both should point to the same parent union
            assert_eq!(get_union(&checker, circle_id), Some(u_id));
            assert_eq!(get_union(&checker, square_id), Some(u_id));
        }
    }

    #[test]
    fn test_get_name_non_variant() {
        let checker = check("model Foo {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        assert_eq!(get_name(&checker, foo_id), None);
    }

    #[test]
    fn test_get_type_non_variant() {
        let checker = check("model Foo {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        assert_eq!(get_type(&checker, foo_id), None);
    }

    #[test]
    fn test_get_union_non_variant() {
        let checker = check("model Foo {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        assert_eq!(get_union(&checker, foo_id), None);
    }
}
