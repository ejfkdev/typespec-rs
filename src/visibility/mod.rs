//! Visibility system for TypeSpec properties
//!
//! Ported from TypeSpec compiler/src/core/visibility/
//!
//! The visibility system determines when properties of a conceptual resource
//! are present. It's based on visibility classes (represented by TypeSpec enums)
//! with visibility modifiers (enum members).
//!
//! NOTE: This is a simplified port. The full TS implementation uses Program-scoped
//! state (WeakMap, useStateMap/useStateSet) which requires infrastructure not yet
//! available. The core data structures and logic are preserved.

use crate::checker::types::TypeId;
use std::collections::{HashMap, HashSet};

// ============================================================================
// Lifecycle visibility (ported from visibility/lifecycle.ts)
// ============================================================================

/// Get the Lifecycle visibility enum TypeId from the standard library.
/// Ported from TS getLifecycleVisibilityEnum().
/// Returns None if the TypeSpec.Lifecycle enum is not available (e.g., stdlib not loaded).
pub fn get_lifecycle_visibility_enum(checker: &crate::checker::Checker) -> Option<TypeId> {
    checker.get_std_type("Lifecycle")
}

/// Visibility modifiers per visibility class (enum type)
pub type VisibilityModifiers = HashMap<TypeId, HashSet<TypeId>>;

/// Visibility store for tracking property visibility
#[derive(Debug, Default)]
pub struct VisibilityStore {
    /// Map of property TypeId → visibility modifiers
    store: HashMap<TypeId, VisibilityModifiers>,
    /// Set of properties with sealed visibility
    sealed_properties: HashSet<TypeId>,
    /// Set of properties with sealed visibility per class
    sealed_classes: HashMap<TypeId, HashSet<TypeId>>,
    /// Default modifier sets per visibility class
    default_modifiers: HashMap<TypeId, HashSet<TypeId>>,
    /// Whether the entire program's visibility is sealed
    program_sealed: bool,
}

impl VisibilityStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a property's visibility is sealed
    pub fn is_sealed(&self, property_id: TypeId, visibility_class: Option<TypeId>) -> bool {
        if self.program_sealed {
            return true;
        }
        if self.sealed_properties.contains(&property_id) {
            return true;
        }
        if let Some(class_id) = visibility_class
            && let Some(sealed) = self.sealed_classes.get(&property_id)
        {
            return sealed.contains(&class_id);
        }
        false
    }

    /// Seal a property's visibility modifiers
    pub fn seal_visibility(&mut self, property_id: TypeId, visibility_class: Option<TypeId>) {
        if let Some(class_id) = visibility_class {
            self.sealed_classes
                .entry(property_id)
                .or_default()
                .insert(class_id);
        } else {
            self.sealed_properties.insert(property_id);
        }
    }

    /// Seal all visibility modifiers for the program
    pub fn seal_program(&mut self) {
        self.program_sealed = true;
    }

    /// Get or initialize visibility modifiers for a property
    pub fn get_or_initialize(&mut self, property_id: TypeId) -> &mut VisibilityModifiers {
        self.store.entry(property_id).or_default()
    }

    /// Get the visibility modifiers for a property and class
    pub fn get_visibility_for_class(
        &mut self,
        property_id: TypeId,
        visibility_class: TypeId,
    ) -> HashSet<TypeId> {
        let modifiers = self.store.entry(property_id).or_default();
        modifiers
            .get(&visibility_class)
            .cloned()
            .unwrap_or_else(|| self.get_default_modifier_set(visibility_class))
    }

    /// Add visibility modifiers to a property
    pub fn add_visibility_modifiers(
        &mut self,
        property_id: TypeId,
        class_id: TypeId,
        modifiers: &[TypeId],
    ) -> bool {
        if self.is_sealed(property_id, Some(class_id)) {
            return false;
        }
        let vis = self.store.entry(property_id).or_default();
        let set = vis.entry(class_id).or_default();
        for &m in modifiers {
            set.insert(m);
        }
        true
    }

    /// Remove visibility modifiers from a property
    pub fn remove_visibility_modifiers(
        &mut self,
        property_id: TypeId,
        class_id: TypeId,
        modifiers: &[TypeId],
    ) -> bool {
        if self.is_sealed(property_id, Some(class_id)) {
            return false;
        }
        let default_set = self.get_default_modifier_set(class_id);
        let vis = self.store.entry(property_id).or_default();
        let set = vis.entry(class_id).or_insert(default_set);
        for &m in modifiers {
            set.remove(&m);
        }
        true
    }

    /// Get the default modifier set for a visibility class
    fn get_default_modifier_set(&self, visibility_class: TypeId) -> HashSet<TypeId> {
        self.default_modifiers
            .get(&visibility_class)
            .cloned()
            .unwrap_or_default()
    }

    /// Set the default modifier set for a visibility class
    pub fn set_default_modifier_set(
        &mut self,
        visibility_class: TypeId,
        default_set: HashSet<TypeId>,
    ) {
        self.default_modifiers.insert(visibility_class, default_set);
    }

    /// Clear all visibility modifiers for a property in a given class
    pub fn clear_visibility_for_class(&mut self, property_id: TypeId, class_id: TypeId) {
        if let Some(modifiers) = self.store.get_mut(&property_id) {
            modifiers.remove(&class_id);
        }
    }

    /// Reset visibility modifiers for a property in a given class to the default set
    pub fn reset_visibility_for_class(&mut self, property_id: TypeId, class_id: TypeId) {
        self.clear_visibility_for_class(property_id, class_id);
        if let Some(defaults) = self.default_modifiers.get(&class_id) {
            let entry = self.store.entry(property_id).or_default();
            entry.insert(class_id, defaults.clone());
        }
    }
}

