//! TypeKit macros for generating repetitive `is_xxx` and `get_xxx` functions

/// Generate an `is_xxx` type check function.
/// Creates: `pub fn $name(checker: &Checker, id: TypeId) -> bool`
/// that checks if the type matches `Type::$variant(_)`.
///
/// Example: `define_type_check!(is_model, Model);`
macro_rules! define_type_check {
    ($name:ident, $variant:ident) => {
        pub fn $name(checker: &crate::checker::Checker, id: crate::checker::types::TypeId) -> bool {
            matches!(
                checker.get_type(id),
                Some(crate::checker::types::Type::$variant(_))
            )
        }
    };
}

/// Generate a `get_xxx` field getter function.
///
/// Forms:
/// 1. `define_type_field_getter!(name, Variant, field, Option<T>)` — field is `Option<T>`, returns `Option<T>`
/// 2. `define_type_field_getter!(name, Variant, field, str)` — field is `String`, returns `Option<&str>`
/// 3. `define_type_field_getter!(name, Variant, field, TypeId)` — field is `TypeId`, returns `Option<TypeId>`
macro_rules! define_type_field_getter {
    // Option<T> field getter — field itself is Option<T>
    ($name:ident, $variant:ident, $field:ident, Option<$ret:ty>) => {
        pub fn $name(
            checker: &crate::checker::Checker,
            id: crate::checker::types::TypeId,
        ) -> Option<$ret> {
            match checker.get_type(id) {
                Some(crate::checker::types::Type::$variant(v)) => v.$field,
                _ => None,
            }
        }
    };
    // &str getter — field is String, returns Option<&str>
    ($name:ident, $variant:ident, $field:ident, str) => {
        pub fn $name(
            checker: &crate::checker::Checker,
            id: crate::checker::types::TypeId,
        ) -> Option<&str> {
            match checker.get_type(id) {
                Some(crate::checker::types::Type::$variant(v)) => Some(v.$field.as_str()),
                _ => None,
            }
        }
    };
    // TypeId getter — field is TypeId (not Option), returns Option<TypeId>
    ($name:ident, $variant:ident, $field:ident, TypeId) => {
        pub fn $name(
            checker: &crate::checker::Checker,
            id: crate::checker::types::TypeId,
        ) -> Option<crate::checker::types::TypeId> {
            match checker.get_type(id) {
                Some(crate::checker::types::Type::$variant(v)) => Some(v.$field),
                _ => None,
            }
        }
    };
}

/// Generate an `is_xxx` value check function for Value enum.
/// Creates: `pub fn $name(value: &Value) -> bool`
///
/// Example: `define_value_check!(is_string, StringValue);`
macro_rules! define_value_check {
    ($name:ident, $variant:ident) => {
        pub fn $name(value: &crate::checker::types::Value) -> bool {
            matches!(value, crate::checker::types::Value::$variant(_))
        }
    };
}

/// Generate a `get_xxx` field getter function for Value enum.
///
/// Example: `define_value_field_getter!(get_string_value, StringValue, value, &str);`
macro_rules! define_value_field_getter {
    ($name:ident, $variant:ident, $field:ident, &$ret:ty) => {
        pub fn $name(value: &crate::checker::types::Value) -> Option<&$ret> {
            match value {
                crate::checker::types::Value::$variant(v) => Some(&v.$field),
                _ => None,
            }
        }
    };
    ($name:ident, $variant:ident, $field:ident, $ret:ty) => {
        pub fn $name(value: &crate::checker::types::Value) -> Option<$ret> {
            match value {
                crate::checker::types::Value::$variant(v) => Some(v.$field),
                _ => None,
            }
        }
    };
}
