//! Intrinsic type state for TypeSpec-Rust
//! Ported from TypeSpec compiler/src/core/intrinsic-type-state.ts
//!
//! Contains getters/setters for intrinsic decorator data:
//! @minValue, @maxValue, @minValueExclusive, @maxValueExclusive,
//! @minLength, @maxLength, @minItems, @maxItems,
//! @doc, @discriminator

use crate::checker::types::TypeId;
use crate::numeric::Numeric;
use crate::state_accessors::StateAccessors;
use std::fmt;

/// Macro to generate the _as_numeric + f64 getter pair for numeric state.
macro_rules! define_numeric_state_getter {
    ($get_as_numeric:ident, $get_f64:ident, $key:expr) => {
        pub fn $get_as_numeric(state: &StateAccessors, target: TypeId) -> Option<Numeric> {
            let stored = state.get_state($key, target)?;
            decode_as_numeric(stored)
        }
        pub fn $get_f64(state: &StateAccessors, target: TypeId) -> Option<f64> {
            $get_as_numeric(state, target).and_then(|n| n.as_f64())
        }
    };
}

// State key constants - aligned with compiler.rs STATE_* constants
const MIN_VALUES: &str = "TypeSpec.minValue";
const MAX_VALUES: &str = "TypeSpec.maxValue";
const MIN_VALUE_EXCLUSIVE: &str = "TypeSpec.minValueExclusive";
const MAX_VALUE_EXCLUSIVE: &str = "TypeSpec.maxValueExclusive";
const MIN_LENGTH: &str = "TypeSpec.minLength";
const MAX_LENGTH: &str = "TypeSpec.maxLength";
const MIN_ITEMS: &str = "TypeSpec.minItems";
const MAX_ITEMS: &str = "TypeSpec.maxItems";
const DOCS: &str = "TypeSpec.doc";
const RETURN_DOCS: &str = "TypeSpec.returnsDoc";
const ERRORS_DOCS: &str = "TypeSpec.errorsDoc";
const DISCRIMINATOR: &str = "TypeSpec.discriminator";

/// Documentation target kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocTarget {
    Self_,
    Returns,
    Errors,
}

/// Documentation data with source tracking
#[derive(Debug, Clone)]
pub struct DocData {
    /// The documentation text
    pub value: String,
    /// How the doc was set: via decorator or comment
    pub source: DocSource,
}

/// Source of documentation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocSource {
    Decorator,
    Comment,
}

impl fmt::Display for DocSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DocSource::Decorator => write!(f, "decorator"),
            DocSource::Comment => write!(f, "comment"),
        }
    }
}

impl DocSource {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            DocSource::Decorator => "decorator",
            DocSource::Comment => "comment",
        }
    }

    /// Parse from string
    pub fn parse_str(s: &str) -> Option<Self> {
        match s {
            "decorator" => Some(DocSource::Decorator),
            "comment" => Some(DocSource::Comment),
            _ => None,
        }
    }
}

/// Discriminator information
#[derive(Debug, Clone)]
pub struct Discriminator {
    /// The property name used as discriminator
    pub property_name: String,
}

/// A scalar value (non-numeric, like datetime strings)
#[derive(Debug, Clone)]
pub struct ScalarValue {
    /// The scalar value as a string
    pub value: String,
}

impl fmt::Display for ScalarValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Value that can be either a Numeric or a ScalarValue
#[derive(Debug, Clone)]
pub enum NumericOrScalar {
    Numeric(Numeric),
    Scalar(ScalarValue),
}

// Helper: encode a NumericOrScalar as a string for storage
fn encode_numeric_or_scalar(value: &NumericOrScalar) -> String {
    match value {
        NumericOrScalar::Numeric(n) => format!("n:{}", n.as_string()),
        NumericOrScalar::Scalar(s) => format!("s:{}", s.value),
    }
}

// Helper: decode a stored string as Numeric (returns None if it's a scalar)
fn decode_as_numeric(stored: &str) -> Option<Numeric> {
    if let Some(num_str) = stored.strip_prefix("n:") {
        Numeric::new(num_str).ok()
    } else {
        None
    }
}

// Helper: decode a stored string as ScalarValue (returns None if it's numeric)
fn decode_as_scalar(stored: &str) -> Option<ScalarValue> {
    stored.strip_prefix("s:").map(|val| ScalarValue {
        value: val.to_string(),
    })
}

