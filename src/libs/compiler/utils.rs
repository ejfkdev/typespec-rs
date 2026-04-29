//! Utility functions and intrinsic decorators
//!
//! Ported from TypeSpec compiler/src/lib/utils.ts and lib/intrinsic/decorators.ts

use crate::checker::types::TypeId;
use crate::state_accessors::StateAccessors;

/// State key for indexer decorator
pub const STATE_INDEXER: &str = "TypeSpec.indexer";
/// State key for prototype getter
pub const STATE_PROTOTYPE_GETTER: &str = "TypeSpec.Prototypes.getter";

/// Model indexer data.
/// Ported from TS ModelIndexer interface.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelIndexer {
    /// Key type (TypeId reference to a Scalar)
    pub key: TypeId,
    /// Value type (TypeId reference)
    pub value: TypeId,
}

/// Apply indexer decorator (internal).
/// Ported from TS indexerDecorator().
pub fn apply_indexer(state: &mut StateAccessors, target: TypeId, key: TypeId, value: TypeId) {
    // Store as "key_id::value_id"
    state.set_state(STATE_INDEXER, target, format!("{}::{}", key, value));
}

/// Get indexer for a type.
/// Ported from TS getIndexer().
pub fn get_indexer(state: &StateAccessors, target: TypeId) -> Option<ModelIndexer> {
    state.get_state(STATE_INDEXER, target).and_then(|s| {
        let parts: Vec<&str> = s.split("::").collect();
        if parts.len() == 2 {
            let key = parts[0].parse::<TypeId>().ok()?;
            let value = parts[1].parse::<TypeId>().ok()?;
            Some(ModelIndexer { key, value })
        } else {
            None
        }
    })
}

/// Apply prototype getter decorator (internal).
/// Ported from TS getterDecorator().
pub fn apply_prototype_getter(state: &mut StateAccessors, target: TypeId) {
    state.set_state(STATE_PROTOTYPE_GETTER, target, "true".to_string());
}

/// Check if type is a prototype getter.
/// Ported from TS isPrototypeGetter().
pub fn is_prototype_getter(state: &StateAccessors, target: TypeId) -> bool {
    state.get_state(STATE_PROTOTYPE_GETTER, target).is_some()
}

/// Replace templated string placeholders with property values.
///
/// Given a format string with `{propertyName}` placeholders, replaces each
/// placeholder with the value from the provided property map.
///
/// Ported from TS replaceTemplatedStringFromProperties().
///
/// # Arguments
/// * `format_string` - The template string with {propertyName} placeholders
/// * `properties` - Map of property names to their string values
///
/// # Examples
/// ```
/// use typespec_rs::libs::compiler::replace_templated_string_from_properties;
/// let props = vec![("name", "Pet"), ("id", "123")];
/// let result = replace_templated_string_from_properties("{name}List{id}", &props);
/// assert_eq!(result, "PetList123");
/// ```
pub fn replace_templated_string_from_properties(
    format_string: &str,
    properties: &[(&str, &str)],
) -> String {
    let mut result = format_string.to_string();
    for (key, value) in properties {
        let placeholder = format!("{{{}}}", key.trim());
        result = result.replace(&placeholder, value);
    }
    result
}

/// Filter model properties in place by removing entries that don't satisfy the predicate.
///
/// Since Rust's type system doesn't allow mutating a RekeyableMap while iterating,
/// this returns the keys to remove, which the caller can then remove.
///
/// Ported from TS filterModelPropertiesInPlace().
///
/// # Arguments
/// * `entries` - Key-value pairs to filter
/// * `filter` - Predicate function; entries where filter returns false will be removed
///
/// # Returns
/// Vec of keys that should be removed
pub fn filter_model_properties_keys<K, V, F>(entries: &[(K, V)], filter: F) -> Vec<usize>
where
    F: Fn(&V) -> bool,
{
    entries
        .iter()
        .enumerate()
        .filter_map(|(i, (_, v))| if !filter(v) { Some(i) } else { None })
        .collect()
}

#[cfg(test)]
mod utils_tests {
    use super::*;

    #[test]
    fn test_replace_templated_string_simple() {
        let props = vec![("name", "Pet"), ("id", "123")];
        let result = replace_templated_string_from_properties("{name}List{id}", &props);
        assert_eq!(result, "PetList123");
    }

    #[test]
    fn test_replace_templated_string_no_match() {
        let props = vec![("name", "Pet")];
        let result = replace_templated_string_from_properties("{unknown}", &props);
        assert_eq!(result, "{unknown}");
    }

    #[test]
    fn test_replace_templated_string_empty_format() {
        let props = vec![("name", "Pet")];
        let result = replace_templated_string_from_properties("", &props);
        assert_eq!(result, "");
    }

    #[test]
    fn test_replace_templated_string_no_placeholders() {
        let props = vec![("name", "Pet")];
        let result = replace_templated_string_from_properties("plain text", &props);
        assert_eq!(result, "plain text");
    }

    #[test]
    fn test_replace_templated_string_whitespace_key() {
        let props = vec![(" name ", "Pet")];
        let result = replace_templated_string_from_properties("{name}", &props);
        assert_eq!(result, "Pet");
    }

    #[test]
    fn test_filter_model_properties_keys() {
        let entries = vec![("a", 1), ("b", 2), ("c", 3)];
        let to_remove = filter_model_properties_keys(&entries, |v| *v > 1);
        assert_eq!(to_remove, vec![0]); // "a" with value 1 should be removed
    }
}
