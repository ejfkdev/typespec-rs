//! @typespec/openapi3 - OpenAPI 3.x Emitter Decorators and Types
//!
//! Ported from TypeSpec packages/openapi3
//!
//! Provides decorators and types for OpenAPI 3.x emitter:
//! - `@useRef` - Override $ref for a model or property
//! - `@oneOf` - Emit union as oneOf instead of anyOf
//!
//! Also defines emitter options and OpenAPI version types.

use crate::diagnostics::{DiagnosticDefinition, DiagnosticMap};
use std::collections::HashMap;

// ============================================================================
// Namespace and state keys
// ============================================================================

/// Namespace for OpenAPI3 types
pub const OPENAPI3_NAMESPACE: &str = "TypeSpec.OpenAPI";

/// State key for @useRef decorator
pub const STATE_USE_REF: &str = "TypeSpec.OpenAPI.useRef";
/// State key for @oneOf decorator (openapi3-specific)
pub const STATE_OPENAPI_ONE_OF: &str = "TypeSpec.OpenAPI.oneOf";

// ============================================================================
// Diagnostic codes
// ============================================================================

/// Diagnostic: @oneOf can only be applied to a union or model property with union type
pub const DIAG_ONEOF_UNION: &str = "oneof-union";
/// Diagnostic: Inconsistent shared route request visibility
pub const DIAG_INCONSISTENT_SHARED_ROUTE: &str = "inconsistent-shared-route-request-visibility";
/// Diagnostic: Invalid server variable type
pub const DIAG_INVALID_SERVER_VARIABLE: &str = "invalid-server-variable";
/// Diagnostic: Invalid format for collection format
pub const DIAG_INVALID_FORMAT: &str = "invalid-format";
/// Diagnostic: Invalid style for parameter
pub const DIAG_INVALID_STYLE: &str = "invalid-style";
/// Diagnostic: Reserved expansion in path parameter
pub const DIAG_PATH_RESERVED_EXPANSION: &str = "path-reserved-expansion";
/// Diagnostic: Resource must be on namespace
pub const DIAG_RESOURCE_NAMESPACE: &str = "resource-namespace";
/// Diagnostic: Path contains query string
pub const DIAG_PATH_QUERY: &str = "path-query";
/// Diagnostic: Duplicate header across content types
pub const DIAG_DUPLICATE_HEADER: &str = "duplicate-header";
/// Diagnostic: Status code in default response
pub const DIAG_STATUS_CODE_IN_DEFAULT_RESPONSE: &str = "status-code-in-default-response";
/// Diagnostic: Cannot get schema for type
pub const DIAG_INVALID_SCHEMA: &str = "invalid-schema";
/// Diagnostic: Union containing only null types
pub const DIAG_UNION_NULL: &str = "union-null";
/// Diagnostic: Empty union not supported
pub const DIAG_EMPTY_UNION: &str = "empty-union";
/// Diagnostic: Empty enum not supported
pub const DIAG_EMPTY_ENUM: &str = "empty-enum";
/// Diagnostic: Enum options must be same literal type
pub const DIAG_ENUM_UNIQUE_TYPE: &str = "enum-unique-type";
/// Diagnostic: Inline cycle detected
pub const DIAG_INLINE_CYCLE: &str = "inline-cycle";
/// Diagnostic: Unsupported status code range
pub const DIAG_UNSUPPORTED_STATUS_CODE_RANGE: &str = "unsupported-status-code-range";
/// Diagnostic: Invalid model property type
pub const DIAG_INVALID_MODEL_PROPERTY: &str = "invalid-model-property";
/// Diagnostic: Unsupported auth type
pub const DIAG_UNSUPPORTED_AUTH: &str = "unsupported-auth";
/// Diagnostic: XML attribute invalid property type
pub const DIAG_XML_ATTRIBUTE_INVALID: &str = "xml-attribute-invalid-property-type";
/// Diagnostic: XML unwrapped invalid property type
pub const DIAG_XML_UNWRAPPED_INVALID: &str = "xml-unwrapped-invalid-property-type";
/// Diagnostic: Invalid component fixed field key
pub const DIAG_INVALID_COMPONENT_KEY: &str = "invalid-component-fixed-field-key";
/// Diagnostic: Streams not supported in OpenAPI 3.0
pub const DIAG_STREAMS_NOT_SUPPORTED: &str = "streams-not-supported";
/// Diagnostic: Default value not supported in OpenAPI 3.0
pub const DIAG_DEFAULT_NOT_SUPPORTED: &str = "default-not-supported";

