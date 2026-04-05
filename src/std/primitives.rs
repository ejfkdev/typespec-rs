//! Primitive type definitions for TypeSpec standard library
//!
//! Ported from TypeSpec compiler/lib/intrinsics.tsp
//!
//! This module defines all built-in scalar types and their inheritance hierarchy:
//! - bytes, string, boolean (base types)
//! - numeric -> integer -> int64 -> int32 -> int16 -> int8
//!                     -> uint64 -> uint32 -> uint16 -> uint8
//!                     -> safeint
//!           -> float -> float64 -> float32
//!           -> decimal -> decimal128
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
];

/// Built-in model types with their indexer kinds
pub const MODEL_INDEXERS: &[(&str, &str, &str)] = &[
    // (model_name, indexer_key, element_type_param)
    ("Array", "integer", "Element"),
    ("Record", "string", "Element"),
];
