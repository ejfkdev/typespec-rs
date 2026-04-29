//! @typespec/json-schema - JSON Schema Decorators and Types
//!
//! Ported from TypeSpec packages/json-schema
//!
//! Provides decorators for JSON Schema modeling:
//! - `@jsonSchema` - Mark a type as a JSON Schema declaration
//! - `@baseUri` - Set base URI for a namespace
//! - `@id` - Set JSON Schema $id
//! - `@multipleOf` - Set multipleOf constraint
//! - `@oneOf` - Mark as oneOf
//! - `@contains` - Set contains constraint
//! - `@minContains`, `@maxContains` - Contains count constraints
//! - `@uniqueItems` - Mark array as having unique items
//! - `@minProperties`, `@maxProperties` - Property count constraints
//! - `@contentEncoding`, `@contentMediaType`, `@contentSchema` - Content constraints
//! - `@prefixItems` - Tuple validation
//! - `@extension` - Custom extension key-value pairs

use crate::checker::types::TypeId;
use crate::diagnostics::{DiagnosticDefinition, DiagnosticMap};
use crate::state_accessors::StateAccessors;
use std::collections::HashMap;

// ============================================================================
// Namespace and state keys
// ============================================================================

/// Namespace for JSON Schema types
pub const JSON_SCHEMA_NAMESPACE: &str = "TypeSpec.JsonSchema";

/// State key for @jsonSchema decorator
pub const STATE_JSON_SCHEMA: &str = "TypeSpec.JsonSchema.JsonSchema";
/// State key for @baseUri decorator
pub const STATE_BASE_URI: &str = "TypeSpec.JsonSchema.baseURI";
/// State key for @multipleOf decorator
pub const STATE_MULTIPLE_OF: &str = "TypeSpec.JsonSchema.multipleOf";
/// State key for @id decorator
pub const STATE_ID: &str = "TypeSpec.JsonSchema.id";
/// State key for @oneOf decorator
pub const STATE_ONE_OF: &str = "TypeSpec.JsonSchema.oneOf";
/// State key for @contains decorator
pub const STATE_CONTAINS: &str = "TypeSpec.JsonSchema.contains";
/// State key for @minContains decorator
pub const STATE_MIN_CONTAINS: &str = "TypeSpec.JsonSchema.minContains";
/// State key for @maxContains decorator
pub const STATE_MAX_CONTAINS: &str = "TypeSpec.JsonSchema.maxContains";
/// State key for @uniqueItems decorator
pub const STATE_UNIQUE_ITEMS: &str = "TypeSpec.JsonSchema.uniqueItems";
/// State key for @minProperties decorator
pub const STATE_MIN_PROPERTIES: &str = "TypeSpec.JsonSchema.minProperties";
/// State key for @maxProperties decorator
pub const STATE_MAX_PROPERTIES: &str = "TypeSpec.JsonSchema.maxProperties";
/// State key for @contentEncoding decorator
pub const STATE_CONTENT_ENCODING: &str = "TypeSpec.JsonSchema.contentEncoding";
/// State key for @contentMediaType decorator
pub const STATE_CONTENT_MEDIA_TYPE: &str = "TypeSpec.JsonSchema.contentMediaType";
/// State key for @contentSchema decorator
pub const STATE_CONTENT_SCHEMA: &str = "TypeSpec.JsonSchema.contentSchema";
/// State key for @prefixItems decorator
pub const STATE_PREFIX_ITEMS: &str = "TypeSpec.JsonSchema.prefixItems";
/// State key for @extension decorator
pub const STATE_EXTENSION: &str = "TypeSpec.JsonSchema.extension";

// ============================================================================
// Diagnostic codes
// ============================================================================

/// Diagnostic: Invalid default value type
pub const DIAG_INVALID_DEFAULT: &str = "invalid-default";
/// Diagnostic: Duplicate $id
pub const DIAG_DUPLICATE_ID: &str = "duplicate-id";
/// Diagnostic: Unknown scalar type
pub const DIAG_UNKNOWN_SCALAR: &str = "unknown-scalar";

// ============================================================================
// Types
// ============================================================================

string_enum! {
    /// File type for JSON Schema output
    pub enum FileType {
        Json => "json",
        Yaml => "yaml",
    }
}

string_enum! {
    /// Strategy for handling int64 type in JSON Schema
    pub enum Int64Strategy {
        String => "string",
        Number => "number",
    }
}

string_enum! {
    /// Strategy for polymorphic models in JSON Schema
    pub enum PolymorphicModelsStrategy {
        Ignore => "ignore",
        OneOf => "oneOf",
        AnyOf => "anyOf",
    }
}

