//! Deprecation tracking for TypeSpec-Rust
//! Ported from TypeSpec compiler/src/core/deprecation.ts

use crate::checker::types::TypeId;
use std::collections::HashMap;

/// Details about a type's deprecation
#[derive(Debug, Clone)]
pub struct DeprecationDetails {
    /// The deprecation message to display when the type is used
    pub message: String,
}

/// Tracker for deprecated types
#[derive(Debug, Clone, Default)]
pub struct DeprecationTracker {
    /// Map from TypeId to its deprecation details
    deprecated: HashMap<TypeId, DeprecationDetails>,
}

impl DeprecationTracker {
    pub fn new() -> Self {
        Self {
            deprecated: HashMap::new(),
        }
    }

    /// Check if the given type is deprecated
    pub fn is_deprecated(&self, type_id: TypeId) -> bool {
        self.deprecated.contains_key(&type_id)
    }

    /// Get deprecation details for a type
    pub fn get_deprecation_details(&self, type_id: TypeId) -> Option<&DeprecationDetails> {
        self.deprecated.get(&type_id)
    }

    /// Mark a type as deprecated
    pub fn mark_deprecated(&mut self, type_id: TypeId, details: DeprecationDetails) {
        self.deprecated.insert(type_id, details);
    }

    /// Remove a type's deprecation status
    pub fn unmark_deprecated(&mut self, type_id: TypeId) {
        self.deprecated.remove(&type_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deprecation_tracking() {
        let mut tracker = DeprecationTracker::new();
        assert!(!tracker.is_deprecated(10));

        tracker.mark_deprecated(
            10,
            DeprecationDetails {
                message: "Use NewType instead".to_string(),
            },
        );
        assert!(tracker.is_deprecated(10));
        assert_eq!(
            tracker.get_deprecation_details(10).unwrap().message,
            "Use NewType instead"
        );

        tracker.unmark_deprecated(10);
        assert!(!tracker.is_deprecated(10));
    }
}