/// A visibility filter that determines if a property is visible
#[derive(Debug, Clone, Default)]
pub struct VisibilityFilter {
    /// Property must have ALL of these visibility modifiers
    pub all: Option<HashSet<TypeId>>,
    /// Property must have ANY of these visibility modifiers
    pub any: Option<HashSet<TypeId>>,
    /// Property must have NONE of these visibility modifiers
    pub none: Option<HashSet<TypeId>>,
}

/// Check if a property is visible according to the given filter
pub fn is_visible(
    store: &mut VisibilityStore,
    property_id: TypeId,
    filter: &VisibilityFilter,
) -> bool {
    // Check ALL constraint
    if let Some(ref all) = filter.all {
        for &modifier_id in all {
            // Simplified: just check if modifier is in any class's set
            if let Some(modifiers) = store.store.get(&property_id) {
                let found = modifiers.values().any(|set| set.contains(&modifier_id));
                if !found {
                    return false;
                }
            } else {
                return false;
            }
        }
    }

    // Check NONE constraint
    if let Some(ref none) = filter.none {
        for &modifier_id in none {
            if let Some(modifiers) = store.store.get(&property_id) {
                let found = modifiers.values().any(|set| set.contains(&modifier_id));
                if found {
                    return false;
                }
            }
        }
    }

    // Check ANY constraint
    if let Some(ref any) = filter.any {
        if any.is_empty() {
            return false;
        }
        let mut found_any = false;
        for &modifier_id in any {
            if let Some(modifiers) = store.store.get(&property_id)
                && modifiers.values().any(|set| set.contains(&modifier_id))
            {
                found_any = true;
                break;
            }
        }
        if !found_any {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visibility_store_seal_property() {
        let mut store = VisibilityStore::new();
        assert!(!store.is_sealed(1, None));
        store.seal_visibility(1, None);
        assert!(store.is_sealed(1, None));
    }

    #[test]
    fn test_visibility_store_seal_class() {
        let mut store = VisibilityStore::new();
        assert!(!store.is_sealed(1, Some(10)));
        store.seal_visibility(1, Some(10));
        assert!(store.is_sealed(1, Some(10)));
        assert!(!store.is_sealed(1, Some(20)));
    }

    #[test]
    fn test_visibility_store_seal_program() {
        let mut store = VisibilityStore::new();
        store.seal_program();
        assert!(store.is_sealed(1, None));
        assert!(store.is_sealed(1, Some(10)));
    }

    #[test]
    fn test_add_visibility_modifiers() {
        let mut store = VisibilityStore::new();
        assert!(store.add_visibility_modifiers(1, 10, &[100, 101]));
        let vis = store.get_visibility_for_class(1, 10);
        assert!(vis.contains(&100));
        assert!(vis.contains(&101));
    }

    #[test]
    fn test_add_visibility_when_sealed() {
        let mut store = VisibilityStore::new();
        store.seal_visibility(1, Some(10));
        assert!(!store.add_visibility_modifiers(1, 10, &[100]));
    }

    #[test]
    fn test_remove_visibility_modifiers() {
        let mut store = VisibilityStore::new();
        store.add_visibility_modifiers(1, 10, &[100, 101]);
        assert!(store.remove_visibility_modifiers(1, 10, &[100]));
        let vis = store.get_visibility_for_class(1, 10);
        assert!(!vis.contains(&100));
        assert!(vis.contains(&101));
    }

    #[test]
    fn test_default_modifier_set() {
        let mut store = VisibilityStore::new();
        store.set_default_modifier_set(10, HashSet::from([100, 101, 102]));
        let vis = store.get_visibility_for_class(1, 10);
        assert_eq!(vis.len(), 3);
    }

    #[test]
    fn test_is_visible_with_all_filter() {
        let mut store = VisibilityStore::new();
        store.add_visibility_modifiers(1, 10, &[100, 101]);
        let filter = VisibilityFilter {
            all: Some(HashSet::from([100, 101])),
            ..Default::default()
        };
        assert!(is_visible(&mut store, 1, &filter));
    }

    #[test]
    fn test_is_visible_with_all_filter_missing() {
        let mut store = VisibilityStore::new();
        store.add_visibility_modifiers(1, 10, &[100]);
        let filter = VisibilityFilter {
            all: Some(HashSet::from([100, 101])),
            ..Default::default()
        };
        assert!(!is_visible(&mut store, 1, &filter));
    }

    #[test]
    fn test_is_visible_with_any_filter() {
        let mut store = VisibilityStore::new();
        store.add_visibility_modifiers(1, 10, &[100]);
        let filter = VisibilityFilter {
            any: Some(HashSet::from([100, 101])),
            ..Default::default()
        };
        assert!(is_visible(&mut store, 1, &filter));
    }

    #[test]
    fn test_is_visible_with_any_filter_empty() {
        let mut store = VisibilityStore::new();
        let filter = VisibilityFilter {
            any: Some(HashSet::new()),
            ..Default::default()
        };
        assert!(!is_visible(&mut store, 1, &filter));
    }

    #[test]
    fn test_is_visible_with_none_filter() {
        let mut store = VisibilityStore::new();
        store.add_visibility_modifiers(1, 10, &[100, 101]);
        let filter = VisibilityFilter {
            none: Some(HashSet::from([102])),
            ..Default::default()
        };
        assert!(is_visible(&mut store, 1, &filter));
    }

    #[test]
    fn test_is_visible_with_none_filter_excluded() {
        let mut store = VisibilityStore::new();
        store.add_visibility_modifiers(1, 10, &[100]);
        let filter = VisibilityFilter {
            none: Some(HashSet::from([100])),
            ..Default::default()
        };
        assert!(!is_visible(&mut store, 1, &filter));
    }

    // ============================================================================
    // Additional visibility tests (ported from TS visibility.test.ts)
    // ============================================================================

    #[test]
    fn test_default_visibility_no_modifiers() {
        // A property with no visibility modifiers should not be visible with any filter
        let mut store = VisibilityStore::new();
        let filter = VisibilityFilter {
            any: Some(HashSet::from([100])),
            ..Default::default()
        };
        assert!(!is_visible(&mut store, 1, &filter));
    }

    #[test]
    fn test_visibility_multiple_classes() {
        // A property can have visibility in multiple classes
        let mut store = VisibilityStore::new();
        store.add_visibility_modifiers(1, 10, &[100]);
        store.add_visibility_modifiers(1, 20, &[200]);
        // Should be visible in class 10
        let vis_10 = store.get_visibility_for_class(1, 10);
        assert!(vis_10.contains(&100));
        // Should be visible in class 20
        let vis_20 = store.get_visibility_for_class(1, 20);
        assert!(vis_20.contains(&200));
    }

    #[test]
    fn test_visibility_add_same_modifier_twice() {
        // Adding the same modifier twice should be idempotent
        let mut store = VisibilityStore::new();
        assert!(store.add_visibility_modifiers(1, 10, &[100]));
        assert!(store.add_visibility_modifiers(1, 10, &[100]));
        let vis = store.get_visibility_for_class(1, 10);
        assert!(vis.contains(&100));
    }

    #[test]
    fn test_visibility_remove_nonexistent() {
        // Removing a modifier that doesn't exist should return true (no error)
        let mut store = VisibilityStore::new();
        assert!(store.remove_visibility_modifiers(1, 10, &[100]));
    }

    #[test]
    fn test_visibility_clear_modifiers() {
        // Clear all visibility for a class
        let mut store = VisibilityStore::new();
        store.add_visibility_modifiers(1, 10, &[100, 101]);
        store.clear_visibility_for_class(1, 10);
        let vis = store.get_visibility_for_class(1, 10);
        assert!(vis.is_empty());
    }

    #[test]
    fn test_visibility_reset_modifiers() {
        // Reset visibility to default for a class
        let mut store = VisibilityStore::new();
        store.set_default_modifier_set(10, HashSet::from([100, 101]));
        store.add_visibility_modifiers(1, 10, &[200]);
        store.reset_visibility_for_class(1, 10);
        let vis = store.get_visibility_for_class(1, 10);
        // Should be back to defaults
        assert!(vis.contains(&100));
        assert!(vis.contains(&101));
        assert!(!vis.contains(&200));
    }

    #[test]
    fn test_visibility_seal_then_add_fails() {
        // Adding modifiers after sealing should fail
        let mut store = VisibilityStore::new();
        store.add_visibility_modifiers(1, 10, &[100]);
        store.seal_visibility(1, Some(10));
        assert!(!store.add_visibility_modifiers(1, 10, &[101]));
        // Original modifier should still be there
        let vis = store.get_visibility_for_class(1, 10);
        assert!(vis.contains(&100));
        assert!(!vis.contains(&101));
    }

    #[test]
    fn test_visibility_seal_program_then_add_fails() {
        // Adding modifiers after sealing the program should fail
        let mut store = VisibilityStore::new();
        store.add_visibility_modifiers(1, 10, &[100]);
        store.seal_program();
        assert!(!store.add_visibility_modifiers(2, 10, &[100]));
    }

    #[test]
    fn test_visibility_remove_when_sealed_fails() {
        // Removing modifiers after sealing should fail
        let mut store = VisibilityStore::new();
        store.add_visibility_modifiers(1, 10, &[100]);
        store.seal_visibility(1, Some(10));
        assert!(!store.remove_visibility_modifiers(1, 10, &[100]));
    }

    #[test]
    fn test_is_visible_combined_filters() {
        // Test with both all and any filters
        let mut store = VisibilityStore::new();
        store.add_visibility_modifiers(1, 10, &[100, 101]);
        let filter = VisibilityFilter {
            all: Some(HashSet::from([100])),
            any: Some(HashSet::from([101])),
            ..Default::default()
        };
        assert!(is_visible(&mut store, 1, &filter));
    }

    #[test]
    fn test_is_visible_combined_filters_fails() {
        // Combined filter where one part fails
        let mut store = VisibilityStore::new();
        store.add_visibility_modifiers(1, 10, &[100]);
        let filter = VisibilityFilter {
            all: Some(HashSet::from([100, 101])), // 101 not present
            any: Some(HashSet::from([100])),
            ..Default::default()
        };
        assert!(!is_visible(&mut store, 1, &filter));
    }

    #[test]
    fn test_is_visible_no_filters() {
        // No filters means everything is visible
        let mut store = VisibilityStore::new();
        let filter = VisibilityFilter::default();
        assert!(is_visible(&mut store, 1, &filter));
    }

    #[test]
    fn test_visibility_default_modifiers_applied_to_new_property() {
        // When a default modifier set exists, new properties get those defaults
        let mut store = VisibilityStore::new();
        store.set_default_modifier_set(10, HashSet::from([100, 101]));
        let vis = store.get_visibility_for_class(1, 10);
        assert!(vis.contains(&100));
        assert!(vis.contains(&101));
    }

    #[test]
    fn test_visibility_preserves_other_classes() {
        // Modifying visibility for one class shouldn't affect another
        let mut store = VisibilityStore::new();
        store.add_visibility_modifiers(1, 10, &[100]);
        store.add_visibility_modifiers(1, 20, &[200]);
        store.clear_visibility_for_class(1, 10);
        // Class 10 should be empty
        assert!(store.get_visibility_for_class(1, 10).is_empty());
        // Class 20 should still have 200
        assert!(store.get_visibility_for_class(1, 20).contains(&200));
    }

    #[test]
    fn test_visibility_multiple_properties() {
        // Multiple properties with different visibility
        let mut store = VisibilityStore::new();
        store.add_visibility_modifiers(1, 10, &[100]);
        store.add_visibility_modifiers(2, 10, &[101]);
        let filter = VisibilityFilter {
            any: Some(HashSet::from([100])),
            ..Default::default()
        };
        assert!(is_visible(&mut store, 1, &filter));
        assert!(!is_visible(&mut store, 2, &filter));
    }

    #[test]
    fn test_visibility_none_filter_with_multiple_classes() {
        // None filter should exclude based on any class
        let mut store = VisibilityStore::new();
        store.add_visibility_modifiers(1, 10, &[100]);
        store.add_visibility_modifiers(1, 20, &[200]);
        let filter = VisibilityFilter {
            none: Some(HashSet::from([100])),
            ..Default::default()
        };
        assert!(!is_visible(&mut store, 1, &filter));
    }

    #[test]
    fn test_visibility_all_filter_with_multiple_classes() {
        // All filter should require modifiers across all classes
        let mut store = VisibilityStore::new();
        store.add_visibility_modifiers(1, 10, &[100]);
        store.add_visibility_modifiers(1, 20, &[200]);
        let filter = VisibilityFilter {
            all: Some(HashSet::from([100, 200])),
            ..Default::default()
        };
        assert!(is_visible(&mut store, 1, &filter));
    }

    // ==================== Lifecycle visibility tests ====================

    #[test]
    fn test_get_lifecycle_visibility_enum() {
        use crate::parser::parse;
        let result = parse("");
        let mut checker = crate::checker::Checker::new();
        checker.set_parse_result(result.root_id, result.builder);
        checker.check_program();
        // Without stdlib loaded, Lifecycle enum may not be available
        let result = get_lifecycle_visibility_enum(&checker);
        // Just verify the function doesn't panic
        let _ = result;
    }
}