/// JSON Schema emitter options.
/// Ported from TS JSONSchemaEmitterOptions interface.
#[derive(Debug, Clone)]
pub struct JsonSchemaEmitterOptions {
    /// File type (json or yaml)
    pub file_type: Option<FileType>,
    /// How to handle int64 values
    pub int64_strategy: Option<Int64Strategy>,
    /// Bundle all schemas into a single document with this ID
    pub bundle_id: Option<String>,
    /// Emit all models without requiring @jsonSchema
    pub emit_all_models: bool,
    /// Emit all references as JSON Schema files
    pub emit_all_refs: bool,
    /// Seal object schemas with unevaluatedProperties
    pub seal_object_schemas: bool,
    /// Strategy for polymorphic models
    pub polymorphic_models_strategy: PolymorphicModelsStrategy,
}

impl Default for JsonSchemaEmitterOptions {
    fn default() -> Self {
        Self {
            file_type: None,
            int64_strategy: None,
            bundle_id: None,
            emit_all_models: false,
            emit_all_refs: false,
            seal_object_schemas: false,
            polymorphic_models_strategy: PolymorphicModelsStrategy::Ignore,
        }
    }
}

/// Extension record - a custom key-value pair in JSON Schema.
/// Ported from TS ExtensionRecord interface.
#[derive(Debug, Clone)]
pub struct ExtensionRecord {
    /// Extension key (must start with x-)
    pub key: String,
    /// Extension value (serialized as string in state)
    pub value: String,
}

// ============================================================================
// Library creation
// ============================================================================

/// Create the @typespec/json-schema library diagnostic map.
pub fn create_json_schema_library() -> DiagnosticMap {
    HashMap::from([
        (
            DIAG_INVALID_DEFAULT.to_string(),
            DiagnosticDefinition::error("Invalid type '{type}' for a default value"),
        ),
        (
            DIAG_DUPLICATE_ID.to_string(),
            DiagnosticDefinition::error("There are multiple types with the same id \"{id}\"."),
        ),
        (
            DIAG_UNKNOWN_SCALAR.to_string(),
            DiagnosticDefinition::warning(
                "Scalar '{name}' is not a known scalar type and doesn't extend a known scalar type.",
            ),
        ),
    ])
}

// ============================================================================
// Decorator implementations
// ============================================================================

/// Apply @jsonSchema decorator
pub fn apply_json_schema(state: &mut StateAccessors, target: TypeId, id_or_uri: Option<&str>) {
    state.add_to_state(STATE_JSON_SCHEMA, target);
    if let Some(id) = id_or_uri {
        state.set_state(STATE_ID, target, id.to_string());
    }
}

/// Check if a type is annotated with @jsonSchema
pub fn is_json_schema(state: &StateAccessors, target: TypeId) -> bool {
    state.has_state(STATE_JSON_SCHEMA, target)
}

string_decorator!(apply_base_uri, get_base_uri, STATE_BASE_URI);
string_decorator!(apply_id, get_id, STATE_ID);
numeric_decorator!(apply_multiple_of, get_multiple_of, STATE_MULTIPLE_OF, f64);
flag_decorator!(apply_one_of, is_one_of, STATE_ONE_OF);
string_decorator!(apply_contains, get_contains, STATE_CONTAINS);
numeric_decorator!(
    apply_min_contains,
    get_min_contains,
    STATE_MIN_CONTAINS,
    i64
);
numeric_decorator!(
    apply_max_contains,
    get_max_contains,
    STATE_MAX_CONTAINS,
    i64
);

flag_decorator!(apply_unique_items, is_unique_items, STATE_UNIQUE_ITEMS);

numeric_decorator!(
    apply_min_properties,
    get_min_properties,
    STATE_MIN_PROPERTIES,
    i64
);
numeric_decorator!(
    apply_max_properties,
    get_max_properties,
    STATE_MAX_PROPERTIES,
    i64
);
string_decorator!(
    apply_content_encoding,
    get_content_encoding,
    STATE_CONTENT_ENCODING
);
string_decorator!(
    apply_content_media_type,
    get_content_media_type,
    STATE_CONTENT_MEDIA_TYPE
);
string_decorator!(
    apply_content_schema,
    get_content_schema,
    STATE_CONTENT_SCHEMA
);
typeid_decorator!(apply_prefix_items, get_prefix_items, STATE_PREFIX_ITEMS);

