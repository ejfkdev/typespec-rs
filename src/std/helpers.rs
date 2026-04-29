//! Helper functions for the TypeSpec standard library
//!
//! Provides utilities for working with built-in types, creating
//! standard library entities, and checking type relationships.

use crate::std::primitives::{BuiltInInterfaceKind, BuiltInModelKind, BuiltInScalarKind};

/// Checks if a name refers to a built-in scalar type.
/// Delegates to `get_builtin_scalar_kind` as the single source of truth.
pub fn is_builtin_scalar(name: &str) -> bool {
    get_builtin_scalar_kind(name).is_some()
}

/// Converts a scalar name to its BuiltInScalarKind, if applicable
pub fn get_builtin_scalar_kind(name: &str) -> Option<BuiltInScalarKind> {
    match name {
        "bytes" => Some(BuiltInScalarKind::Bytes),
        "string" => Some(BuiltInScalarKind::String),
        "boolean" => Some(BuiltInScalarKind::Boolean),
        "numeric" => Some(BuiltInScalarKind::Numeric),
        "integer" => Some(BuiltInScalarKind::Integer),
        "int64" => Some(BuiltInScalarKind::Int64),
        "int32" => Some(BuiltInScalarKind::Int32),
        "int16" => Some(BuiltInScalarKind::Int16),
        "int8" => Some(BuiltInScalarKind::Int8),
        "uint64" => Some(BuiltInScalarKind::Uint64),
        "uint32" => Some(BuiltInScalarKind::Uint32),
        "uint16" => Some(BuiltInScalarKind::Uint16),
        "uint8" => Some(BuiltInScalarKind::Uint8),
        "safeint" => Some(BuiltInScalarKind::Safeint),
        "float" => Some(BuiltInScalarKind::Float),
        "float64" => Some(BuiltInScalarKind::Float64),
        "float32" => Some(BuiltInScalarKind::Float32),
        "decimal" => Some(BuiltInScalarKind::Decimal),
        "decimal128" => Some(BuiltInScalarKind::Decimal128),
        "plainDate" => Some(BuiltInScalarKind::PlainDate),
        "plainTime" => Some(BuiltInScalarKind::PlainTime),
        "utcDateTime" => Some(BuiltInScalarKind::UtcDateTime),
        "offsetDateTime" => Some(BuiltInScalarKind::OffsetDateTime),
        "duration" => Some(BuiltInScalarKind::Duration),
        "url" => Some(BuiltInScalarKind::Url),
        "unixTimestamp32" => Some(BuiltInScalarKind::UnixTimestamp32),
        _ => None,
    }
}

/// Checks if a name refers to a built-in model type
pub fn is_builtin_model(name: &str) -> bool {
    matches!(name, "Array" | "Record")
}

/// Converts a model name to its BuiltInModelKind, if applicable
pub fn get_builtin_model_kind(name: &str) -> Option<BuiltInModelKind> {
    match name {
        "Array" => Some(BuiltInModelKind::Array),
        "Record" => Some(BuiltInModelKind::Record),
        _ => None,
    }
}

/// Checks if a name refers to a built-in prototype interface
pub fn is_builtin_interface(name: &str) -> bool {
    matches!(name, "ModelProperty" | "Operation" | "Array")
}

/// Converts an interface name to its BuiltInInterfaceKind, if applicable
pub fn get_builtin_interface_kind(name: &str) -> Option<BuiltInInterfaceKind> {
    match name {
        "ModelProperty" => Some(BuiltInInterfaceKind::ModelProperty),
        "Operation" => Some(BuiltInInterfaceKind::Operation),
        "Array" => Some(BuiltInInterfaceKind::ArrayInterface),
        _ => None,
    }
}

/// Returns the TypeSpec namespace name for built-in types
pub use crate::libs::compiler::TYPESPEC_NAMESPACE;

/// Returns the TypeSpec.Prototypes namespace name
pub const PROTOTYPES_NAMESPACE: &str = "TypeSpec.Prototypes";

/// Checks if a namespace name is a built-in namespace
pub fn is_builtin_namespace(name: &str) -> bool {
    name == TYPESPEC_NAMESPACE || name == PROTOTYPES_NAMESPACE || name == "TypeSpec.Reflection"
}

/// Returns the scalar hierarchy as a flat list of (name, parent_name) pairs.
/// Delegates to the SCALAR_HIERARCHY constant in primitives.rs.
pub fn get_scalar_hierarchy() -> &'static [(&'static str, Option<&'static str>)] {
    super::primitives::SCALAR_HIERARCHY
}

/// Checks if a scalar is a descendant of another in the type hierarchy
pub fn is_scalar_descendant_of(kind: BuiltInScalarKind, ancestor: BuiltInScalarKind) -> bool {
    if kind == ancestor {
        return true;
    }
    let mut current = kind.extends();
    while let Some(parent) = current {
        if parent == ancestor {
            return true;
        }
        current = parent.extends();
    }
    false
}

/// Returns all ancestor scalar types for a given scalar
pub fn get_scalar_ancestors(kind: BuiltInScalarKind) -> Vec<BuiltInScalarKind> {
    let mut ancestors = Vec::new();
    let mut current = kind.extends();
    while let Some(parent) = current {
        ancestors.push(parent);
        current = parent.extends();
    }
    ancestors
}

