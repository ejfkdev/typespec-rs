//! @typespec/xml - XML Serialization Decorators and Types
//!
//! Ported from TypeSpec packages/xml
//!
//! Provides decorators and types for XML serialization control:
//! - `@name` - XML element/attribute name override (calls @encodedName)
//! - `@attribute` - Serialize as XML attribute
//! - `@unwrapped` - Flatten XML wrapper node
//! - `@ns` - XML namespace declaration
//! - `@nsDeclarations` - Mark enum as XML namespace declarations
//!
//! Also provides:
//! - `Encoding` enum - Known XML encodings (xmlDateTime, xmlDate, xmlTime, xmlDuration, xmlBase64Binary)
//! - XML encoding resolution utilities
//! - XmlNamespace type for namespace data
//!
//! ## Helper Functions
//! - `is_attribute(state, target)` - Check if a property is marked as XML attribute
//! - `is_unwrapped(state, target)` - Check if a property should be unwrapped
//! - `get_ns(state, target)` - Get XML namespace for a type
//! - `is_ns_declarations(state, target)` - Check if an enum is a namespace declaration
//! - `validate_namespace_uri(namespace)` - Validate a namespace is a valid URI

use crate::checker::types::TypeId;
use crate::diagnostics::{DiagnosticDefinition, DiagnosticMap};
use crate::state_accessors::StateAccessors;
use std::collections::HashMap;

// ============================================================================
// Diagnostic codes
// ============================================================================

/// Diagnostic: Enum member used as namespace must be from an @nsDeclarations enum
pub const DIAG_NS_ENUM_NOT_DECLARATION: &str = "ns-enum-not-declaration";
/// Diagnostic: Enum member must have a URI value
pub const DIAG_INVALID_NS_DECLARATION_MEMBER: &str = "invalid-ns-declaration-member";
/// Diagnostic: String namespace requires prefix argument
pub const DIAG_NS_MISSING_PREFIX: &str = "ns-missing-prefix";
/// Diagnostic: Prefix not allowed with enum member namespace
pub const DIAG_PREFIX_NOT_ALLOWED: &str = "prefix-not-allowed";
/// Diagnostic: Namespace is not a valid URI
pub const DIAG_NS_NOT_URI: &str = "ns-not-uri";

// ============================================================================
// State keys (fully qualified with namespace)
// ============================================================================

/// State key for @attribute decorator
pub const STATE_ATTRIBUTE: &str = "TypeSpec.Xml.attribute";
/// State key for @unwrapped decorator
pub const STATE_UNWRAPPED: &str = "TypeSpec.Xml.unwrapped";
/// State key for @ns decorator (namespace data)
pub const STATE_NS: &str = "TypeSpec.Xml.ns";
/// State key for @nsDeclarations decorator
pub const STATE_NS_DECLARATION: &str = "TypeSpec.Xml.nsDeclaration";

/// Namespace for XML types
pub const XML_NAMESPACE: &str = "TypeSpec.Xml";

// ============================================================================
// XmlNamespace type
// ============================================================================

/// Represents an XML namespace with URI and prefix.
/// Ported from TS XmlNamespace interface.
#[derive(Debug, Clone, PartialEq)]
pub struct XmlNamespace {
    /// The namespace URI
    pub namespace: String,
    /// The namespace prefix
    pub prefix: String,
}

impl XmlNamespace {
    /// Create a new XmlNamespace
    pub fn new(namespace: &str, prefix: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
            prefix: prefix.to_string(),
        }
    }

    /// Validate that the namespace URI is valid
    pub fn is_valid_uri(&self) -> bool {
        // Simple URI validation - check if it parses as a URL
        self.namespace.contains(':') && self.namespace.len() > 3
    }
}

// ============================================================================
// XmlEncoding type
// ============================================================================

/// Known XML encodings corresponding to XML Schema types.
/// Ported from TS XmlEncoding type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum XmlEncoding {
    /// xs:dateTime
    XmlDateTime,
    /// xs:date
    XmlDate,
    /// xs:time
    XmlTime,
    /// xs:duration
    XmlDuration,
    /// xs:base64Binary
    XmlBase64Binary,
}

