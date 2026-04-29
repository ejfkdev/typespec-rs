//! @typespec/protobuf - Protocol Buffers Types and Decorators
//!
//! Ported from TypeSpec packages/protobuf
//!
//! Provides types and decorators for Protocol Buffers (Protobuf) modeling:
//! - `@message` - Declare a model as a Protobuf message
//! - `@field(index)` - Define field index for a message property
//! - `@reserve(...)` - Reserve field indices, ranges, or names
//! - `@service` - Declare an interface as a Protobuf service
//! - `@package(details?)` - Declare a namespace as a Protobuf package
//! - `@stream(mode)` - Set streaming mode for an operation
//!
//! Custom scalar types:
//! - `sint32`, `sint64` - Signed integer encodings
//! - `sfixed32`, `sfixed64` - Signed fixed-width encodings
//! - `fixed32`, `fixed64` - Unsigned fixed-width encodings
//!
//! Types:
//! - `Extern<Path, Name>` - External Protobuf reference
//! - `Map<Key, Value>` - Protobuf map type
//! - `StreamMode` enum - Streaming mode for operations
//! - `PackageDetails` model - Package configuration
//!
//! Well-known types:
//! - `Empty` - google.protobuf.Empty
//! - `Timestamp` - google.protobuf.Timestamp
//! - `Any` - google.protobuf.Any
//! - `LatLng` - google.type.LatLng
//!
//! ## Helper Functions
//! - `is_map(state, target)` - Check if a model is a Protobuf map
//! - `is_message(state, target)` - Check if a type is a Protobuf message

use crate::checker::types::TypeId;
use crate::diagnostics::{DiagnosticDefinition, DiagnosticMap};
use crate::state_accessors::StateAccessors;
use std::collections::HashMap;

// ============================================================================
// Diagnostic codes
// ============================================================================

pub const DIAG_FIELD_INDEX_MISSING: &str = "field-index/missing";
pub const DIAG_FIELD_INDEX_INVALID: &str = "field-index/invalid";
pub const DIAG_FIELD_INDEX_OUT_OF_BOUNDS: &str = "field-index/out-of-bounds";
pub const DIAG_FIELD_INDEX_RESERVED: &str = "field-index/reserved";
pub const DIAG_FIELD_INDEX_USER_RESERVED: &str = "field-index/user-reserved";
pub const DIAG_FIELD_INDEX_USER_RESERVED_RANGE: &str = "field-index/user-reserved-range";
pub const DIAG_FIELD_NAME_USER_RESERVED: &str = "field-name/user-reserved";
pub const DIAG_ROOT_OPERATION: &str = "root-operation";
pub const DIAG_UNSUPPORTED_INTRINSIC: &str = "unsupported-intrinsic";
pub const DIAG_UNSUPPORTED_RETURN_TYPE: &str = "unsupported-return-type";
pub const DIAG_UNSUPPORTED_INPUT_TYPE: &str = "unsupported-input-type";
pub const DIAG_UNSUPPORTED_FIELD_TYPE: &str = "unsupported-field-type";
pub const DIAG_NAMESPACE_COLLISION: &str = "namespace-collision";
pub const DIAG_UNCONVERTIBLE_ENUM: &str = "unconvertible-enum";
pub const DIAG_NESTED_ARRAY: &str = "nested-array";
pub const DIAG_INVALID_PACKAGE_NAME: &str = "invalid-package-name";
pub const DIAG_ILLEGAL_RESERVATION: &str = "illegal-reservation";
pub const DIAG_MODEL_NOT_IN_PACKAGE: &str = "model-not-in-package";
pub const DIAG_ANONYMOUS_MODEL: &str = "anonymous-model";
pub const DIAG_UNSPEAKABLE_TEMPLATE_ARGUMENT: &str = "unspeakable-template-argument";
pub const DIAG_PACKAGE: &str = "package";

// ============================================================================
// State keys (fully qualified with namespace)
// ============================================================================