// ============================================================================
// Types
// ============================================================================

string_enum! {
    /// OpenAPI specification version
    #[derive(PartialOrd, Ord)]
    pub enum OpenApiVersion {
        V3_0_0 => "3.0.0",
        V3_1_0 => "3.1.0",
        V3_2_0 => "3.2.0",
    }
}

impl OpenApiVersion {
    /// Get all supported versions
    pub fn all() -> &'static [OpenApiVersion] {
        &[
            OpenApiVersion::V3_0_0,
            OpenApiVersion::V3_1_0,
            OpenApiVersion::V3_2_0,
        ]
    }
}

string_enum! {
    /// Newline character setting for emitter output
    pub enum NewLine {
        Lf => "lf",
        Crlf => "crlf",
    }
}

impl NewLine {
    /// Parse from string value (case-insensitive).
    pub fn parse_str_ci(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "lf" => Some(NewLine::Lf),
            "crlf" => Some(NewLine::Crlf),
            _ => None,
        }
    }
}

string_enum! {
    /// Strategy for generating operation IDs when @operationId is not used.
    /// Ported from TS OperationIdStrategy type.
    pub enum OperationIdStrategy {
        /// Use parent container + operation name (default)
        ParentContainer => "parent-container",
        /// Use fully qualified name from service root
        Fqn => "fqn",
        /// Only use explicitly defined operation IDs
        ExplicitOnly => "explicit-only",
    }
}

string_enum! {
    /// Strategy for emitting parameter examples.
    /// Ported from TS ExperimentalParameterExamplesStrategy type.
    pub enum ExperimentalParameterExamplesStrategy {
        /// Use data directly
        Data => "data",
        /// Serialized form
        Serialized => "serialized",
    }
}

/// Strategy for handling @oneOf in OpenAPI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OneOfStrategy {
    /// Use oneOf for unions
    OneOf,
}

string_enum! {
    /// Strategy for handling safeint type
    pub enum SafeIntStrategy {
        /// Emit as type: integer, format: double-int
        DoubleInt => "double-int",
        /// Emit as type: integer, format: int64
        Int64 => "int64",
    }
}

string_enum! {
    /// When to include x-typespec-name extension
    pub enum IncludeXTypeSpecName {
        InlineOnly => "inline-only",
        Never => "never",
    }
}

/// OpenAPI 3 emitter options.
/// Ported from TS OpenAPI3EmitterOptions interface.
#[derive(Debug, Clone)]
pub struct OpenApi3EmitterOptions {
    /// File type(s) to emit (yaml or json)
    pub file_type: Vec<crate::libs::json_schema::FileType>,
    /// Output file name pattern
    pub output_file: Option<String>,
    /// OpenAPI specification versions to emit
    pub openapi_versions: Vec<OpenApiVersion>,
    /// Newline character setting
    pub new_line: NewLine,
    /// Omit unreachable types
    pub omit_unreachable_types: bool,
    /// When to include x-typespec-name extension
    pub include_x_typespec_name: IncludeXTypeSpecName,
    /// How to handle safeint type
    pub safeint_strategy: SafeIntStrategy,
    /// Seal object schemas with additionalProperties: false
    pub seal_object_schemas: bool,
    /// Strategy for generating operation IDs
    pub operation_id_strategy: OperationIdStrategy,
    /// Optional separator for operation ID segments
    pub operation_id_separator: Option<String>,
    /// Strategy for parameter examples (experimental)
    pub experimental_parameter_examples: Option<ExperimentalParameterExamplesStrategy>,
}

