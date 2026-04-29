//! Intrinsic type operations
//!
//! Ported from TypeSpec compiler/src/typekit/kits/intrinsic.ts

use crate::checker::type_utils;
use crate::checker::types::{IntrinsicTypeName, TypeId};
use crate::checker::{Checker, Type};

define_type_check!(is_intrinsic, Intrinsic);

/// Get the intrinsic type name
pub fn get_intrinsic_name(checker: &Checker, id: TypeId) -> Option<IntrinsicTypeName> {
    checker.get_type(id).and_then(|t| match t {
        Type::Intrinsic(i) => Some(i.name),
        _ => None,
    })
}

/// Check if a type is the void intrinsic type
pub fn is_void(checker: &Checker, id: TypeId) -> bool {
    checker.get_type(id).is_some_and(type_utils::is_void_type)
}

/// Check if a type is the never intrinsic type
pub fn is_never(checker: &Checker, id: TypeId) -> bool {
    checker.get_type(id).is_some_and(type_utils::is_never_type)
}

/// Check if a type is the unknown intrinsic type
pub fn is_unknown(checker: &Checker, id: TypeId) -> bool {
    checker
        .get_type(id)
        .is_some_and(type_utils::is_unknown_type)
}

/// Check if a type is the null intrinsic type
pub fn is_null(checker: &Checker, id: TypeId) -> bool {
    checker.get_type(id).is_some_and(type_utils::is_null_type)
}

/// Check if a type is the error type.
/// Delegates to `type_kind::is_error` which checks both the intrinsic
/// ErrorType and the `checker.error_type` TypeId.
pub fn is_error(checker: &Checker, id: TypeId) -> bool {
    super::type_kind::is_error(checker, id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::test_utils::check;

    #[test]
    fn test_is_intrinsic() {
        let checker = check("");
        assert!(is_intrinsic(&checker, checker.void_type));
        assert!(is_intrinsic(&checker, checker.never_type));
    }

    #[test]
    fn test_is_intrinsic_not_model() {
        let checker = check("model Foo {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        assert!(!is_intrinsic(&checker, foo_id));
    }

    #[test]
    fn test_intrinsic_names() {
        let checker = check("");
        assert_eq!(
            get_intrinsic_name(&checker, checker.void_type),
            Some(IntrinsicTypeName::Void)
        );
        assert_eq!(
            get_intrinsic_name(&checker, checker.never_type),
            Some(IntrinsicTypeName::Never)
        );
        assert_eq!(
            get_intrinsic_name(&checker, checker.error_type),
            Some(IntrinsicTypeName::ErrorType)
        );
    }

    #[test]
    fn test_intrinsic_name_unknown() {
        let checker = check("");
        assert_eq!(
            get_intrinsic_name(&checker, checker.unknown_type),
            Some(IntrinsicTypeName::Unknown)
        );
    }

    #[test]
    fn test_intrinsic_name_null() {
        let checker = check("");
        assert_eq!(
            get_intrinsic_name(&checker, checker.null_type),
            Some(IntrinsicTypeName::Null)
        );
    }

    #[test]
    fn test_is_void_never() {
        let checker = check("");
        assert!(is_void(&checker, checker.void_type));
        assert!(is_never(&checker, checker.never_type));
        assert!(!is_void(&checker, checker.never_type));
    }

    #[test]
    fn test_is_unknown() {
        let checker = check("");
        assert!(is_unknown(&checker, checker.unknown_type));
        assert!(!is_unknown(&checker, checker.void_type));
    }

    #[test]
    fn test_is_null() {
        let checker = check("");
        assert!(is_null(&checker, checker.null_type));
        assert!(!is_null(&checker, checker.void_type));
    }

    #[test]
    fn test_is_error() {
        let checker = check("");
        assert!(is_error(&checker, checker.error_type));
        assert!(!is_error(&checker, checker.void_type));
    }

    #[test]
    fn test_is_void_model_is_not() {
        let checker = check("model Foo {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        assert!(!is_void(&checker, foo_id));
    }

    #[test]
    fn test_get_intrinsic_name_non_intrinsic() {
        let checker = check("model Foo {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        assert_eq!(get_intrinsic_name(&checker, foo_id), None);
    }
}