/// State key for @field decorator
pub const STATE_FIELD_INDEX: &str = "TypeSpec.Protobuf.fieldIndex";
/// State key for @package decorator
pub const STATE_PACKAGE: &str = "TypeSpec.Protobuf.package";
/// State key for @service decorator
pub const STATE_SERVICE: &str = "TypeSpec.Protobuf.service";
/// State key for extern reference
pub const STATE_EXTERN_REF: &str = "TypeSpec.Protobuf.externRef";
/// State key for @stream decorator
pub const STATE_STREAM: &str = "TypeSpec.Protobuf.stream";
/// State key for @reserve decorator
pub const STATE_RESERVE: &str = "TypeSpec.Protobuf.reserve";
/// State key for @message decorator
pub const STATE_MESSAGE: &str = "TypeSpec.Protobuf.message";
/// State key for Map marker
pub const STATE_MAP: &str = "TypeSpec.Protobuf._map";

/// Namespace for Protobuf types
pub const PROTOBUF_NAMESPACE: &str = "TypeSpec.Protobuf";

// ============================================================================
// StreamMode enum
// ============================================================================

string_enum! {
    /// Streaming mode for an operation.
    /// Ported from TS StreamMode enum.
    pub enum StreamMode {
        /// Both input and output are streaming
        Duplex => "Duplex",
        /// Input is streaming (client streaming)
        In => "In",
        /// Output is streaming (server streaming)
        Out => "Out",
        /// Neither input nor output are streaming
        None => "None",
    }
}

// ============================================================================
// Protobuf scalar types (beyond standard TypeSpec scalars)
// ============================================================================

string_enum! {
    /// Custom Protobuf scalar types.
    /// These extend standard TypeSpec integer types with Protobuf-specific encodings.
    pub enum ProtobufScalarKind {
        /// signed int32 (sint32 encoding)
        Sint32 => "sint32",
        /// signed int64 (sint64 encoding)
        Sint64 => "sint64",
        /// signed fixed 32 (sfixed32 encoding)
        Sfixed32 => "sfixed32",
        /// signed fixed 64 (sfixed64 encoding)
        Sfixed64 => "sfixed64",
        /// unsigned fixed 32 (fixed32 encoding)
        Fixed32 => "fixed32",
        /// unsigned fixed 64 (fixed64 encoding)
        Fixed64 => "fixed64",
    }
}

impl ProtobufScalarKind {
    /// Get the parent TypeSpec scalar
    pub fn extends_scalar(&self) -> &'static str {
        match self {
            ProtobufScalarKind::Sint32 | ProtobufScalarKind::Sfixed32 => "int32",
            ProtobufScalarKind::Sint64 | ProtobufScalarKind::Sfixed64 => "int64",
            ProtobufScalarKind::Fixed32 => "uint32",
            ProtobufScalarKind::Fixed64 => "uint64",
        }
    }

    /// Get all custom protobuf scalar kinds
    pub fn all() -> &'static [ProtobufScalarKind] {
        &[
            ProtobufScalarKind::Sint32,
            ProtobufScalarKind::Sint64,
            ProtobufScalarKind::Sfixed32,
            ProtobufScalarKind::Sfixed64,
            ProtobufScalarKind::Fixed32,
            ProtobufScalarKind::Fixed64,
        ]
    }
}

// ============================================================================
// Field reservation types
// ============================================================================

/// A field reservation entry.
/// Ported from TS @reserve decorator arguments.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldReservation {
    /// Reserved field name
    Name(String),
    /// Reserved field index
    Index(u32),
    /// Reserved field index range (inclusive)
    Range(u32, u32),
}

impl FieldReservation {
    /// Check if a field index falls within this reservation
    pub fn contains_index(&self, index: u32) -> bool {
        match self {
            FieldReservation::Name(_) => false,
            FieldReservation::Index(i) => *i == index,
            FieldReservation::Range(start, end) => index >= *start && index <= *end,
        }
    }

    /// Check if a field name matches this reservation
    pub fn contains_name(&self, name: &str) -> bool {
        match self {
            FieldReservation::Name(n) => n == name,
            _ => false,
        }
    }
}

/// Protobuf field index constraints
pub const FIELD_INDEX_MIN: u32 = 1;
pub const FIELD_INDEX_MAX: u32 = 536870911; // 2^29 - 1
/// Implementation-reserved range
pub const FIELD_INDEX_RESERVED_MIN: u32 = 19000;
pub const FIELD_INDEX_RESERVED_MAX: u32 = 19999;