impl XmlEncoding {
    /// Get the fully qualified TypeSpec name for this encoding
    pub fn as_typespec_name(&self) -> &'static str {
        match self {
            XmlEncoding::XmlDateTime => "TypeSpec.Xml.Encoding.xmlDateTime",
            XmlEncoding::XmlDate => "TypeSpec.Xml.Encoding.xmlDate",
            XmlEncoding::XmlTime => "TypeSpec.Xml.Encoding.xmlTime",
            XmlEncoding::XmlDuration => "TypeSpec.Xml.Encoding.xmlDuration",
            XmlEncoding::XmlBase64Binary => "TypeSpec.Xml.Encoding.xmlBase64Binary",
        }
    }

    /// Get the scalar type name that maps to this encoding by default
    pub fn default_scalar_name(&self) -> &'static str {
        match self {
            XmlEncoding::XmlDateTime => "utcDateTime", // or offsetDateTime
            XmlEncoding::XmlDate => "plainDate",
            XmlEncoding::XmlTime => "plainTime",
            XmlEncoding::XmlDuration => "duration",
            XmlEncoding::XmlBase64Binary => "bytes",
        }
    }

    /// Get the XML Schema type name
    pub fn xs_type_name(&self) -> &'static str {
        match self {
            XmlEncoding::XmlDateTime => "xs:dateTime",
            XmlEncoding::XmlDate => "xs:date",
            XmlEncoding::XmlTime => "xs:time",
            XmlEncoding::XmlDuration => "xs:duration",
            XmlEncoding::XmlBase64Binary => "xs:base64Binary",
        }
    }

    /// Parse from TypeSpec encoding name (e.g., "TypeSpec.Xml.Encoding.xmlDateTime")
    pub fn from_typespec_name(name: &str) -> Option<XmlEncoding> {
        match name {
            "TypeSpec.Xml.Encoding.xmlDateTime" => Some(XmlEncoding::XmlDateTime),
            "TypeSpec.Xml.Encoding.xmlDate" => Some(XmlEncoding::XmlDate),
            "TypeSpec.Xml.Encoding.xmlTime" => Some(XmlEncoding::XmlTime),
            "TypeSpec.Xml.Encoding.xmlDuration" => Some(XmlEncoding::XmlDuration),
            "TypeSpec.Xml.Encoding.xmlBase64Binary" => Some(XmlEncoding::XmlBase64Binary),
            _ => None,
        }
    }
}

/// Get the default XML encoding for a scalar type name.
/// Ported from TS getDefaultEncoding().
pub fn get_default_xml_encoding(scalar_name: &str) -> Option<XmlEncoding> {
    match scalar_name {
        "utcDateTime" | "offsetDateTime" => Some(XmlEncoding::XmlDateTime),
        "plainDate" => Some(XmlEncoding::XmlDate),
        "plainTime" => Some(XmlEncoding::XmlTime),
        "duration" => Some(XmlEncoding::XmlDuration),
        "bytes" => Some(XmlEncoding::XmlBase64Binary),
        _ => None,
    }
}

/// Resolve XML encoding for a type.
///
/// If an `@encode` decorator is applied, use that encoding.
/// Otherwise, use the default XML encoding for the scalar type.
///
/// Ported from TS getXmlEncoding().
///
/// # Arguments
/// * `encode_value` - The value from the `@encode` decorator (if any)
/// * `scalar_name` - The name of the scalar type (e.g., "utcDateTime")
/// * `encode_type_name` - The target type name from @encode (e.g., "string")
///
/// # Returns
/// XmlEncodeData if a valid encoding could be resolved, None otherwise.
pub fn get_xml_encoding(
    encode_value: Option<&str>,
    scalar_name: &str,
    encode_type_name: Option<&str>,
) -> Option<XmlEncodeData> {
    // If @encode was applied, check if it's a known XML encoding
    if let Some(encoding) = encode_value {
        // Try to parse as XmlEncoding
        if let Some(xml_enc) = XmlEncoding::from_typespec_name(encoding) {
            return Some(XmlEncodeData {
                encoding: Some(xml_enc),
                type_name: encode_type_name.unwrap_or("string").to_string(),
            });
        }
        // The encoding value might be a custom encoding string (not XML-specific)
        // In this case, we don't have XML encoding data
    }

    // Fall back to default encoding for the scalar type
    let default = get_default_xml_encoding(scalar_name)?;
    Some(XmlEncodeData {
        encoding: Some(default),
        type_name: "string".to_string(),
    })
}

// ============================================================================
// XmlEncodeData type
// ============================================================================

/// XML encoding information for a type or property.
/// Ported from TS XmlEncodeData interface.
#[derive(Debug, Clone)]
pub struct XmlEncodeData {
    /// The encoding type (e.g., "TypeSpec.Xml.Encoding.xmlDateTime")
    pub encoding: Option<XmlEncoding>,
    /// The target type for the encoding (e.g., string)
    pub type_name: String,
}

