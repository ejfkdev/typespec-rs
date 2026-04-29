//! State accessors for TypeSpec-Rust
//! Ported from TypeSpec compiler/src/core/state-accessors.ts
//!
//! In TS, state maps use Symbol keys for type-safe access.
//! In Rust, we use string keys with `HashMap<TypeId, V>` / `HashSet<TypeId>`.

use crate::checker::types::TypeId;
use std::collections::{HashMap, HashSet};

/// A map from TypeId to some string value, keyed by a state key name
/// In TS, this uses Map<Type, unknown>; in Rust we use string values
/// since all our state data can be represented as strings.
pub type StateMap = HashMap<TypeId, String>;

/// A set of TypeIds, keyed by a state key name
pub type StateSet = HashSet<TypeId>;

/// Container for all program state maps and sets
#[derive(Debug, Clone, Default)]
pub struct StateAccessors {
    /// Named state maps: key_name → (`TypeId` → string value)
    state_maps: HashMap<String, HashMap<TypeId, String>>,
    /// Named state sets: key_name → `Set<TypeId>`
    state_sets: HashMap<String, HashSet<TypeId>>,
}

impl StateAccessors {
    /// Create a new empty state accessors container
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or create a state map for the given key
    pub fn state_map(&mut self, key: &str) -> &mut HashMap<TypeId, String> {
        self.state_maps.entry(key.to_string()).or_default()
    }

    /// Get a state map for the given key (read-only)
    pub fn get_state_map(&self, key: &str) -> Option<&HashMap<TypeId, String>> {
        self.state_maps.get(key)
    }

    /// Get or create a state set for the given key
    pub fn state_set(&mut self, key: &str) -> &mut HashSet<TypeId> {
        self.state_sets.entry(key.to_string()).or_default()
    }

    /// Get a state set for the given key (read-only)
    pub fn get_state_set(&self, key: &str) -> Option<&HashSet<TypeId>> {
        self.state_sets.get(key)
    }

    /// Set a value in a state map
    pub fn set_state(&mut self, key: &str, type_id: TypeId, value: String) {
        self.state_maps
            .entry(key.to_string())
            .or_default()
            .insert(type_id, value);
    }

    /// Get a value from a state map
    pub fn get_state(&self, key: &str, type_id: TypeId) -> Option<&str> {
        self.state_maps
            .get(key)
            .and_then(|m| m.get(&type_id).map(|s| s.as_str()))
    }

    /// Check if a type is in a state set
    pub fn has_state(&self, key: &str, type_id: TypeId) -> bool {
        self.state_sets
            .get(key)
            .is_some_and(|s| s.contains(&type_id))
    }

    /// Add a type to a state set
    pub fn add_to_state(&mut self, key: &str, type_id: TypeId) {
        self.state_sets
            .entry(key.to_string())
            .or_default()
            .insert(type_id);
    }

    /// Remove a type from a state set
    pub fn remove_from_state(&mut self, key: &str, type_id: &TypeId) -> bool {
        self.state_sets
            .get_mut(key)
            .is_some_and(|s| s.remove(type_id))
    }

    /// Get the number of entries in a state map
    pub fn state_map_len(&self, key: &str) -> usize {
        self.state_maps.get(key).map_or(0, |m| m.len())
    }

    /// Get the number of entries in a state set
    pub fn state_set_len(&self, key: &str) -> usize {
        self.state_sets.get(key).map_or(0, |s| s.len())
    }

    /// Clear all entries in a state map
    pub fn clear_state_map(&mut self, key: &str) {
        if let Some(m) = self.state_maps.get_mut(key) {
            m.clear();
        }
    }

    /// Clear all entries in a state set
    pub fn clear_state_set(&mut self, key: &str) {
        if let Some(s) = self.state_sets.get_mut(key) {
            s.clear();
        }
    }