/// Check if a field index is valid (not in reserved range)
pub fn is_valid_field_index(index: u32) -> bool {
    (FIELD_INDEX_MIN..=FIELD_INDEX_MAX).contains(&index)
        && !(FIELD_INDEX_RESERVED_MIN..=FIELD_INDEX_RESERVED_MAX).contains(&index)
}

/// Check if a field index is reserved by the user
pub fn is_user_reserved(index: u32, reservations: &[FieldReservation]) -> bool {
    reservations.iter().any(|r| r.contains_index(index))
}

/// Check if a field name is reserved by the user
pub fn is_name_reserved(name: &str, reservations: &[FieldReservation]) -> bool {
    reservations.iter().any(|r| r.contains_name(name))
}

// ============================================================================
// Protobuf identifier validation
// ============================================================================

/// Protobuf full identifier regex.
/// Defined in the [ProtoBuf Language Spec](https://developers.google.com/protocol-buffers/docs/reference/proto3-spec#identifiers).
/// ident = letter { letter | decimalDigit | "_" }
/// fullIdent = ident { "." ident }
pub const PROTO_FULL_IDENT: &str = r"([a-zA-Z][a-zA-Z0-9_]*)+";

/// Check if a string is a valid Protobuf package name.
/// Must consist of letters and numbers separated by ".".
pub fn is_valid_proto_package_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    for segment in name.split('.') {
        if segment.is_empty() {
            return false;
        }
        let chars: Vec<char> = segment.chars().collect();
        if !chars[0].is_ascii_alphabetic() {
            return false;
        }
        if !chars.iter().all(|c| c.is_ascii_alphanumeric() || *c == '_') {
            return false;
        }
    }
    true
}

// ============================================================================
// Decorator implementations
// ============================================================================

flag_decorator!(apply_message, is_message, STATE_MESSAGE);
flag_decorator!(apply_service, is_service, STATE_SERVICE);

/// Implementation of the `@package` decorator.
/// Associates a namespace with a Protobuf package.
/// Ported from TS $package.
pub fn apply_package(state: &mut StateAccessors, target: TypeId, details: Option<&PackageDetails>) {
    let value = details
        .as_ref()
        .map(|d| d.name.as_deref().unwrap_or(""))
        .unwrap_or("");
    state.set_state(STATE_PACKAGE, target, value.to_string());
}

/// Get the package details for a namespace.
pub fn get_package(state: &StateAccessors, target: TypeId) -> Option<String> {
    state
        .get_state(STATE_PACKAGE, target)
        .map(|s| s.to_string())
}

/// Implementation of the `@field` decorator.
/// Sets the field index for a model property.
/// Ported from TS $field.
/// Returns Ok(()) if valid, or Err(diag_code) if invalid.
pub fn apply_field(
    state: &mut StateAccessors,
    target: TypeId,
    field_index: u32,
) -> Result<(), &'static str> {
    if !is_valid_field_index(field_index) {
        if field_index == 0 || !((FIELD_INDEX_MIN..=FIELD_INDEX_MAX).contains(&field_index)) {
            if field_index == 0 {
                return Err(DIAG_FIELD_INDEX_INVALID);
            }
            return Err(DIAG_FIELD_INDEX_OUT_OF_BOUNDS);
        }
        if (FIELD_INDEX_RESERVED_MIN..=FIELD_INDEX_RESERVED_MAX).contains(&field_index) {
            return Err(DIAG_FIELD_INDEX_RESERVED);
        }
    }
    state.set_state(STATE_FIELD_INDEX, target, field_index.to_string());
    Ok(())
}

/// Get the field index for a model property.
pub fn get_field_index(state: &StateAccessors, target: TypeId) -> Option<u32> {
    state
        .get_state(STATE_FIELD_INDEX, target)
        .and_then(|s| s.parse::<u32>().ok())
}

