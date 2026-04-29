//! TypeSpec Standard Library Decorators
//!
//! Ported from TypeSpec compiler/lib/std/decorators.tsp
//!
//! Provides the core TypeSpec decorator definitions:
//!
//! ## Documentation
//! - `@doc`, `@summary`, `@returnsDoc`, `@errorsDoc`
//!
//! ## Service
//! - `@service` (with ServiceOptions)
//! - `@error` - Mark a model as an error type
//!
//! ## Validation / Constraints
//! - `@format`, `@pattern` - String validation
//! - `@minLength`, `@maxLength` - String length constraints
//! - `@minValue`, `@maxValue`, `@minValueExclusive`, `@maxValueExclusive` - Numeric range
//! - `@minItems`, `@maxItems` - Array size constraints
//! - `@secret` - Mark as sensitive value
//!
//! ## Naming
//! - `@tag`, `@friendlyName`, `@encodedName`, `@key`
//!
//! ## Type Manipulation
//! - `@overload`, `@discriminator`, `@discriminated`
//! - `@withOptionalProperties`, `@withoutDefaultValues`
//! - `@withoutOmittedProperties`, `@withPickedProperties`
//!
//! ## Encoding
//! - `@encode` - Specify type encoding
//! - `@mediaTypeHint` - Media type hint
//!
//! ## Paging
//! - `@list`, `@offset`, `@pageIndex`, `@pageSize`, `@pageItems`
//! - `@continuationToken`, `@nextLink`, `@prevLink`, `@firstLink`, `@lastLink`
//!   (see `paging` module)
//!
//! ## Examples
//! - `@example`, `@opExample`
//!
//! ## Debugging
//! - `@inspectType`, `@inspectTypeName`

use crate::checker::types::TypeId;
use crate::diagnostics::{DiagnosticDefinition, DiagnosticMap};
use crate::state_accessors::StateAccessors;
use std::collections::HashMap;

pub mod encoded_names;
pub use encoded_names::*;

pub mod paging;
pub use paging::*;

pub mod types;
pub use types::*;

pub mod tsp_sources;
#[allow(unused_imports)]
pub use tsp_sources::*;

pub mod utils;
pub use utils::*;

pub mod visibility;
pub use visibility::*;

/// Namespace for TypeSpec standard library
pub const TYPESPEC_NAMESPACE: &str = "TypeSpec";

// ============================================================================
// State keys
// ============================================================================

/// State key for @doc decorator
pub const STATE_DOC: &str = "TypeSpec.doc";
/// State key for @summary decorator
pub const STATE_SUMMARY: &str = "TypeSpec.summary";
/// State key for @returnsDoc decorator
pub const STATE_RETURNS_DOC: &str = "TypeSpec.returnsDoc";
/// State key for @errorsDoc decorator
pub const STATE_ERRORS_DOC: &str = "TypeSpec.errorsDoc";
/// State key for @error decorator
pub const STATE_ERROR: &str = "TypeSpec.error";
/// State key for @service decorator
pub const STATE_SERVICE: &str = "TypeSpec.service";
/// State key for @format decorator
pub const STATE_FORMAT: &str = "TypeSpec.format";
/// State key for @pattern decorator
pub const STATE_PATTERN: &str = "TypeSpec.pattern";
/// State key for @minLength decorator
pub const STATE_MIN_LENGTH: &str = "TypeSpec.minLength";
/// State key for @maxLength decorator
pub const STATE_MAX_LENGTH: &str = "TypeSpec.maxLength";
/// State key for @minValue decorator
pub const STATE_MIN_VALUE: &str = "TypeSpec.minValue";
/// State key for @maxValue decorator
pub const STATE_MAX_VALUE: &str = "TypeSpec.maxValue";
/// State key for @minValueExclusive decorator
pub const STATE_MIN_VALUE_EXCLUSIVE: &str = "TypeSpec.minValueExclusive";
/// State key for @maxValueExclusive decorator
pub const STATE_MAX_VALUE_EXCLUSIVE: &str = "TypeSpec.maxValueExclusive";
/// State key for @minItems decorator
pub const STATE_MIN_ITEMS: &str = "TypeSpec.minItems";
/// State key for @maxItems decorator
pub const STATE_MAX_ITEMS: &str = "TypeSpec.maxItems";
/// State key for @secret decorator
pub const STATE_SECRET: &str = "TypeSpec.secret";
/// State key for @tag decorator
pub const STATE_TAG: &str = "TypeSpec.tag";
/// State key for @friendlyName decorator
pub const STATE_FRIENDLY_NAME: &str = "TypeSpec.friendlyName";
/// State key for @key decorator
pub const STATE_KEY: &str = "TypeSpec.key";
/// State key for @overload decorator
pub const STATE_OVERLOAD: &str = "TypeSpec.overload";
/// State key for @discriminator decorator
pub const STATE_DISCRIMINATOR: &str = "TypeSpec.discriminator";
/// State key for @discriminated decorator
pub const STATE_DISCRIMINATED: &str = "TypeSpec.discriminated";
/// State key for @encode decorator
pub const STATE_ENCODE: &str = "TypeSpec.encode";
/// State key for @mediaTypeHint decorator
pub const STATE_MEDIA_TYPE_HINT: &str = "TypeSpec.mediaTypeHint";
/// State key for @example decorator
pub const STATE_EXAMPLE: &str = "TypeSpec.example";
/// State key for @opExample decorator
pub const STATE_OP_EXAMPLE: &str = "TypeSpec.opExample";
/// State key for @withOptionalProperties decorator
pub const STATE_WITH_OPTIONAL_PROPERTIES: &str = "TypeSpec.withOptionalProperties";
/// State key for @withoutDefaultValues decorator
pub const STATE_WITHOUT_DEFAULT_VALUES: &str = "TypeSpec.withoutDefaultValues";
/// State key for @withoutOmittedProperties decorator
pub const STATE_WITHOUT_OMITTED_PROPERTIES: &str = "TypeSpec.withoutOmittedProperties";
/// State key for @withPickedProperties decorator
pub const STATE_WITH_PICKED_PROPERTIES: &str = "TypeSpec.withPickedProperties";
/// State key for @inspectType decorator
pub const STATE_INSPECT_TYPE: &str = "TypeSpec.inspectType";
/// State key for @inspectTypeName decorator
pub const STATE_INSPECT_TYPE_NAME: &str = "TypeSpec.inspectTypeName";

// Types are in types.rs, re-exported via `pub use types::*`

// ============================================================================
// Library creation
// ============================================================================

