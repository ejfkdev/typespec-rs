//! Helper functions for the TypeSpec standard library
//!
//! Provides utilities for working with built-in types, creating
//! standard library entities, and checking type relationships.

use crate::ast::node::NodeId;
use crate::std::primitives::{BuiltInInterfaceKind, BuiltInModelKind, BuiltInScalarKind};
use crate::types::ModelIndexer;

/// Checks if a name refers to a built-in scalar type
pub fn is_builtin_scalar(name: &str) -> bool {
    matches!(
        name,
        "bytes"
            | "string"
            | "boolean"
            | "numeric"
            | "integer"
            | "int64"
            | "int32"
            | "int16"
            | "int8"
            | "uint64"
            | "uint32"
            | "uint16"
            | "uint8"
            | "safeint"
            | "float"
            | "float64"
            | "float32"
            | "decimal"
            | "decimal128"
            | "plainDate"
            | "plainTime"
            | "utcDateTime"
            | "offsetDateTime"
            | "duration"
            | "url"
    )
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
    matches!(
        name,
        "ModelProperty" | "Operation" | "Array"
    )
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
pub const TYPESPEC_NAMESPACE: &str = "TypeSpec";

/// Returns the TypeSpec.Prototypes namespace name
pub const PROTOTYPES_NAMESPACE: &str = "TypeSpec.Prototypes";

/// Checks if a namespace name is a built-in namespace
pub fn is_builtin_namespace(name: &str) -> bool {
    name == TYPESPEC_NAMESPACE || name == PROTOTYPES_NAMESPACE
}

/// Creates an indexer for the Array model type
pub fn create_array_indexer(key_type: NodeId, value_type: NodeId) -> ModelIndexer {
    ModelIndexer {
        key: key_type,
        value: value_type,
    }
}

/// Creates an indexer for the Record model type
pub fn create_record_indexer(key_type: NodeId, value_type: NodeId) -> ModelIndexer {
    ModelIndexer {
        key: key_type,
        value: value_type,
    }
}

/// Returns the scalar hierarchy as a flat list of (name, parent_name) pairs
pub fn get_scalar_hierarchy() -> &'static [(&'static str, Option<&'static str>)] {
    &[
        ("bytes", None),
        ("string", None),
        ("boolean", None),
        ("url", None),
        ("numeric", None),
        ("integer", Some("numeric")),
        ("int64", Some("integer")),
        ("int32", Some("int64")),
        ("int16", Some("int32")),
        ("int8", Some("int16")),
        ("uint64", Some("integer")),
        ("uint32", Some("uint64")),
        ("uint16", Some("uint32")),
        ("uint8", Some("uint16")),
        ("safeint", Some("int64")),
        ("float", Some("numeric")),
        ("float64", Some("float")),
        ("float32", Some("float64")),
        ("decimal", Some("numeric")),
        ("decimal128", Some("decimal")),
        ("plainDate", None),
        ("plainTime", None),
        ("utcDateTime", None),
        ("offsetDateTime", None),
        ("duration", None),
    ]
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