// Helper: encode doc data as string
// Uses \x00 (null byte) as separator since it cannot appear in TypeSpec doc strings.
fn encode_doc_data(data: &DocData) -> String {
    format!("d:{}\x00{}", data.source, data.value)
}

// Helper: decode doc data from string
fn decode_doc_data(stored: &str) -> Option<DocData> {
    let rest = stored.strip_prefix("d:")?;
    let (source_str, value) = rest.split_once('\x00')?;
    let source = match source_str {
        "decorator" => DocSource::Decorator,
        "comment" => DocSource::Comment,
        _ => DocSource::Comment,
    };
    Some(DocData {
        value: value.to_string(),
        source,
    })
}

// Helper: encode discriminator
fn encode_discriminator(disc: &Discriminator) -> String {
    format!("disc:{}", disc.property_name)
}

// Helper: decode discriminator
fn decode_discriminator(stored: &str) -> Option<Discriminator> {
    let prop_name = stored.strip_prefix("disc:")?;
    Some(Discriminator {
        property_name: prop_name.to_string(),
    })
}

// ============================================================================
// @minValue
// ============================================================================

/// Set the min value for a type
pub fn set_min_value(state: &mut StateAccessors, target: TypeId, value: NumericOrScalar) {
    state.set_state(MIN_VALUES, target, encode_numeric_or_scalar(&value));
}

// Get the min value as a Numeric (returns None for scalar values) and as f64
define_numeric_state_getter!(get_min_value_as_numeric, get_min_value, MIN_VALUES);

/// Get the min value for a scalar type
pub fn get_min_value_for_scalar(state: &StateAccessors, target: TypeId) -> Option<ScalarValue> {
    let stored = state.get_state(MIN_VALUES, target)?;
    decode_as_scalar(stored)
}

// ============================================================================
// @maxValue
// ============================================================================

/// Set the max value for a type
pub fn set_max_value(state: &mut StateAccessors, target: TypeId, value: NumericOrScalar) {
    state.set_state(MAX_VALUES, target, encode_numeric_or_scalar(&value));
}

define_numeric_state_getter!(get_max_value_as_numeric, get_max_value, MAX_VALUES);

/// Get the max value for a scalar type
pub fn get_max_value_for_scalar(state: &StateAccessors, target: TypeId) -> Option<ScalarValue> {
    let stored = state.get_state(MAX_VALUES, target)?;
    decode_as_scalar(stored)
}

// ============================================================================
// @minValueExclusive / @maxValueExclusive
// ============================================================================

/// Set the exclusive min value for a type
pub fn set_min_value_exclusive(state: &mut StateAccessors, target: TypeId, value: NumericOrScalar) {
    state.set_state(
        MIN_VALUE_EXCLUSIVE,
        target,
        encode_numeric_or_scalar(&value),
    );
}

define_numeric_state_getter!(
    get_min_value_exclusive_as_numeric,
    get_min_value_exclusive,
    MIN_VALUE_EXCLUSIVE
);

/// Set the exclusive max value for a type
pub fn set_max_value_exclusive(state: &mut StateAccessors, target: TypeId, value: NumericOrScalar) {
    state.set_state(
        MAX_VALUE_EXCLUSIVE,
        target,
        encode_numeric_or_scalar(&value),
    );
}

define_numeric_state_getter!(
    get_max_value_exclusive_as_numeric,
    get_max_value_exclusive,
    MAX_VALUE_EXCLUSIVE
);

// ============================================================================
// @minLength / @maxLength
// ============================================================================

/// Set the min length for a string type
pub fn set_min_length(state: &mut StateAccessors, target: TypeId, value: &Numeric) {
    state.set_state(MIN_LENGTH, target, format!("n:{}", value.as_string()));
}

define_numeric_state_getter!(get_min_length_as_numeric, get_min_length, MIN_LENGTH);

/// Set the max length for a string type
pub fn set_max_length(state: &mut StateAccessors, target: TypeId, value: &Numeric) {
    state.set_state(MAX_LENGTH, target, format!("n:{}", value.as_string()));
}

define_numeric_state_getter!(get_max_length_as_numeric, get_max_length, MAX_LENGTH);

// ============================================================================
// @minItems / @maxItems
// ============================================================================