impl Default for OpenApi3EmitterOptions {
    fn default() -> Self {
        Self {
            file_type: vec![crate::libs::json_schema::FileType::Yaml],
            output_file: None,
            openapi_versions: vec![OpenApiVersion::V3_0_0],
            new_line: NewLine::Lf,
            omit_unreachable_types: false,
            include_x_typespec_name: IncludeXTypeSpecName::Never,
            safeint_strategy: SafeIntStrategy::Int64,
            seal_object_schemas: false,
            operation_id_strategy: OperationIdStrategy::ParentContainer,
            operation_id_separator: None,
            experimental_parameter_examples: None,
        }
    }
}

// ============================================================================
// Library creation
// ============================================================================

/// Create the @typespec/openapi3 library diagnostic map.
/// Ported from TS $lib diagnostics in lib.ts.
pub fn create_openapi3_library() -> DiagnosticMap {
    HashMap::from([
        (
            DIAG_ONEOF_UNION.to_string(),
            DiagnosticDefinition::error(
                "@oneOf decorator can only be used on a union or a model property which type is a union.",
            ),
        ),
        (
            DIAG_INCONSISTENT_SHARED_ROUTE.to_string(),
            DiagnosticDefinition::error(
                "All operations with `@sharedRoutes` must have the same `@requestVisibility`.",
            ),
        ),
        (
            DIAG_INVALID_SERVER_VARIABLE.to_string(),
            DiagnosticDefinition::error(
                "Server variable '{propName}' must be assignable to 'string'. It must either be a string, enum of string or union of strings.",
            ),
        ),
        (
            DIAG_INVALID_FORMAT.to_string(),
            DiagnosticDefinition::warning(
                "Collection format '{value}' is not supported in OpenAPI3 {paramType} parameters. Defaulting to type 'string'.",
            ),
        ),
        (
            DIAG_INVALID_STYLE.to_string(),
            DiagnosticDefinition::warning_with_messages(vec![
                (
                    "default",
                    "Style '{style}' is not supported in OpenAPI3 {paramType} parameters. Defaulting to style 'simple'.",
                ),
                (
                    "optionalPath",
                    "Style '{style}' is not supported in OpenAPI3 {paramType} parameters. The style {style} could be introduced by an optional parameter. Defaulting to style 'simple'.",
                ),
            ]),
        ),
        (
            DIAG_PATH_RESERVED_EXPANSION.to_string(),
            DiagnosticDefinition::warning(
                "Reserved expansion of path parameter with '+' operator #{allowReserved: true} is not supported in OpenAPI3.",
            ),
        ),
        (
            DIAG_RESOURCE_NAMESPACE.to_string(),
            DiagnosticDefinition::error("Resource goes on namespace"),
        ),
        (
            DIAG_PATH_QUERY.to_string(),
            DiagnosticDefinition::error("OpenAPI does not allow paths containing a query string."),
        ),
        (
            DIAG_DUPLICATE_HEADER.to_string(),
            DiagnosticDefinition::error(
                "The header {header} is defined across multiple content types",
            ),
        ),
        (
            DIAG_STATUS_CODE_IN_DEFAULT_RESPONSE.to_string(),
            DiagnosticDefinition::error(
                "A default response should not have an explicit status code",
            ),
        ),
        (
            DIAG_INVALID_SCHEMA.to_string(),
            DiagnosticDefinition::error("Couldn't get schema for type {type}"),
        ),
        (
            DIAG_UNION_NULL.to_string(),
            DiagnosticDefinition::error("Cannot have a union containing only null types."),
        ),
        (
            DIAG_EMPTY_UNION.to_string(),
            DiagnosticDefinition::error(
                "Empty unions are not supported for OpenAPI v3 - enums must have at least one value.",
            ),
        ),
        (
            DIAG_EMPTY_ENUM.to_string(),
            DiagnosticDefinition::error(
                "Empty enums are not supported for OpenAPI v3 - enums must have at least one value.",
            ),
        ),
        (
            DIAG_ENUM_UNIQUE_TYPE.to_string(),
            DiagnosticDefinition::error(
                "Enums are not supported unless all options are literals of the same type.",
            ),
        ),
        (
            DIAG_INLINE_CYCLE.to_string(),
            DiagnosticDefinition::error(
                "Cycle detected in '{type}'. Use @friendlyName decorator to assign an OpenAPI definition name and make it non-inline.",
            ),
        ),
        (
            DIAG_UNSUPPORTED_STATUS_CODE_RANGE.to_string(),
            DiagnosticDefinition::error(
                "Status code range '{start}' to '{end}' is not supported. OpenAPI 3.0 can only represent range 1XX, 2XX, 3XX, 4XX and 5XX. Example: `@minValue(400) @maxValue(499)` for 4XX.",
            ),
        ),
        (
            DIAG_INVALID_MODEL_PROPERTY.to_string(),
            DiagnosticDefinition::error("'{type}' cannot be specified as a model property."),
        ),
        (
            DIAG_UNSUPPORTED_AUTH.to_string(),
            DiagnosticDefinition::warning(
                "Authentication \"{authType}\" is not a known authentication by the openapi3 emitter, it will be ignored.",
            ),
        ),
        (
            DIAG_XML_ATTRIBUTE_INVALID.to_string(),
            DiagnosticDefinition::warning(
                "XML `@attribute` can only be primitive types in the OpenAPI 3 emitter, Property '{name}' type will be changed to type: string.",
            ),
        ),
        (
            DIAG_XML_UNWRAPPED_INVALID.to_string(),
            DiagnosticDefinition::warning(
                "XML `@unwrapped` can only used on array properties or primitive ones in the OpenAPI 3 emitter, Property '{name}' will be ignored.",
            ),
        ),
        (
            DIAG_INVALID_COMPONENT_KEY.to_string(),
            DiagnosticDefinition::warning(
                "Invalid key '{value}' used in a fixed field of the Component object. Only alphanumerics, dot (.), hyphen (-), and underscore (_) characters are allowed in keys.",
            ),
        ),
        (
            DIAG_STREAMS_NOT_SUPPORTED.to_string(),
            DiagnosticDefinition::warning(
                "Streams with itemSchema are only fully supported in OpenAPI 3.2.0 or above. The response will be emitted without itemSchema. Consider using OpenAPI 3.2.0 for full stream support.",
            ),
        ),
        (
            DIAG_DEFAULT_NOT_SUPPORTED.to_string(),
            DiagnosticDefinition::warning(
                "Default value is not supported in OpenAPI 3.0 {message}",
            ),
        ),
    ])
}

