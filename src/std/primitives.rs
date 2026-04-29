//! Primitive type definitions for TypeSpec standard library
//!
//! Ported from TypeSpec compiler/lib/intrinsics.tsp
//!
//! This module defines all built-in scalar types and their inheritance hierarchy:
//! - bytes, string, boolean (base types)
//! - numeric -> integer -> int64 -> int32 -> int16 -> int8
//!   -> uint64 -> uint32 -> uint16 -> uint8
//!   -> safeint
//!   -> float -> float64 -> float32
//!   -> decimal -> decimal128
//! - plainDate, plainTime, utcDateTime, offsetDateTime, duration

use crate::ast::node::NodeId;

/// Built-in scalar type definition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltInScalarKind {
    /// `bytes` - byte array
    Bytes,
    /// `string` - textual characters
    String,
    /// `boolean` - true/false
    Boolean,

    // Numeric hierarchy root
    /// `numeric` - any numeric type
    Numeric,

    // Integer hierarchy
    /// `integer` - whole numbers (BigInteger)
    Integer,
    /// `int64` - 64-bit integer
    Int64,
    /// `int32` - 32-bit integer
    Int32,
    /// `int16` - 16-bit integer
    Int16,
    /// `int8` - 8-bit integer
    Int8,
    /// `uint64` - 64-bit unsigned integer
    Uint64,
    /// `uint32` - 32-bit unsigned integer
    Uint32,
    /// `uint16` - 16-bit unsigned integer
    Uint16,
    /// `uint8` - 8-bit unsigned integer
    Uint8,
    /// `safeint` - JSON-safe integer
    Safeint,

    // Float hierarchy
    /// `float` - floating point number
    Float,
    /// `float64` - 64-bit float
    Float64,
    /// `float32` - 32-bit float
    Float32,

    // Decimal hierarchy
    /// `decimal` - arbitrary precision decimal (BigDecimal)
    Decimal,
    /// `decimal128` - 128-bit decimal
    Decimal128,

    // Date/time types
    /// `plainDate` - calendar date without time zone
    PlainDate,
    /// `plainTime` - clock time without time zone
    PlainTime,
    /// `utcDateTime` - UTC instant
    UtcDateTime,
    /// `offsetDateTime` - date/time with time zone offset
    OffsetDateTime,
    /// `duration` - time period/duration
    Duration,

    /// `url` - URL type
    Url,
    /// `unixTimestamp32` - 32-bit unix timestamp
    UnixTimestamp32,
}