/// Set the min items for an array type
pub fn set_min_items(state: &mut StateAccessors, target: TypeId, value: &Numeric) {
    state.set_state(MIN_ITEMS, target, format!("n:{}", value.as_string()));
}

define_numeric_state_getter!(get_min_items_as_numeric, get_min_items, MIN_ITEMS);

/// Set the max items for an array type
pub fn set_max_items(state: &mut StateAccessors, target: TypeId, value: &Numeric) {
    state.set_state(MAX_ITEMS, target, format!("n:{}", value.as_string()));
}

define_numeric_state_getter!(get_max_items_as_numeric, get_max_items, MAX_ITEMS);

// ============================================================================
// @doc
// ============================================================================

/// Set documentation data for a type
pub fn set_doc_data(
    state: &mut StateAccessors,
    target: TypeId,
    doc_target: DocTarget,
    data: DocData,
) {
    let key = doc_key(doc_target);
    state.set_state(key, target, encode_doc_data(&data));
}

/// Get documentation data for a type
pub fn get_doc_data(
    state: &StateAccessors,
    target: TypeId,
    doc_target: DocTarget,
) -> Option<DocData> {
    let key = doc_key(doc_target);
    let stored = state.get_state(key, target)?;
    decode_doc_data(stored)
}

/// Get documentation for the type itself (convenience)
pub fn get_doc(state: &StateAccessors, target: TypeId) -> Option<String> {
    get_doc_data(state, target, DocTarget::Self_).map(|d| d.value)
}

fn doc_key(target: DocTarget) -> &'static str {
    match target {
        DocTarget::Self_ => DOCS,
        DocTarget::Returns => RETURN_DOCS,
        DocTarget::Errors => ERRORS_DOCS,
    }
}

// ============================================================================
// @discriminator
// ============================================================================

/// Set the discriminator for a type
pub fn set_discriminator(state: &mut StateAccessors, target: TypeId, discriminator: Discriminator) {
    state.set_state(DISCRIMINATOR, target, encode_discriminator(&discriminator));
}

/// Get the discriminator for a type
pub fn get_discriminator(state: &StateAccessors, target: TypeId) -> Option<Discriminator> {
    let stored = state.get_state(DISCRIMINATOR, target)?;
    decode_discriminator(stored)
}

/// Get all types that have a discriminator set.
/// Ported from TS getDiscriminatedTypes().
/// Returns a list of (TypeId, Discriminator) pairs for all types with @discriminator.
pub fn get_discriminated_types(state: &StateAccessors) -> Vec<(TypeId, Discriminator)> {
    let mut result = Vec::new();
    if let Some(state_map) = state.get_state_map(DISCRIMINATOR) {
        for (&type_id, stored) in state_map {
            if let Some(disc) = decode_discriminator(stored) {
                result.push((type_id, disc));
            }
        }
    }
    result
}

// ============================================================================
// @discriminated options
// ============================================================================

const DISCRIMINATED_OPTIONS: &str = "discriminated";

/// Discriminated union options
#[derive(Debug, Clone)]
pub struct DiscriminatedOptions {
    /// The discriminator property name
    pub property_name: String,
}

fn encode_discriminated_options(opts: &DiscriminatedOptions) -> String {
    format!("dopt:{}", opts.property_name)
}

fn decode_discriminated_options(stored: &str) -> Option<DiscriminatedOptions> {
    let prop = stored.strip_prefix("dopt:")?;
    Some(DiscriminatedOptions {
        property_name: prop.to_string(),
    })
}

/// Set discriminated options for a union type.
/// Ported from TS setDiscriminatedOptions().
pub fn set_discriminated_options(
    state: &mut StateAccessors,
    target: TypeId,
    opts: DiscriminatedOptions,
) {
    state.set_state(
        DISCRIMINATED_OPTIONS,
        target,
        encode_discriminated_options(&opts),
    );
}

