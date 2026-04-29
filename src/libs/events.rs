//! @typespec/events - Event System Decorators
//!
//! Ported from TypeSpec packages/events
//!
//! Provides decorators for describing event types in TypeSpec:
//! - `@events` - Mark a union as describing a set of events
//! - `@contentType` - Specify content type for event envelope/body/payload
//! - `@data` - Identify the payload field of an event
//!
//! ## Diagnostics
//! - `invalid-content-type-target` - @contentType on non-top-level, non-payload field
//! - `multiple-event-payloads` - Multiple @data on same event
//!
//! ## Helper Functions
//! - `is_events(state, target)` - Check if a union is decorated with @events
//! - `get_content_type(state, target)` - Get content type for a variant or property
//! - `is_event_data(state, target)` - Check if a property is marked as @data

use crate::diagnostics::{DiagnosticDefinition, DiagnosticMap};
#[cfg(test)]
use crate::state_accessors::StateAccessors;
use std::collections::HashMap;

// ============================================================================
// Diagnostic codes
// ============================================================================

/// Diagnostic: @contentType on invalid target
pub const DIAG_INVALID_CONTENT_TYPE_TARGET: &str = "invalid-content-type-target";
/// Diagnostic: Multiple event payloads in same event
pub const DIAG_MULTIPLE_EVENT_PAYLOADS: &str = "multiple-event-payloads";

// ============================================================================
// State keys (fully qualified with namespace)
// ============================================================================

/// State key for @events decorator
pub const STATE_EVENTS: &str = "TypeSpec.Events.events";
/// State key for @contentType decorator
pub const STATE_CONTENT_TYPE: &str = "TypeSpec.Events.contentType";
/// State key for @data decorator
pub const STATE_DATA: &str = "TypeSpec.Events.data";

/// Namespace for events types
pub const EVENTS_NAMESPACE: &str = "TypeSpec.Events";

// ============================================================================
// Event definition types
// ============================================================================

/// Definition of an event extracted from a union variant.
/// Ported from TS EventDefinition interface.
#[derive(Debug, Clone)]
pub struct EventDefinition {
    /// Optional event type name (from named union variant)
    pub event_type: Option<String>,
    /// Content type of the event envelope (from @contentType on variant)
    pub envelope_content_type: Option<String>,
    /// The payload type of the event
    pub payload_type: Option<String>,
    /// Content type of the payload (from @contentType on @data property)
    pub payload_content_type: Option<String>,
    /// Path to the payload property
    pub path_to_payload: String,
}

// ============================================================================
// Library creation
// ============================================================================

/// Create the @typespec/events library diagnostic map.
/// Ported from TS $lib definition in lib.ts.
pub fn create_events_library() -> DiagnosticMap {
    HashMap::from([
        (
            DIAG_INVALID_CONTENT_TYPE_TARGET.to_string(),
            DiagnosticDefinition::error(
                "@contentType can only be specified on the top-level event envelope, or the event payload marked with @data",
            ),
        ),
        (
            DIAG_MULTIPLE_EVENT_PAYLOADS.to_string(),
            DiagnosticDefinition::error_with_messages(vec![
                (
                    "default",
                    "Event payload already applied to {dataPath} but also exists under {currentPath}",
                ),
                (
                    "payloadInIndexedModel",
                    "Event payload applied from inside a Record or Array at {dataPath}",
                ),
            ]),
        ),
    ])
}

// ============================================================================
// Decorator implementations
// ============================================================================

flag_decorator!(apply_events, is_events, STATE_EVENTS);
string_decorator!(apply_content_type, get_content_type, STATE_CONTENT_TYPE);
flag_decorator!(apply_data, is_event_data, STATE_DATA);

// ============================================================================
// TSP Sources
// ============================================================================

/// The TypeSpec source for the events library decorators
pub const EVENTS_DECORATORS_TSP: &str = r#"
using TypeSpec.Reflection;

namespace TypeSpec.Events;

/**
 * Specify that this union describes a set of events.
 *
 * @example
 *
 * ```typespec
 * @events
 * union MixedEvents {
 *   pingEvent: string;
 *
 *   doneEvent: "done";
 * }
 * ```
 */