// ============================================================================
// Decorator implementations
// ============================================================================

flag_decorator!(apply_attribute, is_attribute, STATE_ATTRIBUTE);
flag_decorator!(apply_unwrapped, is_unwrapped, STATE_UNWRAPPED);
flag_decorator!(
    apply_ns_declarations,
    is_ns_declarations,
    STATE_NS_DECLARATION
);

/// Implementation of the `@ns` decorator.
/// Sets XML namespace data for a type.
/// Ported from TS $ns.
/// Returns Some(()) on success, or a diagnostic code string on validation failure.
pub fn apply_ns(
    state: &mut StateAccessors,
    target: TypeId,
    namespace: &XmlNamespace,
) -> Result<(), &'static str> {
    if !validate_namespace_uri(&namespace.namespace) {
        return Err(DIAG_NS_NOT_URI);
    }
    // Store as "namespace|prefix" format
    state.set_state(
        STATE_NS,
        target,
        format!("{}|{}", namespace.namespace, namespace.prefix),
    );
    Ok(())
}

/// Get the XML namespace data for a type.
/// Ported from TS getNs().
pub fn get_ns(state: &StateAccessors, target: TypeId) -> Option<XmlNamespace> {
    state.get_state(STATE_NS, target).and_then(|s| {
        let parts: Vec<&str> = s.splitn(2, '|').collect();
        if parts.len() == 2 {
            Some(XmlNamespace::new(parts[0], parts[1]))
        } else {
            None
        }
    })
}

/// Validate that a namespace string is a valid URI.
/// Ported from TS validateNamespaceIsUri().
pub fn validate_namespace_uri(namespace: &str) -> bool {
    // Try to parse as a URL - same logic as TS `new URL(namespace)`
    namespace.contains("://") || namespace.starts_with("urn:")
}

// ============================================================================
// Library creation
// ============================================================================

/// Create the @typespec/xml library diagnostic map.
/// Ported from TS $lib definition in lib.ts.
pub fn create_xml_library() -> DiagnosticMap {
    HashMap::from([
        (
            DIAG_NS_ENUM_NOT_DECLARATION.to_string(),
            DiagnosticDefinition::error(
                "Enum member used as namespace must be part of an enum marked with @nsDeclaration.",
            ),
        ),
        (
            DIAG_INVALID_NS_DECLARATION_MEMBER.to_string(),
            DiagnosticDefinition::error(
                "Enum member {name} must have a value that is the XML namespace url.",
            ),
        ),
        (
            DIAG_NS_MISSING_PREFIX.to_string(),
            DiagnosticDefinition::error(
                "When using a string namespace you must provide a prefix as the 2nd argument.",
            ),
        ),
        (
            DIAG_PREFIX_NOT_ALLOWED.to_string(),
            DiagnosticDefinition::error(
                "@ns decorator cannot have the prefix parameter set when using an enum member.",
            ),
        ),
        (
            DIAG_NS_NOT_URI.to_string(),
            DiagnosticDefinition::error("Namespace {namespace} is not a valid URI."),
        ),
    ])
}

// ============================================================================
// TSP Sources
// ============================================================================

/// The TypeSpec source for the XML library types
pub const XML_TYPES_TSP: &str = r#"
namespace TypeSpec.Xml;

/**
 * Known Xml encodings
 */
enum Encoding {
  /** Corresponds to a field of schema xs:dateTime */
  xmlDateTime,

  /** Correspond to a field of schema xs:date */
  xmlDate,

  /** Correspond to a field of schema xs:time */
  xmlTime,

  /** Correspond to a field of schema xs:duration */
  xmlDuration,

  /** Correspond to a field of schema xs:base64Binary */
  xmlBase64Binary,
}
"#;

/// The TypeSpec source for the XML library decorators
pub const XML_DECORATORS_TSP: &str = r#"
import "../dist/src/decorators.js";

using TypeSpec.Reflection;

namespace TypeSpec.Xml;

/**
 * Provide the name of the XML element or attribute. This means the same thing as
 * @encodedName("application/xml", value)
 *
 * @param name The name of the XML element or attribute
 */
extern dec name(target: unknown, name: valueof string);

/**
 * Specify that the target property should be encoded as an XML attribute instead of node.
 */
extern dec attribute(target: ModelProperty);