/// Checks if a scalar type has the `fromISO` initializer
pub fn has_from_iso_initializer(kind: BuiltInScalarKind) -> bool {
    matches!(
        kind,
        BuiltInScalarKind::PlainDate
            | BuiltInScalarKind::PlainTime
            | BuiltInScalarKind::UtcDateTime
            | BuiltInScalarKind::OffsetDateTime
            | BuiltInScalarKind::Duration
    )
}

/// Checks if a scalar type has the `now` initializer
pub fn has_now_initializer(kind: BuiltInScalarKind) -> bool {
    matches!(
        kind,
        BuiltInScalarKind::PlainDate
            | BuiltInScalarKind::PlainTime
            | BuiltInScalarKind::UtcDateTime
            | BuiltInScalarKind::OffsetDateTime
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_builtin_scalar() {
        assert!(is_builtin_scalar("string"));
        assert!(is_builtin_scalar("int32"));
        assert!(is_builtin_scalar("boolean"));
        assert!(is_builtin_scalar("duration"));
        assert!(!is_builtin_scalar("Foo"));
        assert!(!is_builtin_scalar("myType"));
    }

    #[test]
    fn test_get_builtin_scalar_kind() {
        assert_eq!(
            get_builtin_scalar_kind("string"),
            Some(BuiltInScalarKind::String)
        );
        assert_eq!(
            get_builtin_scalar_kind("int32"),
            Some(BuiltInScalarKind::Int32)
        );
        assert_eq!(get_builtin_scalar_kind("unknown"), None);
    }

    #[test]
    fn test_is_builtin_model() {
        assert!(is_builtin_model("Array"));
        assert!(is_builtin_model("Record"));
        assert!(!is_builtin_model("Foo"));
    }

    #[test]
    fn test_get_builtin_model_kind() {
        assert_eq!(
            get_builtin_model_kind("Array"),
            Some(BuiltInModelKind::Array)
        );
        assert_eq!(
            get_builtin_model_kind("Record"),
            Some(BuiltInModelKind::Record)
        );
        assert_eq!(get_builtin_model_kind("Foo"), None);
    }

    #[test]
    fn test_is_builtin_namespace() {
        assert!(is_builtin_namespace("TypeSpec"));
        assert!(is_builtin_namespace("TypeSpec.Prototypes"));
        assert!(!is_builtin_namespace("MyNamespace"));
    }

    #[test]
    fn test_scalar_hierarchy() {
        let hierarchy = get_scalar_hierarchy();
        // int32 -> int64 -> integer -> numeric
        let int32_entry = hierarchy.iter().find(|(name, _)| *name == "int32");
        assert!(int32_entry.is_some());
        assert_eq!(int32_entry.unwrap().1, Some("int64"));

        let integer_entry = hierarchy.iter().find(|(name, _)| *name == "integer");
        assert!(integer_entry.is_some());
        assert_eq!(integer_entry.unwrap().1, Some("numeric"));
    }

    #[test]
    fn test_is_scalar_descendant_of() {
        // int32 extends int64 extends integer extends numeric
        assert!(is_scalar_descendant_of(
            BuiltInScalarKind::Int32,
            BuiltInScalarKind::Int64
        ));
        assert!(is_scalar_descendant_of(
            BuiltInScalarKind::Int32,
            BuiltInScalarKind::Integer
        ));
        assert!(is_scalar_descendant_of(
            BuiltInScalarKind::Int32,
            BuiltInScalarKind::Numeric
        ));
        // A type is its own descendant
        assert!(is_scalar_descendant_of(
            BuiltInScalarKind::String,
            BuiltInScalarKind::String
        ));
        // string does not extend numeric
        assert!(!is_scalar_descendant_of(
            BuiltInScalarKind::String,
            BuiltInScalarKind::Numeric
        ));
    }

    #[test]
    fn test_get_scalar_ancestors() {
        let ancestors = get_scalar_ancestors(BuiltInScalarKind::Int32);
        // int32 -> int64 -> integer -> numeric
        assert!(ancestors.contains(&BuiltInScalarKind::Int64));
        assert!(ancestors.contains(&BuiltInScalarKind::Integer));
        assert!(ancestors.contains(&BuiltInScalarKind::Numeric));

        // string has no ancestors
        let string_ancestors = get_scalar_ancestors(BuiltInScalarKind::String);
        assert!(string_ancestors.is_empty());
    }

    #[test]
    fn test_has_from_iso_initializer() {
        assert!(has_from_iso_initializer(BuiltInScalarKind::PlainDate));
        assert!(has_from_iso_initializer(BuiltInScalarKind::UtcDateTime));
        assert!(!has_from_iso_initializer(BuiltInScalarKind::String));
        assert!(!has_from_iso_initializer(BuiltInScalarKind::Int32));
    }

    #[test]
    fn test_has_now_initializer() {
        assert!(has_now_initializer(BuiltInScalarKind::PlainDate));
        assert!(has_now_initializer(BuiltInScalarKind::UtcDateTime));
        assert!(!has_now_initializer(BuiltInScalarKind::Duration));
        assert!(!has_now_initializer(BuiltInScalarKind::String));
    }
}