/// Apply @extension decorator.
/// Extensions are stored as "key::value||key2::value2" format.
pub fn apply_extension(state: &mut StateAccessors, target: TypeId, key: &str, value: &str) {
    let existing = state.get_state(STATE_EXTENSION, target).unwrap_or("");
    let entry = format!("{}::{}", key, value);
    let new_value = if existing.is_empty() {
        entry
    } else {
        format!("{}||{}", existing, entry)
    };
    state.set_state(STATE_EXTENSION, target, new_value);
}

/// Get all extensions for a type.
/// Returns a Vec of (key, value) pairs.
pub fn get_extensions(state: &StateAccessors, target: TypeId) -> Vec<ExtensionRecord> {
    let raw = match state.get_state(STATE_EXTENSION, target) {
        Some(s) => s,
        None => return Vec::new(),
    };
    raw.split("||")
        .filter(|s| !s.is_empty())
        .filter_map(|entry| {
            if let Some((key, value)) = entry.split_once("::") {
                Some(ExtensionRecord {
                    key: key.to_string(),
                    value: value.to_string(),
                })
            } else {
                None
            }
        })
        .collect()
}

// ============================================================================
// TSP Sources
// ============================================================================

/// The TypeSpec source for the JSON Schema library decorators
pub const JSON_SCHEMA_DECORATORS_TSP: &str = r#"
namespace TypeSpec.JsonSchema;

using TypeSpec.Reflection;

extern dec jsonSchema(target: Model | Union | Enum | Scalar | Namespace, baseUriOrId?: valueof string);
extern dec baseUri(target: Namespace, uri: valueof string);
extern dec id(target: Model | Union | Enum | Scalar, id: valueof string);
extern dec multipleOf(target: numeric | ModelProperty, value: valueof numeric);
extern dec oneOf(target: Union | ModelProperty);
extern dec contains(target: unknown[] | ModelProperty, value: unknown);
extern dec minContains(target: unknown[] | ModelProperty, value: valueof integer);
extern dec maxContains(target: unknown[] | ModelProperty, value: valueof integer);
extern dec uniqueItems(target: unknown[] | ModelProperty);
extern dec minProperties(target: Model | ModelProperty, value: valueof integer);
extern dec maxProperties(target: Model | ModelProperty, value: valueof integer);
extern dec contentEncoding(target: string | ModelProperty, value: valueof string);
extern dec contentMediaType(target: string | ModelProperty, value: valueof string);
extern dec contentSchema(target: string | ModelProperty, value: unknown);
extern dec prefixItems(target: unknown[] | ModelProperty, value: Tuple);
extern dec extension(target: unknown, key: valueof string, value: unknown);

