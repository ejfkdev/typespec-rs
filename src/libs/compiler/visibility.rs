//! Visibility-related decorators
//!
//! Ported from compiler/src/lib/visibility.ts
//!
//! NOTE: The full TS visibility system depends on Program, Mutator, Realm,
//! and the visibility core (isVisible, addVisibilityModifiers, etc.).
//! These state-based functions provide the storage layer; the runtime
//! filtering/transformation logic will be added when the checker is complete.

use crate::checker::types::TypeId;
use crate::state_accessors::StateAccessors;

// ============================================================================
// State keys
// ============================================================================

/// State key for @visibility decorator
pub const STATE_VISIBILITY: &str = "TypeSpec.visibility";
/// State key for @removeVisibility decorator
pub const STATE_REMOVE_VISIBILITY: &str = "TypeSpec.removeVisibility";
/// State key for @invisible decorator
pub const STATE_INVISIBLE: &str = "TypeSpec.invisible";
/// State key for @defaultVisibility decorator
pub const STATE_DEFAULT_VISIBILITY: &str = "TypeSpec.defaultVisibility";
/// State key for @withVisibility decorator
pub const STATE_WITH_VISIBILITY: &str = "TypeSpec.withVisibility";
/// State key for @withUpdateableProperties decorator
pub const STATE_WITH_UPDATEABLE_PROPERTIES: &str = "TypeSpec.withUpdateableProperties";
/// State key for @withVisibilityFilter decorator
pub const STATE_WITH_VISIBILITY_FILTER: &str = "TypeSpec.withVisibilityFilter";
/// State key for @withLifecycleUpdate decorator
pub const STATE_WITH_LIFECYCLE_UPDATE: &str = "TypeSpec.withLifecycleUpdate";
/// State key for @parameterVisibility decorator
pub const STATE_PARAMETER_VISIBILITY: &str = "TypeSpec.parameterVisibility";
/// State key for @returnTypeVisibility decorator
pub const STATE_RETURN_TYPE_VISIBILITY: &str = "TypeSpec.returnTypeVisibility";
/// State key for @withDefaultKeyVisibility decorator
pub const STATE_WITH_DEFAULT_KEY_VISIBILITY: &str = "TypeSpec.withDefaultKeyVisibility";

// ============================================================================
// Visibility decorators
// ============================================================================

/// Helper: get a comma-separated state value as a `Vec<String>`
fn get_comma_list(state: &StateAccessors, key: &str, target: TypeId) -> Vec<String> {
    match state.get_state(key, target) {
        Some(s) if !s.is_empty() => s.split(',').map(|s| s.to_string()).collect(),
        _ => Vec::new(),
    }
}

/// Apply @visibility decorator.
/// Stores visibility modifiers as a comma-separated string of enum member names.
pub fn apply_visibility(state: &mut StateAccessors, target: TypeId, modifiers: &[&str]) {
    let value = modifiers.join(",");
    // Append to existing visibility
    let existing = state.get_state(STATE_VISIBILITY, target).unwrap_or("");
    let new_value = if existing.is_empty() {
        value
    } else {
        format!("{},{}", existing, value)
    };
    state.set_state(STATE_VISIBILITY, target, new_value);
}

/// Get @visibility modifiers
pub fn get_visibility(state: &StateAccessors, target: TypeId) -> Vec<String> {
    get_comma_list(state, STATE_VISIBILITY, target)
}

/// Apply @removeVisibility decorator.
/// Stores visibility modifiers to remove as a comma-separated string.
pub fn apply_remove_visibility(state: &mut StateAccessors, target: TypeId, modifiers: &[&str]) {
    let value = modifiers.join(",");
    let existing = state
        .get_state(STATE_REMOVE_VISIBILITY, target)
        .unwrap_or("");
    let new_value = if existing.is_empty() {
        value
    } else {
        format!("{},{}", existing, value)
    };
    state.set_state(STATE_REMOVE_VISIBILITY, target, new_value);
}

/// Get @removeVisibility modifiers
pub fn get_remove_visibility(state: &StateAccessors, target: TypeId) -> Vec<String> {
    get_comma_list(state, STATE_REMOVE_VISIBILITY, target)
}

string_decorator!(apply_invisible, get_invisible, STATE_INVISIBLE);

/// Apply @defaultVisibility decorator
pub fn apply_default_visibility(state: &mut StateAccessors, target: TypeId, modifiers: &[&str]) {
    state.set_state(STATE_DEFAULT_VISIBILITY, target, modifiers.join(","));
}

/// Get @defaultVisibility modifiers
pub fn get_default_visibility(state: &StateAccessors, target: TypeId) -> Vec<String> {
    get_comma_list(state, STATE_DEFAULT_VISIBILITY, target)
}

/// Apply @withVisibility decorator (marks that the model was processed with visibility filter)
pub fn apply_with_visibility(state: &mut StateAccessors, target: TypeId, modifiers: &[&str]) {
    state.set_state(STATE_WITH_VISIBILITY, target, modifiers.join(","));
}

/// Get @withVisibility modifiers
pub fn get_with_visibility(state: &StateAccessors, target: TypeId) -> Vec<String> {
    get_comma_list(state, STATE_WITH_VISIBILITY, target)
}

flag_decorator!(
    apply_with_updateable_properties,
    is_with_updateable_properties,
    STATE_WITH_UPDATEABLE_PROPERTIES
);

string_decorator!(
    apply_with_visibility_filter,
    get_with_visibility_filter,
    STATE_WITH_VISIBILITY_FILTER
);

flag_decorator!(
    apply_with_lifecycle_update,
    is_with_lifecycle_update,
    STATE_WITH_LIFECYCLE_UPDATE
);

/// Apply @parameterVisibility decorator
pub fn apply_parameter_visibility(state: &mut StateAccessors, target: TypeId, modifiers: &[&str]) {
    state.set_state(STATE_PARAMETER_VISIBILITY, target, modifiers.join(","));
}

/// Get @parameterVisibility modifiers
pub fn get_parameter_visibility(state: &StateAccessors, target: TypeId) -> Vec<String> {
    get_comma_list(state, STATE_PARAMETER_VISIBILITY, target)
}

/// Apply @returnTypeVisibility decorator
pub fn apply_return_type_visibility(
    state: &mut StateAccessors,
    target: TypeId,
    modifiers: &[&str],
) {
    state.set_state(STATE_RETURN_TYPE_VISIBILITY, target, modifiers.join(","));
}

/// Get @returnTypeVisibility modifiers
pub fn get_return_type_visibility(state: &StateAccessors, target: TypeId) -> Vec<String> {
    get_comma_list(state, STATE_RETURN_TYPE_VISIBILITY, target)
}

string_decorator!(
    apply_with_default_key_visibility,
    get_with_default_key_visibility,
    STATE_WITH_DEFAULT_KEY_VISIBILITY
);