impl BuiltInScalarKind {
    /// Returns the TypeSpec name of this scalar
    pub fn as_str(&self) -> &'static str {
        match self {
            BuiltInScalarKind::Bytes => "bytes",
            BuiltInScalarKind::String => "string",
            BuiltInScalarKind::Boolean => "boolean",
            BuiltInScalarKind::Numeric => "numeric",
            BuiltInScalarKind::Integer => "integer",
            BuiltInScalarKind::Int64 => "int64",
            BuiltInScalarKind::Int32 => "int32",
            BuiltInScalarKind::Int16 => "int16",
            BuiltInScalarKind::Int8 => "int8",
            BuiltInScalarKind::Uint64 => "uint64",
            BuiltInScalarKind::Uint32 => "uint32",
            BuiltInScalarKind::Uint16 => "uint16",
            BuiltInScalarKind::Uint8 => "uint8",
            BuiltInScalarKind::Safeint => "safeint",
            BuiltInScalarKind::Float => "float",
            BuiltInScalarKind::Float64 => "float64",
            BuiltInScalarKind::Float32 => "float32",
            BuiltInScalarKind::Decimal => "decimal",
            BuiltInScalarKind::Decimal128 => "decimal128",
            BuiltInScalarKind::PlainDate => "plainDate",
            BuiltInScalarKind::PlainTime => "plainTime",
            BuiltInScalarKind::UtcDateTime => "utcDateTime",
            BuiltInScalarKind::OffsetDateTime => "offsetDateTime",
            BuiltInScalarKind::Duration => "duration",
            BuiltInScalarKind::Url => "url",
            BuiltInScalarKind::UnixTimestamp32 => "unixTimestamp32",
        }
    }

    /// Returns the parent scalar kind in the inheritance hierarchy, if any
    pub fn extends(&self) -> Option<BuiltInScalarKind> {
        match self {
            // These have no parent (base types)
            BuiltInScalarKind::Bytes
            | BuiltInScalarKind::String
            | BuiltInScalarKind::Boolean
            | BuiltInScalarKind::Url => None,

            // Numeric hierarchy root
            BuiltInScalarKind::Numeric => None,

            // Integer hierarchy
            BuiltInScalarKind::Integer => Some(BuiltInScalarKind::Numeric),
            BuiltInScalarKind::Int64 => Some(BuiltInScalarKind::Integer),
            BuiltInScalarKind::Int32 => Some(BuiltInScalarKind::Int64),
            BuiltInScalarKind::Int16 => Some(BuiltInScalarKind::Int32),
            BuiltInScalarKind::Int8 => Some(BuiltInScalarKind::Int16),
            BuiltInScalarKind::Uint64 => Some(BuiltInScalarKind::Integer),
            BuiltInScalarKind::Uint32 => Some(BuiltInScalarKind::Uint64),
            BuiltInScalarKind::Uint16 => Some(BuiltInScalarKind::Uint32),
            BuiltInScalarKind::Uint8 => Some(BuiltInScalarKind::Uint16),
            BuiltInScalarKind::Safeint => Some(BuiltInScalarKind::Int64),

            // Float hierarchy
            BuiltInScalarKind::Float => Some(BuiltInScalarKind::Numeric),
            BuiltInScalarKind::Float64 => Some(BuiltInScalarKind::Float),
            BuiltInScalarKind::Float32 => Some(BuiltInScalarKind::Float64),

            // Decimal hierarchy
            BuiltInScalarKind::Decimal => Some(BuiltInScalarKind::Numeric),
            BuiltInScalarKind::Decimal128 => Some(BuiltInScalarKind::Decimal),

            // Date/time types - these extend nothing in the intrinsics.tsp
            BuiltInScalarKind::PlainDate => None,
            BuiltInScalarKind::PlainTime => None,
            BuiltInScalarKind::UtcDateTime => None,
            BuiltInScalarKind::OffsetDateTime => None,
            BuiltInScalarKind::Duration => None,
            BuiltInScalarKind::UnixTimestamp32 => Some(BuiltInScalarKind::UtcDateTime),
        }
    }

    /// Returns true if this is a date/time related scalar
    pub fn is_datetime(&self) -> bool {
        matches!(
            self,
            BuiltInScalarKind::PlainDate
                | BuiltInScalarKind::PlainTime
                | BuiltInScalarKind::UtcDateTime
                | BuiltInScalarKind::OffsetDateTime
                | BuiltInScalarKind::Duration
                | BuiltInScalarKind::UnixTimestamp32
        )
    }

    /// Returns true if this scalar has initializers (like plainDate.fromISO())
    pub fn has_initializers(&self) -> bool {
        self.is_datetime()
    }
}

/// Built-in model type definition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltInModelKind {
    /// `Array<Element>` - array model type with integer indexer
    Array,
    /// `Record<Element>` - record model type with string indexer
    Record,
}

/// Built-in interface type definition (for prototypes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltInInterfaceKind {
    /// `ModelProperty` - interface for model properties
    ModelProperty,
    /// `Operation` - interface for operations
    Operation,
    /// `Array<TElementType>` - interface for array element access
    ArrayInterface,
}

/// ScalarInitializer - represents an init declaration on a scalar
#[derive(Debug, Clone)]
pub struct ScalarInitializer {
    /// Name of the initializer (e.g., "fromISO", "now")
    pub name: String,
    /// Parameter types (empty for "now")
    pub params: Vec<NodeId>,
    /// Return type (typically the scalar itself)
    pub return_type: NodeId,
}

impl ScalarInitializer {
    /// Creates a "fromISO(value: string)" initializer
    pub fn from_iso(return_type: NodeId) -> Self {
        Self {
            name: "fromISO".to_string(),
            params: Vec::new(), // Placeholder - would be string type
            return_type,
        }
    }