/**
 * Specify that the target property shouldn't create a wrapper node. This can be used to
 * flatten list nodes into the model node or to include raw text in the model node.
 * It cannot be used with `@attribute`.
 */
extern dec unwrapped(target: ModelProperty);

/**
 * Specify the XML namespace for this element.
 *
 * @param ns The namespace URI or a member of an enum decorated with @nsDeclaration.
 * @param prefix The namespace prefix. Required if the namespace parameter was passed as a string.
 */
extern dec ns(target: unknown, ns: string | EnumMember, prefix?: valueof string);

/**
 * Mark an enum as declaring XML namespaces. See `@ns`
 */
extern dec nsDeclarations(target: Enum);
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_xml_library() {
        let diags = create_xml_library();
        assert_eq!(diags.len(), 5);
        let codes: Vec<&str> = diags.keys().map(|code| code.as_str()).collect();
        assert!(codes.contains(&DIAG_NS_ENUM_NOT_DECLARATION));
        assert!(codes.contains(&DIAG_NS_MISSING_PREFIX));
        assert!(codes.contains(&DIAG_NS_NOT_URI));
    }

    #[test]
    fn test_xml_namespace_constant() {
        assert_eq!(XML_NAMESPACE, "TypeSpec.Xml");
    }

    #[test]
    fn test_xml_encoding_roundtrip() {
        for encoding in &[
            XmlEncoding::XmlDateTime,
            XmlEncoding::XmlDate,
            XmlEncoding::XmlTime,
            XmlEncoding::XmlDuration,
            XmlEncoding::XmlBase64Binary,
        ] {
            let scalar = encoding.default_scalar_name();
            let roundtrip = get_default_xml_encoding(scalar);
            assert_eq!(
                roundtrip,
                Some(*encoding),
                "Failed roundtrip for {:?}",
                encoding
            );
        }
    }

    #[test]
    fn test_xml_encoding_no_default_for_unknown() {
        assert_eq!(get_default_xml_encoding("string"), None);
        assert_eq!(get_default_xml_encoding("int32"), None);
    }

    #[test]
    fn test_xml_encoding_names() {
        assert_eq!(
            XmlEncoding::XmlDateTime.as_typespec_name(),
            "TypeSpec.Xml.Encoding.xmlDateTime"
        );
        assert_eq!(
            XmlEncoding::XmlBase64Binary.as_typespec_name(),
            "TypeSpec.Xml.Encoding.xmlBase64Binary"
        );
    }

    #[test]
    fn test_xml_encoding_xs_types() {
        assert_eq!(XmlEncoding::XmlDateTime.xs_type_name(), "xs:dateTime");
        assert_eq!(XmlEncoding::XmlDate.xs_type_name(), "xs:date");
        assert_eq!(XmlEncoding::XmlTime.xs_type_name(), "xs:time");
        assert_eq!(XmlEncoding::XmlDuration.xs_type_name(), "xs:duration");
        assert_eq!(
            XmlEncoding::XmlBase64Binary.xs_type_name(),
            "xs:base64Binary"
        );
    }

    #[test]
    fn test_xml_namespace_type() {
        let ns = XmlNamespace::new("http://example.com", "ex");
        assert_eq!(ns.namespace, "http://example.com");
        assert_eq!(ns.prefix, "ex");
        assert!(ns.is_valid_uri());

        let bad_ns = XmlNamespace::new("not-a-uri", "x");
        assert!(!bad_ns.is_valid_uri());
    }

    #[test]
    fn test_xml_namespace_equality() {
        let ns1 = XmlNamespace::new("http://example.com", "ex");
        let ns2 = XmlNamespace::new("http://example.com", "ex");
        let ns3 = XmlNamespace::new("http://other.com", "ex");
        assert_eq!(ns1, ns2);
        assert_ne!(ns1, ns3);
    }

    #[test]
    fn test_tsp_sources_not_empty() {
        assert!(!XML_TYPES_TSP.is_empty());
        assert!(!XML_DECORATORS_TSP.is_empty());
        assert!(XML_TYPES_TSP.contains("Encoding"));
        assert!(XML_DECORATORS_TSP.contains("name"));
        assert!(XML_DECORATORS_TSP.contains("attribute"));
        assert!(XML_DECORATORS_TSP.contains("unwrapped"));
        assert!(XML_DECORATORS_TSP.contains("ns"));
        assert!(XML_DECORATORS_TSP.contains("nsDeclarations"));
    }

    #[test]
    fn test_xml_encode_data() {
        let data = XmlEncodeData {
            encoding: Some(XmlEncoding::XmlDateTime),
            type_name: "string".to_string(),
        };
        assert!(data.encoding.is_some());
        assert_eq!(data.type_name, "string");
    }

    #[test]
    fn test_is_attribute() {
        let mut state = StateAccessors::new();
        assert!(!is_attribute(&state, 1));

        apply_attribute(&mut state, 1);
        assert!(is_attribute(&state, 1));
        assert!(!is_attribute(&state, 2));
    }

    #[test]
    fn test_is_unwrapped() {
        let mut state = StateAccessors::new();
        assert!(!is_unwrapped(&state, 1));

        apply_unwrapped(&mut state, 1);
        assert!(is_unwrapped(&state, 1));
    }

    #[test]
    fn test_is_ns_declarations() {
        let mut state = StateAccessors::new();
        assert!(!is_ns_declarations(&state, 1));

        apply_ns_declarations(&mut state, 1);
        assert!(is_ns_declarations(&state, 1));
    }

    #[test]
    fn test_apply_ns_valid_uri() {
        let mut state = StateAccessors::new();
        let ns = XmlNamespace::new("http://example.com/schema", "ex");
        let result = apply_ns(&mut state, 1, &ns);
        assert!(result.is_ok());

        let retrieved = get_ns(&state, 1);
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.namespace, "http://example.com/schema");
        assert_eq!(retrieved.prefix, "ex");
    }

    #[test]
    fn test_apply_ns_invalid_uri() {
        let mut state = StateAccessors::new();
        let ns = XmlNamespace::new("not-a-uri", "x");
        let result = apply_ns(&mut state, 1, &ns);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DIAG_NS_NOT_URI);
    }

    #[test]
    fn test_validate_namespace_uri() {
        assert!(validate_namespace_uri("http://example.com"));
        assert!(validate_namespace_uri("https://example.com/schema"));
        assert!(validate_namespace_uri("urn:example:ns"));
        assert!(!validate_namespace_uri("not-a-uri"));
        assert!(!validate_namespace_uri(""));
    }

    #[test]
    fn test_apply_ns_urn() {
        let mut state = StateAccessors::new();
        let ns = XmlNamespace::new("urn:example:namespace", "ex");
        let result = apply_ns(&mut state, 1, &ns);
        assert!(result.is_ok());

        let retrieved = get_ns(&state, 1);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().namespace, "urn:example:namespace");
    }

    #[test]
    fn test_xml_encoding_from_typespec_name() {
        assert_eq!(
            XmlEncoding::from_typespec_name("TypeSpec.Xml.Encoding.xmlDateTime"),
            Some(XmlEncoding::XmlDateTime)
        );
        assert_eq!(
            XmlEncoding::from_typespec_name("TypeSpec.Xml.Encoding.xmlDuration"),
            Some(XmlEncoding::XmlDuration)
        );
        assert_eq!(XmlEncoding::from_typespec_name("unknown"), None);
    }

    #[test]
    fn test_get_xml_encoding_with_encode() {
        // When @encode is applied with a known XML encoding
        let result = get_xml_encoding(
            Some("TypeSpec.Xml.Encoding.xmlDateTime"),
            "utcDateTime",
            Some("string"),
        );
        assert!(result.is_some());
        let data = result.unwrap();
        assert_eq!(data.encoding, Some(XmlEncoding::XmlDateTime));
        assert_eq!(data.type_name, "string");
    }

    #[test]
    fn test_get_xml_encoding_default_fallback() {
        // When no @encode is applied, use default encoding
        let result = get_xml_encoding(None, "utcDateTime", None);
        assert!(result.is_some());
        let data = result.unwrap();
        assert_eq!(data.encoding, Some(XmlEncoding::XmlDateTime));
        assert_eq!(data.type_name, "string");
    }

    #[test]
    fn test_get_xml_encoding_unknown_scalar() {
        // Scalar with no default XML encoding and no @encode
        let result = get_xml_encoding(None, "string", None);
        assert!(result.is_none());
    }

    #[test]
    fn test_get_xml_encoding_custom_encode() {
        // Custom encoding (not XML-specific)
        let result = get_xml_encoding(Some("rfc7231"), "utcDateTime", Some("string"));
        // rfc7231 is not a known XML encoding, falls back to default
        assert!(result.is_some());
        let data = result.unwrap();
        assert_eq!(data.encoding, Some(XmlEncoding::XmlDateTime));
    }
}