/// Create the TypeSpec standard library diagnostic map.
pub fn create_std_library() -> DiagnosticMap {
    // The TS compiler defines these diagnostics in messages.ts
    // These are the core decorator validation diagnostics
    HashMap::from([
        (
            "decorator-wrong-target".to_string(),
            DiagnosticDefinition::error("Decorator cannot be used on this target"),
        ),
        (
            "decorator-duplicate".to_string(),
            DiagnosticDefinition::error("Decorator can only be applied once"),
        ),
        (
            "invalid-encoding".to_string(),
            DiagnosticDefinition::error("Invalid encoding value"),
        ),
        (
            "no-key-type".to_string(),
            DiagnosticDefinition::error("Type used as a resource must have a key property"),
        ),
        (
            "invalid-pattern-regex".to_string(),
            DiagnosticDefinition::error("Invalid pattern regex"),
        ),
    ])
}

// ============================================================================
// Decorator implementations (state-based)
// ============================================================================

/// Apply @doc decorator.
/// Delegates to `intrinsic_type_state::set_doc_data()`.
pub fn apply_doc(state: &mut StateAccessors, target: TypeId, doc: &str) {
    crate::intrinsic_type_state::set_doc_data(
        state,
        target,
        crate::intrinsic_type_state::DocTarget::Self_,
        crate::intrinsic_type_state::DocData {
            value: doc.to_string(),
            source: crate::intrinsic_type_state::DocSource::Decorator,
        },
    );
}

/// Get @doc value.
/// Delegates to `intrinsic_type_state::get_doc()`.
pub fn get_doc(state: &StateAccessors, target: TypeId) -> Option<String> {
    crate::intrinsic_type_state::get_doc(state, target)
}

string_decorator!(apply_summary, get_summary, STATE_SUMMARY);

flag_decorator!(apply_error, is_error, STATE_ERROR);

// ============================================================================
// Type checker functions
// Ported from TS lib/decorators.ts: isStringType, isNumericType, isDateTimeType
// ============================================================================

/// State key for string type check
pub const STATE_IS_STRING_TYPE: &str = "TypeSpec.isStringType";
/// State key for numeric type check
pub const STATE_IS_NUMERIC_TYPE: &str = "TypeSpec.isNumericType";
/// State key for datetime type check
pub const STATE_IS_DATETIME_TYPE: &str = "TypeSpec.isDateTimeType";

flag_decorator!(set_string_type, is_string_type, STATE_IS_STRING_TYPE);
flag_decorator!(set_numeric_type, is_numeric_type, STATE_IS_NUMERIC_TYPE);
flag_decorator!(
    set_date_time_type,
    is_date_time_type,
    STATE_IS_DATETIME_TYPE
);

// ============================================================================
// Deprecated support
// Ported from TS core/deprecation.ts: getDeprecationDetails()
// ============================================================================

/// State key for @deprecated decorator
pub const STATE_DEPRECATED: &str = "TypeSpec.deprecated";

string_decorator!(apply_deprecated, get_deprecated, STATE_DEPRECATED);

// ============================================================================
// Doc data support
// Ported from TS core/intrinsic-type-state.ts: DocData
// ============================================================================

/// State key for doc data source tracking
pub const STATE_DOC_SOURCE: &str = "TypeSpec.docSource";

/// Apply @doc decorator with source tracking.
/// Delegates to `intrinsic_type_state::set_doc_data()`.
/// Ported from TS docFromCommentDecorator / $doc.
pub fn apply_doc_with_source(
    state: &mut StateAccessors,
    target: TypeId,
    doc: &str,
    source: crate::intrinsic_type_state::DocSource,
) {
    crate::intrinsic_type_state::set_doc_data(
        state,
        target,
        crate::intrinsic_type_state::DocTarget::Self_,
        crate::intrinsic_type_state::DocData {
            value: doc.to_string(),
            source,
        },
    );
}

/// Get full doc data including source.
/// Delegates to `intrinsic_type_state::get_doc_data()`.
/// Ported from TS getDocData().
pub fn get_doc_data(
    state: &StateAccessors,
    target: TypeId,
) -> Option<crate::intrinsic_type_state::DocData> {
    crate::intrinsic_type_state::get_doc_data(
        state,
        target,
        crate::intrinsic_type_state::DocTarget::Self_,
    )
}

/// Apply @service decorator.
/// Ported from TS $service() which calls addService().
/// Stores Service data as `title` format (empty string if no title).
pub fn apply_service(state: &mut StateAccessors, target: TypeId, title: Option<&str>) {
    state.set_state(STATE_SERVICE, target, title.unwrap_or("").to_string());
}

/// Check if a namespace is a service.
/// Ported from TS isService().
pub fn is_service(state: &StateAccessors, target: TypeId) -> bool {
    state.get_state(STATE_SERVICE, target).is_some()
}

/// Get service title.
pub fn get_service_title(state: &StateAccessors, target: TypeId) -> Option<String> {
    state
        .get_state(STATE_SERVICE, target)
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
}

/// Add a service for the given namespace.
/// Ported from TS addService(). Merges with existing service details.
pub fn add_service(state: &mut StateAccessors, target: TypeId, details: ServiceDetails) {
    let existing_title = get_service_title(state, target);
    let title = details.title.or(existing_title).unwrap_or_default();
    state.set_state(STATE_SERVICE, target, title);
}

/// Get the Service information for the given namespace.
/// Ported from TS getService().
pub fn get_service(state: &StateAccessors, target: TypeId) -> Option<Service> {
    if !is_service(state, target) {
        return None;
    }
    Some(Service {
        namespace_type: target,
        details: ServiceDetails {
            title: get_service_title(state, target),
        },
    })
}

/// List all services defined in the program.
/// Ported from TS listServices().
pub fn list_services(state: &StateAccessors) -> Vec<Service> {
    state
        .get_state_map(STATE_SERVICE)
        .map(|map| {
            map.keys()
                .filter_map(|&target| get_service(state, target))
                .collect()
        })
        .unwrap_or_default()
}

string_decorator!(apply_format, get_format, STATE_FORMAT);

/// Apply @pattern decorator.
/// Stores PatternData as `pattern[::validationMessage]` format.
/// Ported from TS $pattern() which stores PatternData { pattern, validationMessage }.
/// Apply @pattern decorator.
///
/// Note: Uses `::` as internal separator between pattern and validation message.
/// This matches the TS compiler behavior. Patterns containing `::` may be parsed
/// incorrectly by `get_pattern_data()`.
pub fn apply_pattern(
    state: &mut StateAccessors,
    target: TypeId,
    pattern: &str,
    validation_message: Option<&str>,
) {
    let value = match validation_message {
        Some(msg) => format!("{}::{}", pattern, msg),
        None => pattern.to_string(),
    };
    state.set_state(STATE_PATTERN, target, value);
}

/// Get @pattern value (just the pattern string).
pub fn get_pattern(state: &StateAccessors, target: TypeId) -> Option<String> {
    state.get_state(STATE_PATTERN, target).map(|s| {
        // Pattern data may contain "pattern::validationMessage" format
        if let Some(pos) = s.find("::") {
            s[..pos].to_string()
        } else {
            s.to_string()
        }
    })
}

