//! Encoded name validation and resolution
//!
//! Ported from TypeSpec compiler/src/lib/encoded-names.ts

use crate::checker::types::TypeId;
use crate::state_accessors::StateAccessors;

/// State key for @encodedName decorator
pub const STATE_ENCODED_NAME: &str = "TypeSpec.encodedName";

/// Apply @encodedName decorator
///
/// Supports multiple encoded names per target (one per mime type).
/// Stored as `mime1::name1||mime2::name2` internally.
pub fn apply_encoded_name(state: &mut StateAccessors, target: TypeId, mime_type: &str, name: &str) {
    let existing = state
        .get_state(STATE_ENCODED_NAME, target)
        .unwrap_or("")
        .to_string();
    let new_entry = format!("{}::{}", mime_type, name);
    let new_value = if existing.is_empty() {
        new_entry
    } else {
        // Check if this mime type already has an entry, update it
        let entries: Vec<&str> = existing.split("||").collect();
        let mut found = false;
        let updated: Vec<String> = entries
            .iter()
            .map(|e| {
                if let Some(colon_pos) = e.find("::") {
                    let mt = &e[..colon_pos];
                    if mt == mime_type {
                        found = true;
                        new_entry.clone()
                    } else {
                        e.to_string()
                    }
                } else {
                    e.to_string()
                }
            })
            .collect();
        if found {
            updated.join("||")
        } else {
            format!("{}||{}", existing, new_entry)
        }
    };
    state.set_state(STATE_ENCODED_NAME, target, new_value);
}

/// Get @encodedName value for a specific mime type
///
/// Supports MIME type suffix resolution (e.g., "application/merge-patch+json"
/// will also match "application/json" entries).
///
/// Ported from TS `getEncodedName()`.
pub fn get_encoded_name(state: &StateAccessors, target: TypeId, mime_type: &str) -> Option<String> {
    // Parse the MIME type for suffix resolution
    let parsed = crate::mime_type::parse_mime_type(mime_type);
    let resolved_mime_type = parsed.as_ref().and_then(|p| {
        p.suffix
            .as_ref()
            .map(|suffix| format!("{}/{}", p.mime_type, suffix))
    });

    state.get_state(STATE_ENCODED_NAME, target).and_then(|s| {
        let entries: Vec<&str> = s.split("||").collect();
        // First, try exact match
        for entry in &entries {
            if let Some(colon_pos) = entry.find("::") {
                let mt = &entry[..colon_pos];
                if mt == mime_type {
                    return Some(entry[colon_pos + 2..].to_string());
                }
            }
        }
        // If no exact match, try resolving MIME type with suffix
        if let Some(resolved) = resolved_mime_type {
            for entry in &entries {
                if let Some(colon_pos) = entry.find("::") {
                    let mt = &entry[..colon_pos];
                    if mt == resolved {
                        return Some(entry[colon_pos + 2..].to_string());
                    }
                }
            }
        }
        None
    })
}

/// Get all encoded names for a target
///
/// Returns a vector of (mime_type, name) pairs.
pub fn get_all_encoded_names(state: &StateAccessors, target: TypeId) -> Vec<(String, String)> {
    state
        .get_state(STATE_ENCODED_NAME, target)
        .map(|s| {
            s.split("||")
                .filter_map(|entry| {
                    let colon_pos = entry.find("::")?;
                    Some((
                        entry[..colon_pos].to_string(),
                        entry[colon_pos + 2..].to_string(),
                    ))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Resolve the encoded name for a type when serialized to a given mime type.
/// If a specific value was provided by `@encodedName` decorator for that mime type
/// it will return that, otherwise it will return the default name.
///
/// Ported from TS resolveEncodedName().
pub fn resolve_encoded_name(
    state: &StateAccessors,
    target: TypeId,
    mime_type: &str,
    default_name: &str,
) -> String {
    get_encoded_name(state, target, mime_type).unwrap_or_else(|| default_name.to_string())
}

/// Result of validating encoded names for conflicts.
///
/// In the TS version, this function reports diagnostics via the Program.
/// Here we return a list of conflicts for the caller to handle.
#[derive(Debug, Clone)]
pub struct EncodedNameConflict {
    /// The TypeId that has the conflicting encoded name
    pub target: TypeId,
    /// The MIME type where the conflict occurs
    pub mime_type: String,
    /// The encoded name that conflicts
    pub name: String,
    /// The kind of conflict
    pub kind: EncodedNameConflictKind,
}

/// Kind of encoded name conflict
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodedNameConflictKind {
    /// The encoded name conflicts with an existing member name in the same scope
    MemberConflict,
    /// The encoded name duplicates another encoded name for the same MIME type
    Duplicate,
}

/// Validate encoded names for conflicts across all types in the state.
///
/// Checks:
/// 1. No encoded name should conflict with an existing member name in the same scope
/// 2. No two encoded names should map to the same name for the same MIME type
///
/// Ported from TS `validateEncodedNamesConflicts()`.
/// Note: The TS version uses the full type graph to determine scope.
/// This simplified version requires the caller to provide a set of
/// existing member names per parent type.
pub fn validate_encoded_names_conflicts(
    state: &StateAccessors,
    existing_names: &std::collections::HashMap<TypeId, Vec<String>>,
) -> Vec<EncodedNameConflict> {
    let mut conflicts = Vec::new();

    // Track duplicates: (mime_type, encoded_name) -> [type_ids]
    let mut duplicate_tracker: std::collections::HashMap<(String, String), Vec<TypeId>> =
        std::collections::HashMap::new();

    // Iterate all encoded names
    for (&target_id, names) in existing_names.iter() {
        let all_encoded = get_all_encoded_names(state, target_id);

        for (mime_type, encoded_name) in &all_encoded {
            // Check if encoded name conflicts with existing member names
            if names.iter().any(|n| n == encoded_name) {
                conflicts.push(EncodedNameConflict {
                    target: target_id,
                    mime_type: mime_type.clone(),
                    name: encoded_name.clone(),
                    kind: EncodedNameConflictKind::MemberConflict,
                });
            }

            // Track for duplicate detection
            let key = (mime_type.clone(), encoded_name.clone());
            duplicate_tracker.entry(key).or_default().push(target_id);
        }
    }

    // Report duplicates
    for ((mime_type, encoded_name), type_ids) in duplicate_tracker.iter() {
        if type_ids.len() > 1 {
            for &target_id in type_ids {
                conflicts.push(EncodedNameConflict {
                    target: target_id,
                    mime_type: mime_type.clone(),
                    name: encoded_name.clone(),
                    kind: EncodedNameConflictKind::Duplicate,
                });
            }
        }
    }

    conflicts
}
