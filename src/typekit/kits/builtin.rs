//! Builtin type operations
//!
//! Ported from TypeSpec compiler/src/typekit/kits/builtin.ts
//!
//! Provides access to TypeSpec's built-in scalar types.

use crate::checker::Checker;
use crate::checker::types::TypeId;

/// Get a built-in scalar type by name
pub fn get_scalar(checker: &Checker, name: &str) -> Option<TypeId> {
    checker.get_std_type(name)
}

/// Get the built-in string scalar type
pub fn string(checker: &Checker) -> Option<TypeId> {
    checker.get_std_type("string")
}

/// Get the built-in int8 scalar type
pub fn int8(checker: &Checker) -> Option<TypeId> {
    checker.get_std_type("int8")
}

/// Get the built-in int16 scalar type
pub fn int16(checker: &Checker) -> Option<TypeId> {
    checker.get_std_type("int16")
}

/// Get the built-in int32 scalar type
pub fn int32(checker: &Checker) -> Option<TypeId> {
    checker.get_std_type("int32")
}

/// Get the built-in int64 scalar type
pub fn int64(checker: &Checker) -> Option<TypeId> {
    checker.get_std_type("int64")
}

/// Get the built-in float32 scalar type
pub fn float32(checker: &Checker) -> Option<TypeId> {
    checker.get_std_type("float32")
}

/// Get the built-in float64 scalar type
pub fn float64(checker: &Checker) -> Option<TypeId> {
    checker.get_std_type("float64")
}

/// Get the built-in boolean scalar type
pub fn boolean(checker: &Checker) -> Option<TypeId> {
    checker.get_std_type("boolean")
}

/// Get the built-in bytes scalar type
pub fn bytes(checker: &Checker) -> Option<TypeId> {
    checker.get_std_type("bytes")
}

/// Get the built-in decimal scalar type
pub fn decimal(checker: &Checker) -> Option<TypeId> {
    checker.get_std_type("decimal")
}

/// Get the built-in decimal128 scalar type
pub fn decimal128(checker: &Checker) -> Option<TypeId> {
    checker.get_std_type("decimal128")
}

/// Get the built-in duration scalar type
pub fn duration(checker: &Checker) -> Option<TypeId> {
    checker.get_std_type("duration")
}

/// Get the built-in plainDate scalar type
pub fn plain_date(checker: &Checker) -> Option<TypeId> {
    checker.get_std_type("plainDate")
}

/// Get the built-in plainTime scalar type
pub fn plain_time(checker: &Checker) -> Option<TypeId> {
    checker.get_std_type("plainTime")
}

/// Get the built-in utcDateTime scalar type
pub fn utc_date_time(checker: &Checker) -> Option<TypeId> {
    checker.get_std_type("utcDateTime")
}

/// Get the built-in offsetDateTime scalar type
pub fn offset_date_time(checker: &Checker) -> Option<TypeId> {
    checker.get_std_type("offsetDateTime")
}

/// Get the built-in numeric scalar type
pub fn numeric(checker: &Checker) -> Option<TypeId> {
    checker.get_std_type("numeric")
}

/// Get the built-in integer scalar type
pub fn integer(checker: &Checker) -> Option<TypeId> {
    checker.get_std_type("integer")
}

/// Get the built-in url scalar type
pub fn url(checker: &Checker) -> Option<TypeId> {
    checker.get_std_type("url")
}

/// Get the built-in uint8 scalar type
pub fn uint8(checker: &Checker) -> Option<TypeId> {
    checker.get_std_type("uint8")
}

/// Get the built-in uint16 scalar type
pub fn uint16(checker: &Checker) -> Option<TypeId> {
    checker.get_std_type("uint16")
}

/// Get the built-in uint32 scalar type
pub fn uint32(checker: &Checker) -> Option<TypeId> {
    checker.get_std_type("uint32")
}

/// Get the built-in uint64 scalar type
pub fn uint64(checker: &Checker) -> Option<TypeId> {
    checker.get_std_type("uint64")
}

/// Get the built-in safeint scalar type
pub fn safe_int(checker: &Checker) -> Option<TypeId> {
    checker.get_std_type("safeint")
}

/// Get the built-in float scalar type
pub fn float(checker: &Checker) -> Option<TypeId> {
    checker.get_std_type("float")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::test_utils::check;

    #[test]
    fn test_builtin_string() {
        let checker = check("");
        assert!(string(&checker).is_some());
    }

    #[test]
    fn test_builtin_int32() {
        let checker = check("");
        assert!(int32(&checker).is_some());
    }

    #[test]
    fn test_builtin_boolean() {
        let checker = check("");
        assert!(boolean(&checker).is_some());
    }

    #[test]
    fn test_builtin_get_scalar() {
        let checker = check("");
        assert!(get_scalar(&checker, "string").is_some());
        assert!(get_scalar(&checker, "int32").is_some());
        assert!(get_scalar(&checker, "nonexistent").is_none());
    }

    #[test]
    fn test_builtin_int8() {
        let checker = check("");
        assert!(int8(&checker).is_some());
    }

    #[test]
    fn test_builtin_int16() {
        let checker = check("");
        assert!(int16(&checker).is_some());
    }

    #[test]
    fn test_builtin_int64() {
        let checker = check("");
        assert!(int64(&checker).is_some());
    }

    #[test]
    fn test_builtin_float32() {
        let checker = check("");
        assert!(float32(&checker).is_some());
    }

    #[test]
    fn test_builtin_float64() {
        let checker = check("");
        assert!(float64(&checker).is_some());
    }

    #[test]
    fn test_builtin_decimal() {
        let checker = check("");
        assert!(decimal(&checker).is_some());
    }

    #[test]
    fn test_builtin_decimal128() {
        let checker = check("");
        assert!(decimal128(&checker).is_some());
    }

    #[test]
    fn test_builtin_numeric() {
        let checker = check("");
        assert!(numeric(&checker).is_some());
    }

    #[test]
    fn test_builtin_integer() {
        let checker = check("");
        assert!(integer(&checker).is_some());
    }

    #[test]
    fn test_builtin_scalar_extends_chain() {
        let checker = check("");
        // int32 extends int64 extends integer extends numeric
        let int32_id = int32(&checker).unwrap();
        let int64_id = int64(&checker).unwrap();
        assert!(crate::typekit::kits::scalar::extends_scalar(
            &checker, int32_id, int64_id
        ));
    }

    #[test]
    fn test_builtin_url() {
        let checker = check("");
        assert!(url(&checker).is_some());
    }

    #[test]
    fn test_builtin_uint8() {
        let checker = check("");
        assert!(uint8(&checker).is_some());
    }

    #[test]
    fn test_builtin_uint16() {
        let checker = check("");
        assert!(uint16(&checker).is_some());
    }

    #[test]
    fn test_builtin_uint32() {
        let checker = check("");
        assert!(uint32(&checker).is_some());
    }

    #[test]
    fn test_builtin_uint64() {
        let checker = check("");
        assert!(uint64(&checker).is_some());
    }

    #[test]
    fn test_builtin_safe_int() {
        let checker = check("");
        assert!(safe_int(&checker).is_some());
    }

    #[test]
    fn test_builtin_float() {
        let checker = check("");
        assert!(float(&checker).is_some());
    }
}