/// Get full @pattern data including validation message.
/// Ported from TS getPatternData().
pub fn get_pattern_data(state: &StateAccessors, target: TypeId) -> Option<PatternData> {
    state.get_state(STATE_PATTERN, target).map(|s| {
        if let Some(pos) = s.find("::") {
            PatternData {
                pattern: s[..pos].to_string(),
                validation_message: Some(s[pos + 2..].to_string()),
            }
        } else {
            PatternData {
                pattern: s.to_string(),
                validation_message: None,
            }
        }
    })
}

/// Apply @minLength decorator.
/// Delegates to `intrinsic_type_state::set_min_length()`.
pub fn apply_min_length(state: &mut StateAccessors, target: TypeId, value: i64) {
    crate::intrinsic_type_state::set_min_length(
        state,
        target,
        &crate::numeric::Numeric::new(&value.to_string())
            .expect("valid numeric value from TypeSpec"),
    );
}

/// Get @minLength value.
/// Delegates to `intrinsic_type_state::get_min_length()`.
pub fn get_min_length(state: &StateAccessors, target: TypeId) -> Option<i64> {
    crate::intrinsic_type_state::get_min_length(state, target).map(|v| v.round() as i64)
}

/// Apply @maxLength decorator.
/// Delegates to `intrinsic_type_state::set_max_length()`.
pub fn apply_max_length(state: &mut StateAccessors, target: TypeId, value: i64) {
    crate::intrinsic_type_state::set_max_length(
        state,
        target,
        &crate::numeric::Numeric::new(&value.to_string())
            .expect("valid numeric value from TypeSpec"),
    );
}

/// Get @maxLength value.
/// Delegates to `intrinsic_type_state::get_max_length()`.
pub fn get_max_length(state: &StateAccessors, target: TypeId) -> Option<i64> {
    crate::intrinsic_type_state::get_max_length(state, target).map(|v| v.round() as i64)
}

/// Apply @minValue decorator.
/// Delegates to `intrinsic_type_state::set_min_value()`.
pub fn apply_min_value(state: &mut StateAccessors, target: TypeId, value: f64) {
    use crate::intrinsic_type_state::{NumericOrScalar, set_min_value};
    let numeric = crate::numeric::Numeric::new(&value.to_string())
        .expect("valid numeric value from TypeSpec");
    set_min_value(state, target, NumericOrScalar::Numeric(numeric));
}

/// Get @minValue value.
/// Delegates to `intrinsic_type_state::get_min_value()`.
pub fn get_min_value(state: &StateAccessors, target: TypeId) -> Option<f64> {
    crate::intrinsic_type_state::get_min_value(state, target)
}

/// Apply @maxValue decorator.
/// Delegates to `intrinsic_type_state::set_max_value()`.
pub fn apply_max_value(state: &mut StateAccessors, target: TypeId, value: f64) {
    use crate::intrinsic_type_state::{NumericOrScalar, set_max_value};
    let numeric = crate::numeric::Numeric::new(&value.to_string())
        .expect("valid numeric value from TypeSpec");
    set_max_value(state, target, NumericOrScalar::Numeric(numeric));
}

/// Get @maxValue value.
/// Delegates to `intrinsic_type_state::get_max_value()`.
pub fn get_max_value(state: &StateAccessors, target: TypeId) -> Option<f64> {
    crate::intrinsic_type_state::get_max_value(state, target)
}

flag_decorator!(apply_secret, is_secret, STATE_SECRET);

/// Apply @tag decorator.
///
/// Note: Uses `;` as internal separator for multiple tags.
/// This matches the TS compiler behavior. Tag names containing `;` may be
/// split incorrectly by `get_tags()`.
pub fn apply_tag(state: &mut StateAccessors, target: TypeId, tag: &str) {
    // Tags can be multiple, append with separator
    let existing = state.get_state(STATE_TAG, target).unwrap_or("").to_string();
    let new_value = if existing.is_empty() {
        tag.to_string()
    } else {
        format!("{};{}", existing, tag)
    };
    state.set_state(STATE_TAG, target, new_value);
}