model Json<T> {
  value: T;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace() {
        assert_eq!(JSON_SCHEMA_NAMESPACE, "TypeSpec.JsonSchema");
    }

    #[test]
    fn test_create_library() {
        let diags = create_json_schema_library();
        assert_eq!(diags.len(), 3);
    }

    #[test]
    fn test_file_type() {
        assert_eq!(FileType::Json.as_str(), "json");
        assert_eq!(FileType::Yaml.as_str(), "yaml");
        assert_eq!(FileType::parse_str("yaml"), Some(FileType::Yaml));
        assert_eq!(FileType::parse_str("json"), Some(FileType::Json));
    }

    #[test]
    fn test_int64_strategy() {
        assert_eq!(Int64Strategy::String.as_str(), "string");
        assert_eq!(Int64Strategy::Number.as_str(), "number");
        assert_eq!(
            Int64Strategy::parse_str("string"),
            Some(Int64Strategy::String)
        );
    }

    #[test]
    fn test_polymorphic_strategy() {
        assert_eq!(PolymorphicModelsStrategy::Ignore.as_str(), "ignore");
        assert_eq!(PolymorphicModelsStrategy::OneOf.as_str(), "oneOf");
        assert_eq!(PolymorphicModelsStrategy::AnyOf.as_str(), "anyOf");
    }

    #[test]
    fn test_json_schema_decorator() {
        let mut state = StateAccessors::new();
        assert!(!is_json_schema(&state, 1));
        apply_json_schema(&mut state, 1, None);
        assert!(is_json_schema(&state, 1));
    }

    #[test]
    fn test_json_schema_with_id() {
        let mut state = StateAccessors::new();
        apply_json_schema(&mut state, 1, Some("https://example.com/schema"));
        assert!(is_json_schema(&state, 1));
        assert_eq!(
            get_id(&state, 1),
            Some("https://example.com/schema".to_string())
        );
    }

    #[test]
    fn test_base_uri() {
        let mut state = StateAccessors::new();
        assert_eq!(get_base_uri(&state, 1), None);
        apply_base_uri(&mut state, 1, "https://example.com/schemas/");
        assert_eq!(
            get_base_uri(&state, 1),
            Some("https://example.com/schemas/".to_string())
        );
    }

    #[test]
    fn test_id() {
        let mut state = StateAccessors::new();
        apply_id(&mut state, 1, "Pet");
        assert_eq!(get_id(&state, 1), Some("Pet".to_string()));
    }

    #[test]
    fn test_multiple_of() {
        let mut state = StateAccessors::new();
        apply_multiple_of(&mut state, 1, 0.01);
        assert_eq!(get_multiple_of(&state, 1), Some(0.01));
    }

    #[test]
    fn test_one_of() {
        let mut state = StateAccessors::new();
        assert!(!is_one_of(&state, 1));
        apply_one_of(&mut state, 1);
        assert!(is_one_of(&state, 1));
    }

    #[test]
    fn test_contains() {
        let mut state = StateAccessors::new();
        apply_contains(&mut state, 1, "string");
        assert_eq!(get_contains(&state, 1), Some("string".to_string()));
    }

    #[test]
    fn test_min_max_contains() {
        let mut state = StateAccessors::new();
        apply_min_contains(&mut state, 1, 1);
        apply_max_contains(&mut state, 1, 5);
        assert_eq!(get_min_contains(&state, 1), Some(1));
        assert_eq!(get_max_contains(&state, 1), Some(5));
    }

    #[test]
    fn test_unique_items() {
        let mut state = StateAccessors::new();
        assert!(!is_unique_items(&state, 1));
        apply_unique_items(&mut state, 1);
        assert!(is_unique_items(&state, 1));
    }

    #[test]
    fn test_min_max_properties() {
        let mut state = StateAccessors::new();
        apply_min_properties(&mut state, 1, 1);
        apply_max_properties(&mut state, 1, 10);
        assert_eq!(get_min_properties(&state, 1), Some(1));
        assert_eq!(get_max_properties(&state, 1), Some(10));
    }

    #[test]
    fn test_content_encoding() {
        let mut state = StateAccessors::new();
        apply_content_encoding(&mut state, 1, "base64");
        assert_eq!(get_content_encoding(&state, 1), Some("base64".to_string()));
    }

    #[test]
    fn test_content_media_type() {
        let mut state = StateAccessors::new();
        apply_content_media_type(&mut state, 1, "text/html");
        assert_eq!(
            get_content_media_type(&state, 1),
            Some("text/html".to_string())
        );
    }

    #[test]
    fn test_content_schema() {
        let mut state = StateAccessors::new();
        apply_content_schema(&mut state, 1, "string");
        assert_eq!(get_content_schema(&state, 1), Some("string".to_string()));
    }

    #[test]
    fn test_prefix_items() {
        let mut state = StateAccessors::new();
        apply_prefix_items(&mut state, 1, 42);
        assert_eq!(get_prefix_items(&state, 1), Some(42));
    }

    #[test]
    fn test_extension_single() {
        let mut state = StateAccessors::new();
        apply_extension(&mut state, 1, "x-custom", "value1");
        let exts = get_extensions(&state, 1);
        assert_eq!(exts.len(), 1);
        assert_eq!(exts[0].key, "x-custom");
        assert_eq!(exts[0].value, "value1");
    }

    #[test]
    fn test_extension_multiple() {
        let mut state = StateAccessors::new();
        apply_extension(&mut state, 1, "x-custom", "value1");
        apply_extension(&mut state, 1, "x-another", "value2");
        let exts = get_extensions(&state, 1);
        assert_eq!(exts.len(), 2);
    }

    #[test]
    fn test_emitter_options_default() {
        let opts = JsonSchemaEmitterOptions::default();
        assert!(opts.file_type.is_none());
        assert!(opts.int64_strategy.is_none());
        assert!(opts.bundle_id.is_none());
        assert!(!opts.emit_all_models);
        assert!(!opts.emit_all_refs);
        assert!(!opts.seal_object_schemas);
        assert_eq!(
            opts.polymorphic_models_strategy,
            PolymorphicModelsStrategy::Ignore
        );
    }

    #[test]
    fn test_decorators_tsp() {
        assert!(!JSON_SCHEMA_DECORATORS_TSP.is_empty());
        assert!(JSON_SCHEMA_DECORATORS_TSP.contains("dec jsonSchema"));
        assert!(JSON_SCHEMA_DECORATORS_TSP.contains("dec extension"));
        assert!(JSON_SCHEMA_DECORATORS_TSP.contains("dec oneOf"));
    }
}
