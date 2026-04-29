//! Value type operations
//!
//! Ported from TypeSpec compiler/src/typekit/kits/value.ts
//!
//! Provides operations for Value types (StringValue, NumericValue, BooleanValue,
//! ObjectValue, ArrayValue, EnumValue, NullValue, ScalarValue).

use super::builtin;
use crate::checker::Checker;
use crate::checker::types::{Value, ValueId};

define_value_check!(is_string, StringValue);
define_value_check!(is_numeric, NumericValue);
define_value_check!(is_boolean, BooleanValue);
define_value_check!(is_object, ObjectValue);
define_value_check!(is_array, ArrayValue);
define_value_check!(is_enum, EnumValue);
define_value_check!(is_null, NullValue);
define_value_check!(is_scalar, ScalarValue);

define_value_field_getter!(get_string_value, StringValue, value, &str);
define_value_field_getter!(get_numeric_value, NumericValue, value, f64);
define_value_field_getter!(get_boolean_value, BooleanValue, value, bool);

/// Get the TypeId associated with this value's type
pub fn get_type_id(value: &Value) -> crate::checker::types::TypeId {
    value.value_type()
}

/// Check if source value can be assigned to target type
pub fn is_assignable_to(
    checker: &mut Checker,
    source: &Value,
    target_type: crate::checker::types::TypeId,
) -> bool {
    super::entity::is_assignable_to(checker, source.value_type(), target_type)
}

/// Create a string Value
pub fn create_string(checker: &mut Checker, value: &str) -> ValueId {
    let string_type = builtin::string(checker).unwrap_or(checker.error_type);
    let sv = crate::checker::types::StringValue {
        type_id: string_type,
        value: value.to_string(),
        scalar: None,
        node: None,
    };
    checker.create_value(Value::StringValue(sv))
}

/// Create a numeric Value
pub fn create_numeric(checker: &mut Checker, value: f64) -> ValueId {
    let numeric_type = builtin::numeric(checker).unwrap_or(checker.error_type);
    let nv = crate::checker::types::NumericValue {
        type_id: numeric_type,
        value,
        scalar: None,
        node: None,
    };
    checker.create_value(Value::NumericValue(nv))
}

/// Create a boolean Value
pub fn create_boolean(checker: &mut Checker, value: bool) -> ValueId {
    let boolean_type = builtin::boolean(checker).unwrap_or(checker.error_type);
    let bv = crate::checker::types::BooleanValue {
        type_id: boolean_type,
        value,
        scalar: None,
        node: None,
    };
    checker.create_value(Value::BooleanValue(bv))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::test_utils::check;

    #[test]
    fn test_is_string() {
        let mut checker = check("");
        let vid = create_string(&mut checker, "hello");
        let value = checker.get_value(vid).unwrap();
        assert!(is_string(value));
        assert!(!is_numeric(value));
    }

    #[test]
    fn test_is_numeric() {
        let mut checker = check("");
        let vid = create_numeric(&mut checker, 42.0);
        let value = checker.get_value(vid).unwrap();
        assert!(is_numeric(value));
        assert!(!is_string(value));
    }

    #[test]
    fn test_is_boolean() {
        let mut checker = check("");
        let vid = create_boolean(&mut checker, true);
        let value = checker.get_value(vid).unwrap();
        assert!(is_boolean(value));
        assert!(!is_numeric(value));
    }

    #[test]
    fn test_get_string_value() {
        let mut checker = check("");
        let vid = create_string(&mut checker, "hello");
        let value = checker.get_value(vid).unwrap();
        assert_eq!(get_string_value(value), Some("hello"));
    }

    #[test]
    fn test_get_numeric_value() {
        let mut checker = check("");
        let vid = create_numeric(&mut checker, 2.78);
        let value = checker.get_value(vid).unwrap();
        assert_eq!(get_numeric_value(value), Some(2.78));
    }

    #[test]
    fn test_get_boolean_value() {
        let mut checker = check("");
        let vid = create_boolean(&mut checker, false);
        let value = checker.get_value(vid).unwrap();
        assert_eq!(get_boolean_value(value), Some(false));
    }

    #[test]
    fn test_get_string_value_wrong_type() {
        let mut checker = check("");
        let vid = create_numeric(&mut checker, 1.0);
        let value = checker.get_value(vid).unwrap();
        assert_eq!(get_string_value(value), None);
    }

    #[test]
    fn test_is_null_not_other() {
        let mut checker = check("");
        let vid = create_string(&mut checker, "test");
        let value = checker.get_value(vid).unwrap();
        assert!(!is_null(value));
        assert!(!is_object(value));
        assert!(!is_array(value));
        assert!(!is_enum(value));
        assert!(!is_scalar(value));
    }

    #[test]
    fn test_get_type_id() {
        let mut checker = check("");
        let vid = create_string(&mut checker, "test");
        let value = checker.get_value(vid).unwrap();
        let type_id = get_type_id(value);
        assert_eq!(type_id, builtin::string(&checker).unwrap());
    }
}