/// Get all tags for a target
pub fn get_tags(state: &StateAccessors, target: TypeId) -> Vec<String> {
    state
        .get_state(STATE_TAG, target)
        .map(|s| {
            s.split(';')
                .map(String::from)
                .filter(|t| !t.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

optional_name_decorator!(apply_key, is_key, get_key_name, STATE_KEY);

string_decorator!(apply_friendly_name, get_friendly_name, STATE_FRIENDLY_NAME);

/// Apply @discriminator decorator.
/// Delegates to `intrinsic_type_state::set_discriminator()`.
pub fn apply_discriminator(state: &mut StateAccessors, target: TypeId, property_name: &str) {
    crate::intrinsic_type_state::set_discriminator(
        state,
        target,
        crate::intrinsic_type_state::Discriminator {
            property_name: property_name.to_string(),
        },
    );
}

/// Get @discriminator property name.
/// Delegates to `intrinsic_type_state::get_discriminator()`.
pub fn get_discriminator(state: &StateAccessors, target: TypeId) -> Option<String> {
    crate::intrinsic_type_state::get_discriminator(state, target).map(|d| d.property_name)
}

/// Apply @encode decorator.
/// Stores EncodeData as `encoding[::encodeAsTypeId]` format.
/// Ported from TS $encode() which stores EncodeData { encoding?, type }.
pub fn apply_encode(
    state: &mut StateAccessors,
    target: TypeId,
    encoding: Option<&str>,
    encode_as: Option<TypeId>,
) {
    let enc_str = encoding.unwrap_or("");
    let value = match encode_as {
        Some(type_id) => format!("{}::{}", enc_str, type_id),
        None => enc_str.to_string(),
    };
    state.set_state(STATE_ENCODE, target, value);
}

/// Get @encode value (just the encoding string).
pub fn get_encode(state: &StateAccessors, target: TypeId) -> Option<String> {
    state.get_state(STATE_ENCODE, target).map(|s| {
        if let Some(pos) = s.find("::") {
            s[..pos].to_string()
        } else {
            s.to_string()
        }
    })
}

/// Get full @encode data including encodeAs type.
/// Ported from TS getEncode() which returns EncodeData.
pub fn get_encode_data(state: &StateAccessors, target: TypeId) -> Option<EncodeData> {
    state.get_state(STATE_ENCODE, target).map(|s| {
        if let Some(pos) = s.find("::") {
            let encoding_part = &s[..pos];
            let type_part = &s[pos + 2..];
            EncodeData {
                encoding: if encoding_part.is_empty() {
                    None
                } else {
                    Some(encoding_part.to_string())
                },
                encode_as_type: type_part.parse::<TypeId>().ok(),
            }
        } else {
            EncodeData {
                encoding: if s.is_empty() {
                    None
                } else {
                    Some(s.to_string())
                },
                encode_as_type: None,
            }
        }
    })
}

// Encoded name functions are in encoded_names.rs, re-exported via `pub use encoded_names::*`

string_decorator!(
    apply_media_type_hint,
    get_media_type_hint,
    STATE_MEDIA_TYPE_HINT
);

// ============================================================================
// Documentation decorators
// ============================================================================

string_decorator!(apply_returns_doc, get_returns_doc, STATE_RETURNS_DOC);
string_decorator!(apply_errors_doc, get_errors_doc, STATE_ERRORS_DOC);

// ============================================================================
// Exclusive value range decorators
// ============================================================================

/// Apply @minValueExclusive decorator.
/// Delegates to `intrinsic_type_state::set_min_value_exclusive()`.
pub fn apply_min_value_exclusive(state: &mut StateAccessors, target: TypeId, value: f64) {
    use crate::intrinsic_type_state::{NumericOrScalar, set_min_value_exclusive};
    let numeric = crate::numeric::Numeric::new(&value.to_string())
        .expect("valid numeric value from TypeSpec");
    set_min_value_exclusive(state, target, NumericOrScalar::Numeric(numeric));
}

/// Get @minValueExclusive value.
/// Delegates to `intrinsic_type_state::get_min_value_exclusive()`.
pub fn get_min_value_exclusive(state: &StateAccessors, target: TypeId) -> Option<f64> {
    crate::intrinsic_type_state::get_min_value_exclusive(state, target)
}

/// Apply @maxValueExclusive decorator.
/// Delegates to `intrinsic_type_state::set_max_value_exclusive()`.
pub fn apply_max_value_exclusive(state: &mut StateAccessors, target: TypeId, value: f64) {
    use crate::intrinsic_type_state::{NumericOrScalar, set_max_value_exclusive};
    let numeric = crate::numeric::Numeric::new(&value.to_string())
        .expect("valid numeric value from TypeSpec");
    set_max_value_exclusive(state, target, NumericOrScalar::Numeric(numeric));
}

/// Get @maxValueExclusive value.
/// Delegates to `intrinsic_type_state::get_max_value_exclusive()`.
pub fn get_max_value_exclusive(state: &StateAccessors, target: TypeId) -> Option<f64> {
    crate::intrinsic_type_state::get_max_value_exclusive(state, target)
}

// ============================================================================
// Array size constraint decorators
// ============================================================================

/// Apply @minItems decorator.
/// Delegates to `intrinsic_type_state::set_min_items()`.
pub fn apply_min_items(state: &mut StateAccessors, target: TypeId, value: i64) {
    crate::intrinsic_type_state::set_min_items(
        state,
        target,
        &crate::numeric::Numeric::new(&value.to_string())
            .expect("valid numeric value from TypeSpec"),
    );
}

/// Get @minItems value.
/// Delegates to `intrinsic_type_state::get_min_items()`.
pub fn get_min_items(state: &StateAccessors, target: TypeId) -> Option<i64> {
    crate::intrinsic_type_state::get_min_items(state, target).map(|v| v.round() as i64)
}

/// Apply @maxItems decorator.
/// Delegates to `intrinsic_type_state::set_max_items()`.
pub fn apply_max_items(state: &mut StateAccessors, target: TypeId, value: i64) {
    crate::intrinsic_type_state::set_max_items(
        state,
        target,
        &crate::numeric::Numeric::new(&value.to_string())
            .expect("valid numeric value from TypeSpec"),
    );
}

/// Get @maxItems value.
/// Delegates to `intrinsic_type_state::get_max_items()`.
pub fn get_max_items(state: &StateAccessors, target: TypeId) -> Option<i64> {
    crate::intrinsic_type_state::get_max_items(state, target).map(|v| v.round() as i64)
}

// ============================================================================
// Overload decorator
// ============================================================================

typeid_decorator!(apply_overload, get_overload, STATE_OVERLOAD);

// ============================================================================
// Example decorators
// ============================================================================

/// Internal helper: append a titled example entry to state.
///
/// Note: Uses `::` to separate title from value, and `||` between entries.
/// This matches the TS compiler behavior. Values containing these delimiters
/// may be parsed incorrectly by `get_examples_impl()`.
fn apply_example_impl(
    state: &mut StateAccessors,
    state_key: &str,
    target: TypeId,
    title: Option<&str>,
    value: &str,
) {
    let entry = match title {
        Some(t) => format!("{}::{}", t, value),
        None => format!("::{}", value),
    };
    state.append_state(state_key, target, &entry, "||");
}

/// Internal helper: parse titled example entries from state.
fn get_examples_impl(
    state: &StateAccessors,
    state_key: &str,
    target: TypeId,
) -> Vec<(Option<String>, String)> {
    let raw = match state.get_state(state_key, target) {
        Some(s) => s,
        None => return Vec::new(),
    };
    raw.split("||")
        .filter(|s| !s.is_empty())
        .map(|entry| {
            if let Some((title, value)) = entry.split_once("::") {
                if title.is_empty() {
                    (None, value.to_string())
                } else {
                    (Some(title.to_string()), value.to_string())
                }
            } else {
                (None, entry.to_string())
            }
        })
        .collect()
}

/// Apply @example decorator.
/// Stores example data as a string value. Multiple examples are stored
/// using the `::` separator for title and `||` delimiter between entries.
pub fn apply_example(state: &mut StateAccessors, target: TypeId, title: Option<&str>, value: &str) {
    apply_example_impl(state, STATE_EXAMPLE, target, title, value);
}

/// Get all examples for a type.
/// Returns a Vec of (title, value) pairs.
pub fn get_examples(state: &StateAccessors, target: TypeId) -> Vec<(Option<String>, String)> {
    get_examples_impl(state, STATE_EXAMPLE, target)
}

/// Apply @opExample decorator.
/// Stores operation example data as a string value.
pub fn apply_op_example(
    state: &mut StateAccessors,
    target: TypeId,
    title: Option<&str>,
    value: &str,
) {
    apply_example_impl(state, STATE_OP_EXAMPLE, target, title, value);
}

/// Get all operation examples.
/// Returns a Vec of (title, value) pairs.
pub fn get_op_examples(state: &StateAccessors, target: TypeId) -> Vec<(Option<String>, String)> {
    get_examples_impl(state, STATE_OP_EXAMPLE, target)
}

// ============================================================================
// Type manipulation decorators
// ============================================================================

flag_decorator!(
    apply_with_optional_properties,
    is_with_optional_properties,
    STATE_WITH_OPTIONAL_PROPERTIES
);
flag_decorator!(
    apply_without_default_values,
    is_without_default_values,
    STATE_WITHOUT_DEFAULT_VALUES
);

string_decorator!(
    apply_without_omitted_properties,
    get_without_omitted_properties,
    STATE_WITHOUT_OMITTED_PROPERTIES
);
string_decorator!(
    apply_with_picked_properties,
    get_with_picked_properties,
    STATE_WITH_PICKED_PROPERTIES
);

// ============================================================================
// Debug decorators
// ============================================================================

string_decorator!(apply_inspect_type, get_inspect_type, STATE_INSPECT_TYPE);
string_decorator!(
    apply_inspect_type_name,
    get_inspect_type_name,
    STATE_INSPECT_TYPE_NAME
);

// Visibility decorators are in visibility.rs, re-exported via `pub use visibility::*`
// Encoded name functions are in encoded_names.rs, re-exported via `pub use encoded_names::*`
// Utility functions and intrinsic decorators are in utils.rs, re-exported via `pub use utils::*`
// TSP source is in tsp_sources.rs, re-exported via `pub use tsp_sources::*`

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace() {
        assert_eq!(TYPESPEC_NAMESPACE, "TypeSpec");
    }

    #[test]
    fn test_create_std_library() {
        let diags = create_std_library();
        assert!(diags.len() >= 4);
    }

    #[test]
    fn test_apply_doc() {
        let mut state = StateAccessors::new();
        assert_eq!(get_doc(&state, 1), None);
        apply_doc(&mut state, 1, "A pet");
        assert_eq!(get_doc(&state, 1), Some("A pet".to_string()));
    }

    #[test]
    fn test_apply_summary() {
        let mut state = StateAccessors::new();
        apply_summary(&mut state, 1, "Pet model");
        assert_eq!(get_summary(&state, 1), Some("Pet model".to_string()));
    }

    #[test]
    fn test_apply_error() {
        let mut state = StateAccessors::new();
        assert!(!is_error(&state, 1));
        apply_error(&mut state, 1);
        assert!(is_error(&state, 1));
    }

    #[test]
    fn test_apply_service() {
        let mut state = StateAccessors::new();
        assert!(!is_service(&state, 1));
        apply_service(&mut state, 1, Some("PetStore"));
        assert!(is_service(&state, 1));
        assert_eq!(get_service_title(&state, 1), Some("PetStore".to_string()));
    }

    #[test]
    fn test_add_service() {
        let mut state = StateAccessors::new();
        add_service(
            &mut state,
            1,
            ServiceDetails {
                title: Some("MyAPI".to_string()),
            },
        );
        assert!(is_service(&state, 1));
        assert_eq!(get_service_title(&state, 1), Some("MyAPI".to_string()));
    }

    #[test]
    fn test_add_service_merges_title() {
        let mut state = StateAccessors::new();
        apply_service(&mut state, 1, Some("Original"));
        add_service(&mut state, 1, ServiceDetails { title: None });
        // add_service with no title should keep existing title
        assert_eq!(get_service_title(&state, 1), Some("Original".to_string()));
    }

    #[test]
    fn test_get_service() {
        let mut state = StateAccessors::new();
        assert!(get_service(&state, 1).is_none());
        apply_service(&mut state, 1, Some("PetStore"));
        let svc = get_service(&state, 1).unwrap();
        assert_eq!(svc.namespace_type, 1);
        assert_eq!(svc.details.title, Some("PetStore".to_string()));
    }

    #[test]
    fn test_list_services() {
        let mut state = StateAccessors::new();
        assert!(list_services(&state).is_empty());
        apply_service(&mut state, 1, Some("Service1"));
        apply_service(&mut state, 2, Some("Service2"));
        let services = list_services(&state);
        assert_eq!(services.len(), 2);
    }

    #[test]
    fn test_apply_format() {
        let mut state = StateAccessors::new();
        apply_format(&mut state, 1, "uuid");
        assert_eq!(get_format(&state, 1), Some("uuid".to_string()));
    }

    #[test]
    fn test_apply_pattern() {
        let mut state = StateAccessors::new();
        apply_pattern(&mut state, 1, "[a-z]+", None);
        assert_eq!(get_pattern(&state, 1), Some("[a-z]+".to_string()));
    }

    #[test]
    fn test_apply_pattern_with_validation_message() {
        let mut state = StateAccessors::new();
        apply_pattern(&mut state, 1, "[a-z]+", Some("Must be lowercase letters"));
        assert_eq!(get_pattern(&state, 1), Some("[a-z]+".to_string()));
        let data = get_pattern_data(&state, 1).unwrap();
        assert_eq!(data.pattern, "[a-z]+");
        assert_eq!(
            data.validation_message,
            Some("Must be lowercase letters".to_string())
        );
    }

    #[test]
    fn test_pattern_data_no_validation_message() {
        let mut state = StateAccessors::new();
        apply_pattern(&mut state, 1, "^\\d+$", None);
        let data = get_pattern_data(&state, 1).unwrap();
        assert_eq!(data.pattern, "^\\d+$");
        assert_eq!(data.validation_message, None);
    }

    #[test]
    fn test_apply_min_max_length() {
        let mut state = StateAccessors::new();
        apply_min_length(&mut state, 1, 2);
        apply_max_length(&mut state, 1, 20);
        assert_eq!(get_min_length(&state, 1), Some(2));
        assert_eq!(get_max_length(&state, 1), Some(20));
    }

    #[test]
    fn test_apply_min_max_value() {
        let mut state = StateAccessors::new();
        apply_min_value(&mut state, 1, 18.0);
        apply_max_value(&mut state, 1, 200.0);
        assert_eq!(get_min_value(&state, 1), Some(18.0));
        assert_eq!(get_max_value(&state, 1), Some(200.0));
    }

    #[test]
    fn test_apply_secret() {
        let mut state = StateAccessors::new();
        assert!(!is_secret(&state, 1));
        apply_secret(&mut state, 1);
        assert!(is_secret(&state, 1));
    }

    #[test]
    fn test_apply_tag() {
        let mut state = StateAccessors::new();
        apply_tag(&mut state, 1, "pets");
        apply_tag(&mut state, 1, "store");
        let tags = get_tags(&state, 1);
        assert!(tags.contains(&"pets".to_string()));
        assert!(tags.contains(&"store".to_string()));
    }

    #[test]
    fn test_apply_key() {
        let mut state = StateAccessors::new();
        assert!(!is_key(&state, 1));
        apply_key(&mut state, 1, Some("id"));
        assert!(is_key(&state, 1));
        assert_eq!(get_key_name(&state, 1), Some("id".to_string()));
    }

    #[test]
    fn test_apply_friendly_name() {
        let mut state = StateAccessors::new();
        apply_friendly_name(&mut state, 1, "PetList");
        assert_eq!(get_friendly_name(&state, 1), Some("PetList".to_string()));
    }

    #[test]
    fn test_apply_discriminator() {
        let mut state = StateAccessors::new();
        apply_discriminator(&mut state, 1, "kind");
        assert_eq!(get_discriminator(&state, 1), Some("kind".to_string()));
    }

    #[test]
    fn test_apply_encode() {
        let mut state = StateAccessors::new();
        apply_encode(&mut state, 1, Some("rfc7231"), None);
        assert_eq!(get_encode(&state, 1), Some("rfc7231".to_string()));
    }

    #[test]
    fn test_apply_encode_with_encode_as() {
        let mut state = StateAccessors::new();
        apply_encode(&mut state, 1, Some("unixTimestamp"), Some(42));
        assert_eq!(get_encode(&state, 1), Some("unixTimestamp".to_string()));
        let data = get_encode_data(&state, 1).unwrap();
        assert_eq!(data.encoding, Some("unixTimestamp".to_string()));
        assert_eq!(data.encode_as_type, Some(42));
    }

    #[test]
    fn test_encode_data_no_encoding() {
        let mut state = StateAccessors::new();
        apply_encode(&mut state, 1, None, Some(10));
        let data = get_encode_data(&state, 1).unwrap();
        assert_eq!(data.encoding, None);
        assert_eq!(data.encode_as_type, Some(10));
    }

    #[test]
    fn test_apply_encoded_name() {
        let mut state = StateAccessors::new();
        apply_encoded_name(&mut state, 1, "application/json", "exp");
        assert_eq!(
            get_encoded_name(&state, 1, "application/json"),
            Some("exp".to_string())
        );
        assert_eq!(get_encoded_name(&state, 1, "application/xml"), None);
    }

    #[test]
    fn test_apply_encoded_name_multiple_mime_types() {
        let mut state = StateAccessors::new();
        apply_encoded_name(&mut state, 1, "application/json", "exp");
        apply_encoded_name(&mut state, 1, "application/xml", "expiry");
        assert_eq!(
            get_encoded_name(&state, 1, "application/json"),
            Some("exp".to_string())
        );
        assert_eq!(
            get_encoded_name(&state, 1, "application/xml"),
            Some("expiry".to_string())
        );
        assert_eq!(get_encoded_name(&state, 1, "application/yaml"), None);
    }

    #[test]
    fn test_encoded_name_suffix_resolution() {
        // "application/merge-patch+json" should resolve to "application/json" entry
        let mut state = StateAccessors::new();
        apply_encoded_name(&mut state, 1, "application/json", "exp");
        assert_eq!(
            get_encoded_name(&state, 1, "application/merge-patch+json"),
            Some("exp".to_string())
        );
    }

    #[test]
    fn test_get_all_encoded_names() {
        let mut state = StateAccessors::new();
        apply_encoded_name(&mut state, 1, "application/json", "exp");
        apply_encoded_name(&mut state, 1, "application/xml", "expiry");
        let all = get_all_encoded_names(&state, 1);
        assert_eq!(all.len(), 2);
        assert!(all.contains(&("application/json".to_string(), "exp".to_string())));
        assert!(all.contains(&("application/xml".to_string(), "expiry".to_string())));
    }

    #[test]
    fn test_resolve_encoded_name() {
        let mut state = StateAccessors::new();
        apply_encoded_name(&mut state, 1, "application/json", "exp");
        assert_eq!(
            resolve_encoded_name(&state, 1, "application/json", "expireAt"),
            "exp"
        );
        assert_eq!(
            resolve_encoded_name(&state, 1, "application/xml", "expireAt"),
            "expireAt"
        );
    }

    #[test]
    fn test_encoded_name_update_existing() {
        let mut state = StateAccessors::new();
        apply_encoded_name(&mut state, 1, "application/json", "exp");
        apply_encoded_name(&mut state, 1, "application/json", "updated_exp");
        assert_eq!(
            get_encoded_name(&state, 1, "application/json"),
            Some("updated_exp".to_string())
        );
    }

    #[test]
    fn test_apply_media_type_hint() {
        let mut state = StateAccessors::new();
        apply_media_type_hint(&mut state, 1, "application/xml");
        assert_eq!(
            get_media_type_hint(&state, 1),
            Some("application/xml".to_string())
        );
    }

    #[test]
    fn test_paging_decorators() {
        let mut state = StateAccessors::new();
        assert!(!is_list(&state, 1));
        apply_list(&mut state, 1);
        assert!(is_list(&state, 1));

        assert!(!is_page_items(&state, 2));
        apply_page_items(&mut state, 2);
        assert!(is_page_items(&state, 2));

        assert!(!is_continuation_token(&state, 3));
        apply_continuation_token(&mut state, 3);
        assert!(is_continuation_token(&state, 3));

        assert!(!is_next_link(&state, 4));
        apply_next_link(&mut state, 4);
        assert!(is_next_link(&state, 4));
    }

    #[test]
    fn test_encoding_enums() {
        assert_eq!(DateTimeKnownEncoding::Rfc3339.as_str(), "rfc3339");
        assert_eq!(
            DateTimeKnownEncoding::parse_str("unixTimestamp"),
            Some(DateTimeKnownEncoding::UnixTimestamp)
        );

        assert_eq!(DurationKnownEncoding::Iso8601.as_str(), "ISO8601");
        assert_eq!(
            DurationKnownEncoding::parse_str("seconds"),
            Some(DurationKnownEncoding::Seconds)
        );

        assert_eq!(BytesKnownEncoding::Base64.as_str(), "base64");
        assert_eq!(
            BytesKnownEncoding::parse_str("base64url"),
            Some(BytesKnownEncoding::Base64url)
        );

        assert_eq!(ArrayEncoding::PipeDelimited.as_str(), "pipeDelimited");
        assert_eq!(
            ArrayEncoding::parse_str("commaDelimited"),
            Some(ArrayEncoding::CommaDelimited)
        );
    }

    #[test]
    fn test_decorators_tsp_not_empty() {
        assert!(!STD_DECORATORS_TSP.is_empty());
        assert!(STD_DECORATORS_TSP.contains("dec doc"));
        assert!(STD_DECORATORS_TSP.contains("dec error"));
        assert!(STD_DECORATORS_TSP.contains("dec tag"));
        assert!(STD_DECORATORS_TSP.contains("dec key"));
        assert!(STD_DECORATORS_TSP.contains("dec encode"));
        assert!(STD_DECORATORS_TSP.contains("dec list"));
        assert!(STD_DECORATORS_TSP.contains("dec example"));
    }

    #[test]
    fn test_returns_doc() {
        let mut state = StateAccessors::new();
        assert_eq!(get_returns_doc(&state, 1), None);
        apply_returns_doc(&mut state, 1, "Returns a list of pets");
        assert_eq!(
            get_returns_doc(&state, 1),
            Some("Returns a list of pets".to_string())
        );
    }

    #[test]
    fn test_errors_doc() {
        let mut state = StateAccessors::new();
        assert_eq!(get_errors_doc(&state, 1), None);
        apply_errors_doc(&mut state, 1, "Error responses");
        assert_eq!(
            get_errors_doc(&state, 1),
            Some("Error responses".to_string())
        );
    }

    #[test]
    fn test_min_max_value_exclusive() {
        let mut state = StateAccessors::new();
        apply_min_value_exclusive(&mut state, 1, 0.0);
        apply_max_value_exclusive(&mut state, 1, 100.0);
        assert_eq!(get_min_value_exclusive(&state, 1), Some(0.0));
        assert_eq!(get_max_value_exclusive(&state, 1), Some(100.0));
    }

    #[test]
    fn test_min_max_items() {
        let mut state = StateAccessors::new();
        apply_min_items(&mut state, 1, 1);
        apply_max_items(&mut state, 1, 100);
        assert_eq!(get_min_items(&state, 1), Some(1));
        assert_eq!(get_max_items(&state, 1), Some(100));
    }

    #[test]
    fn test_overload() {
        let mut state = StateAccessors::new();
        assert_eq!(get_overload(&state, 1), None);
        apply_overload(&mut state, 1, 42);
        assert_eq!(get_overload(&state, 1), Some(42));
    }

    #[test]
    fn test_example_single() {
        let mut state = StateAccessors::new();
        assert!(get_examples(&state, 1).is_empty());
        apply_example(&mut state, 1, Some("Basic"), r#"{"name": "Fido"}"#);
        let examples = get_examples(&state, 1);
        assert_eq!(examples.len(), 1);
        assert_eq!(examples[0].0, Some("Basic".to_string()));
        assert_eq!(examples[0].1, r#"{"name": "Fido"}"#);
    }

    #[test]
    fn test_example_multiple() {
        let mut state = StateAccessors::new();
        apply_example(&mut state, 1, Some("Example1"), r#"{"name": "Fido"}"#);
        apply_example(&mut state, 1, None, r#"{"name": "Spot"}"#);
        let examples = get_examples(&state, 1);
        assert_eq!(examples.len(), 2);
        assert_eq!(examples[0].0, Some("Example1".to_string()));
        assert_eq!(examples[1].0, None);
    }

    #[test]
    fn test_op_example() {
        let mut state = StateAccessors::new();
        apply_op_example(
            &mut state,
            1,
            Some("GetPet"),
            r#"{"parameters": {"id": 1}}"#,
        );
        let examples = get_op_examples(&state, 1);
        assert_eq!(examples.len(), 1);
        assert_eq!(examples[0].0, Some("GetPet".to_string()));
    }

    #[test]
    fn test_with_optional_properties() {
        let mut state = StateAccessors::new();
        assert!(!is_with_optional_properties(&state, 1));
        apply_with_optional_properties(&mut state, 1);
        assert!(is_with_optional_properties(&state, 1));
    }

    #[test]
    fn test_without_default_values() {
        let mut state = StateAccessors::new();
        assert!(!is_without_default_values(&state, 1));
        apply_without_default_values(&mut state, 1);
        assert!(is_without_default_values(&state, 1));
    }

    #[test]
    fn test_without_omitted_properties() {
        let mut state = StateAccessors::new();
        assert_eq!(get_without_omitted_properties(&state, 1), None);
        apply_without_omitted_properties(&mut state, 1, "password");
        assert_eq!(
            get_without_omitted_properties(&state, 1),
            Some("password".to_string())
        );
    }

    #[test]
    fn test_with_picked_properties() {
        let mut state = StateAccessors::new();
        assert_eq!(get_with_picked_properties(&state, 1), None);
        apply_with_picked_properties(&mut state, 1, "name");
        assert_eq!(
            get_with_picked_properties(&state, 1),
            Some("name".to_string())
        );
    }

    #[test]
    fn test_inspect_type() {
        let mut state = StateAccessors::new();
        assert_eq!(get_inspect_type(&state, 1), None);
        apply_inspect_type(&mut state, 1, "checking model");
        assert_eq!(
            get_inspect_type(&state, 1),
            Some("checking model".to_string())
        );
    }

    #[test]
    fn test_inspect_type_name() {
        let mut state = StateAccessors::new();
        assert_eq!(get_inspect_type_name(&state, 1), None);
        apply_inspect_type_name(&mut state, 1, "Pet");
        assert_eq!(get_inspect_type_name(&state, 1), Some("Pet".to_string()));
    }

    #[test]
    fn test_all_paging_decorators() {
        let mut state = StateAccessors::new();
        apply_prev_link(&mut state, 1);
        apply_first_link(&mut state, 2);
        apply_last_link(&mut state, 3);
        apply_offset(&mut state, 4);
        apply_page_index(&mut state, 5);
        apply_page_size(&mut state, 6);

        assert!(is_prev_link(&state, 1));
        assert!(is_first_link(&state, 2));
        assert!(is_last_link(&state, 3));
        assert!(is_offset(&state, 4));
        assert!(is_page_index(&state, 5));
        assert!(is_page_size(&state, 6));

        assert!(!is_prev_link(&state, 2));
        assert!(!is_offset(&state, 1));
    }

    #[test]
    fn test_paging_property_kind() {
        let mut state = StateAccessors::new();
        assert_eq!(get_paging_property_kind(&state, 1), None);

        apply_offset(&mut state, 1);
        assert_eq!(get_paging_property_kind(&state, 1), Some("offset"));

        apply_next_link(&mut state, 2);
        assert_eq!(get_paging_property_kind(&state, 2), Some("nextLink"));
    }

    #[test]
    fn test_multiple_paging_markers() {
        let mut state = StateAccessors::new();
        assert!(!has_multiple_paging_markers(&state, 1));

        apply_offset(&mut state, 1);
        assert!(!has_multiple_paging_markers(&state, 1));

        apply_page_index(&mut state, 1);
        assert!(has_multiple_paging_markers(&state, 1));
    }

    #[test]
    fn test_paging_operation_struct() {
        let op = PagingOperation {
            input: PagingInput {
                offset: Some(PagingProperty {
                    property: 1,
                    path: vec![1],
                }),
                page_size: Some(PagingProperty {
                    property: 2,
                    path: vec![2],
                }),
                ..Default::default()
            },
            output: PagingOutput {
                page_items: Some(PagingProperty {
                    property: 3,
                    path: vec![3],
                }),
                next_link: Some(PagingProperty {
                    property: 4,
                    path: vec![4],
                }),
                ..Default::default()
            },
        };
        assert!(op.input.offset.is_some());
        assert!(op.output.page_items.is_some());
        assert!(op.input.page_index.is_none());
    }

    // ========================================================================
    // Visibility decorator tests
    // ========================================================================

    #[test]
    fn test_visibility() {
        let mut state = StateAccessors::new();
        assert!(get_visibility(&state, 1).is_empty());
        apply_visibility(&mut state, 1, &["read", "update"]);
        let vis = get_visibility(&state, 1);
        assert!(vis.contains(&"read".to_string()));
        assert!(vis.contains(&"update".to_string()));
    }

    #[test]
    fn test_remove_visibility() {
        let mut state = StateAccessors::new();
        apply_remove_visibility(&mut state, 1, &["create"]);
        let vis = get_remove_visibility(&state, 1);
        assert!(vis.contains(&"create".to_string()));
    }

    #[test]
    fn test_invisible() {
        let mut state = StateAccessors::new();
        assert_eq!(get_invisible(&state, 1), None);
        apply_invisible(&mut state, 1, "Lifecycle");
        assert_eq!(get_invisible(&state, 1), Some("Lifecycle".to_string()));
    }

    #[test]
    fn test_default_visibility() {
        let mut state = StateAccessors::new();
        apply_default_visibility(&mut state, 1, &["read", "update"]);
        let vis = get_default_visibility(&state, 1);
        assert_eq!(vis.len(), 2);
        assert!(vis.contains(&"read".to_string()));
        assert!(vis.contains(&"update".to_string()));
    }

    #[test]
    fn test_with_visibility() {
        let mut state = StateAccessors::new();
        apply_with_visibility(&mut state, 1, &["read"]);
        let vis = get_with_visibility(&state, 1);
        assert!(vis.contains(&"read".to_string()));
    }

    #[test]
    fn test_with_updateable_properties() {
        let mut state = StateAccessors::new();
        assert!(!is_with_updateable_properties(&state, 1));
        apply_with_updateable_properties(&mut state, 1);
        assert!(is_with_updateable_properties(&state, 1));
    }

    #[test]
    fn test_with_visibility_filter() {
        let mut state = StateAccessors::new();
        assert_eq!(get_with_visibility_filter(&state, 1), None);
        apply_with_visibility_filter(&mut state, 1, "read");
        assert_eq!(
            get_with_visibility_filter(&state, 1),
            Some("read".to_string())
        );
    }

    #[test]
    fn test_with_lifecycle_update() {
        let mut state = StateAccessors::new();
        assert!(!is_with_lifecycle_update(&state, 1));
        apply_with_lifecycle_update(&mut state, 1);
        assert!(is_with_lifecycle_update(&state, 1));
    }

    #[test]
    fn test_parameter_visibility() {
        let mut state = StateAccessors::new();
        apply_parameter_visibility(&mut state, 1, &["create", "update"]);
        let vis = get_parameter_visibility(&state, 1);
        assert!(vis.contains(&"create".to_string()));
        assert!(vis.contains(&"update".to_string()));
    }

    #[test]
    fn test_return_type_visibility() {
        let mut state = StateAccessors::new();
        apply_return_type_visibility(&mut state, 1, &["read"]);
        let vis = get_return_type_visibility(&state, 1);
        assert!(vis.contains(&"read".to_string()));
    }

    #[test]
    fn test_with_default_key_visibility() {
        let mut state = StateAccessors::new();
        assert_eq!(get_with_default_key_visibility(&state, 1), None);
        apply_with_default_key_visibility(&mut state, 1, "read");
        assert_eq!(
            get_with_default_key_visibility(&state, 1),
            Some("read".to_string())
        );
    }

    // ========================================================================
    // Type checker function tests
    // ========================================================================

    #[test]
    fn test_is_string_type() {
        let mut state = StateAccessors::new();
        assert!(!is_string_type(&state, 1));
        set_string_type(&mut state, 1);
        assert!(is_string_type(&state, 1));
    }

    #[test]
    fn test_is_numeric_type() {
        let mut state = StateAccessors::new();
        assert!(!is_numeric_type(&state, 1));
        set_numeric_type(&mut state, 1);
        assert!(is_numeric_type(&state, 1));
    }

    #[test]
    fn test_is_date_time_type() {
        let mut state = StateAccessors::new();
        assert!(!is_date_time_type(&state, 1));
        set_date_time_type(&mut state, 1);
        assert!(is_date_time_type(&state, 1));
    }

    #[test]
    fn test_deprecated() {
        let mut state = StateAccessors::new();
        assert_eq!(get_deprecated(&state, 1), None);
        apply_deprecated(&mut state, 1, "Use FooV2 instead");
        assert_eq!(
            get_deprecated(&state, 1),
            Some("Use FooV2 instead".to_string())
        );
    }

    #[test]
    fn test_doc_data_decorator_source() {
        let mut state = StateAccessors::new();
        apply_doc_with_source(&mut state, 1, "Some doc", DocSource::Decorator);
        let data = get_doc_data(&state, 1).unwrap();
        assert_eq!(data.value, "Some doc");
        assert_eq!(data.source, DocSource::Decorator);
    }

    #[test]
    fn test_doc_data_comment_source() {
        let mut state = StateAccessors::new();
        apply_doc_with_source(&mut state, 1, "Doc from comment", DocSource::Comment);
        let data = get_doc_data(&state, 1).unwrap();
        assert_eq!(data.value, "Doc from comment");
        assert_eq!(data.source, DocSource::Comment);
    }

    #[test]
    fn test_doc_source_parse() {
        assert_eq!(
            DocSource::parse_str("decorator"),
            Some(DocSource::Decorator)
        );
        assert_eq!(DocSource::parse_str("comment"), Some(DocSource::Comment));
        assert_eq!(DocSource::parse_str("other"), None);
    }

    #[test]
    fn test_doc_data_default_source() {
        let mut state = StateAccessors::new();
        // apply_doc doesn't set source, so get_doc_data should default to Decorator
        apply_doc(&mut state, 1, "Basic doc");
        let data = get_doc_data(&state, 1).unwrap();
        assert_eq!(data.value, "Basic doc");
        assert_eq!(data.source, DocSource::Decorator); // default
    }

    // ========================================================================
    // Intrinsic decorator tests
    // ========================================================================

    #[test]
    fn test_indexer() {
        let mut state = StateAccessors::new();
        assert_eq!(get_indexer(&state, 1), None);
        apply_indexer(&mut state, 1, 10, 20);
        let idx = get_indexer(&state, 1).unwrap();
        assert_eq!(idx.key, 10);
        assert_eq!(idx.value, 20);
    }

    #[test]
    fn test_prototype_getter() {
        let mut state = StateAccessors::new();
        assert!(!is_prototype_getter(&state, 1));
        apply_prototype_getter(&mut state, 1);
        assert!(is_prototype_getter(&state, 1));
    }
}
