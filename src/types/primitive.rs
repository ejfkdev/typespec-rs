//! Primitive and intrinsic types for TypeSpec-Rust
//! Ported from TypeSpec compiler/src/core/types.ts

use crate::ast::node::NodeId;
use std::collections::HashMap;

/// Intrinsic scalar names - standard scalar types in TypeSpec
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntrinsicScalarName {
    Bytes,
    Numeric,
    Integer,
    Float,
    Int64,
    Int32,
    Int16,
    Int8,
    Uint64,
    Uint32,
    Uint16,
    Uint8,
    Safeint,
    Float32,
    Float64,
    Decimal,
    Decimal128,
    String,
    PlainDate,
    PlainTime,
    UtcDateTime,
    OffsetDateTime,
    Duration,
    Boolean,
    Url,
}

impl IntrinsicScalarName {
    pub fn as_str(&self) -> &'static str {
        match self {
            IntrinsicScalarName::Bytes => "bytes",
            IntrinsicScalarName::Numeric => "numeric",
            IntrinsicScalarName::Integer => "integer",
            IntrinsicScalarName::Float => "float",
            IntrinsicScalarName::Int64 => "int64",
            IntrinsicScalarName::Int32 => "int32",
            IntrinsicScalarName::Int16 => "int16",
            IntrinsicScalarName::Int8 => "int8",
            IntrinsicScalarName::Uint64 => "uint64",
            IntrinsicScalarName::Uint32 => "uint32",
            IntrinsicScalarName::Uint16 => "uint16",
            IntrinsicScalarName::Uint8 => "uint8",
            IntrinsicScalarName::Safeint => "safeint",
            IntrinsicScalarName::Float32 => "float32",
            IntrinsicScalarName::Float64 => "float64",
            IntrinsicScalarName::Decimal => "decimal",
            IntrinsicScalarName::Decimal128 => "decimal128",
            IntrinsicScalarName::String => "string",
            IntrinsicScalarName::PlainDate => "plainDate",
            IntrinsicScalarName::PlainTime => "plainTime",
            IntrinsicScalarName::UtcDateTime => "utcDateTime",
            IntrinsicScalarName::OffsetDateTime => "offsetDateTime",
            IntrinsicScalarName::Duration => "duration",
            IntrinsicScalarName::Boolean => "boolean",
            IntrinsicScalarName::Url => "url",
        }
    }
}

/// Intrinsic type names
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntrinsicTypeName {
    ErrorType,
    Void,
    Never,
    Unknown,
    Null,
}

/// TypeKind - enumeration of all type kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeKind {
    // Composite types
    Model,
    ModelProperty,
    Interface,
    Operation,
    Enum,
    EnumMember,
    Union,
    UnionVariant,
    Scalar,
    ScalarConstructor,

    // Literal types
    String,
    Number,
    Boolean,

    // Template types
    TemplateParameter,
    Tuple,

    // Special types
    Namespace,
    Decorator,
    FunctionType,
    FunctionParameter,

    // Intrinsic types
    Intrinsic,

    // String template
    StringTemplate,
    StringTemplateSpan,

    // Value types (for completeness)
    Value,
    ObjectValue,
    ArrayValue,
    ScalarValue,
    NumericValue,
    StringValue,
    BooleanValue,
    EnumValue,
    NullValue,
    FunctionValue,

    // Special entities
    TypeMapper,
    Indeterminate,
    MixedParameterConstraint,
}

/// Model indexer - enables array-style access on models
#[derive(Debug, Clone)]
pub struct ModelIndexer {
    /// The key type (must be a Scalar like string or integer)
    pub key: NodeId,
    /// The value type
    pub value: NodeId,
}

/// Type mapper - maps template parameters to their actual types
#[derive(Debug, Clone)]
pub struct TypeMapper {
    /// Whether this is a partial mapping
    pub partial: bool,
    /// The mapping from template parameters to types
    pub map: HashMap<NodeId, NodeId>,
    /// Arguments used for instantiation
    pub args: Vec<NodeId>,
    /// Source node used to create this mapper
    pub source_node: Option<NodeId>,
    /// Parent mapper if any
    pub parent_mapper: Option<Box<TypeMapper>>,
}

/// Mixed parameter constraint - represents a type or value constraint
#[derive(Debug, Clone)]
pub struct MixedParameterConstraint {
    /// Type constraint
    pub type_constraint: Option<NodeId>,
    /// Value type constraint
    pub value_type: Option<NodeId>,
    /// Source node
    pub node: Option<NodeId>,
}

/// Indeterminate entity - when something could be a type or value but isn't determined yet
#[derive(Debug, Clone)]
pub struct IndeterminateEntity {
    /// The underlying type
    pub type_id: NodeId,
}