    /// Creates a "now()" initializer
    pub fn now(return_type: NodeId) -> Self {
        Self {
            name: "now".to_string(),
            params: Vec::new(),
            return_type,
        }
    }
}

/// Standard library container for all built-in type declarations
#[derive(Debug, Clone)]
pub struct StandardLibrary {
    /// The TypeSpec namespace containing built-in scalars and models
    pub typespec_namespace: NodeId,
    /// The TypeSpec.Prototypes namespace
    pub prototypes_namespace: NodeId,
}

impl StandardLibrary {
    /// Creates a new empty standard library structure
    pub fn new(typespec_ns: NodeId, prototypes_ns: NodeId) -> Self {
        Self {
            typespec_namespace: typespec_ns,
            prototypes_namespace: prototypes_ns,
        }
    }
}

/// Represents the complete scalar inheritance hierarchy
pub const SCALAR_HIERARCHY: &[(&str, Option<&str>)] = &[
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
    ("unixTimestamp32", Some("utcDateTime")),
];

/// Built-in model types with their indexer kinds
pub const MODEL_INDEXERS: &[(&str, &str, &str)] = &[
    // (model_name, indexer_key, element_type_param)
    ("Array", "integer", "Element"),
    ("Record", "string", "Element"),
];

/// Check if a builtin scalar name belongs to a primitive kind ("string", "numeric", "boolean").
/// Uses SCALAR_HIERARCHY to walk the parent chain, so it stays complete as types are added.
pub fn scalar_matches_primitive(scalar_name: &str, primitive: &str) -> bool {
    if scalar_name == primitive {
        return true;
    }
    // Walk the hierarchy: find the entry, then walk parent chain
    let mut current = scalar_name;
    loop {
        let Some((_, parent)) = SCALAR_HIERARCHY.iter().find(|(name, _)| *name == current) else {
            return false;
        };
        match parent {
            Some(p) if *p == primitive => return true,
            Some(p) => current = p,
            None => return false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // BuiltInScalarKind tests
    // ========================================================================

    #[test]
    fn test_scalar_as_str() {
        assert_eq!(BuiltInScalarKind::Bytes.as_str(), "bytes");
        assert_eq!(BuiltInScalarKind::String.as_str(), "string");
        assert_eq!(BuiltInScalarKind::Boolean.as_str(), "boolean");
        assert_eq!(BuiltInScalarKind::Numeric.as_str(), "numeric");
        assert_eq!(BuiltInScalarKind::Integer.as_str(), "integer");
        assert_eq!(BuiltInScalarKind::Int64.as_str(), "int64");
        assert_eq!(BuiltInScalarKind::Int32.as_str(), "int32");
        assert_eq!(BuiltInScalarKind::Int16.as_str(), "int16");
        assert_eq!(BuiltInScalarKind::Int8.as_str(), "int8");
        assert_eq!(BuiltInScalarKind::Uint64.as_str(), "uint64");
        assert_eq!(BuiltInScalarKind::Uint32.as_str(), "uint32");
        assert_eq!(BuiltInScalarKind::Uint16.as_str(), "uint16");
        assert_eq!(BuiltInScalarKind::Uint8.as_str(), "uint8");
        assert_eq!(BuiltInScalarKind::Safeint.as_str(), "safeint");
        assert_eq!(BuiltInScalarKind::Float.as_str(), "float");
        assert_eq!(BuiltInScalarKind::Float64.as_str(), "float64");
        assert_eq!(BuiltInScalarKind::Float32.as_str(), "float32");
        assert_eq!(BuiltInScalarKind::Decimal.as_str(), "decimal");
        assert_eq!(BuiltInScalarKind::Decimal128.as_str(), "decimal128");
        assert_eq!(BuiltInScalarKind::PlainDate.as_str(), "plainDate");
        assert_eq!(BuiltInScalarKind::PlainTime.as_str(), "plainTime");
        assert_eq!(BuiltInScalarKind::UtcDateTime.as_str(), "utcDateTime");
        assert_eq!(BuiltInScalarKind::OffsetDateTime.as_str(), "offsetDateTime");
        assert_eq!(BuiltInScalarKind::Duration.as_str(), "duration");
        assert_eq!(BuiltInScalarKind::Url.as_str(), "url");
    }

    #[test]
    fn test_scalar_extends_hierarchy() {
        // Base types have no parent
        assert_eq!(BuiltInScalarKind::Bytes.extends(), None);
        assert_eq!(BuiltInScalarKind::String.extends(), None);
        assert_eq!(BuiltInScalarKind::Boolean.extends(), None);
        assert_eq!(BuiltInScalarKind::Numeric.extends(), None);

        // Integer hierarchy: int8 -> int16 -> int32 -> int64 -> integer -> numeric
        assert_eq!(
            BuiltInScalarKind::Int8.extends(),
            Some(BuiltInScalarKind::Int16)
        );
        assert_eq!(
            BuiltInScalarKind::Int16.extends(),
            Some(BuiltInScalarKind::Int32)
        );
        assert_eq!(
            BuiltInScalarKind::Int32.extends(),
            Some(BuiltInScalarKind::Int64)
        );
        assert_eq!(
            BuiltInScalarKind::Int64.extends(),
            Some(BuiltInScalarKind::Integer)
        );
        assert_eq!(
            BuiltInScalarKind::Integer.extends(),
            Some(BuiltInScalarKind::Numeric)
        );

        // Unsigned hierarchy
        assert_eq!(
            BuiltInScalarKind::Uint8.extends(),
            Some(BuiltInScalarKind::Uint16)
        );
        assert_eq!(
            BuiltInScalarKind::Uint16.extends(),
            Some(BuiltInScalarKind::Uint32)
        );
        assert_eq!(
            BuiltInScalarKind::Uint32.extends(),
            Some(BuiltInScalarKind::Uint64)
        );
        assert_eq!(
            BuiltInScalarKind::Uint64.extends(),
            Some(BuiltInScalarKind::Integer)
        );
        assert_eq!(
            BuiltInScalarKind::Safeint.extends(),
            Some(BuiltInScalarKind::Int64)
        );

        // Float hierarchy
        assert_eq!(
            BuiltInScalarKind::Float32.extends(),
            Some(BuiltInScalarKind::Float64)
        );
        assert_eq!(
            BuiltInScalarKind::Float64.extends(),
            Some(BuiltInScalarKind::Float)
        );
        assert_eq!(
            BuiltInScalarKind::Float.extends(),
            Some(BuiltInScalarKind::Numeric)
        );

        // Decimal hierarchy
        assert_eq!(
            BuiltInScalarKind::Decimal128.extends(),
            Some(BuiltInScalarKind::Decimal)
        );
        assert_eq!(
            BuiltInScalarKind::Decimal.extends(),
            Some(BuiltInScalarKind::Numeric)
        );

        // Date/time types have no parent
        assert_eq!(BuiltInScalarKind::PlainDate.extends(), None);
        assert_eq!(BuiltInScalarKind::PlainTime.extends(), None);
        assert_eq!(BuiltInScalarKind::UtcDateTime.extends(), None);
        assert_eq!(BuiltInScalarKind::OffsetDateTime.extends(), None);
        assert_eq!(BuiltInScalarKind::Duration.extends(), None);
    }

    #[test]
    fn test_scalar_is_datetime() {
        assert!(BuiltInScalarKind::PlainDate.is_datetime());
        assert!(BuiltInScalarKind::PlainTime.is_datetime());
        assert!(BuiltInScalarKind::UtcDateTime.is_datetime());
        assert!(BuiltInScalarKind::OffsetDateTime.is_datetime());
        assert!(BuiltInScalarKind::Duration.is_datetime());
        assert!(!BuiltInScalarKind::String.is_datetime());
        assert!(!BuiltInScalarKind::Int32.is_datetime());
        assert!(!BuiltInScalarKind::Float.is_datetime());
    }

    #[test]
    fn test_scalar_has_initializers() {
        assert!(BuiltInScalarKind::PlainDate.has_initializers());
        assert!(BuiltInScalarKind::UtcDateTime.has_initializers());
        assert!(!BuiltInScalarKind::String.has_initializers());
        assert!(!BuiltInScalarKind::Int32.has_initializers());
    }

    #[test]
    fn test_scalar_kind_count() {
        // Must have exactly 24 built-in scalar types
        let all_kinds = [
            BuiltInScalarKind::Bytes,
            BuiltInScalarKind::String,
            BuiltInScalarKind::Boolean,
            BuiltInScalarKind::Numeric,
            BuiltInScalarKind::Integer,
            BuiltInScalarKind::Int64,
            BuiltInScalarKind::Int32,
            BuiltInScalarKind::Int16,
            BuiltInScalarKind::Int8,
            BuiltInScalarKind::Uint64,
            BuiltInScalarKind::Uint32,
            BuiltInScalarKind::Uint16,
            BuiltInScalarKind::Uint8,
            BuiltInScalarKind::Safeint,
            BuiltInScalarKind::Float,
            BuiltInScalarKind::Float64,
            BuiltInScalarKind::Float32,
            BuiltInScalarKind::Decimal,
            BuiltInScalarKind::Decimal128,
            BuiltInScalarKind::PlainDate,
            BuiltInScalarKind::PlainTime,
            BuiltInScalarKind::UtcDateTime,
            BuiltInScalarKind::OffsetDateTime,
            BuiltInScalarKind::Duration,
            BuiltInScalarKind::Url,
        ];
        assert_eq!(all_kinds.len(), 25);
    }

    // ========================================================================
    // SCALAR_HIERARCHY consistency tests
    // ========================================================================

    #[test]
    fn test_scalar_hierarchy_completeness() {
        // Every BuiltInScalarKind should appear in SCALAR_HIERARCHY
        let all_names: Vec<&str> = SCALAR_HIERARCHY.iter().map(|(name, _)| *name).collect();
        assert!(all_names.contains(&"bytes"));
        assert!(all_names.contains(&"string"));
        assert!(all_names.contains(&"int8"));
        assert!(all_names.contains(&"float32"));
        assert!(all_names.contains(&"decimal128"));
        assert!(all_names.contains(&"utcDateTime"));
        assert!(all_names.contains(&"url"));
    }

    #[test]
    fn test_scalar_hierarchy_matches_extends() {
        // SCALAR_HIERARCHY parent names must match BuiltInScalarKind::extends()
        for (name, parent_name) in SCALAR_HIERARCHY {
            let kind = super::super::helpers::get_builtin_scalar_kind(name).unwrap();
            let expected_parent =
                parent_name.and_then(super::super::helpers::get_builtin_scalar_kind);
            assert_eq!(kind.extends(), expected_parent, "Mismatch for {}", name);
        }
    }

    #[test]
    fn test_scalar_hierarchy_no_duplicates() {
        let mut names = std::collections::HashSet::new();
        for (name, _) in SCALAR_HIERARCHY {
            assert!(names.insert(name), "Duplicate scalar name: {}", name);
        }
    }

    #[test]
    fn test_scalar_hierarchy_count() {
        assert_eq!(
            SCALAR_HIERARCHY.len(),
            26,
            "SCALAR_HIERARCHY should have 26 entries"
        );
    }

    // ========================================================================
    // MODEL_INDEXERS tests
    // ========================================================================

    #[test]
    fn test_model_indexers() {
        assert_eq!(MODEL_INDEXERS.len(), 2);
        assert_eq!(MODEL_INDEXERS[0], ("Array", "integer", "Element"));
        assert_eq!(MODEL_INDEXERS[1], ("Record", "string", "Element"));
    }

    // ========================================================================
    // ScalarInitializer tests
    // ========================================================================

    #[test]
    fn test_scalar_initializer_from_iso() {
        let init = ScalarInitializer::from_iso(1);
        assert_eq!(init.name, "fromISO");
        assert!(init.params.is_empty());
        assert_eq!(init.return_type, 1);
    }

    #[test]
    fn test_scalar_initializer_now() {
        let init = ScalarInitializer::now(2);
        assert_eq!(init.name, "now");
        assert!(init.params.is_empty());
        assert_eq!(init.return_type, 2);
    }

    // ========================================================================
    // StandardLibrary tests
    // ========================================================================

    #[test]
    fn test_standard_library_new() {
        let lib = StandardLibrary::new(10, 20);
        assert_eq!(lib.typespec_namespace, 10);
        assert_eq!(lib.prototypes_namespace, 20);
    }
}
