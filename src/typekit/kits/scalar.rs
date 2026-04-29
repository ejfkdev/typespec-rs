//! Scalar type operations
//!
//! Ported from TypeSpec compiler/src/typekit/kits/scalar.ts

use super::builtin;
use crate::checker::types::TypeId;
use crate::checker::{Checker, Type};

define_type_check!(is_scalar, Scalar);

/// Check if a scalar is the built-in string type or extends it
pub fn is_string_scalar(checker: &Checker, id: TypeId) -> bool {
    let string_id = match builtin::string(checker) {
        Some(id) => id,
        None => return false,
    };
    id == string_id || extends_scalar(checker, id, string_id)
}

/// Check if a scalar is the built-in numeric type or extends it
pub fn is_numeric_scalar(checker: &Checker, id: TypeId) -> bool {
    let numeric_id = match builtin::numeric(checker) {
        Some(id) => id,
        None => return false,
    };
    id == numeric_id || extends_scalar(checker, id, numeric_id)
}

define_type_field_getter!(get_base_scalar, Scalar, base_scalar, Option<TypeId>);

/// Get the standard (built-in) base scalar by walking the extends chain.
/// Returns None if the scalar doesn't extend a built-in.
pub fn get_std_base(checker: &Checker, id: TypeId) -> Option<TypeId> {
    let mut current = Some(id);
    while let Some(curr_id) = current {
        let base = get_base_scalar(checker, curr_id);
        if base.is_none() {
            // This is a root scalar — if it's built-in, return it
            if is_builtin_scalar(checker, curr_id) {
                return Some(curr_id);
            }
            return None;
        }
        current = base;
    }
    None
}

/// Check if a scalar is a built-in TypeSpec scalar
fn is_builtin_scalar(checker: &Checker, id: TypeId) -> bool {
    match checker.get_type(id) {
        Some(Type::Scalar(s)) => crate::std::helpers::is_builtin_scalar(&s.name),
        _ => false,
    }
}

/// Scalar encoding information
#[derive(Debug, Clone)]
pub struct ScalarEncoding {
    /// The encoding name
    pub encoding: String,
    /// The encoding type (TypeId)
    pub type_id: TypeId,
}

/// Get the encoding of a scalar (from @encode decorator)
pub fn get_encoding(checker: &Checker, id: TypeId) -> Option<ScalarEncoding> {
    match checker.get_type(id) {
        Some(Type::Scalar(_)) => {
            let encoding = crate::libs::compiler::get_encode(&checker.state_accessors, id)?;
            Some(ScalarEncoding {
                encoding,
                type_id: id,
            })
        }
        _ => None,
    }
}

/// Check if a scalar extends another scalar through the chain
pub fn extends_scalar(checker: &Checker, id: TypeId, target: TypeId) -> bool {
    let mut current = Some(id);
    while let Some(curr_id) = current {
        if curr_id == target {
            return true;
        }
        current = get_base_scalar(checker, curr_id);
    }
    false
}

/// Walk up the scalar extends chain, collecting all ancestors
pub fn get_ancestor_chain(checker: &Checker, id: TypeId) -> Vec<TypeId> {
    let mut chain = Vec::new();
    let mut current = get_base_scalar(checker, id);
    while let Some(curr_id) = current {
        chain.push(curr_id);
        current = get_base_scalar(checker, curr_id);
    }
    chain
}

// ============================================================================
// Scalar type-specific convenience functions
// Ported from TS typekit/kits/scalar.ts isXxx/extendsXxx functions
// ============================================================================

/// Macro to generate is_xxx_scalar and extends_xxx_scalar convenience functions
/// for each built-in scalar type.
macro_rules! define_scalar_checks {
    ($($name:ident => $getter:ident),* $(,)?) => {
        $(
            /// Check if a scalar is the built-in `$name` type or extends it
            pub fn $name(checker: &Checker, id: TypeId) -> bool {
                let target_id = match builtin::$getter(checker) {
                    Some(id) => id,
                    None => return false,
                };
                id == target_id || extends_scalar(checker, id, target_id)
            }
        )*
    };
}

define_scalar_checks! {
    is_boolean_scalar => boolean,
    is_bytes_scalar => bytes,
    is_decimal_scalar => decimal,
    is_decimal128_scalar => decimal128,
    is_duration_scalar => duration,
    is_float_scalar => float,
    is_float32_scalar => float32,
    is_float64_scalar => float64,
    is_int8_scalar => int8,
    is_int16_scalar => int16,
    is_int32_scalar => int32,
    is_int64_scalar => int64,
    is_integer_scalar => integer,
    is_offset_date_time_scalar => offset_date_time,
    is_plain_date_scalar => plain_date,
    is_plain_time_scalar => plain_time,
    is_safe_int_scalar => safe_int,
    is_uint8_scalar => uint8,
    is_uint16_scalar => uint16,
    is_uint32_scalar => uint32,
    is_uint64_scalar => uint64,
    is_url_scalar => url,
    is_utc_date_time_scalar => utc_date_time,
}