/// Implementation of the `@stream` decorator.
/// Sets the streaming mode for an operation.
/// Ported from TS $stream.
pub fn apply_stream(state: &mut StateAccessors, target: TypeId, mode: StreamMode) {
    state.set_state(STATE_STREAM, target, mode.as_str().to_string());
}

/// Get the streaming mode for an operation.
pub fn get_stream_mode(state: &StateAccessors, target: TypeId) -> Option<StreamMode> {
    state
        .get_state(STATE_STREAM, target)
        .and_then(StreamMode::parse_str)
}

/// Implementation of the `@reserve` decorator.
/// Stores field reservations for a type.
/// Ported from TS $reserve.
pub fn apply_reserve(
    state: &mut StateAccessors,
    target: TypeId,
    reservations: &[FieldReservation],
) {
    // Serialize reservations as semicolon-separated entries
    let serialized: String = reservations
        .iter()
        .map(|r| match r {
            FieldReservation::Name(n) => format!("n:{}", n),
            FieldReservation::Index(i) => format!("i:{}", i),
            FieldReservation::Range(s, e) => format!("r:{}-{}", s, e),
        })
        .collect::<Vec<_>>()
        .join(";");
    state.set_state(STATE_RESERVE, target, serialized);
}

/// Get field reservations for a type.
pub fn get_reservations(state: &StateAccessors, target: TypeId) -> Vec<FieldReservation> {
    state
        .get_state(STATE_RESERVE, target)
        .map(|s| {
            if s.is_empty() {
                return Vec::new();
            }
            s.split(';')
                .filter_map(|entry| {
                    let parts: Vec<&str> = entry.splitn(2, ':').collect();
                    if parts.len() != 2 {
                        return None;
                    }
                    match parts[0] {
                        "n" => Some(FieldReservation::Name(parts[1].to_string())),
                        "i" => parts[1].parse::<u32>().ok().map(FieldReservation::Index),
                        "r" => {
                            let range_parts: Vec<&str> = parts[1].splitn(2, '-').collect();
                            if range_parts.len() == 2 {
                                let start = range_parts[0].parse::<u32>().ok()?;
                                let end = range_parts[1].parse::<u32>().ok()?;
                                Some(FieldReservation::Range(start, end))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

flag_decorator!(apply_map, is_map, STATE_MAP);

/// Implementation of the internal `@externRef` decorator.
/// Stores an external Protobuf reference.
/// Ported from TS $externRef.
pub fn apply_extern_ref(state: &mut StateAccessors, target: TypeId, path: &str, name: &str) {
    state.set_state(STATE_EXTERN_REF, target, format!("{}|{}", path, name));
}

/// Get the external Protobuf reference for a model.
/// Returns (path, name) if found.
pub fn get_extern_ref(state: &StateAccessors, target: TypeId) -> Option<(String, String)> {
    state.get_state(STATE_EXTERN_REF, target).and_then(|s| {
        let parts: Vec<&str> = s.splitn(2, '|').collect();
        if parts.len() == 2 {
            Some((parts[0].to_string(), parts[1].to_string()))
        } else {
            None
        }
    })
}

// ============================================================================
// PackageDetails type
// ============================================================================

/// Details for a Protobuf package definition.
/// Ported from TS PackageDetails model.
#[derive(Debug, Clone)]
pub struct PackageDetails {
    /// The package name
    pub name: Option<String>,
    /// Package options
    pub options: Vec<(String, PackageOptionValue)>,
}

/// Value types for package options
#[derive(Debug, Clone, PartialEq)]
pub enum PackageOptionValue {
    /// String option value
    String(String),
    /// Boolean option value
    Boolean(bool),
    /// Numeric option value
    Numeric(f64),
}

// ============================================================================
// Emitter options
// ============================================================================

/// Protobuf emitter options.
/// Ported from TS ProtobufEmitterOptions interface.
#[derive(Debug, Clone, Default)]
pub struct ProtobufEmitterOptions {
    /// Don't emit anything
    pub no_emit: bool,
    /// Omit unreachable types
    pub omit_unreachable_types: bool,
}

// ============================================================================
// Protobuf type mapping
// ============================================================================

/// Mapping from TypeSpec scalar to Protobuf scalar type name.
/// Ported from TS proto.ts conversions.
pub fn typespec_scalar_to_proto(scalar_name: &str) -> Option<&'static str> {
    match scalar_name {
        "boolean" => Some("bool"),
        "string" => Some("string"),
        "bytes" => Some("bytes"),
        "int8" | "int16" | "int32" => Some("int32"),
        "int64" => Some("int64"),
        "uint8" | "uint16" | "uint32" => Some("uint32"),
        "uint64" => Some("uint64"),
        "safeint" => Some("int64"),
        "float32" => Some("float"),
        "float64" => Some("double"),
        "sint32" => Some("sint32"),
        "sint64" => Some("sint64"),
        "sfixed32" => Some("sfixed32"),
        "sfixed64" => Some("sfixed64"),
        "fixed32" => Some("fixed32"),
        "fixed64" => Some("fixed64"),
        _ => None,
    }
}

// ============================================================================
// Library creation
// ============================================================================

/// Create the @typespec/protobuf library diagnostic map.
/// Ported from TS TypeSpecProtobufLibrary definition in lib.ts.
pub fn create_protobuf_library() -> DiagnosticMap {
    HashMap::from([
        (
            "field-index".to_string(),
            DiagnosticDefinition::error_with_messages(vec![
                (
                    "missing",
                    "field {name} does not have a field index, but one is required (try using the '@field' decorator)",
                ),
                (
                    "invalid",
                    "field index {index} is invalid (must be an integer greater than zero)",
                ),
                (
                    "out-of-bounds",
                    "field index {index} is out of bounds (must be less than {max})",
                ),
                (
                    "reserved",
                    "field index {index} falls within the implementation-reserved range of 19000-19999 inclusive",
                ),
                (
                    "user-reserved",
                    "field index {index} was reserved by a call to @reserve on this model",
                ),
                (
                    "user-reserved-range",
                    "field index {index} falls within a range reserved by a call to @reserve on this model",
                ),
            ]),
        ),
        (
            "field-name".to_string(),
            DiagnosticDefinition::error_with_messages(vec![(
                "user-reserved",
                "field name '{name}' was reserved by a call to @reserve on this model",
            )]),
        ),
        (
            DIAG_ROOT_OPERATION.to_string(),
            DiagnosticDefinition::error(
                "operations in the root namespace are not supported (no associated Protobuf service)",
            ),
        ),
        (
            DIAG_UNSUPPORTED_INTRINSIC.to_string(),
            DiagnosticDefinition::error("intrinsic type {name} is not supported in Protobuf"),
        ),
        (
            DIAG_UNSUPPORTED_RETURN_TYPE.to_string(),
            DiagnosticDefinition::error("Protobuf methods must return a named Model"),
        ),
        (
            DIAG_UNSUPPORTED_INPUT_TYPE.to_string(),
            DiagnosticDefinition::error_with_messages(vec![
                (
                    "wrong-number",
                    "Protobuf methods must accept exactly one Model input (an empty model will do)",
                ),
                (
                    "wrong-type",
                    "Protobuf methods may only accept a named Model as an input",
                ),
                (
                    "unconvertible",
                    "input parameters cannot be converted to a Protobuf message",
                ),
            ]),
        ),
        (
            DIAG_UNSUPPORTED_FIELD_TYPE.to_string(),
            DiagnosticDefinition::error_with_messages(vec![
                (
                    "unconvertible",
                    "cannot convert a {type} to a protobuf type (only intrinsic types and models are supported)",
                ),
                (
                    "unknown-intrinsic",
                    "no known protobuf scalar for intrinsic type {name}",
                ),
                (
                    "unknown-scalar",
                    "no known protobuf scalar for TypeSpec scalar type {name}",
                ),
                (
                    "recursive-map",
                    "a protobuf map's 'value' type may not refer to another map",
                ),
                ("union", "a message field's type may not be a union"),
            ]),
        ),
        (
            DIAG_NAMESPACE_COLLISION.to_string(),
            DiagnosticDefinition::error("the package name {name} has already been used"),
        ),
        (
            DIAG_UNCONVERTIBLE_ENUM.to_string(),
            DiagnosticDefinition::error_with_messages(vec![
                (
                    "default",
                    "enums must explicitly assign exactly one integer to each member to be used in a Protobuf message",
                ),
                (
                    "no-zero-first",
                    "the first variant of an enum must be set to zero to be used in a Protobuf message",
                ),
            ]),
        ),
        (
            DIAG_NESTED_ARRAY.to_string(),
            DiagnosticDefinition::error("nested arrays are not supported by the Protobuf emitter"),
        ),
        (
            DIAG_INVALID_PACKAGE_NAME.to_string(),
            DiagnosticDefinition::error(
                "{name} is not a valid package name (must consist of letters and numbers separated by \".\")",
            ),
        ),
        (
            DIAG_ILLEGAL_RESERVATION.to_string(),
            DiagnosticDefinition::error(
                "reservation value must be a string literal, uint32 literal, or a tuple of two uint32 literals denoting a range",
            ),
        ),
        (
            DIAG_MODEL_NOT_IN_PACKAGE.to_string(),
            DiagnosticDefinition::error(
                "model {name} is not in a namespace that uses the '@Protobuf.package' decorator",
            ),
        ),
        (
            DIAG_ANONYMOUS_MODEL.to_string(),
            DiagnosticDefinition::error("anonymous models cannot be used in Protobuf messages"),
        ),
        (
            DIAG_UNSPEAKABLE_TEMPLATE_ARGUMENT.to_string(),
            DiagnosticDefinition::error(
                "template {name} cannot be converted to a Protobuf message because it has an unspeakable argument (try using the '@friendlyName' decorator on the template)",
            ),
        ),
        (
            DIAG_PACKAGE.to_string(),
            DiagnosticDefinition::error_with_messages(vec![(
                "disallowed-option-type",
                "option '{name}' with type '{type}' is not allowed in a package declaration (only string, boolean, and numeric types are allowed)",
            )]),
        ),
    ])
}

// ============================================================================
// TSP Sources
// ============================================================================

/// The TypeSpec source for the Protobuf library (proto.tsp)
pub const PROTOBUF_TSP: &str = r#"
import "../dist/src/tsp-index.js";

namespace TypeSpec.Protobuf;

/**
 * A model that represents an external Protobuf reference.
 * @template Path the relative path to a .proto file to import
 * @template Name the fully-qualified reference to the type
 */
@Private.externRef(Path, Name)
model Extern<Path extends string, Name extends string> {
  _extern: never;
}

/**
 * Contains some common well-known Protobuf types defined by the google.protobuf library.
 */
namespace WellKnown {
  model Empty is Extern<"google/protobuf/empty.proto", "google.protobuf.Empty">;
  model Timestamp is Extern<"google/protobuf/timestamp.proto", "google.protobuf.Timestamp">;
  model Any is Extern<"google/protobuf/any.proto", "google.protobuf.Any">;
  model LatLng is Extern<"google/type/latlng.proto", "google.type.LatLng">;
}

scalar sint32 extends int32;
scalar sint64 extends int64;
scalar sfixed32 extends int32;
scalar sfixed64 extends int64;
scalar fixed32 extends uint32;
scalar fixed64 extends uint64;

alias integral = int32 | int64 | uint32 | uint64 | boolean;

@Private._map
model Map<Key extends integral | string, Value> {}

extern dec message(target: {});
extern dec field(target: TypeSpec.Reflection.ModelProperty, index: valueof uint32);
extern dec reserve(target: {}, ...reservations: valueof (string | [uint32, uint32] | uint32)[]);
extern dec service(target: TypeSpec.Reflection.Interface);

model PackageDetails {
  name?: string;
  options?: Record<string | boolean | numeric>;
}

extern dec `package`(target: TypeSpec.Reflection.Namespace, details?: PackageDetails);

enum StreamMode {
  Duplex,
  In,
  Out,
  None,
}

extern dec stream(target: TypeSpec.Reflection.Operation, mode: StreamMode);

namespace Private {
  extern dec externRef(target: Reflection.Model, path: string, name: string);
  extern dec _map(target: Reflection.Model);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protobuf_namespace() {
        assert_eq!(PROTOBUF_NAMESPACE, "TypeSpec.Protobuf");
    }

    #[test]
    fn test_create_protobuf_library() {
        let diags = create_protobuf_library();
        assert!(diags.len() >= 15);
        let codes: Vec<&str> = diags.keys().map(|code| code.as_str()).collect();
        assert!(codes.contains(&"field-index"));
        assert!(codes.contains(&DIAG_UNSUPPORTED_FIELD_TYPE));
        assert!(codes.contains(&DIAG_NESTED_ARRAY));
    }

    #[test]
    fn test_stream_mode() {
        assert_eq!(StreamMode::Duplex.as_str(), "Duplex");
        assert_eq!(StreamMode::parse_str("Duplex"), Some(StreamMode::Duplex));
        assert_eq!(StreamMode::parse_str("In"), Some(StreamMode::In));
        assert_eq!(StreamMode::parse_str("Out"), Some(StreamMode::Out));
        assert_eq!(StreamMode::parse_str("None"), Some(StreamMode::None));
        assert_eq!(StreamMode::parse_str("unknown"), None);
    }

    #[test]
    fn test_protobuf_scalar_kinds() {
        for kind in ProtobufScalarKind::all() {
            assert!(!kind.as_str().is_empty());
            assert!(!kind.extends_scalar().is_empty());
        }
    }

    #[test]
    fn test_field_reservation() {
        let reservations = vec![
            FieldReservation::Name("test".to_string()),
            FieldReservation::Index(100),
            FieldReservation::Range(8, 15),
        ];

        assert!(is_name_reserved("test", &reservations));
        assert!(!is_name_reserved("other", &reservations));
        assert!(is_user_reserved(100, &reservations));
        assert!(is_user_reserved(10, &reservations)); // in range 8-15
        assert!(!is_user_reserved(7, &reservations));
        assert!(!is_user_reserved(16, &reservations));
    }

    #[test]
    fn test_valid_field_indices() {
        assert!(is_valid_field_index(1));
        assert!(is_valid_field_index(15));
        assert!(is_valid_field_index(18999));
        assert!(is_valid_field_index(20000));
        assert!(!is_valid_field_index(0));
        assert!(!is_valid_field_index(19000)); // reserved
        assert!(!is_valid_field_index(19999)); // reserved
        assert!(!is_valid_field_index(536870912)); // too large
    }

    #[test]
    fn test_typespec_to_proto_mapping() {
        assert_eq!(typespec_scalar_to_proto("boolean"), Some("bool"));
        assert_eq!(typespec_scalar_to_proto("string"), Some("string"));
        assert_eq!(typespec_scalar_to_proto("int32"), Some("int32"));
        assert_eq!(typespec_scalar_to_proto("int64"), Some("int64"));
        assert_eq!(typespec_scalar_to_proto("float32"), Some("float"));
        assert_eq!(typespec_scalar_to_proto("float64"), Some("double"));
        assert_eq!(typespec_scalar_to_proto("sint32"), Some("sint32"));
        assert_eq!(typespec_scalar_to_proto("fixed32"), Some("fixed32"));
        assert_eq!(typespec_scalar_to_proto("unknown"), None);
    }

    #[test]
    fn test_protobuf_tsp_not_empty() {
        assert!(!PROTOBUF_TSP.is_empty());
        assert!(PROTOBUF_TSP.contains("Extern"));
        assert!(PROTOBUF_TSP.contains("sint32"));
        assert!(PROTOBUF_TSP.contains("Map"));
        assert!(PROTOBUF_TSP.contains("StreamMode"));
        assert!(PROTOBUF_TSP.contains("message"));
        assert!(PROTOBUF_TSP.contains("field"));
        assert!(PROTOBUF_TSP.contains("reserve"));
        assert!(PROTOBUF_TSP.contains("service"));
    }

    #[test]
    fn test_package_details() {
        let details = PackageDetails {
            name: Some("test.package".to_string()),
            options: vec![
                (
                    "java_package".to_string(),
                    PackageOptionValue::String("com.test".to_string()),
                ),
                ("optimize_for".to_string(), PackageOptionValue::Numeric(1.0)),
            ],
        };
        assert_eq!(details.name, Some("test.package".to_string()));
        assert_eq!(details.options.len(), 2);
    }

    #[test]
    fn test_is_valid_proto_package_name() {
        assert!(is_valid_proto_package_name("com.example"));
        assert!(is_valid_proto_package_name("google.protobuf"));
        assert!(is_valid_proto_package_name("a"));
        assert!(!is_valid_proto_package_name(""));
        assert!(!is_valid_proto_package_name(".example"));
        assert!(!is_valid_proto_package_name("example."));
        assert!(!is_valid_proto_package_name("123.example"));
        assert!(!is_valid_proto_package_name("com.123"));
    }

    #[test]
    fn test_apply_and_is_message() {
        let mut state = StateAccessors::new();
        assert!(!is_message(&state, 1));
        apply_message(&mut state, 1);
        assert!(is_message(&state, 1));
        assert!(!is_message(&state, 2));
    }

    #[test]
    fn test_apply_and_is_service() {
        let mut state = StateAccessors::new();
        assert!(!is_service(&state, 1));
        apply_service(&mut state, 1);
        assert!(is_service(&state, 1));
    }

    #[test]
    fn test_apply_field_valid() {
        let mut state = StateAccessors::new();
        let result = apply_field(&mut state, 1, 1);
        assert!(result.is_ok());
        assert_eq!(get_field_index(&state, 1), Some(1));
    }

    #[test]
    fn test_apply_field_zero() {
        let mut state = StateAccessors::new();
        let result = apply_field(&mut state, 1, 0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DIAG_FIELD_INDEX_INVALID);
    }

    #[test]
    fn test_apply_field_reserved() {
        let mut state = StateAccessors::new();
        let result = apply_field(&mut state, 1, 19000);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DIAG_FIELD_INDEX_RESERVED);
    }

    #[test]
    fn test_apply_stream() {
        let mut state = StateAccessors::new();
        apply_stream(&mut state, 1, StreamMode::Duplex);
        assert_eq!(get_stream_mode(&state, 1), Some(StreamMode::Duplex));

        apply_stream(&mut state, 2, StreamMode::Out);
        assert_eq!(get_stream_mode(&state, 2), Some(StreamMode::Out));
    }

    #[test]
    fn test_apply_and_get_reservations() {
        let mut state = StateAccessors::new();
        let reservations = vec![
            FieldReservation::Name("test".to_string()),
            FieldReservation::Index(100),
            FieldReservation::Range(8, 15),
        ];
        apply_reserve(&mut state, 1, &reservations);

        let retrieved = get_reservations(&state, 1);
        assert_eq!(retrieved.len(), 3);
        assert!(
            retrieved
                .iter()
                .any(|r| matches!(r, FieldReservation::Name(n) if n == "test"))
        );
        assert!(
            retrieved
                .iter()
                .any(|r| matches!(r, FieldReservation::Index(100)))
        );
        assert!(
            retrieved
                .iter()
                .any(|r| matches!(r, FieldReservation::Range(8, 15)))
        );
    }

    #[test]
    fn test_apply_and_is_map() {
        let mut state = StateAccessors::new();
        assert!(!is_map(&state, 1));
        apply_map(&mut state, 1);
        assert!(is_map(&state, 1));
    }

    #[test]
    fn test_apply_and_get_extern_ref() {
        let mut state = StateAccessors::new();
        apply_extern_ref(
            &mut state,
            1,
            "google/protobuf/empty.proto",
            "google.protobuf.Empty",
        );

        let result = get_extern_ref(&state, 1);
        assert!(result.is_some());
        let (path, name) = result.unwrap();
        assert_eq!(path, "google/protobuf/empty.proto");
        assert_eq!(name, "google.protobuf.Empty");
    }

    #[test]
    fn test_apply_package() {
        let mut state = StateAccessors::new();
        let details = PackageDetails {
            name: Some("test.package".to_string()),
            options: vec![],
        };
        apply_package(&mut state, 1, Some(&details));
        assert_eq!(get_package(&state, 1), Some("test.package".to_string()));

        apply_package(&mut state, 2, None);
        assert_eq!(get_package(&state, 2), Some("".to_string()));
    }
}
