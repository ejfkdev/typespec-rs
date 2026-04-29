//! Utility data structures and functions
//!
//! Ported from TypeSpec compiler/src/utils/ and compiler/src/core/types.ts (RekeyableMap)
//!
//! Includes:
//! - RekeyableMap: Insertion-ordered map with key rekeying support
//! - DuplicateTracker: Track duplicate entries by key
//! - TwoLevelMap: Map with two-level keys
//! - Queue: Efficient queue with amortized O(1) operations
//! - Collection equality utilities (array_equals, map_equals)
//! - Case-insensitive string map
//! - Misc utilities (is_defined, distinct_array)

use std::collections::HashMap;
use std::hash::Hash;
use std::vec::Vec;

// ============================================================================
// RekeyableMap — Ported from TS types.ts RekeyableMap
// ============================================================================

/// A map that allows rekeying (changing keys) while maintaining insertion order.
/// This is similar to JavaScript's Map but with the ability to change keys.
#[derive(Debug, Clone)]
pub struct RekeyableMap<K, V> {
    entries: Vec<(K, V)>,
}

impl<K, V> RekeyableMap<K, V>
where
    K: Eq + Clone,
    V: Clone,
{
    /// Create a new empty RekeyableMap
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Create a RekeyableMap from an iterator of key-value pairs
    pub fn from_entries<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        Self {
            entries: iter.into_iter().collect(),
        }
    }

    /// Get a reference to the value associated with the key
    pub fn get(&self, key: &K) -> Option<&V> {
        self.entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Get a mutable reference to the value associated with the key
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.entries
            .iter_mut()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    /// Insert a key-value pair. If the key already exists, the value is updated.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if let Some(existing) = self.entries.iter_mut().find(|(k, _)| k == &key) {
            let old_value = std::mem::replace(existing, (key, value));
            return Some(old_value.1);
        }
        self.entries.push((key, value));
        None
    }

    /// Set a key-value pair (alias for insert)
    pub fn set(&mut self, key: K, value: V) -> &mut Self {
        self.insert(key, value);
        self
    }

    /// Remove a key from the map
    pub fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(pos) = self.entries.iter().position(|(k, _)| k == key) {
            Some(self.entries.remove(pos).1)
        } else {
            None
        }
    }

    /// Check if the map contains a key
    pub fn contains_key(&self, key: &K) -> bool {
        self.entries.iter().any(|(k, _)| k == key)
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the map is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get an iterator over the entries in insertion order
    pub fn iter(&self) -> std::slice::Iter<'_, (K, V)> {
        self.entries.iter()
    }

    /// Get an iterator over the entries with mutable references
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        self.entries.iter_mut().map(|(k, v)| (&*k, v))
    }

    /// Get an iterator over the keys
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.entries.iter().map(|(k, _)| k)
    }

    /// Get an iterator over the values
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.entries.iter().map(|(_, v)| v)
    }

    /// Rekey a key - change the key while maintaining position in the map.
    pub fn rekey(&mut self, old_key: &K, new_key: K) -> &mut Self {
        let old_pos = match self.entries.iter().position(|(k, _)| k == old_key) {
            Some(pos) => pos,
            None => return self,
        };
        let value = self.entries[old_pos].1.clone();
        let new_pos = self.entries.iter().position(|(k, _)| k == &new_key);
        let final_pos = match new_pos {
            Some(np) if np < old_pos => np,
            Some(np) if np > old_pos => np - 1,
            _ => old_pos,
        };
        self.entries.remove(old_pos);
        if new_pos.is_some() {
            // After removing at old_pos, re-find the new_key entry
            if let Some(remove_pos) = self.entries.iter().position(|(k, _)| k == &new_key) {
                self.entries.remove(remove_pos);
            }
        }
        self.entries.insert(final_pos, (new_key, value));
        self
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl<K, V> Default for RekeyableMap<K, V>
where
    K: Eq + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> IntoIterator for RekeyableMap<K, V>
where
    K: Eq + Clone,
    V: Clone,
{
    type Item = (K, V);
    type IntoIter = std::vec::IntoIter<(K, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl<'a, K, V> IntoIterator for &'a RekeyableMap<K, V>
where
    K: Eq + Clone,
    V: Clone,
{
    type Item = (&'a K, &'a V);
    type IntoIter = std::iter::Map<std::slice::Iter<'a, (K, V)>, fn(&(K, V)) -> (&K, &V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter().map(|(k, v)| (k, v))
    }
}

/// Create a RekeyableMap from an iterator
pub fn create_rekeyable_map<I, K, V>(iter: I) -> RekeyableMap<K, V>
where
    I: IntoIterator<Item = (K, V)>,
    K: Eq + Clone,
    V: Clone,
{
    RekeyableMap::from_entries(iter)
}

// ============================================================================
// DuplicateTracker — Ported from TS utils/duplicate-tracker.ts
// ============================================================================

/// Helper class to track duplicate instances by key
#[derive(Debug, Default)]
pub struct DuplicateTracker<K, V> {
    entries: HashMap<K, Vec<V>>,
}

impl<K: Hash + Eq, V: Clone> DuplicateTracker<K, V> {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Track usage of key with associated value
    pub fn track(&mut self, k: K, v: V) {
        self.entries.entry(k).or_default().push(v);
    }

    /// Return iterator of all duplicate entries (keys with more than one value)
    pub fn duplicates(&self) -> impl Iterator<Item = (&K, &Vec<V>)> {
        self.entries.iter().filter(|(_, v)| v.len() > 1)
    }
}

/// A map with exactly two keys per value
#[derive(Debug, Default)]
pub struct TwoLevelMap<K1, K2, V> {
    map: HashMap<K1, HashMap<K2, V>>,
}

impl<K1: Hash + Eq, K2: Hash + Eq, V> TwoLevelMap<K1, K2, V> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Get an existing entry or add a new one
    pub fn get_or_add<F>(&mut self, key1: K1, key2: K2, create: F) -> &V
    where
        F: FnOnce() -> V,
    {
        self.map
            .entry(key1)
            .or_default()
            .entry(key2)
            .or_insert_with(create)
    }

    /// Get a reference to the value
    pub fn get(&self, key1: &K1, key2: &K2) -> Option<&V> {
        self.map.get(key1).and_then(|m| m.get(key2))
    }
}

/// Efficient queue with amortized O(1) dequeue
#[derive(Debug, Default)]
pub struct Queue<T> {
    elements: Vec<Option<T>>,
    head_index: usize,
}

impl<T> Queue<T> {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            head_index: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head_index == self.elements.len()
    }

    pub fn enqueue(&mut self, item: T) {
        self.elements.push(Some(item));
    }

    pub fn dequeue(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        let result = self.elements[self.head_index].take();
        self.head_index += 1;

        // Compact if more than half is empty and we'd save > 100 slots
        if self.head_index > 100 && self.head_index > self.elements.len() >> 1 {
            self.elements.drain(0..self.head_index);
            self.head_index = 0;
        }

        result
    }

    pub fn len(&self) -> usize {
        self.elements.len() - self.head_index
    }
}