extern dec events(target: Union);

/**
 * Specifies the content type of the event envelope, event body, or event payload.
 * When applied to an event payload, that field must also have a corresponding `@data`
 * decorator.
 *
 * @example
 *
 * ```typespec
 * @events union MixedEvents {
 *   @contentType("application/json")
 *   message: { id: string, text: string, }
 * }
 * ```
 *
 * @example Specify the content type of the event payload.
 *
 * ```typespec
 * @events union MixedEvents {
 *   { done: true },
 *
 *   { done: false, @data @contentType("text/plain") value: string,}
 * }
 * ```
 */
extern dec contentType(target: UnionVariant | ModelProperty, contentType: valueof string);

/**
 * Identifies the payload of an event.
 * Only one field in an event can be marked as the payload.
 *
 * @example
 *
 * ```typespec
 * @events union MixedEvents {
 *   { metadata: Record<string>, @data payload: string,}
 * }
 * ```
 */
extern dec data(target: ModelProperty);
"#;

/// The TypeSpec source for the events library main entry
pub const EVENTS_MAIN_TSP: &str = r#"
import "../dist/src/tsp-index.js";
import "./decorators.tsp";
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_events_namespace() {
        assert_eq!(EVENTS_NAMESPACE, "TypeSpec.Events");
    }

    #[test]
    fn test_state_keys() {
        assert!(STATE_EVENTS.starts_with("TypeSpec.Events"));
        assert!(STATE_CONTENT_TYPE.starts_with("TypeSpec.Events"));
        assert!(STATE_DATA.starts_with("TypeSpec.Events"));
    }

    #[test]
    fn test_create_events_library() {
        let diags = create_events_library();
        assert_eq!(diags.len(), 2);
        let codes: Vec<&str> = diags.keys().map(|code| code.as_str()).collect();
        assert!(codes.contains(&DIAG_INVALID_CONTENT_TYPE_TARGET));
        assert!(codes.contains(&DIAG_MULTIPLE_EVENT_PAYLOADS));
    }

    #[test]
    fn test_decorators_tsp_not_empty() {
        assert!(!EVENTS_DECORATORS_TSP.is_empty());
        assert!(EVENTS_DECORATORS_TSP.contains("events"));
        assert!(EVENTS_DECORATORS_TSP.contains("contentType"));
        assert!(EVENTS_DECORATORS_TSP.contains("data"));
    }

    #[test]
    fn test_event_definition() {
        let def = EventDefinition {
            event_type: Some("pingEvent".to_string()),
            envelope_content_type: Some("application/json".to_string()),
            payload_type: Some("string".to_string()),
            payload_content_type: None,
            path_to_payload: "payload".to_string(),
        };
        assert_eq!(def.event_type, Some("pingEvent".to_string()));
        assert!(def.payload_content_type.is_none());
    }

    #[test]
    fn test_is_events() {
        let mut state = StateAccessors::new();
        assert!(!is_events(&state, 1));

        apply_events(&mut state, 1);
        assert!(is_events(&state, 1));
        assert!(!is_events(&state, 2));
    }

    #[test]
    fn test_get_content_type() {
        let mut state = StateAccessors::new();
        assert_eq!(get_content_type(&state, 1), None);

        apply_content_type(&mut state, 1, "application/json");
        assert_eq!(
            get_content_type(&state, 1),
            Some("application/json".to_string())
        );
    }

    #[test]
    fn test_is_event_data() {
        let mut state = StateAccessors::new();
        assert!(!is_event_data(&state, 1));

        apply_data(&mut state, 1);
        assert!(is_event_data(&state, 1));
        assert!(!is_event_data(&state, 2));
    }

    #[test]
    fn test_content_type_overwrite() {
        let mut state = StateAccessors::new();
        apply_content_type(&mut state, 1, "text/plain");
        assert_eq!(get_content_type(&state, 1), Some("text/plain".to_string()));

        apply_content_type(&mut state, 1, "application/json");
        assert_eq!(
            get_content_type(&state, 1),
            Some("application/json".to_string())
        );
    }
}