/// Get discriminated options for a union type.
/// Ported from TS getDiscriminatedOptions().
pub fn get_discriminated_options(
    state: &StateAccessors,
    target: TypeId,
) -> Option<DiscriminatedOptions> {
    let stored = state.get_state(DISCRIMINATED_OPTIONS, target)?;
    decode_discriminated_options(stored)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_max_value() {
        let mut state = StateAccessors::new();
        let target: TypeId = 10;

        set_min_value(
            &mut state,
            target,
            NumericOrScalar::Numeric(Numeric::new("0").unwrap()),
        );
        set_max_value(
            &mut state,
            target,
            NumericOrScalar::Numeric(Numeric::new("100").unwrap()),
        );

        let min = get_min_value_as_numeric(&state, target);
        assert!(min.is_some());
        assert_eq!(min.unwrap().as_string(), "0");

        let max = get_max_value_as_numeric(&state, target);
        assert!(max.is_some());
        assert_eq!(max.unwrap().as_string(), "100");
    }

    #[test]
    fn test_scalar_value() {
        let mut state = StateAccessors::new();
        let target: TypeId = 20;

        set_min_value(
            &mut state,
            target,
            NumericOrScalar::Scalar(ScalarValue {
                value: "2023-01-01".to_string(),
            }),
        );

        // Should not return as numeric
        let min_num = get_min_value_as_numeric(&state, target);
        assert!(min_num.is_none());

        // Should return as scalar
        let min_scalar = get_min_value_for_scalar(&state, target);
        assert!(min_scalar.is_some());
        assert_eq!(min_scalar.unwrap().value, "2023-01-01");
    }

    #[test]
    fn test_min_max_length() {
        let mut state = StateAccessors::new();
        let target: TypeId = 30;

        set_min_length(&mut state, target, &Numeric::new("1").unwrap());
        set_max_length(&mut state, target, &Numeric::new("255").unwrap());

        let min = get_min_length_as_numeric(&state, target);
        assert!(min.is_some());
        assert_eq!(min.unwrap().as_string(), "1");

        let max = get_max_length_as_numeric(&state, target);
        assert!(max.is_some());
        assert_eq!(max.unwrap().as_string(), "255");
    }

    #[test]
    fn test_min_max_items() {
        let mut state = StateAccessors::new();
        let target: TypeId = 40;

        set_min_items(&mut state, target, &Numeric::new("0").unwrap());
        set_max_items(&mut state, target, &Numeric::new("100").unwrap());

        let min = get_min_items_as_numeric(&state, target);
        assert!(min.is_some());

        let max = get_max_items_as_numeric(&state, target);
        assert!(max.is_some());
    }

    #[test]
    fn test_doc_data() {
        let mut state = StateAccessors::new();
        let target: TypeId = 50;

        set_doc_data(
            &mut state,
            target,
            DocTarget::Self_,
            DocData {
                value: "This is a model".to_string(),
                source: DocSource::Decorator,
            },
        );

        let doc = get_doc_data(&state, target, DocTarget::Self_);
        assert!(doc.is_some());
        let doc = doc.unwrap();
        assert_eq!(doc.value, "This is a model");
        assert_eq!(doc.source, DocSource::Decorator);

        let simple_doc = get_doc(&state, target);
        assert_eq!(simple_doc, Some("This is a model".to_string()));
    }

    #[test]
    fn test_doc_targets() {
        let mut state = StateAccessors::new();
        let target: TypeId = 60;

        set_doc_data(
            &mut state,
            target,
            DocTarget::Self_,
            DocData {
                value: "self doc".to_string(),
                source: DocSource::Comment,
            },
        );
        set_doc_data(
            &mut state,
            target,
            DocTarget::Returns,
            DocData {
                value: "returns doc".to_string(),
                source: DocSource::Decorator,
            },
        );
        set_doc_data(
            &mut state,
            target,
            DocTarget::Errors,
            DocData {
                value: "errors doc".to_string(),
                source: DocSource::Decorator,
            },
        );

        assert_eq!(get_doc(&state, target), Some("self doc".to_string()));
        assert_eq!(
            get_doc_data(&state, target, DocTarget::Returns).map(|d| d.value),
            Some("returns doc".to_string())
        );
        assert_eq!(
            get_doc_data(&state, target, DocTarget::Errors).map(|d| d.value),
            Some("errors doc".to_string())
        );
    }

    #[test]
    fn test_discriminator() {
        let mut state = StateAccessors::new();
        let target: TypeId = 70;

        set_discriminator(
            &mut state,
            target,
            Discriminator {
                property_name: "kind".to_string(),
            },
        );

        let disc = get_discriminator(&state, target);
        assert!(disc.is_some());
        assert_eq!(disc.unwrap().property_name, "kind");
    }

    #[test]
    fn test_exclusive_min_max_value() {
        let mut state = StateAccessors::new();
        let target: TypeId = 80;

        set_min_value_exclusive(
            &mut state,
            target,
            NumericOrScalar::Numeric(Numeric::new("0").unwrap()),
        );
        set_max_value_exclusive(
            &mut state,
            target,
            NumericOrScalar::Numeric(Numeric::new("100").unwrap()),
        );

        let min = get_min_value_exclusive_as_numeric(&state, target);
        assert!(min.is_some());
        assert_eq!(min.unwrap().as_string(), "0");

        let max = get_max_value_exclusive_as_numeric(&state, target);
        assert!(max.is_some());
        assert_eq!(max.unwrap().as_string(), "100");
    }

    // ============================================================================
    // Convenience f64 getter tests
    // ============================================================================

    #[test]
    fn test_get_min_value_f64() {
        let mut state = StateAccessors::new();
        let target: TypeId = 100;

        set_min_value(
            &mut state,
            target,
            NumericOrScalar::Numeric(Numeric::new("42").unwrap()),
        );
        assert_eq!(get_min_value(&state, target), Some(42.0));
    }

    #[test]
    fn test_get_max_value_f64() {
        let mut state = StateAccessors::new();
        let target: TypeId = 101;

        set_max_value(
            &mut state,
            target,
            NumericOrScalar::Numeric(Numeric::new("99.5").unwrap()),
        );
        let val = get_max_value(&state, target);
        assert!(val.is_some());
        assert!((val.unwrap() - 99.5).abs() < 0.001);
    }

    #[test]
    fn test_get_min_length_f64() {
        let mut state = StateAccessors::new();
        let target: TypeId = 102;

        set_min_length(&mut state, target, &Numeric::new("1").unwrap());
        assert_eq!(get_min_length(&state, target), Some(1.0));
    }

    #[test]
    fn test_get_max_length_f64() {
        let mut state = StateAccessors::new();
        let target: TypeId = 103;

        set_max_length(&mut state, target, &Numeric::new("255").unwrap());
        assert_eq!(get_max_length(&state, target), Some(255.0));
    }

    #[test]
    fn test_get_min_items_f64() {
        let mut state = StateAccessors::new();
        let target: TypeId = 104;

        set_min_items(&mut state, target, &Numeric::new("0").unwrap());
        assert_eq!(get_min_items(&state, target), Some(0.0));
    }

    #[test]
    fn test_get_max_items_f64() {
        let mut state = StateAccessors::new();
        let target: TypeId = 105;

        set_max_items(&mut state, target, &Numeric::new("100").unwrap());
        assert_eq!(get_max_items(&state, target), Some(100.0));
    }

    #[test]
    fn test_get_min_value_scalar_returns_none_f64() {
        let mut state = StateAccessors::new();
        let target: TypeId = 106;

        // Scalar values should return None for f64 getter
        set_min_value(
            &mut state,
            target,
            NumericOrScalar::Scalar(ScalarValue {
                value: "2023-01-01".to_string(),
            }),
        );
        assert_eq!(get_min_value(&state, target), None);
    }

    #[test]
    fn test_exclusive_convenience_getters() {
        let mut state = StateAccessors::new();
        let target: TypeId = 107;

        set_min_value_exclusive(
            &mut state,
            target,
            NumericOrScalar::Numeric(Numeric::new("0").unwrap()),
        );
        set_max_value_exclusive(
            &mut state,
            target,
            NumericOrScalar::Numeric(Numeric::new("100").unwrap()),
        );

        assert_eq!(get_min_value_exclusive(&state, target), Some(0.0));
        assert_eq!(get_max_value_exclusive(&state, target), Some(100.0));
    }

    // ============================================================================
    // Discriminated options tests
    // ============================================================================

    #[test]
    fn test_discriminated_options() {
        let mut state = StateAccessors::new();
        let target: TypeId = 200;

        set_discriminated_options(
            &mut state,
            target,
            DiscriminatedOptions {
                property_name: "kind".to_string(),
            },
        );

        let opts = get_discriminated_options(&state, target);
        assert!(opts.is_some());
        assert_eq!(opts.unwrap().property_name, "kind");
    }

    #[test]
    fn test_discriminated_options_not_set() {
        let state = StateAccessors::new();
        let target: TypeId = 201;
        assert!(get_discriminated_options(&state, target).is_none());
    }
}