/// Check if two arrays have the same elements
pub fn array_equals<T, F>(left: &[T], right: &[T], equals: F) -> bool
where
    F: Fn(&T, &T) -> bool,
{
    if left.len() != right.len() {
        return false;
    }
    for i in 0..left.len() {
        if !equals(&left[i], &right[i]) {
            return false;
        }
    }
    true
}

/// Check if two HashMaps have the same entries
pub fn map_equals<K, V, F>(left: &HashMap<K, V>, right: &HashMap<K, V>, equals: F) -> bool
where
    K: Hash + Eq,
    F: Fn(&V, &V) -> bool,
{
    if left.len() != right.len() {
        return false;
    }
    for (key, value) in left {
        match right.get(key) {
            Some(rv) if equals(value, rv) => {}
            _ => return false,
        }
    }
    true
}

/// Case-insensitive string map
#[derive(Debug, Default)]
pub struct CaseInsensitiveMap<V> {
    inner: HashMap<String, V>,
}

impl<V> CaseInsensitiveMap<V> {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&V> {
        self.inner.get(&key.to_uppercase())
    }

    pub fn insert(&mut self, key: String, value: V) -> Option<V> {
        self.inner.insert(key.to_uppercase(), value)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.inner.contains_key(&key.to_uppercase())
    }