/// Macro to generate extends_xxx_scalar convenience functions
macro_rules! define_extends_checks {
    ($($name:ident => $getter:ident),* $(,)?) => {
        $(
            /// Check if a scalar extends the built-in `$name` type (but is not the type itself)
            pub fn $name(checker: &Checker, id: TypeId) -> bool {
                let target_id = match builtin::$getter(checker) {
                    Some(id) => id,
                    None => return false,
                };
                id != target_id && extends_scalar(checker, id, target_id)
            }
        )*
    };
}

define_extends_checks! {
    extends_boolean_scalar => boolean,
    extends_bytes_scalar => bytes,
    extends_decimal_scalar => decimal,
    extends_decimal128_scalar => decimal128,
    extends_duration_scalar => duration,
    extends_float_scalar => float,
    extends_float32_scalar => float32,
    extends_float64_scalar => float64,
    extends_int8_scalar => int8,
    extends_int16_scalar => int16,
    extends_int32_scalar => int32,
    extends_int64_scalar => int64,
    extends_integer_scalar => integer,
    extends_offset_date_time_scalar => offset_date_time,
    extends_plain_date_scalar => plain_date,
    extends_plain_time_scalar => plain_time,
    extends_safe_int_scalar => safe_int,
    extends_uint8_scalar => uint8,
    extends_uint16_scalar => uint16,
    extends_uint32_scalar => uint32,
    extends_uint64_scalar => uint64,
    extends_url_scalar => url,
    extends_utc_date_time_scalar => utc_date_time,
    extends_string_scalar => string,
    extends_numeric_scalar => numeric,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::test_utils::check;
    use crate::typekit::kits::builtin;

    #[test]
    fn test_is_scalar() {
        let checker = check("scalar MyS extends string;");
        let s_id = checker.declared_types.get("MyS").copied().unwrap();
        assert!(is_scalar(&checker, s_id));
    }

    #[test]
    fn test_is_scalar_not_model() {
        let checker = check("model Foo {}");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        assert!(!is_scalar(&checker, foo_id));
    }

    #[test]
    fn test_is_scalar_builtin() {
        let checker = check("");
        let string_id = builtin::string(&checker).unwrap();
        assert!(is_scalar(&checker, string_id));
    }

    #[test]
    fn test_get_base_scalar() {
        let checker = check("scalar MyS extends string;");
        let s_id = checker.declared_types.get("MyS").copied().unwrap();
        let base = get_base_scalar(&checker, s_id);
        assert!(base.is_some());
    }

    #[test]
    fn test_get_base_scalar_builtin_has_none() {
        let checker = check("");
        let string_id = builtin::string(&checker).unwrap();
        // string is a root scalar, has no base
        assert!(get_base_scalar(&checker, string_id).is_none());
    }

    #[test]
    fn test_extends_scalar() {
        let checker = check("scalar Base extends string; scalar Derived extends Base;");
        let derived_id = checker.declared_types.get("Derived").copied().unwrap();
        let base_id = checker.declared_types.get("Base").copied().unwrap();

        assert!(extends_scalar(&checker, derived_id, base_id));
        assert!(!extends_scalar(&checker, base_id, derived_id));
    }

    #[test]
    fn test_extends_scalar_chain() {
        let checker = check("scalar A extends int32; scalar B extends A;");
        let b_id = checker.declared_types.get("B").copied().unwrap();
        let int32_id = builtin::int32(&checker).unwrap();
        // B → A → int32, so B extends int32
        assert!(extends_scalar(&checker, b_id, int32_id));
    }

    #[test]
    fn test_extends_scalar_builtin_int32_extends_integer() {
        let checker = check("");
        let int32_id = builtin::int32(&checker).unwrap();
        let integer_id = builtin::integer(&checker).unwrap();
        assert!(extends_scalar(&checker, int32_id, integer_id));
    }

    #[test]
    fn test_extends_scalar_builtin_int32_extends_numeric() {
        let checker = check("");
        let int32_id = builtin::int32(&checker).unwrap();
        let numeric_id = builtin::numeric(&checker).unwrap();
        assert!(extends_scalar(&checker, int32_id, numeric_id));
    }

    #[test]
    fn test_get_ancestor_chain() {
        let checker = check("scalar Base extends string; scalar Derived extends Base;");
        let derived_id = checker.declared_types.get("Derived").copied().unwrap();
        let chain = get_ancestor_chain(&checker, derived_id);
        assert!(chain.len() >= 2); // Base → string
    }

    #[test]
    fn test_get_ancestor_chain_builtin() {
        let checker = check("");
        let int32_id = builtin::int32(&checker).unwrap();
        let chain = get_ancestor_chain(&checker, int32_id);
        // int32 → int64 → integer → numeric
        assert!(chain.len() >= 2);
    }

    #[test]
    fn test_extends_scalar_self() {
        let checker = check("");
        let int32_id = builtin::int32(&checker).unwrap();
        assert!(extends_scalar(&checker, int32_id, int32_id));
    }

    // ==================== Ported from TS typekit/scalar.test.ts ====================

    #[test]
    fn test_is_string_scalar() {
        let checker = check("alias Foo = string; alias Bar = boolean;");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let bar_id = checker.declared_types.get("Bar").copied().unwrap();
        let foo_resolved = checker.resolve_alias_chain(foo_id);
        let bar_resolved = checker.resolve_alias_chain(bar_id);
        assert!(is_string_scalar(&checker, foo_resolved));
        assert!(!is_string_scalar(&checker, bar_resolved));
    }

    #[test]
    fn test_is_string_scalar_custom() {
        let checker = check("scalar MyS extends string;");
        let s_id = checker.declared_types.get("MyS").copied().unwrap();
        assert!(is_string_scalar(&checker, s_id));
    }

    #[test]
    fn test_is_numeric_scalar() {
        let checker = check("alias Foo = numeric; alias Bar = int32;");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let bar_id = checker.declared_types.get("Bar").copied().unwrap();
        let foo_resolved = checker.resolve_alias_chain(foo_id);
        let bar_resolved = checker.resolve_alias_chain(bar_id);
        assert!(is_numeric_scalar(&checker, foo_resolved));
        // int32 extends integer extends numeric — so it IS a numeric scalar
        assert!(is_numeric_scalar(&checker, bar_resolved));
    }

    #[test]
    fn test_get_std_base_custom_scalar() {
        let checker = check("scalar foo extends string;");
        let foo_id = checker.declared_types.get("foo").copied().unwrap();
        let std_base = get_std_base(&checker, foo_id);
        assert!(std_base.is_some());
        let base = std_base.unwrap();
        assert!(is_string_scalar(&checker, base));
    }

    #[test]
    fn test_get_std_base_root_scalar() {
        let checker = check("scalar bar;");
        let bar_id = checker.declared_types.get("bar").copied().unwrap();
        // bar doesn't extend any built-in
        let std_base = get_std_base(&checker, bar_id);
        assert!(std_base.is_none());
    }

    #[test]
    fn test_get_std_base_builtin() {
        let checker = check("");
        let string_id = builtin::string(&checker).unwrap();
        let std_base = get_std_base(&checker, string_id);
        assert!(std_base.is_some());
        assert_eq!(std_base, Some(string_id));
    }

    // ==================== Scalar isXxx/extendsXxx tests ====================

    #[test]
    fn test_is_boolean_scalar() {
        let checker = check("alias Foo = boolean;");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = checker.resolve_alias_chain(foo_id);
        assert!(is_boolean_scalar(&checker, resolved));
    }

    #[test]
    fn test_is_int32_scalar() {
        let checker = check("alias Foo = int32;");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = checker.resolve_alias_chain(foo_id);
        assert!(is_int32_scalar(&checker, resolved));
    }

    #[test]
    fn test_is_integer_scalar() {
        let checker = check("alias Foo = int32;");
        let foo_id = checker.declared_types.get("Foo").copied().unwrap();
        let resolved = checker.resolve_alias_chain(foo_id);
        // int32 extends integer
        assert!(is_integer_scalar(&checker, resolved));
    }

    #[test]
    fn test_extends_string_scalar_custom() {
        let checker = check("scalar MyS extends string;");
        let s_id = checker.declared_types.get("MyS").copied().unwrap();
        assert!(extends_string_scalar(&checker, s_id));
    }

    #[test]
    fn test_extends_int32_scalar_custom() {
        let checker = check("scalar MyInt extends int32;");
        let s_id = checker.declared_types.get("MyInt").copied().unwrap();
        assert!(extends_int32_scalar(&checker, s_id));
    }

    #[test]
    fn test_not_extends_for_builtin_itself() {
        let checker = check("");
        let string_id = builtin::string(&checker).unwrap();
        // string itself does NOT "extend" string — extends means "is a descendant"
        assert!(!extends_string_scalar(&checker, string_id));
    }
}