// ============================================================================
// Decorator implementations
// ============================================================================

string_decorator!(apply_use_ref, get_use_ref, STATE_USE_REF);

flag_decorator!(
    apply_openapi_one_of,
    is_openapi_one_of,
    STATE_OPENAPI_ONE_OF
);

// ============================================================================
// TSP Sources
// ============================================================================

/// The TypeSpec source for the OpenAPI3 library decorators
pub const OPENAPI3_DECORATORS_TSP: &str = r#"
namespace TypeSpec.OpenAPI;

extern dec useRef(target: Model | ModelProperty, refUrl: valueof string);
extern dec oneOf(target: Union | ModelProperty);
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_accessors::StateAccessors;

    #[test]
    fn test_namespace() {
        assert_eq!(OPENAPI3_NAMESPACE, "TypeSpec.OpenAPI");
    }

    #[test]
    fn test_create_library() {
        let diags = create_openapi3_library();
        assert!(diags.len() >= 24);
        let codes: Vec<&str> = diags.keys().map(|code| code.as_str()).collect();
        assert!(codes.contains(&DIAG_ONEOF_UNION));
        assert!(codes.contains(&DIAG_PATH_QUERY));
        assert!(codes.contains(&DIAG_EMPTY_UNION));
        assert!(codes.contains(&DIAG_INLINE_CYCLE));
        assert!(codes.contains(&DIAG_UNSUPPORTED_AUTH));
    }

    #[test]
    fn test_openapi_version() {
        assert_eq!(OpenApiVersion::V3_0_0.as_str(), "3.0.0");
        assert_eq!(OpenApiVersion::V3_1_0.as_str(), "3.1.0");
        assert_eq!(OpenApiVersion::V3_2_0.as_str(), "3.2.0");
        assert_eq!(
            OpenApiVersion::parse_str("3.0.0"),
            Some(OpenApiVersion::V3_0_0)
        );
        assert_eq!(
            OpenApiVersion::parse_str("3.1.0"),
            Some(OpenApiVersion::V3_1_0)
        );
        assert_eq!(OpenApiVersion::parse_str("2.0"), None);
    }

    #[test]
    fn test_newline() {
        assert_eq!(NewLine::Lf.as_str(), "lf");
        assert_eq!(NewLine::Crlf.as_str(), "crlf");
        assert_eq!(NewLine::parse_str("lf"), Some(NewLine::Lf));
        assert_eq!(NewLine::parse_str("crlf"), Some(NewLine::Crlf));
    }

    #[test]
    fn test_safeint_strategy() {
        assert_eq!(SafeIntStrategy::DoubleInt.as_str(), "double-int");
        assert_eq!(SafeIntStrategy::Int64.as_str(), "int64");
        assert_eq!(
            SafeIntStrategy::parse_str("int64"),
            Some(SafeIntStrategy::Int64)
        );
    }

    #[test]
    fn test_include_x_typespec_name() {
        assert_eq!(IncludeXTypeSpecName::InlineOnly.as_str(), "inline-only");
        assert_eq!(IncludeXTypeSpecName::Never.as_str(), "never");
    }

    #[test]
    fn test_use_ref() {
        let mut state = StateAccessors::new();
        assert_eq!(get_use_ref(&state, 1), None);
        apply_use_ref(&mut state, 1, "#/components/schemas/Pet");
        assert_eq!(
            get_use_ref(&state, 1),
            Some("#/components/schemas/Pet".to_string())
        );
    }

    #[test]
    fn test_openapi_one_of() {
        let mut state = StateAccessors::new();
        assert!(!is_openapi_one_of(&state, 1));
        apply_openapi_one_of(&mut state, 1);
        assert!(is_openapi_one_of(&state, 1));
    }

    #[test]
    fn test_emitter_options_default() {
        let opts = OpenApi3EmitterOptions::default();
        assert_eq!(opts.openapi_versions, vec![OpenApiVersion::V3_0_0]);
        assert_eq!(opts.new_line, NewLine::Lf);
        assert!(!opts.omit_unreachable_types);
        assert_eq!(opts.include_x_typespec_name, IncludeXTypeSpecName::Never);
        assert_eq!(opts.safeint_strategy, SafeIntStrategy::Int64);
        assert!(!opts.seal_object_schemas);
        assert_eq!(
            opts.operation_id_strategy,
            OperationIdStrategy::ParentContainer
        );
        assert!(opts.operation_id_separator.is_none());
        assert!(opts.experimental_parameter_examples.is_none());
    }

    #[test]
    fn test_operation_id_strategy() {
        assert_eq!(
            OperationIdStrategy::ParentContainer.as_str(),
            "parent-container"
        );
        assert_eq!(OperationIdStrategy::Fqn.as_str(), "fqn");
        assert_eq!(OperationIdStrategy::ExplicitOnly.as_str(), "explicit-only");
        assert_eq!(
            OperationIdStrategy::parse_str("parent-container"),
            Some(OperationIdStrategy::ParentContainer)
        );
        assert_eq!(
            OperationIdStrategy::parse_str("fqn"),
            Some(OperationIdStrategy::Fqn)
        );
        assert_eq!(OperationIdStrategy::parse_str("unknown"), None);
    }

    #[test]
    fn test_parameter_examples_strategy() {
        assert_eq!(ExperimentalParameterExamplesStrategy::Data.as_str(), "data");
        assert_eq!(
            ExperimentalParameterExamplesStrategy::Serialized.as_str(),
            "serialized"
        );
        assert_eq!(
            ExperimentalParameterExamplesStrategy::parse_str("data"),
            Some(ExperimentalParameterExamplesStrategy::Data)
        );
        assert_eq!(
            ExperimentalParameterExamplesStrategy::parse_str("unknown"),
            None
        );
    }

    #[test]
    fn test_decorators_tsp() {
        assert!(!OPENAPI3_DECORATORS_TSP.is_empty());
        assert!(OPENAPI3_DECORATORS_TSP.contains("dec useRef"));
        assert!(OPENAPI3_DECORATORS_TSP.contains("dec oneOf"));
    }
}