    /// Get an iterator over all state map keys and their entries
    pub fn iter_state_maps(&self) -> impl Iterator<Item = (&str, &HashMap<TypeId, String>)> {
        self.state_maps.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Get an iterator over all state set keys and their entries
    pub fn iter_state_sets(&self) -> impl Iterator<Item = (&str, &HashSet<TypeId>)> {
        self.state_sets.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Delete a type from a state map
    pub fn delete_from_state_map(&mut self, key: &str, type_id: &TypeId) -> bool {
        self.state_maps
            .get_mut(key)
            .is_some_and(|m| m.remove(type_id).is_some())
    }

    /// Parse a state map value as a typed value.
    /// Returns `None` if the key is not set or the value cannot be parsed.
    ///
    /// This eliminates the common pattern:
    /// `state.get_state(key, type_id).and_then(|s| s.parse::<T>().ok())`
    pub fn get_parsed<T: std::str::FromStr>(&self, key: &str, type_id: TypeId) -> Option<T> {
        self.get_state(key, type_id)
            .and_then(|s| s.parse::<T>().ok())
    }

    /// Check if a state map entry exists for the given key and type.
    /// Unlike `has_state` which checks the state sets, this checks the state maps.
    pub fn has_state_value(&self, key: &str, type_id: TypeId) -> bool {
        self.state_maps
            .get(key)
            .is_some_and(|m| m.contains_key(&type_id))
    }

    /// Append a value to an existing state map entry, with a separator.
    /// If no existing value, just sets the new value.
    ///
    /// This eliminates the common pattern:
    /// ```ignore
    /// let existing = state.get_state(key, target).unwrap_or("");
    /// let new_value = if existing.is_empty() { value } else { format!("{}{}{}", existing, sep, value) };
    /// state.set_state(key, target, new_value);
    /// ```
    pub fn append_state(&mut self, key: &str, type_id: TypeId, value: &str, separator: &str) {
        let current = self.get_state(key, type_id).unwrap_or("");
        if current.is_empty() {
            self.set_state(key, type_id, value.to_string());
        } else {
            self.set_state(key, type_id, format!("{}{}{}", current, separator, value));
        }
    }
}

/// Create a state accessors factory that returns stateMap and stateSet functions.
/// Ported from TS createStateAccessors().
/// In TS, this returns closures keyed by Symbol; in Rust, we use string keys.
pub fn create_state_accessors() -> StateAccessors {
    StateAccessors::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_map() {
        let mut sa = StateAccessors::new();
        sa.set_state("minValues", 10, "42".to_string());
        sa.set_state("minValues", 20, "100".to_string());

        assert_eq!(sa.get_state("minValues", 10), Some("42"));
        assert_eq!(sa.get_state("minValues", 20), Some("100"));
        assert_eq!(sa.get_state("minValues", 30), None);
    }

    #[test]
    fn test_state_set() {
        let mut sa = StateAccessors::new();
        assert!(!sa.has_state("deprecated", 10));

        sa.add_to_state("deprecated", 10);
        assert!(sa.has_state("deprecated", 10));
        assert!(!sa.has_state("deprecated", 20));

        assert!(sa.remove_from_state("deprecated", &10));
        assert!(!sa.has_state("deprecated", 10));
    }

    #[test]
    fn test_state_map_direct_access() {
        let mut sa = StateAccessors::new();
        let map = sa.state_map("docs");
        map.insert(5, "Some doc".to_string());

        let map = sa.get_state_map("docs");
        assert!(map.is_some());
        assert_eq!(map.unwrap().len(), 1);
    }

    #[test]
    fn test_multiple_state_keys() {
        let mut sa = StateAccessors::new();
        sa.set_state("minValues", 1, "0".to_string());
        sa.set_state("maxValues", 1, "100".to_string());
        sa.add_to_state("deprecated", 2);

        assert_eq!(sa.get_state("minValues", 1), Some("0"));
        assert_eq!(sa.get_state("maxValues", 1), Some("100"));
        assert!(sa.has_state("deprecated", 2));
        assert!(!sa.has_state("deprecated", 1));
    }

    #[test]
    fn test_create_state_accessors() {
        let mut sa = create_state_accessors();
        sa.set_state("test", 1, "hello".to_string());
        assert_eq!(sa.get_state("test", 1), Some("hello"));
    }

    #[test]
    fn test_state_map_len() {
        let mut sa = StateAccessors::new();
        assert_eq!(sa.state_map_len("test"), 0);
        sa.set_state("test", 1, "a".to_string());
        sa.set_state("test", 2, "b".to_string());
        assert_eq!(sa.state_map_len("test"), 2);
    }

    #[test]
    fn test_state_set_len() {
        let mut sa = StateAccessors::new();
        assert_eq!(sa.state_set_len("items"), 0);
        sa.add_to_state("items", 1);
        sa.add_to_state("items", 2);
        assert_eq!(sa.state_set_len("items"), 2);
    }

    #[test]
    fn test_clear_state_map() {
        let mut sa = StateAccessors::new();
        sa.set_state("data", 1, "a".to_string());
        sa.clear_state_map("data");
        assert_eq!(sa.state_map_len("data"), 0);
    }

    #[test]
    fn test_clear_state_set() {
        let mut sa = StateAccessors::new();
        sa.add_to_state("items", 1);
        sa.clear_state_set("items");
        assert_eq!(sa.state_set_len("items"), 0);
    }

    #[test]
    fn test_delete_from_state_map() {
        let mut sa = StateAccessors::new();
        sa.set_state("data", 1, "a".to_string());
        assert!(sa.delete_from_state_map("data", &1));
        assert_eq!(sa.get_state("data", 1), None);
        assert!(!sa.delete_from_state_map("data", &1)); // already deleted
    }

    #[test]
    fn test_state_map_overwrite() {
        let mut sa = StateAccessors::new();
        sa.set_state("val", 1, "old".to_string());
        sa.set_state("val", 1, "new".to_string());
        assert_eq!(sa.get_state("val", 1), Some("new"));
    }

    #[test]
    fn test_state_set_duplicate_add() {
        let mut sa = StateAccessors::new();
        sa.add_to_state("items", 1);
        sa.add_to_state("items", 1); // duplicate, no effect
        assert_eq!(sa.state_set_len("items"), 1);
    }

    #[test]
    fn test_get_parsed() {
        let mut sa = StateAccessors::new();
        sa.set_state("count", 1, "42".to_string());
        sa.set_state("ratio", 2, "3.14".to_string());
        sa.set_state("bad", 3, "not_a_number".to_string());

        assert_eq!(sa.get_parsed::<i64>("count", 1), Some(42));
        assert_eq!(
            sa.get_parsed::<f64>("ratio", 2),
            Some(std::f64::consts::PI.floor() + 0.14)
        );
        assert_eq!(sa.get_parsed::<i64>("bad", 3), None);
        assert_eq!(sa.get_parsed::<i64>("missing", 1), None);
    }

    #[test]
    fn test_has_state_value() {
        let mut sa = StateAccessors::new();
        sa.set_state("key", 1, "value".to_string());
        assert!(sa.has_state_value("key", 1));
        assert!(!sa.has_state_value("key", 2));
        assert!(!sa.has_state_value("missing", 1));
    }

    #[test]
    fn test_append_state() {
        let mut sa = StateAccessors::new();
        sa.append_state("tags", 1, "tag1", ";");
        assert_eq!(sa.get_state("tags", 1), Some("tag1"));

        sa.append_state("tags", 1, "tag2", ";");
        assert_eq!(sa.get_state("tags", 1), Some("tag1;tag2"));

        sa.append_state("tags", 1, "tag3", ";");
        assert_eq!(sa.get_state("tags", 1), Some("tag1;tag2;tag3"));
    }
}