    pub fn remove(&mut self, key: &str) -> Option<V> {
        self.inner.remove(&key.to_uppercase())
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

/// Return distinct items from an array based on a key selector
pub fn distinct_array<T, K, F>(arr: &[T], key_selector: F) -> Vec<&T>
where
    K: Hash + Eq,
    F: Fn(&T) -> K,
{
    let mut seen = HashMap::new();
    for item in arr {
        let key = key_selector(item);
        seen.entry(key).or_insert(item);
    }
    seen.into_values().collect()
}

/// Check if argument is not None
pub fn is_defined<T>(arg: &Option<T>) -> bool {
    arg.is_some()
}

/// Check if a string is whitespace or empty
pub fn is_whitespace_string_or_undefined(s: Option<&str>) -> bool {
    match s {
        None => true,
        Some(str) => str.trim().is_empty(),
    }
}

/// Get the index of the first non-whitespace character
pub fn first_non_whitespace_character_index(line: &str) -> usize {
    line.chars()
        .position(|c| !c.is_whitespace())
        .unwrap_or(line.len())
}

/// Remove entries with None values from a HashMap
pub fn omit_none<V>(data: &HashMap<String, Option<V>>) -> HashMap<String, &V> {
    data.iter()
        .filter_map(|(k, v)| v.as_ref().map(|val| (k.clone(), val)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== RekeyableMap tests ====================

    #[test]
    fn test_rekeyable_map_new() {
        let map: RekeyableMap<String, i32> = RekeyableMap::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_rekeyable_map_from_iter() {
        let map = RekeyableMap::from_entries(vec![
            ("a".to_string(), 1),
            ("b".to_string(), 2),
            ("c".to_string(), 3),
        ]);
        assert_eq!(map.len(), 3);
        assert_eq!(map.get(&"a".to_string()), Some(&1));
    }

    #[test]
    fn test_rekeyable_map_insert_update() {
        let mut map = RekeyableMap::new();
        map.insert("a".to_string(), 1);
        map.insert("b".to_string(), 2);
        assert_eq!(map.len(), 2);
        map.insert("a".to_string(), 10);
        assert_eq!(map.get(&"a".to_string()), Some(&10));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_rekeyable_map_keys_values_order() {
        let mut map = RekeyableMap::new();
        map.insert("a".to_string(), 1);
        map.insert("b".to_string(), 2);
        map.insert("c".to_string(), 3);
        let keys: Vec<_> = map.keys().cloned().collect();
        assert_eq!(keys, vec!["a", "b", "c"]);
        let values: Vec<_> = map.values().cloned().collect();
        assert_eq!(values, vec![1, 2, 3]);
    }

    #[test]
    fn test_rekeyable_map_remove() {
        let mut map = RekeyableMap::from_entries(vec![
            ("a".to_string(), 1),
            ("b".to_string(), 2),
            ("c".to_string(), 3),
        ]);
        let removed = map.remove(&"b".to_string());
        assert_eq!(removed, Some(2));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_rekeyable_map_rekey() {
        let mut map = RekeyableMap::from_entries(vec![
            ("a".to_string(), 1),
            ("b".to_string(), 2),
            ("c".to_string(), 3),
        ]);
        map.rekey(&"b".to_string(), "renamed".to_string());
        let entries: Vec<_> = map.iter().map(|(k, v)| (k.clone(), *v)).collect();
        assert_eq!(
            entries,
            vec![
                ("a".to_string(), 1),
                ("renamed".to_string(), 2),
                ("c".to_string(), 3),
            ]
        );
    }

    #[test]
    fn test_rekeyable_map_rekey_to_existing() {
        let mut map = RekeyableMap::from_entries(vec![
            ("a".to_string(), 1),
            ("b".to_string(), 2),
            ("c".to_string(), 3),
        ]);
        map.rekey(&"c".to_string(), "b".to_string());
        assert_eq!(map.get(&"b".to_string()), Some(&3)); // c's value
    }

    #[test]
    fn test_rekeyable_map_iter_mut() {
        let mut map = RekeyableMap::from_entries(vec![("a".to_string(), 1), ("b".to_string(), 2)]);
        for (_, v) in map.iter_mut() {
            *v *= 10;
        }
        assert_eq!(map.get(&"a".to_string()), Some(&10));
    }

    #[test]
    fn test_create_rekeyable_map() {
        let map = create_rekeyable_map(vec![("x", 1), ("y", 2)]);
        assert_eq!(map.len(), 2);
    }

    // ==================== DuplicateTracker tests ====================

    #[test]
    fn test_duplicate_tracker_no_duplicates() {
        let mut tracker = DuplicateTracker::new();
        tracker.track("a", 1);
        tracker.track("b", 2);
        assert_eq!(tracker.duplicates().count(), 0);
    }

    #[test]
    fn test_duplicate_tracker_with_duplicates() {
        let mut tracker = DuplicateTracker::new();
        tracker.track("a", 1);
        tracker.track("a", 2);
        tracker.track("b", 3);
        let dups: Vec<_> = tracker.duplicates().collect();
        assert_eq!(dups.len(), 1);
        assert_eq!(dups[0].0, &"a");
        assert_eq!(dups[0].1, &[1, 2]);
    }

    #[test]
    fn test_two_level_map() {
        let mut map = TwoLevelMap::new();
        let val = map.get_or_add("a", "b", || 42);
        assert_eq!(*val, 42);
        assert_eq!(map.get(&"a", &"b"), Some(&42));
    }

    #[test]
    fn test_queue() {
        let mut q = Queue::new();
        assert!(q.is_empty());
        q.enqueue(1);
        q.enqueue(2);
        q.enqueue(3);
        assert_eq!(q.dequeue(), Some(1));
        assert_eq!(q.dequeue(), Some(2));
        assert_eq!(q.dequeue(), Some(3));
        assert_eq!(q.dequeue(), None);
    }

    #[test]
    fn test_array_equals() {
        assert!(array_equals(&[1, 2, 3], &[1, 2, 3], |a, b| a == b));
        assert!(!array_equals(&[1, 2, 3], &[1, 2, 4], |a, b| a == b));
        assert!(!array_equals(&[1, 2], &[1, 2, 3], |a, b| a == b));
    }

    #[test]
    fn test_map_equals() {
        let left = HashMap::from([("a", 1), ("b", 2)]);
        let right = HashMap::from([("a", 1), ("b", 2)]);
        assert!(map_equals(&left, &right, |a, b| a == b));

        let right2 = HashMap::from([("a", 1), ("b", 3)]);
        assert!(!map_equals(&left, &right2, |a, b| a == b));
    }

    #[test]
    fn test_case_insensitive_map() {
        let mut m = CaseInsensitiveMap::new();
        m.insert("Hello".to_string(), 42);
        assert_eq!(m.get("hello"), Some(&42));
        assert_eq!(m.get("HELLO"), Some(&42));
        assert!(m.contains_key("hElLo"));
    }

    #[test]
    fn test_distinct_array() {
        let arr = vec!["a", "b", "a", "c", "b"];
        let result = distinct_array(&arr, |s| *s);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_is_whitespace_string() {
        assert!(is_whitespace_string_or_undefined(None));
        assert!(is_whitespace_string_or_undefined(Some("")));
        assert!(is_whitespace_string_or_undefined(Some("   ")));
        assert!(!is_whitespace_string_or_undefined(Some("hello")));
    }

    #[test]
    fn test_first_non_whitespace_character_index() {
        assert_eq!(first_non_whitespace_character_index("hello"), 0);
        assert_eq!(first_non_whitespace_character_index("  hello"), 2);
        assert_eq!(first_non_whitespace_character_index("   "), 3);
    }
}
